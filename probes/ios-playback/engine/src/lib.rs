//! iPhone playback feasibility probe: the pinned librespot engine behind a
//! narrow C ABI.
//!
//! librespot decodes Spotify audio and runs Spotify Connect. Its sink here
//! does not open a device: it hands 44.1 kHz stereo f32 PCM to a bounded
//! queue that the Swift side drains from an `AVAudioSourceNode` render block.
//! The app owns the `AVAudioSession`; this crate never touches audio
//! hardware.
//!
//! Nothing here logs audio, access tokens or reusable credentials. Events
//! carry track metadata and counters only. Credentials cross the boundary
//! once, through the dedicated callback, so the app can keep them in the
//! Keychain.

use std::collections::VecDeque;
use std::ffi::{CStr, CString, c_char, c_void};
use std::future::Future;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::Duration;

use librespot_connect::{
    ConnectConfig, LoadContextOptions, LoadRequest, LoadRequestOptions, Options, PlaybackSnapshot,
    Spirc,
};
use librespot_core::{
    authentication::Credentials,
    cache::Cache,
    config::{DeviceType, SessionConfig},
    session::Session,
};
use librespot_metadata::audio::UniqueFields;
use librespot_playback::{
    audio_backend::{Sink, SinkResult},
    config::PlayerConfig,
    convert::Converter,
    decoder::AudioPacket,
    mixer::{self, MixerConfig},
    player::{Player, PlayerEvent},
};
use serde_json::json;

/// librespot always decodes to this.
pub const SAMPLE_RATE: u32 = 44_100;
pub const CHANNELS: usize = 2;
/// Half a second of interleaved samples. The decoder blocks above this, so
/// it runs at playback speed and a pause wastes little.
const QUEUE_CAP: usize = SAMPLE_RATE as usize * CHANNELS / 2;

pub type EventCallback = extern "C" fn(json: *const c_char, ctx: *mut c_void);
pub type CredentialsCallback = extern "C" fn(data: *const u8, len: usize, ctx: *mut c_void);

/// Decoded PCM waiting for the render block, plus counters for evidence.
pub struct Pcm {
    queue: Mutex<VecDeque<f32>>,
    space: Condvar,
    /// Frames that left the queue into the render block.
    rendered_frames: AtomicU64,
    /// Frames the render block asked for while the queue was empty or busy.
    silent_frames: AtomicU64,
    /// Frames librespot's player handed to the sink.
    decoded_frames: AtomicU64,
    /// RMS of the most recent non-silent render, as f32 bits.
    last_rms: AtomicU64,
}

impl Pcm {
    const fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            space: Condvar::new(),
            rendered_frames: AtomicU64::new(0),
            silent_frames: AtomicU64::new(0),
            decoded_frames: AtomicU64::new(0),
            last_rms: AtomicU64::new(0),
        }
    }

    /// Blocks the player thread while the queue is full.
    pub fn push(&self, samples: &[f32]) {
        self.push_active(samples, &AtomicBool::new(true));
    }

    fn push_active(&self, samples: &[f32], active: &AtomicBool) {
        let mut queue = self.queue.lock().unwrap_or_else(|p| p.into_inner());
        while active.load(Ordering::Acquire)
            && queue.len() + samples.len() > QUEUE_CAP.max(samples.len())
        {
            queue = self
                .space
                .wait_timeout(queue, Duration::from_millis(250))
                .unwrap_or_else(|p| p.into_inner())
                .0;
        }
        if !active.load(Ordering::Acquire) {
            return;
        }
        queue.extend(samples);
        self.decoded_frames
            .fetch_add((samples.len() / CHANNELS) as u64, Ordering::Relaxed);
    }

    /// Fills `left`/`right` from the queue and pads with silence. Never
    /// blocks: the render block runs on Core Audio's real-time thread.
    pub fn render(&self, left: &mut [f32], right: &mut [f32]) -> usize {
        let frames = left.len().min(right.len());
        // ponytail: try_lock on the real-time thread; an SPSC ring (rtrb)
        // would remove the rare silent block if contention ever shows up.
        let filled = match self.queue.try_lock() {
            Ok(mut queue) => {
                let available = (queue.len() / CHANNELS).min(frames);
                let mut energy = 0f64;
                for frame in 0..available {
                    let l = queue.pop_front().unwrap_or(0.0);
                    let r = queue.pop_front().unwrap_or(0.0);
                    energy += f64::from(l * l + r * r);
                    left[frame] = l;
                    right[frame] = r;
                }
                drop(queue);
                if available > 0 {
                    let rms = (energy / (available * CHANNELS) as f64).sqrt() as f32;
                    self.last_rms
                        .store(u64::from(rms.to_bits()), Ordering::Relaxed);
                    self.space.notify_one();
                }
                available
            }
            Err(_) => 0,
        };
        left[filled..frames].fill(0.0);
        right[filled..frames].fill(0.0);
        self.rendered_frames
            .fetch_add(filled as u64, Ordering::Relaxed);
        self.silent_frames
            .fetch_add((frames - filled) as u64, Ordering::Relaxed);
        filled
    }

    pub fn clear(&self) {
        self.queue.lock().unwrap_or_else(|p| p.into_inner()).clear();
        self.space.notify_all();
    }
}

static PCM: Pcm = Pcm::new();

struct QueueSink {
    active: Arc<AtomicBool>,
}

impl Sink for QueueSink {
    fn write(&mut self, packet: AudioPacket, converter: &mut Converter) -> SinkResult<()> {
        if let AudioPacket::Samples(samples) = packet {
            PCM.push_active(&converter.f64_to_f32(&samples), &self.active);
        }
        Ok(())
    }
}

struct Host {
    on_event: EventCallback,
    on_credentials: CredentialsCallback,
    ctx: usize,
}

impl Host {
    fn emit(&self, value: serde_json::Value) {
        if let Ok(line) = CString::new(value.to_string()) {
            (self.on_event)(line.as_ptr(), self.ctx as *mut c_void);
        }
    }
}

struct Probe {
    runtime: tokio::runtime::Runtime,
    host: Arc<Host>,
    cache_dir: String,
    device_name: String,
    connections: Arc<ConnectionAttempts>,
}

#[derive(Default)]
struct ConnectionState {
    generation: u64,
    task: Option<tokio::task::JoinHandle<()>>,
    active: Option<Arc<AtomicBool>>,
    spirc: Option<Arc<Spirc>>,
}

impl ConnectionState {
    fn cancel(&mut self) -> Option<tokio::task::JoinHandle<()>> {
        self.generation = self.generation.wrapping_add(1);
        if let Some(active) = self.active.take() {
            active.store(false, Ordering::Release);
            // Release a decoder blocked on a full queue before Player drops.
            PCM.space.notify_all();
        }
        let task = self.task.take();
        if let Some(task) = &task {
            task.abort();
        }
        if let Some(spirc) = self.spirc.take() {
            let _ = spirc.shutdown();
        }
        task
    }
}

/// One tracked attempt, including authentication and reconnect backoff. A
/// replacement waits until cancellation has dropped the prior session guards.
#[derive(Default)]
struct ConnectionAttempts {
    state: Mutex<ConnectionState>,
    serial: tokio::sync::Mutex<()>,
}

impl ConnectionAttempts {
    fn start<F: Future<Output = ()> + Send + 'static>(
        self: &Arc<Self>,
        runtime: &tokio::runtime::Handle,
        run: impl FnOnce(u64, Arc<AtomicBool>) -> F + Send + 'static,
    ) {
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        drop(state.cancel());
        let generation = state.generation;
        let active = Arc::new(AtomicBool::new(true));
        state.active = Some(Arc::clone(&active));
        let connections = Arc::clone(self);
        state.task = Some(runtime.spawn(async move {
            let _exclusive = connections.serial.lock().await;
            if connections.is_current(generation) {
                run(generation, active).await;
            }
        }));
    }

    fn cancel(&self) -> Option<tokio::task::JoinHandle<()>> {
        self.state
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .cancel()
    }

    fn is_current(&self, generation: u64) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .generation
            == generation
    }

    fn current(&self) -> Option<Arc<Spirc>> {
        self.state
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .spirc
            .clone()
    }

    fn publish(&self, generation: u64, spirc: Arc<Spirc>) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        if state.generation != generation {
            return false;
        }
        state.spirc = Some(spirc);
        true
    }
}

/// Cancellation also covers a session still authenticating, its event task,
/// and the Connect loop. Dropping the future must not leave any of them alive.
struct SessionLifetime<'a> {
    session: Session,
    connections: &'a ConnectionAttempts,
    spirc: Option<Arc<Spirc>>,
    events: tokio::task::JoinHandle<()>,
}

impl Drop for SessionLifetime<'_> {
    fn drop(&mut self) {
        self.events.abort();
        if let Some(spirc) = &self.spirc {
            let _ = spirc.shutdown();
            let mut state = self
                .connections
                .state
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            if state.spirc.as_ref().is_some_and(|s| Arc::ptr_eq(s, spirc)) {
                state.spirc.take();
            }
        }
        self.session.shutdown();
        PCM.clear();
    }
}

static PROBE: OnceLock<Probe> = OnceLock::new();

/// Forwards librespot warnings and errors only. Lower levels can include
/// request details; these two describe failures.
struct HostLogger;

impl log::Log for HostLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Warn
    }
    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata())
            && let Some(probe) = PROBE.get()
        {
            probe.host.emit(json!({
                "t": "log",
                "level": record.level().as_str(),
                "target": record.target(),
                "msg": record.args().to_string(),
            }));
        }
    }
    fn flush(&self) {}
}

static LOGGER: HostLogger = HostLogger;

fn c_str(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: the caller passes a NUL-terminated string valid for this call.
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .ok()
        .map(str::to_owned)
}

/// Starts the runtime. Call once, before anything else.
///
/// # Safety
/// `cache_dir` and `device_name` are NUL-terminated UTF-8. The callbacks and
/// `ctx` stay valid for the life of the process.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_start(
    cache_dir: *const c_char,
    device_name: *const c_char,
    on_event: EventCallback,
    on_credentials: CredentialsCallback,
    ctx: *mut c_void,
) -> i32 {
    let (Some(cache_dir), Some(device_name)) = (c_str(cache_dir), c_str(device_name)) else {
        return -1;
    };
    let Ok(runtime) = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
    else {
        return -2;
    };
    let probe = Probe {
        runtime,
        host: Arc::new(Host {
            on_event,
            on_credentials,
            ctx: ctx as usize,
        }),
        cache_dir,
        device_name,
        connections: Arc::default(),
    };
    if PROBE.set(probe).is_err() {
        return -3;
    }
    let _ = log::set_logger(&LOGGER).map(|()| log::set_max_level(log::LevelFilter::Warn));
    0
}

/// Signs in and announces this phone on Spotify Connect.
///
/// `kind` 0: `data` is an access token for Spotify's streaming scope.
/// `kind` 1: `data` is the JSON reusable credential this probe stored.
///
/// # Safety
/// `data` points to `len` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_connect(kind: u32, data: *const u8, len: usize) -> i32 {
    let Some(probe) = PROBE.get() else { return -1 };
    if data.is_null() {
        return -2;
    }
    // SAFETY: guaranteed by the caller.
    let bytes = unsafe { std::slice::from_raw_parts(data, len) }.to_vec();
    let credentials = match kind {
        0 => match String::from_utf8(bytes) {
            Ok(token) => Credentials::with_access_token(token),
            Err(_) => return -3,
        },
        1 => match serde_json::from_slice::<Credentials>(&bytes) {
            Ok(credentials) => credentials,
            Err(_) => return -3,
        },
        _ => return -3,
    };
    let host = Arc::clone(&probe.host);
    probe.connections.start(
        probe.runtime.handle(),
        move |generation, active| async move {
            if let Err(error) = connect(probe, generation, active, credentials).await
                && probe.connections.is_current(generation)
            {
                host.emit(json!({ "t": "error", "stage": "connect", "msg": error }));
            }
        },
    );
    0
}

/// Runs Connect sessions until one ends without playback to resume. When
/// Spotify drops the connection mid-playback (seen after about 13 minutes on
/// the phone), reconnect with the reusable credential and restore the exact
/// queue, as the desktop engine does.
async fn connect(
    probe: &'static Probe,
    generation: u64,
    active: Arc<AtomicBool>,
    mut credentials: Credentials,
) -> Result<(), String> {
    const RETRY_DELAYS_S: [u64; 6] = [1, 2, 4, 8, 16, 32];
    let mut pending: Option<Arc<PlaybackSnapshot>> = None;
    let mut failures = 0;
    loop {
        if !probe.connections.is_current(generation) {
            return Ok(());
        }
        match run_session(
            probe,
            generation,
            Arc::clone(&active),
            credentials.clone(),
            pending.clone(),
        )
        .await
        {
            Ok((None, _)) => {
                // Silent when a newer session already took over.
                if probe.connections.is_current(generation) && probe.connections.current().is_none()
                {
                    probe.host.emit(json!({ "t": "session_ended" }));
                }
                return Ok(());
            }
            Ok((Some(snapshot), stored)) => {
                probe
                    .host
                    .emit(json!({ "t": "reconnecting", "reason": "session_lost_during_playback" }));
                pending = Some(snapshot);
                credentials = stored;
                failures = 0;
            }
            Err(error) if pending.is_some() && failures < RETRY_DELAYS_S.len() => {
                probe.host.emit(
                    json!({ "t": "reconnect_failed", "attempt": failures + 1, "msg": error }),
                );
                tokio::time::sleep(Duration::from_secs(RETRY_DELAYS_S[failures])).await;
                failures += 1;
            }
            Err(error) => return Err(error),
        }
    }
}

/// One Connect session. Returns playback to restore when the session was lost
/// while active, with the credential to reconnect with.
async fn run_session(
    probe: &'static Probe,
    generation: u64,
    active: Arc<AtomicBool>,
    credentials: Credentials,
    restore: Option<Arc<PlaybackSnapshot>>,
) -> Result<(Option<Arc<PlaybackSnapshot>>, Credentials), String> {
    let cache = Cache::new(None, Some(probe.cache_dir.as_str()), None, None)
        .map(Cache::with_memory_credentials)
        .map_err(|e| e.to_string())?;
    // librespot's iOS identity is internally consistent only with its own
    // default client ID: the client token is always requested for it, so a
    // keymaster override makes login5 answer BAD_REQUEST.
    let session = Session::new(SessionConfig::default(), Some(cache));
    let mixer =
        mixer::find(Some("softvol")).ok_or("soft volume mixer missing")?(MixerConfig::default())
            .map_err(|e| e.to_string())?;
    let player = Player::new(
        PlayerConfig {
            position_update_interval: Some(Duration::from_secs(5)),
            ..PlayerConfig::default()
        },
        session.clone(),
        mixer.get_soft_volume(),
        move || Box::new(QueueSink { active }),
    );
    let events = tokio::spawn(forward_events(
        player.get_player_event_channel(),
        Arc::clone(&probe.host),
        Arc::clone(&probe.connections),
        generation,
    ));
    let mut lifetime = SessionLifetime {
        session: session.clone(),
        connections: &probe.connections,
        spirc: None,
        events,
    };
    let connected = Spirc::new(
        ConnectConfig {
            name: probe.device_name.clone(),
            device_type: DeviceType::Smartphone,
            initial_volume: u16::MAX,
            ..ConnectConfig::default()
        },
        session.clone(),
        credentials.clone(),
        player,
        mixer,
    )
    .await;
    // The access point issues the reusable credential before Connect starts,
    // so keep it even when a later step fails: the next launch reconnects
    // without another browser sign-in.
    if probe.connections.is_current(generation)
        && let Some(stored) = session.cache().and_then(|cache| cache.credentials())
        && let Ok(blob) = serde_json::to_vec(&stored)
    {
        (probe.host.on_credentials)(blob.as_ptr(), blob.len(), probe.host.ctx as *mut c_void);
    }
    let (spirc, task) = connected.map_err(|e| e.to_string())?;
    let spirc = Arc::new(spirc);
    lifetime.spirc = Some(Arc::clone(&spirc));
    if !probe.connections.publish(generation, Arc::clone(&spirc)) {
        return Ok((None, credentials));
    }
    let restored = match restore {
        Some(snapshot) => spirc.restore_playback(snapshot).is_ok(),
        None => false,
    };
    probe
        .host
        .emit(json!({ "t": "connected", "restored": restored }));
    task.await;
    let stored = session
        .cache()
        .and_then(|cache| cache.credentials())
        .unwrap_or(credentials);
    Ok((spirc.disconnected_playback(), stored))
}

async fn forward_events(
    mut events: tokio::sync::mpsc::UnboundedReceiver<PlayerEvent>,
    host: Arc<Host>,
    connections: Arc<ConnectionAttempts>,
    generation: u64,
) {
    while let Some(event) = events.recv().await {
        if !connections.is_current(generation) {
            return;
        }
        let value = match event {
            PlayerEvent::TrackChanged { audio_item } => {
                let artists = match &audio_item.unique_fields {
                    UniqueFields::Track { artists, .. } => artists
                        .iter()
                        .map(|a| a.name.clone())
                        .collect::<Vec<_>>()
                        .join(", "),
                    UniqueFields::Episode { show_name, .. } => show_name.clone(),
                    UniqueFields::Local { artists, .. } => artists.clone().unwrap_or_default(),
                };
                json!({
                    "t": "track",
                    "uri": audio_item.uri,
                    "name": audio_item.name,
                    "artists": artists,
                    "duration_ms": audio_item.duration_ms,
                })
            }
            PlayerEvent::Playing {
                track_id,
                position_ms,
                ..
            } => {
                json!({ "t": "playing", "uri": track_id.to_uri().unwrap_or_default(), "position_ms": position_ms })
            }
            PlayerEvent::Paused { position_ms, .. } => {
                // The decoder is paused but Core Audio keeps requesting frames.
                // Discard its bounded tail before reporting the pause to Swift.
                PCM.clear();
                json!({ "t": "paused", "position_ms": position_ms })
            }
            PlayerEvent::Stopped { .. } => {
                PCM.clear();
                json!({ "t": "stopped" })
            }
            PlayerEvent::Seeked { position_ms, .. } => {
                PCM.clear();
                json!({ "t": "seeked", "position_ms": position_ms })
            }
            PlayerEvent::PositionChanged { position_ms, .. } => {
                json!({ "t": "position", "position_ms": position_ms })
            }
            PlayerEvent::EndOfTrack { .. } => json!({ "t": "end_of_track" }),
            PlayerEvent::Unavailable { track_id, .. } => {
                json!({ "t": "unavailable", "uri": track_id.to_uri().unwrap_or_default() })
            }
            PlayerEvent::AudioKeyUnavailable { .. } => json!({ "t": "audio_key_unavailable" }),
            PlayerEvent::SessionClientChanged {
                client_name,
                client_brand_name,
                client_model_name,
                ..
            } => {
                json!({ "t": "controller", "name": client_name, "brand": client_brand_name, "model": client_model_name })
            }
            PlayerEvent::SessionDisconnected { .. } => json!({ "t": "session_disconnected" }),
            _ => continue,
        };
        host.emit(value);
    }
}

fn with_spirc(action: impl FnOnce(&Spirc) -> Result<(), librespot_core::Error>) -> i32 {
    let Some(probe) = PROBE.get() else { return -1 };
    let Some(spirc) = probe.connections.current() else {
        return -2;
    };
    match action(&spirc) {
        Ok(()) => 0,
        Err(error) => {
            probe
                .host
                .emit(json!({ "t": "error", "stage": "command", "msg": error.to_string() }));
            -3
        }
    }
}

/// Makes this phone the active Connect device and plays `context_uri`.
///
/// # Safety
/// `context_uri` is a NUL-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_load(context_uri: *const c_char) -> i32 {
    let Some(uri) = c_str(context_uri) else {
        return -4;
    };
    with_spirc(|spirc| {
        spirc.activate()?;
        spirc.load(LoadRequest::from_context_uri(
            uri,
            LoadRequestOptions {
                start_playing: true,
                seek_to: 0,
                playing_track: None,
                context_options: Some(LoadContextOptions::Options(Options::default())),
            },
        ))
    })
}

/// 0 play, 1 pause, 2 next, 3 previous, 4 take over playback from the
/// active Connect device.
#[unsafe(no_mangle)]
pub extern "C" fn probe_command(command: u32) -> i32 {
    with_spirc(|spirc| match command {
        0 => spirc.play(),
        1 => spirc.pause(),
        2 => spirc.next(),
        3 => spirc.prev(),
        4 => spirc.transfer(None),
        _ => Ok(()),
    })
}

/// Called from the render block. Returns the frames that carried audio.
///
/// # Safety
/// `left` and `right` each point to `frames` writable floats.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_render(left: *mut f32, right: *mut f32, frames: usize) -> usize {
    if left.is_null() || right.is_null() {
        return 0;
    }
    // SAFETY: guaranteed by the caller; the buffers do not overlap.
    let (left, right) = unsafe {
        (
            std::slice::from_raw_parts_mut(left, frames),
            std::slice::from_raw_parts_mut(right, frames),
        )
    };
    PCM.render(left, right)
}

#[repr(C)]
pub struct ProbeStats {
    pub decoded_frames: u64,
    pub rendered_frames: u64,
    pub silent_frames: u64,
    pub last_rms: f32,
}

#[unsafe(no_mangle)]
pub extern "C" fn probe_stats() -> ProbeStats {
    ProbeStats {
        decoded_frames: PCM.decoded_frames.load(Ordering::Relaxed),
        rendered_frames: PCM.rendered_frames.load(Ordering::Relaxed),
        silent_frames: PCM.silent_frames.load(Ordering::Relaxed),
        last_rms: f32::from_bits(PCM.last_rms.load(Ordering::Relaxed) as u32),
    }
}

/// Ends the current Connect session, for example one left stale while iOS
/// suspended the app. No playback is restored; reconnect with
/// `probe_connect`.
#[unsafe(no_mangle)]
pub extern "C" fn probe_disconnect() {
    let Some(probe) = PROBE.get() else { return };
    drop(probe.connections.cancel());
}

/// Drops buffered audio, for a pause the system forced on the app.
#[unsafe(no_mangle)]
pub extern "C" fn probe_flush() {
    PCM.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSession {
        active: Arc<AtomicU64>,
        cleaned: Arc<AtomicBool>,
    }

    impl Drop for TestSession {
        fn drop(&mut self) {
            self.active.fetch_sub(1, Ordering::SeqCst);
            self.cleaned.store(true, Ordering::SeqCst);
        }
    }

    #[tokio::test]
    async fn replacements_wait_for_cleanup_and_only_the_latest_attempt_starts() {
        let connections = Arc::new(ConnectionAttempts::default());
        let active = Arc::new(AtomicU64::new(0));
        let cleaned = Arc::new(AtomicBool::new(false));
        let (started, first) = tokio::sync::oneshot::channel();
        connections.start(&tokio::runtime::Handle::current(), {
            let active = Arc::clone(&active);
            let cleaned = Arc::clone(&cleaned);
            move |generation, _| async move {
                assert_eq!(active.fetch_add(1, Ordering::SeqCst), 0);
                let _session = TestSession { active, cleaned };
                started.send(generation).unwrap();
                std::future::pending::<()>().await;
            }
        });
        let old = first.await.unwrap();
        let superseded = Arc::new(AtomicBool::new(false));
        connections.start(&tokio::runtime::Handle::current(), {
            let superseded = Arc::clone(&superseded);
            move |_, _| async move { superseded.store(true, Ordering::SeqCst) }
        });
        let (started, latest) = tokio::sync::oneshot::channel();
        connections.start(&tokio::runtime::Handle::current(), {
            let active = Arc::clone(&active);
            let cleaned = Arc::clone(&cleaned);
            move |generation, _| async move {
                assert!(cleaned.load(Ordering::SeqCst));
                assert_eq!(active.fetch_add(1, Ordering::SeqCst), 0);
                let _session = TestSession { active, cleaned };
                started.send(generation).unwrap();
                std::future::pending::<()>().await;
            }
        });
        let latest = latest.await.unwrap();
        assert!(!connections.is_current(old));
        assert!(connections.is_current(latest));
        assert!(!superseded.load(Ordering::SeqCst));
        assert!(
            connections
                .cancel()
                .unwrap()
                .await
                .unwrap_err()
                .is_cancelled()
        );
        assert_eq!(active.load(Ordering::SeqCst), 0);
        assert!(!connections.is_current(latest));
    }

    #[tokio::test]
    async fn disconnect_cancels_authentication_and_reconnect_backoff() {
        let connections = Arc::new(ConnectionAttempts::default());
        for backoff in [false, true] {
            let continued = Arc::new(AtomicBool::new(false));
            let (started, ready) = tokio::sync::oneshot::channel();
            connections.start(&tokio::runtime::Handle::current(), {
                let continued = Arc::clone(&continued);
                move |generation, _| async move {
                    started.send(generation).unwrap();
                    if backoff {
                        tokio::time::sleep(Duration::from_secs(60)).await;
                    } else {
                        std::future::pending::<()>().await;
                    }
                    continued.store(true, Ordering::SeqCst);
                }
            });
            let generation = ready.await.unwrap();
            assert!(
                connections
                    .cancel()
                    .unwrap()
                    .await
                    .unwrap_err()
                    .is_cancelled()
            );
            assert!(!continued.load(Ordering::SeqCst));
            assert!(!connections.is_current(generation));
            assert!(connections.current().is_none());
        }
    }

    #[test]
    fn cancellation_releases_a_decoder_waiting_for_queue_space() {
        let pcm = Arc::new(Pcm::new());
        pcm.push(&vec![0.25; QUEUE_CAP]);
        let active = Arc::new(AtomicBool::new(true));
        let (started, ready) = std::sync::mpsc::channel();
        let (finished, done) = std::sync::mpsc::channel();
        let decoder = std::thread::spawn({
            let pcm = Arc::clone(&pcm);
            let active = Arc::clone(&active);
            move || {
                started.send(()).unwrap();
                pcm.push_active(&[0.5, -0.5], &active);
                finished.send(()).unwrap();
            }
        });
        ready.recv_timeout(Duration::from_secs(2)).unwrap();
        active.store(false, Ordering::Release);
        pcm.space.notify_all();
        done.recv_timeout(Duration::from_secs(2)).unwrap();
        decoder.join().unwrap();
        assert_eq!(pcm.queue.lock().unwrap().len(), QUEUE_CAP);
        assert_eq!(
            pcm.decoded_frames.load(Ordering::Relaxed),
            (QUEUE_CAP / CHANNELS) as u64
        );
    }

    #[tokio::test]
    async fn a_player_pause_flushes_the_buffer_before_reporting_paused() {
        extern "C" fn event(line: *const c_char, ctx: *mut c_void) {
            // SAFETY: this test owns both pointers until forwarding completes.
            unsafe { *ctx.cast::<String>() = CStr::from_ptr(line).to_str().unwrap().into() };
        }
        extern "C" fn credential(_: *const u8, _: usize, _: *mut c_void) {}
        let mut reported = String::new();
        let host = Arc::new(Host {
            on_event: event,
            on_credentials: credential,
            ctx: (&mut reported as *mut String) as usize,
        });
        PCM.push(&vec![0.25; QUEUE_CAP]);
        let pause = PlayerEvent::Paused {
            play_request_id: 1,
            track_id: librespot_core::SpotifyUri::from_uri("spotify:track:0DiWol3AO6WpXZgp0goxAV")
                .unwrap(),
            position_ms: 1234,
        };
        let stale = Arc::new(ConnectionAttempts::default());
        drop(stale.cancel());
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        sender.send(pause.clone()).unwrap();
        drop(sender);
        forward_events(receiver, Arc::clone(&host), stale, 0).await;
        assert!(
            reported.is_empty(),
            "obsolete sessions cannot report a pause"
        );
        assert_eq!(PCM.queue.lock().unwrap().len(), QUEUE_CAP);
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        sender.send(pause).unwrap();
        drop(sender);
        forward_events(receiver, host, Arc::default(), 0).await;
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&reported).unwrap()["t"],
            "paused"
        );
        let (mut left, mut right) = ([1.0; 64], [1.0; 64]);
        assert_eq!(PCM.render(&mut left, &mut right), 0);
        assert_eq!(left, [0.0; 64]);
        assert_eq!(right, [0.0; 64]);
        PCM.push(&[0.5, -0.5]);
        assert_eq!(PCM.render(&mut left, &mut right), 1);
        assert_eq!((left[0], right[0]), (0.5, -0.5));
    }

    #[test]
    fn render_drains_queue_in_order_pads_silence_and_measures_rms() {
        let pcm = Pcm::new();
        pcm.push(&[0.5, -0.5, 0.5, -0.5]);
        let (mut left, mut right) = ([9.0f32; 3], [9.0f32; 3]);
        assert_eq!(pcm.render(&mut left, &mut right), 2);
        assert_eq!(left, [0.5, 0.5, 0.0]);
        assert_eq!(right, [-0.5, -0.5, 0.0]);
        assert_eq!(pcm.rendered_frames.load(Ordering::Relaxed), 2);
        assert_eq!(pcm.silent_frames.load(Ordering::Relaxed), 1);
        assert_eq!(pcm.decoded_frames.load(Ordering::Relaxed), 2);
        assert_eq!(
            f32::from_bits(pcm.last_rms.load(Ordering::Relaxed) as u32),
            0.5
        );
    }

    #[test]
    fn credentials_round_trip_through_the_keychain_blob() {
        let credentials = Credentials::with_access_token("dummy-token");
        let blob = serde_json::to_vec(&credentials).unwrap();
        assert_eq!(
            serde_json::from_slice::<Credentials>(&blob).unwrap(),
            credentials
        );
    }
}

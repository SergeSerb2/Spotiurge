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
use std::sync::atomic::{AtomicU64, Ordering};
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
        let mut queue = self.queue.lock().unwrap_or_else(|p| p.into_inner());
        while queue.len() + samples.len() > QUEUE_CAP.max(samples.len()) {
            queue = self
                .space
                .wait_timeout(queue, Duration::from_millis(250))
                .unwrap_or_else(|p| p.into_inner())
                .0;
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

struct QueueSink;

impl Sink for QueueSink {
    fn write(&mut self, packet: AudioPacket, converter: &mut Converter) -> SinkResult<()> {
        if let AudioPacket::Samples(samples) = packet {
            PCM.push(&converter.f64_to_f32(&samples));
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
    spirc: Mutex<Option<Arc<Spirc>>>,
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
        spirc: Mutex::new(None),
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
    probe.runtime.spawn(async move {
        if let Err(error) = connect(probe, credentials).await {
            host.emit(json!({ "t": "error", "stage": "connect", "msg": error }));
        }
    });
    0
}

/// Runs Connect sessions until one ends without playback to resume. When
/// Spotify drops the connection mid-playback (seen after about 13 minutes on
/// the phone), reconnect with the reusable credential and restore the exact
/// queue, as the desktop engine does.
async fn connect(probe: &'static Probe, mut credentials: Credentials) -> Result<(), String> {
    const RETRY_DELAYS_S: [u64; 6] = [1, 2, 4, 8, 16, 32];
    let mut pending: Option<Arc<PlaybackSnapshot>> = None;
    let mut failures = 0;
    loop {
        match run_session(probe, credentials.clone(), pending.clone()).await {
            Ok((None, _)) => {
                // Silent when a newer session already took over.
                if probe
                    .spirc
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .is_none()
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
        || Box::new(QueueSink),
    );
    tokio::spawn(forward_events(
        player.get_player_event_channel(),
        Arc::clone(&probe.host),
    ));
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
    if let Some(stored) = session.cache().and_then(|cache| cache.credentials())
        && let Ok(blob) = serde_json::to_vec(&stored)
    {
        (probe.host.on_credentials)(blob.as_ptr(), blob.len(), probe.host.ctx as *mut c_void);
    }
    let (spirc, task) = connected.map_err(|e| e.to_string())?;
    let spirc = Arc::new(spirc);
    let restored = match restore {
        Some(snapshot) => spirc.restore_playback(snapshot).is_ok(),
        None => false,
    };
    *probe.spirc.lock().unwrap_or_else(|p| p.into_inner()) = Some(Arc::clone(&spirc));
    probe
        .host
        .emit(json!({ "t": "connected", "restored": restored }));
    task.await;
    {
        // A newer session may already have replaced this one.
        let mut current = probe.spirc.lock().unwrap_or_else(|p| p.into_inner());
        if current.as_ref().is_some_and(|c| Arc::ptr_eq(c, &spirc)) {
            current.take();
        }
    }
    let stored = session
        .cache()
        .and_then(|cache| cache.credentials())
        .unwrap_or(credentials);
    Ok((spirc.disconnected_playback(), stored))
}

async fn forward_events(
    mut events: tokio::sync::mpsc::UnboundedReceiver<PlayerEvent>,
    host: Arc<Host>,
) {
    while let Some(event) = events.recv().await {
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
    let Some(spirc) = probe
        .spirc
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .clone()
    else {
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
    if let Some(spirc) = probe.spirc.lock().unwrap_or_else(|p| p.into_inner()).take() {
        let _ = spirc.shutdown();
    }
}

/// Drops buffered audio, for a pause the system forced on the app.
#[unsafe(no_mangle)]
pub extern "C" fn probe_flush() {
    PCM.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

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

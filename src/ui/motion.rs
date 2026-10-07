//! Finite, interruptible transitions.
//!
//! Every helper here moves toward its target over a fixed time with an
//! exponential ease-out, asks for a repaint only while it is still moving,
//! and stops asking the moment it settles. Retargeting mid-flight starts
//! from wherever the value currently is, so a quick second click never
//! jumps. With reduced motion every helper returns its target at once.

use egui::{Color32, Context, Id};

/// Hover and press feedback.
pub const FEEDBACK: f32 = 0.12;
/// Selection moving between rows or chips, and fills changing state.
pub const STATE: f32 = 0.22;
/// A page settling in after navigation.
pub const PAGE: f32 = 0.22;
/// The ambient light crossing over to a new cover.
pub const AMBIENT: f32 = 0.6;
/// How far, in points, a page rises while it settles in.
pub const PAGE_RISE: f32 = 6.0;

const REDUCED_ID: &str = "spotiurge-reduced-motion";

/// Whether motion is reduced for this frame, by the setting or the system.
pub fn reduced(ctx: &Context) -> bool {
    ctx.data(|data| data.get_temp::<bool>(Id::new(REDUCED_ID)))
        .unwrap_or(false)
}

/// Records the frame's motion preference and keeps egui's own animations
/// (popups fading in, scroll smoothing) in step with it.
pub fn set_reduced(ctx: &Context, reduced: bool) {
    let id = Id::new(REDUCED_ID);
    if ctx.data(|data| data.get_temp::<bool>(id)) == Some(reduced) {
        return;
    }
    ctx.data_mut(|data| data.insert_temp(id, reduced));
    ctx.all_styles_mut(|style| apply_to_style(style, reduced));
}

/// Whether the system asks apps to reduce motion. macOS answers through
/// its accessibility setting, read at most every two seconds; elsewhere the
/// app's own setting decides.
pub fn system_prefers_reduced(ctx: &Context) -> bool {
    #[cfg(target_os = "macos")]
    {
        let id = Id::new("spotiurge-system-reduced-motion");
        let now = ctx.input(|input| input.time);
        if let Some((read_at, reduced)) = ctx.data(|data| data.get_temp::<(f64, bool)>(id))
            && now - read_at < 2.0
        {
            return reduced;
        }
        let reduced = macos_reduce_motion();
        ctx.data_mut(|data| data.insert_temp(id, (now, reduced)));
        reduced
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = ctx;
        false
    }
}

#[cfg(target_os = "macos")]
fn macos_reduce_motion() -> bool {
    use objc2::runtime::{AnyClass, AnyObject};
    let Some(class) = AnyClass::get(c"NSWorkspace") else {
        return false;
    };
    // SAFETY: `sharedWorkspace` returns the process's shared workspace,
    // and the property is a plain BOOL getter available since macOS 10.12.
    unsafe {
        let workspace: *mut AnyObject = objc2::msg_send![class, sharedWorkspace];
        if workspace.is_null() {
            return false;
        }
        objc2::msg_send![workspace, accessibilityDisplayShouldReduceMotion]
    }
}

/// egui's animation settings for the motion preference.
pub fn apply_to_style(style: &mut egui::Style, reduced: bool) {
    if reduced {
        style.animation_time = 0.0;
        style.scroll_animation = egui::style::ScrollAnimation::none();
    } else {
        style.animation_time = FEEDBACK;
        style.scroll_animation = egui::style::ScrollAnimation::default();
    }
}

/// Exponential ease-out, exactly 0 at the start and 1 at the end.
pub fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t >= 1.0 {
        1.0
    } else {
        (1.0 - 2f32.powf(-10.0 * t)) / (1.0 - 2f32.powf(-10.0))
    }
}

#[derive(Clone, Copy)]
struct Tween {
    from: f32,
    to: f32,
    start: f64,
}

impl Tween {
    fn at(&self, now: f64, duration: f32) -> (f32, bool) {
        let t = if duration <= 0.0 {
            1.0
        } else {
            ((now - self.start) as f32 / duration).clamp(0.0, 1.0)
        };
        let value = self.from + (self.to - self.from) * ease_out(t);
        (if t >= 1.0 { self.to } else { value }, t < 1.0)
    }
}

/// `target`, reached over `duration` seconds from wherever the value was.
/// The first call shows the target directly: nothing animates into view.
pub fn value(ctx: &Context, id: Id, target: f32, duration: f32) -> f32 {
    let (value, moving) = value_step(ctx, id, target, duration);
    if moving {
        ctx.request_repaint();
    }
    value
}

/// The value this frame, and whether it is still moving.
fn value_step(ctx: &Context, id: Id, target: f32, duration: f32) -> (f32, bool) {
    let now = ctx.input(|input| input.time);
    let reduced = reduced(ctx);
    ctx.data_mut(|data| {
        let tween = data.get_temp_mut_or_insert_with(id, || Tween {
            from: target,
            to: target,
            start: f64::NEG_INFINITY,
        });
        if reduced {
            *tween = Tween {
                from: target,
                to: target,
                start: f64::NEG_INFINITY,
            };
            return (target, false);
        }
        if tween.to != target {
            let (current, _) = tween.at(now, duration);
            *tween = Tween {
                from: current,
                to: target,
                start: now,
            };
        }
        tween.at(now, duration)
    })
}

/// Puts a [`value`] at `target` at once, ending any glide: for a value
/// whose subject moved with the layout rather than changed.
pub fn snap(ctx: &Context, id: Id, target: f32) {
    ctx.data_mut(|data| {
        data.insert_temp(
            id,
            Tween {
                from: target,
                to: target,
                start: f64::NEG_INFINITY,
            },
        );
    });
}

/// 0 when `on` is false and 1 when true, eased between them.
pub fn toggle(ctx: &Context, id: Id, on: bool, duration: f32) -> f32 {
    value(ctx, id, if on { 1.0 } else { 0.0 }, duration)
}

/// A colour crossing over to `target`, channel by channel.
pub fn color(ctx: &Context, id: Id, target: Color32, duration: f32) -> Color32 {
    let [r, g, b, a] = target.to_srgba_unmultiplied();
    let channel = |axis: &'static str, value: u8| {
        self::value(ctx, id.with(axis), f32::from(value), duration)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    let eased = [
        channel("r", r),
        channel("g", g),
        channel("b", b),
        channel("a", a),
    ];
    if eased == [r, g, b, a] {
        target
    } else {
        Color32::from_rgba_unmultiplied(eased[0], eased[1], eased[2], eased[3])
    }
}

/// How far, from 0 to 1, a transition has come since `key` last changed:
/// the entrance of whatever `key` names. The first key seen counts as
/// already arrived.
pub fn entrance(ctx: &Context, id: Id, key: u64, duration: f32) -> f32 {
    let (progress, moving) = entrance_step(ctx, id, key, duration);
    if moving {
        ctx.request_repaint();
    }
    progress
}

fn entrance_step(ctx: &Context, id: Id, key: u64, duration: f32) -> (f32, bool) {
    let now = ctx.input(|input| input.time);
    let reduced = reduced(ctx);
    let start = ctx.data_mut(|data| {
        let entry = data.get_temp_mut_or_insert_with(id, || (key, f64::NEG_INFINITY));
        if entry.0 != key {
            *entry = (key, now);
        }
        entry.1
    });
    if reduced || duration <= 0.0 {
        return (1.0, false);
    }
    let t = ((now - start) as f32 / duration).clamp(0.0, 1.0);
    (ease_out(t), t < 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs one frame at `time` seconds and returns what `f` read.
    fn frame<R>(ctx: &Context, time: f64, f: impl FnOnce(&Context) -> R) -> R {
        let mut out = None;
        let mut f = Some(f);
        let mut output = ctx.run_ui(
            egui::RawInput {
                time: Some(time),
                ..Default::default()
            },
            |ui| {
                if let Some(f) = f.take() {
                    out = Some(f(ui.ctx()));
                }
            },
        );
        output.textures_delta.clear();
        out.unwrap()
    }

    #[test]
    fn easing_starts_at_zero_and_lands_exactly() {
        assert_eq!(ease_out(0.0), 0.0);
        assert_eq!(ease_out(1.0), 1.0);
        assert!(ease_out(0.3) > 0.7, "decelerates: most of the way early");
        let mut last = 0.0;
        for step in 1..=20 {
            let eased = ease_out(step as f32 / 20.0);
            assert!(eased >= last);
            last = eased;
        }
    }

    /// A value glides to its new target, then reports it has stopped, so
    /// nothing asks for another frame.
    #[test]
    fn a_value_settles_and_then_goes_quiet() {
        let ctx = Context::default();
        let id = Id::new("glide");
        let (first, _) = frame(&ctx, 0.0, |ctx| value_step(ctx, id, 10.0, STATE));
        assert_eq!(first, 10.0, "nothing animates into view");

        let (start, moving) = frame(&ctx, 1.0, |ctx| value_step(ctx, id, 50.0, STATE));
        assert_eq!(start, 10.0);
        assert!(moving, "a moving value asks for the next frame");

        let (middle, moving) = frame(&ctx, 1.0 + 0.1, |ctx| value_step(ctx, id, 50.0, STATE));
        assert!(10.0 < middle && middle < 50.0, "{middle}");
        assert!(moving);

        let (end, moving) = frame(&ctx, 1.0 + f64::from(STATE), |ctx| {
            value_step(ctx, id, 50.0, STATE)
        });
        assert_eq!(end, 50.0);
        assert!(!moving, "a settled value leaves the app idle");
        let (_, moving) = frame(&ctx, 5.0, |ctx| value_step(ctx, id, 50.0, STATE));
        assert!(!moving);
    }

    /// Retargeting mid-flight continues from where the value is, not from
    /// the old start or the old end.
    #[test]
    fn a_new_target_mid_flight_starts_from_the_current_value() {
        let ctx = Context::default();
        let id = Id::new("retarget");
        frame(&ctx, 0.0, |ctx| value_step(ctx, id, 0.0, STATE));
        frame(&ctx, 1.0, |ctx| value_step(ctx, id, 100.0, STATE));
        let (middle, _) = frame(&ctx, 1.05, |ctx| value_step(ctx, id, 100.0, STATE));
        let (turned, _) = frame(&ctx, 1.05, |ctx| value_step(ctx, id, 0.0, STATE));
        assert_eq!(turned, middle, "no jump when the target changes");
        let (back, moving) = frame(&ctx, 1.05 + f64::from(STATE), |ctx| {
            value_step(ctx, id, 0.0, STATE)
        });
        assert_eq!((back, moving), (0.0, false));
    }

    /// Reduced motion shows every state at once and reports nothing moving.
    #[test]
    fn reduced_motion_is_immediate_and_idle() {
        let ctx = Context::default();
        let id = Id::new("reduced");
        frame(&ctx, 0.0, |ctx| {
            set_reduced(ctx, true);
            value_step(ctx, id, 0.0, STATE)
        });
        let (jumped, moving) = frame(&ctx, 1.0, |ctx| value_step(ctx, id, 80.0, STATE));
        assert_eq!((jumped, moving), (80.0, false));
        let (entered, moving) = frame(&ctx, 1.0, |ctx| {
            entrance_step(ctx, id.with("page"), 7, PAGE)
        });
        assert_eq!((entered, moving), (1.0, false));
        let (_, moving) = frame(&ctx, 1.0, |ctx| {
            entrance_step(ctx, id.with("page"), 8, PAGE)
        });
        assert!(!moving);
        let style = ctx.global_style();
        assert_eq!(style.animation_time, 0.0, "egui's popups stop fading too");

        frame(&ctx, 2.0, |ctx| set_reduced(ctx, false));
        assert_eq!(ctx.global_style().animation_time, FEEDBACK);
    }

    /// A page enters once per change of key; staying put is not re-entering.
    #[test]
    fn an_entrance_runs_once_per_change() {
        let ctx = Context::default();
        let id = Id::new("page");
        let (first, moving) = frame(&ctx, 0.0, |ctx| entrance_step(ctx, id, 1, PAGE));
        assert_eq!(
            (first, moving),
            (1.0, false),
            "the first page is already there"
        );
        let (start, moving) = frame(&ctx, 1.0, |ctx| entrance_step(ctx, id, 2, PAGE));
        assert_eq!(start, 0.0);
        assert!(moving);
        let (done, moving) = frame(&ctx, 1.0 + f64::from(PAGE), |ctx| {
            entrance_step(ctx, id, 2, PAGE)
        });
        assert_eq!((done, moving), (1.0, false));
    }

    #[test]
    fn colours_cross_over_and_land_on_the_target() {
        let ctx = Context::default();
        let id = Id::new("tint");
        let red = Color32::from_rgb(200, 40, 40);
        let blue = Color32::from_rgb(40, 40, 200);
        frame(&ctx, 0.0, |ctx| color(ctx, id, red, AMBIENT));
        let between = frame(&ctx, 1.1, |ctx| color(ctx, id, blue, AMBIENT));
        assert_eq!(between, red, "the crossing starts from the old colour");
        let between = frame(&ctx, 1.2, |ctx| color(ctx, id, blue, AMBIENT));
        assert!(between.r() < 200 && between.b() > 40);
        let end = frame(&ctx, 1.1 + f64::from(AMBIENT), |ctx| {
            color(ctx, id, blue, AMBIENT)
        });
        assert_eq!(end, blue);
    }
}

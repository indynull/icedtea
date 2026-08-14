//! Material Design 3 duration and easing tokens.
//!
//! Desktop chrome uses the short and medium steps. Reduced motion
//! collapses every duration to 0 ms so progress snaps to the target.

use iced::time::Duration;

/// Named M3 duration step (milliseconds at full motion).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DurationStep {
    Short1,
    Short2,
    Short3,
    Short4,
    Medium1,
    Medium2,
    Medium3,
    Medium4,
    Long1,
    Long2,
    Long3,
    Long4,
}

impl DurationStep {
    /// Full-motion length in milliseconds.
    pub fn millis(self) -> u64 {
        match self {
            Self::Short1 => 50,
            Self::Short2 => 100,
            Self::Short3 => 150,
            Self::Short4 => 200,
            Self::Medium1 => 250,
            Self::Medium2 => 300,
            Self::Medium3 => 350,
            Self::Medium4 => 400,
            Self::Long1 => 450,
            Self::Long2 => 500,
            Self::Long3 => 550,
            Self::Long4 => 600,
        }
    }

    /// Duration, or zero when reduced motion is on.
    pub fn duration(self, reduced: bool) -> Duration {
        if reduced {
            Duration::ZERO
        } else {
            Duration::from_millis(self.millis())
        }
    }

    pub const ALL: [DurationStep; 12] = [
        Self::Short1,
        Self::Short2,
        Self::Short3,
        Self::Short4,
        Self::Medium1,
        Self::Medium2,
        Self::Medium3,
        Self::Medium4,
        Self::Long1,
        Self::Long2,
        Self::Long3,
        Self::Long4,
    ];
}

/// Named M3 easing set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ease {
    /// Most overlay enter (expressive, ends at rest).
    Emphasized,
    EmphasizedDecelerate,
    EmphasizedAccelerate,
    Standard,
    StandardDecelerate,
    StandardAccelerate,
}

impl Ease {
    /// Cubic Bézier control points (x1, y1, x2, y2).
    pub fn controls(self) -> (f32, f32, f32, f32) {
        match self {
            Self::Emphasized => (0.2, 0.0, 0.0, 1.0),
            Self::EmphasizedDecelerate => (0.05, 0.7, 0.1, 1.0),
            Self::EmphasizedAccelerate => (0.3, 0.0, 0.8, 0.15),
            Self::Standard => (0.2, 0.0, 0.0, 1.0),
            Self::StandardDecelerate => (0.0, 0.0, 0.0, 1.0),
            Self::StandardAccelerate => (0.3, 0.0, 1.0, 1.0),
        }
    }

    /// Sample this curve at `t` in 0..=1.
    pub fn sample(self, t: f32) -> f32 {
        let (x1, y1, x2, y2) = self.controls();
        cubic_bezier(x1, y1, x2, y2, t.clamp(0.0, 1.0))
    }

    /// lilt easing for `iced::Animation::easing`.
    pub fn lilt(self) -> iced::animation::Easing {
        match self {
            Self::Emphasized | Self::EmphasizedDecelerate | Self::StandardDecelerate => {
                iced::animation::Easing::EaseOutCubic
            }
            Self::EmphasizedAccelerate | Self::StandardAccelerate => {
                iced::animation::Easing::EaseInCubic
            }
            Self::Standard => iced::animation::Easing::EaseInOutCubic,
        }
    }

    pub const ALL: [Ease; 6] = [
        Self::Emphasized,
        Self::EmphasizedDecelerate,
        Self::EmphasizedAccelerate,
        Self::Standard,
        Self::StandardDecelerate,
        Self::StandardAccelerate,
    ];
}

/// Overlay enter / exit (fade + short slide).
pub const OVERLAY: DurationStep = DurationStep::Short4;
/// Side sheet slide.
pub const SHEET: DurationStep = DurationStep::Short4;
/// Toast enter and the last slice of TTL.
pub const TOAST: DurationStep = DurationStep::Short3;
/// Expander / accordion height.
pub const EXPAND: DurationStep = DurationStep::Medium1;
/// Determinate progress value change.
pub const PROGRESS: DurationStep = DurationStep::Medium2;

/// Slide distance at progress 0 (desktop dp).
pub const OVERLAY_SLIDE: f32 = 12.0;
/// Side sheet slide at progress 0.
pub const SHEET_SLIDE: f32 = 16.0;
/// Toast slide at progress 0.
pub const TOAST_SLIDE: f32 = 8.0;

fn sample_bezier_x(x1: f32, x2: f32, t: f32) -> f32 {
    let u = 1.0 - t;
    3.0 * u * u * t * x1 + 3.0 * u * t * t * x2 + t * t * t
}

fn sample_bezier_y(y1: f32, y2: f32, t: f32) -> f32 {
    let u = 1.0 - t;
    3.0 * u * u * t * y1 + 3.0 * u * t * t * y2 + t * t * t
}

fn sample_bezier_dx(x1: f32, x2: f32, t: f32) -> f32 {
    let u = 1.0 - t;
    3.0 * u * u * x1 + 6.0 * u * t * (x2 - x1) + 3.0 * t * t * (1.0 - x2)
}

/// Unit cubic Bézier Y at X=`x` (P0=(0,0), P3=(1,1)).
pub fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let mut t = x;
    for _ in 0..8 {
        let xt = sample_bezier_x(x1, x2, t);
        let d = sample_bezier_dx(x1, x2, t);
        if d.abs() < 1e-6 {
            break;
        }
        t = (t - (xt - x) / d).clamp(0.0, 1.0);
    }
    sample_bezier_y(y1, y2, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_steps_match_m3_millis() {
        assert_eq!(DurationStep::Short1.millis(), 50);
        assert_eq!(DurationStep::Short4.millis(), 200);
        assert_eq!(DurationStep::Medium1.millis(), 250);
        assert_eq!(DurationStep::Long4.millis(), 600);
        assert_eq!(OVERLAY.millis(), 200);
        assert_eq!(TOAST.millis(), 150);
        assert_eq!(EXPAND.millis(), 250);
        assert_eq!(PROGRESS.millis(), 300);
        for step in DurationStep::ALL {
            assert!(step.millis() > 0);
            assert_eq!(step.duration(true), Duration::ZERO);
            assert_eq!(step.duration(false), Duration::from_millis(step.millis()));
        }
    }

    #[test]
    fn ease_samples_endpoints_and_mid() {
        for ease in Ease::ALL {
            assert!((ease.sample(0.0) - 0.0).abs() < 1e-4);
            assert!((ease.sample(1.0) - 1.0).abs() < 1e-4);
            let mid = ease.sample(0.5);
            assert!(mid > 0.0 && mid < 1.0, "{ease:?} mid={mid}");
            let _ = ease.lilt();
        }
        // Decelerate spends more time near the end: mid is already high.
        assert!(Ease::EmphasizedDecelerate.sample(0.5) > 0.7);
        // Accelerate stays low in the first half.
        assert!(Ease::EmphasizedAccelerate.sample(0.5) < 0.4);
    }

    #[test]
    fn cubic_bezier_identity_is_linear() {
        for x in [0.0, 0.25, 0.5, 0.75, 1.0] {
            assert!((cubic_bezier(0.0, 0.0, 1.0, 1.0, x) - x).abs() < 0.02);
        }
        // Flat tangent near the origin takes the Newton early-exit.
        let _ = cubic_bezier(0.0, 0.0, 0.0, 0.0, 0.0001);
    }
}

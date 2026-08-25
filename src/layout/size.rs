//! Size policy and space distribution (Qt stretch / min / max).

/// How a child wants space on one axis.
///
/// ```
/// use icedtea::layout::SizePolicy;
/// let sizes = icedtea::layout::distribute(100.0, &[
///     SizePolicy::fixed(20.0),
///     SizePolicy::expand(1.0),
/// ]);
/// assert_eq!(sizes[0], 20.0);
/// assert_eq!(sizes[1], 80.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SizePolicy {
    pub min: f32,
    pub preferred: f32,
    pub max: f32,
    pub stretch: f32,
}

impl SizePolicy {
    pub fn fixed(px: f32) -> Self {
        Self {
            min: px,
            preferred: px,
            max: px,
            stretch: 0.0,
        }
    }

    pub fn expand(stretch: f32) -> Self {
        Self {
            min: 0.0,
            preferred: 0.0,
            max: f32::INFINITY,
            stretch: stretch.max(0.0),
        }
    }

    pub fn between(min: f32, preferred: f32, max: f32, stretch: f32) -> Self {
        let min = min.max(0.0);
        let preferred = preferred.max(min);
        let max = max.max(preferred);
        Self {
            min,
            preferred,
            max,
            stretch: stretch.max(0.0),
        }
    }
}

/// Assign `total` from preferred sizes: shrink toward min, then grow
/// only children with stretch. Leftover after max caps is left unused
/// so the box can pack it.
///
/// ```
/// use icedtea::layout::{allocate, SizePolicy};
/// let sizes = allocate(100.0, &[
///     SizePolicy::fixed(20.0),
///     SizePolicy::expand(1.0),
/// ]);
/// assert!((sizes[0] - 20.0).abs() < 0.01);
/// assert!((sizes[1] - 80.0).abs() < 0.01);
/// ```
pub fn allocate(total: f32, policies: &[SizePolicy]) -> Vec<f32> {
    if policies.is_empty() {
        return Vec::new();
    }
    let total = total.max(0.0);
    let mut sizes: Vec<f32> = policies
        .iter()
        .map(|p| p.preferred.clamp(p.min, p.max))
        .collect();
    let used: f32 = sizes.iter().sum();
    if used > total + 0.01 {
        let room: f32 = sizes
            .iter()
            .zip(policies)
            .map(|(s, p)| (*s - p.min).max(0.0))
            .sum();
        if room <= 0.01 {
            let scale = total / used;
            for s in &mut sizes {
                *s *= scale;
            }
            return sizes;
        }
        let overflow = used - total;
        for (i, p) in policies.iter().enumerate() {
            let share = (sizes[i] - p.min).max(0.0) / room * overflow;
            sizes[i] = (sizes[i] - share).max(p.min);
        }
        return sizes;
    }
    let leftover = total - used;
    let stretch_sum: f32 = policies.iter().map(|p| p.stretch).sum();
    if leftover > 0.0 && stretch_sum > 0.0 {
        for (i, p) in policies.iter().enumerate() {
            if p.stretch <= 0.0 {
                continue;
            }
            let extra = leftover * (p.stretch / stretch_sum);
            sizes[i] = (sizes[i] + extra).min(p.max);
        }
    }
    let used: f32 = sizes.iter().sum();
    if used < total - 0.01 {
        let mut best: Option<usize> = None;
        let mut best_stretch = 0.0;
        for (i, p) in policies.iter().enumerate() {
            if sizes[i] < p.max - 0.01 && p.stretch > best_stretch {
                best_stretch = p.stretch;
                best = Some(i);
            }
        }
        if let Some(i) = best {
            sizes[i] = (sizes[i] + (total - used)).min(policies[i].max);
        }
    }
    sizes
}

/// Distribute `total` across policies: mins first, then stretch leftover.
pub fn distribute(total: f32, policies: &[SizePolicy]) -> Vec<f32> {
    if policies.is_empty() {
        return Vec::new();
    }
    let total = total.max(0.0);
    let mut sizes: Vec<f32> = policies.iter().map(|p| p.min).collect();
    let used: f32 = sizes.iter().sum();
    if used >= total {
        if used == 0.0 {
            return sizes;
        }
        let scale = total / used;
        for s in &mut sizes {
            *s *= scale;
        }
        return sizes;
    }
    let leftover = total - used;
    let stretch_sum: f32 = policies.iter().map(|p| p.stretch).sum();
    if stretch_sum > 0.0 {
        for (i, p) in policies.iter().enumerate() {
            let extra = leftover * (p.stretch / stretch_sum);
            sizes[i] = (sizes[i] + extra).min(p.max);
        }
    } else {
        let n = policies.len() as f32;
        for (i, p) in policies.iter().enumerate() {
            let extra = leftover / n;
            sizes[i] = (sizes[i] + extra).min(p.max);
        }
    }
    let used: f32 = sizes.iter().sum();
    if used < total - 0.01 {
        let mut best: Option<usize> = None;
        let mut best_stretch = f32::NEG_INFINITY;
        for (i, p) in policies.iter().enumerate() {
            if sizes[i] < p.max && p.stretch >= best_stretch {
                best_stretch = p.stretch;
                best = Some(i);
            }
        }
        if let Some(i) = best {
            sizes[i] = (sizes[i] + (total - used)).min(policies[i].max);
        }
    }
    sizes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distribute_fixed_and_stretch() {
        assert!(distribute(100.0, &[]).is_empty());
        let s = distribute(100.0, &[SizePolicy::fixed(20.0), SizePolicy::expand(1.0)]);
        assert!((s[0] - 20.0).abs() < 0.01);
        assert!((s[1] - 80.0).abs() < 0.01);
        let even = distribute(
            90.0,
            &[
                SizePolicy::between(10.0, 10.0, 1000.0, 0.0),
                SizePolicy::between(10.0, 10.0, 1000.0, 0.0),
            ],
        );
        assert!((even[0] - 45.0).abs() < 0.01);
        let stuck = distribute(90.0, &[SizePolicy::fixed(10.0), SizePolicy::fixed(10.0)]);
        assert!((stuck[0] - 10.0).abs() < 0.01);
        let squeezed = distribute(10.0, &[SizePolicy::fixed(20.0), SizePolicy::fixed(20.0)]);
        assert!((squeezed[0] - 5.0).abs() < 0.01);
        let zero = distribute(0.0, &[SizePolicy::expand(1.0)]);
        assert_eq!(zero[0], 0.0);
        let capped = distribute(
            200.0,
            &[
                SizePolicy::between(10.0, 10.0, 30.0, 1.0),
                SizePolicy::expand(1.0),
            ],
        );
        assert!(capped[0] <= 30.0);
        assert!(capped[1] > 100.0);
        let two_stretch = distribute(100.0, &[SizePolicy::expand(1.0), SizePolicy::expand(3.0)]);
        assert!(two_stretch[1] > two_stretch[0]);
        let leftover = distribute(
            100.0,
            &[
                SizePolicy::between(0.0, 0.0, 20.0, 1.0),
                SizePolicy::between(0.0, 0.0, 1000.0, 0.0),
            ],
        );
        assert!(leftover[0] <= 20.0 + 0.01);
        assert!(leftover[1] > 50.0);
        let _ = SizePolicy::between(10.0, 5.0, 8.0, -1.0);
        assert_eq!(distribute(0.0, &[SizePolicy::fixed(0.0)]), vec![0.0]);
        let rem = distribute(
            80.0,
            &[
                SizePolicy::between(10.0, 10.0, 15.0, 1.0),
                SizePolicy::between(10.0, 10.0, 200.0, 1.0),
            ],
        );
        assert!(rem[0] <= 15.01);
        assert!(rem[1] >= 60.0);
    }

    #[test]
    fn allocate_starts_at_preferred_and_leaves_hug_leftover() {
        assert!(allocate(100.0, &[]).is_empty());
        let hug = allocate(
            100.0,
            &[
                SizePolicy::between(10.0, 20.0, 20.0, 0.0),
                SizePolicy::between(10.0, 20.0, 20.0, 0.0),
            ],
        );
        assert!((hug[0] - 20.0).abs() < 0.01);
        assert!((hug[1] - 20.0).abs() < 0.01);
        let share = allocate(
            100.0,
            &[
                SizePolicy::between(10.0, 20.0, 20.0, 0.0),
                SizePolicy::expand(1.0),
            ],
        );
        assert!((share[0] - 20.0).abs() < 0.01);
        assert!((share[1] - 80.0).abs() < 0.01);
        let squeezed = allocate(
            20.0,
            &[
                SizePolicy::between(10.0, 30.0, 40.0, 0.0),
                SizePolicy::between(10.0, 30.0, 40.0, 0.0),
            ],
        );
        assert!((squeezed[0] - 10.0).abs() < 0.01);
        assert!((squeezed[1] - 10.0).abs() < 0.01);
        let zero = allocate(0.0, &[SizePolicy::fixed(0.0), SizePolicy::fixed(0.0)]);
        assert_eq!(zero, vec![0.0, 0.0]);
        let scale = allocate(10.0, &[SizePolicy::fixed(20.0), SizePolicy::fixed(20.0)]);
        assert!((scale[0] - 5.0).abs() < 0.01);
        let rem = allocate(
            100.0,
            &[
                SizePolicy::between(0.0, 0.0, 20.0, 1.0),
                SizePolicy::between(0.0, 0.0, 1000.0, 1.0),
            ],
        );
        assert!(rem[0] <= 20.01);
        assert!(rem[1] >= 79.0);
    }
}

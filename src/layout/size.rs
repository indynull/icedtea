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
        if let Some((i, _)) = policies
            .iter()
            .enumerate()
            .filter(|(i, p)| sizes[*i] < p.max)
            .max_by(|a, b| a.1.stretch.partial_cmp(&b.1.stretch).unwrap())
        {
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
    }
}

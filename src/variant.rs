//! Named control variants.

/// Appearance role for buttons, chips, and similar controls.
///
/// ```
/// assert_ne!(icedtea::variant::Variant::Primary, icedtea::variant::Variant::Danger);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Variant {
    #[default]
    Primary,
    Quiet,
    Danger,
    Ghost,
    Chip,
}

impl Variant {
    pub const ALL: [Variant; 5] = [
        Variant::Primary,
        Variant::Quiet,
        Variant::Danger,
        Variant::Ghost,
        Variant::Chip,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_variants_unique() {
        assert_eq!(Variant::default(), Variant::Primary);
        let mut seen = std::collections::HashSet::new();
        for v in Variant::ALL {
            assert!(seen.insert(v));
        }
        assert_eq!(seen.len(), 5);
    }
}

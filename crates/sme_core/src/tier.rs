/// Fidelity tier controls optional rendering quality features.
/// Tiers add visual polish — they NEVER change simulation or determinism.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FidelityTier {
    /// Mobile-safe baseline: no dynamic lights, no post-processing.
    #[default]
    Tier0,
    /// PC polish: optional bloom, vignette, enhanced colors.
    Tier2,
}

impl FidelityTier {
    /// All tiers in display order.
    pub const ALL: &'static [FidelityTier] = &[FidelityTier::Tier0, FidelityTier::Tier2];

    /// Short human-readable label for overlay display.
    pub fn label(self) -> &'static str {
        match self {
            Self::Tier0 => "Tier 0 (Mobile)",
            Self::Tier2 => "Tier 2 (PC)",
        }
    }

    /// Cycle to the next tier (wraps around).
    pub fn next(self) -> Self {
        match self {
            Self::Tier0 => Self::Tier2,
            Self::Tier2 => Self::Tier0,
        }
    }
}

impl std::fmt::Display for FidelityTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_tier0() {
        assert_eq!(FidelityTier::default(), FidelityTier::Tier0);
    }

    #[test]
    fn next_cycles_through_tiers() {
        assert_eq!(FidelityTier::Tier0.next(), FidelityTier::Tier2);
        assert_eq!(FidelityTier::Tier2.next(), FidelityTier::Tier0);
    }

    #[test]
    fn label_returns_readable_string() {
        assert!(FidelityTier::Tier0.label().contains("Tier 0"));
        assert!(FidelityTier::Tier2.label().contains("Tier 2"));
    }

    #[test]
    fn display_matches_label() {
        for &tier in FidelityTier::ALL {
            assert_eq!(format!("{}", tier), tier.label());
        }
    }

    #[test]
    fn all_contains_every_variant() {
        assert_eq!(FidelityTier::ALL.len(), 2);
        assert!(FidelityTier::ALL.contains(&FidelityTier::Tier0));
        assert!(FidelityTier::ALL.contains(&FidelityTier::Tier2));
    }
}

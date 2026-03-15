use serde::{Deserialize, Serialize};

/// Reputation tiers based on credit score
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReputationTier {
    Novice,    // 0-200
    Regular,   // 201-500
    Active,    // 501-700
    Veteran,   // 701-850
    Expert,    // 851-1000
}

impl ReputationTier {
    pub fn as_str(&self) -> &str {
        match self {
            ReputationTier::Novice => "novice",
            ReputationTier::Regular => "regular",
            ReputationTier::Active => "active",
            ReputationTier::Veteran => "veteran",
            ReputationTier::Expert => "expert",
        }
    }

    /// 从字符串解析等级
    pub fn parse_tier(s: &str) -> Option<Self> {
        match s {
            "novice" => Some(ReputationTier::Novice),
            "regular" => Some(ReputationTier::Regular),
            "active" => Some(ReputationTier::Active),
            "veteran" => Some(ReputationTier::Veteran),
            "expert" => Some(ReputationTier::Expert),
            _ => None,
        }
    }
}

/// Get reputation tier from credit score
pub fn get_reputation_tier(score: i32) -> ReputationTier {
    match score {
        0..=200 => ReputationTier::Novice,
        201..=500 => ReputationTier::Regular,
        501..=700 => ReputationTier::Active,
        701..=850 => ReputationTier::Veteran,
        _ => ReputationTier::Expert,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_calculation() {
        assert_eq!(get_reputation_tier(0), ReputationTier::Novice);
        assert_eq!(get_reputation_tier(200), ReputationTier::Novice);
        assert_eq!(get_reputation_tier(201), ReputationTier::Regular);
        assert_eq!(get_reputation_tier(500), ReputationTier::Regular);
        assert_eq!(get_reputation_tier(501), ReputationTier::Active);
        assert_eq!(get_reputation_tier(700), ReputationTier::Active);
        assert_eq!(get_reputation_tier(701), ReputationTier::Veteran);
        assert_eq!(get_reputation_tier(850), ReputationTier::Veteran);
        assert_eq!(get_reputation_tier(851), ReputationTier::Expert);
        assert_eq!(get_reputation_tier(1000), ReputationTier::Expert);
    }

    #[test]
    fn test_tier_string_conversion() {
        assert_eq!(ReputationTier::Novice.as_str(), "novice");
        assert_eq!(ReputationTier::Regular.as_str(), "regular");
        assert_eq!(ReputationTier::Active.as_str(), "active");
        assert_eq!(ReputationTier::Veteran.as_str(), "veteran");
        assert_eq!(ReputationTier::Expert.as_str(), "expert");
    }
}

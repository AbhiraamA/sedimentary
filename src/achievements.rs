//! Hidden discoveries. Their names stay "???" until earned.
//!
//! Unlock conditions live in `rock.rs` (they are pet logic); this module only
//! holds identities and descriptions, keyed by stable string ids so old saves
//! keep working when new discoveries are added.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Achievement {
    ContinentalDrift,
    PlateTectonics,
    CommitmentToTheBit,
    FloodPlain,
    MirrorFinish,
    Unblinking,
    IdentityCrisis,
    RareEarth,
    RockBottom,
    ZenGarden,
    GlacialErratic,
    ProdigalOwner,
    TemporalAnomaly,
    DeepTime,
}

impl Achievement {
    pub const ALL: [Achievement; 14] = [
        Achievement::ContinentalDrift,
        Achievement::PlateTectonics,
        Achievement::CommitmentToTheBit,
        Achievement::FloodPlain,
        Achievement::MirrorFinish,
        Achievement::Unblinking,
        Achievement::IdentityCrisis,
        Achievement::RareEarth,
        Achievement::RockBottom,
        Achievement::ZenGarden,
        Achievement::GlacialErratic,
        Achievement::ProdigalOwner,
        Achievement::TemporalAnomaly,
        Achievement::DeepTime,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Achievement::ContinentalDrift => "continental_drift",
            Achievement::PlateTectonics => "plate_tectonics",
            Achievement::CommitmentToTheBit => "commitment_to_the_bit",
            Achievement::FloodPlain => "flood_plain",
            Achievement::MirrorFinish => "mirror_finish",
            Achievement::Unblinking => "unblinking",
            Achievement::IdentityCrisis => "identity_crisis",
            Achievement::RareEarth => "rare_earth",
            Achievement::RockBottom => "rock_bottom",
            Achievement::ZenGarden => "zen_garden",
            Achievement::GlacialErratic => "glacial_erratic",
            Achievement::ProdigalOwner => "prodigal_owner",
            Achievement::TemporalAnomaly => "temporal_anomaly",
            Achievement::DeepTime => "deep_time",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Achievement::ContinentalDrift => "Continental Drift",
            Achievement::PlateTectonics => "Plate Tectonics",
            Achievement::CommitmentToTheBit => "Commitment to the Bit",
            Achievement::FloodPlain => "Flood Plain",
            Achievement::MirrorFinish => "Mirror Finish",
            Achievement::Unblinking => "Unblinking",
            Achievement::IdentityCrisis => "Identity Crisis",
            Achievement::RareEarth => "Rare Earth",
            Achievement::RockBottom => "Rock Bottom",
            Achievement::ZenGarden => "Zen Garden",
            Achievement::GlacialErratic => "Glacial Erratic",
            Achievement::ProdigalOwner => "The Prodigal Owner",
            Achievement::TemporalAnomaly => "Temporal Anomaly",
            Achievement::DeepTime => "Deep Time",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Achievement::ContinentalDrift => {
                "Your rock moved one character under its own power. Witnesses wept."
            }
            Achievement::PlateTectonics => {
                "Five characters of lifetime displacement. Cartographers are furious."
            }
            Achievement::CommitmentToTheBit => "Fed a rock ten times. It still has no mouth.",
            Achievement::FloodPlain => "Moisture hit 100. Your rock is now technically a wetland.",
            Achievement::MirrorFinish => {
                "Polish hit 100. It can see you clearly now. That's worse."
            }
            Achievement::Unblinking => "Stared five times in a row. You lost all five.",
            Achievement::IdentityCrisis => {
                "Renamed your rock three times. It has stopped answering to any of them."
            }
            Achievement::RareEarth => {
                "Witnessed a rare event. Tell no one. They won't believe you."
            }
            Achievement::RockBottom => {
                "Disappointment hit 100. Genuinely impressive, in the wrong direction."
            }
            Achievement::ZenGarden => {
                "Boredom hit 0. For one shining moment, your rock was entertained."
            }
            Achievement::GlacialErratic => {
                "Took your rock on ten walks. It was carried every time. It's not counting them."
            }
            Achievement::ProdigalOwner => {
                "Came back after a week or more away. Your rock noticed. Your rock noticed everything."
            }
            Achievement::TemporalAnomaly => {
                "Your clock went backwards. Your rock has seen worse. Subduction, mostly."
            }
            Achievement::DeepTime => "Kept the same rock for 30 days. Geologically, a blink.",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|a| a.id() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn ids_are_unique_and_round_trip() {
        let ids: HashSet<_> = Achievement::ALL.iter().map(|a| a.id()).collect();
        assert_eq!(ids.len(), Achievement::ALL.len());
        for a in Achievement::ALL {
            assert_eq!(Achievement::from_id(a.id()), Some(a));
            assert!(!a.title().is_empty() && !a.description().is_empty());
        }
        assert_eq!(Achievement::from_id("nonsense"), None);
    }
}

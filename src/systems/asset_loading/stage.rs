use bevy::prelude::*;

#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum LoadStage {
    #[default]
    Boot,
    Index,
    Declarations,
    Resolve,
    Payload,
    Derive,
    World,
    Instantiate,
    Warm,
    Settled,
}

impl LoadStage {
    pub const ORDER: &'static [LoadStage] = &[
        Self::Boot,
        Self::Index,
        Self::Declarations,
        Self::Resolve,
        Self::Payload,
        Self::Derive,
        Self::World,
        Self::Instantiate,
        Self::Warm,
        Self::Settled,
    ];

    pub fn position(self) -> usize {
        Self::ORDER
            .iter()
            .position(|stage| *stage == self)
            .expect("every `LoadStage` variant must appear in `LoadStage::ORDER`")
    }

    pub fn next(self) -> LoadStage {
        *Self::ORDER.get(self.position() + 1).unwrap_or(&Self::Settled)
    }

    pub fn earliest(self, other: LoadStage) -> LoadStage {
        if other.position() < self.position() {
            other
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_covers_every_variant() {
        for stage in LoadStage::ORDER {
            match stage {
                LoadStage::Boot
                | LoadStage::Index
                | LoadStage::Declarations
                | LoadStage::Resolve
                | LoadStage::Payload
                | LoadStage::Derive
                | LoadStage::World
                | LoadStage::Instantiate
                | LoadStage::Warm
                | LoadStage::Settled => {}
            }
        }

        let mut seen = LoadStage::ORDER.to_vec();
        seen.dedup();
        assert_eq!(seen.len(), LoadStage::ORDER.len(), "`ORDER` has duplicates");
        assert_eq!(LoadStage::ORDER.len(), 10);
        assert_eq!(LoadStage::default(), LoadStage::Boot);
    }

    #[test]
    fn next_walks_forward_and_stops_at_settled() {
        assert_eq!(LoadStage::Boot.next(), LoadStage::Index);
        assert_eq!(LoadStage::Warm.next(), LoadStage::Settled);
        assert_eq!(LoadStage::Settled.next(), LoadStage::Settled);
    }

    #[test]
    fn earliest_is_order_position_not_declaration_order() {
        assert_eq!(
            LoadStage::Instantiate.earliest(LoadStage::Derive),
            LoadStage::Derive
        );
        assert_eq!(
            LoadStage::Derive.earliest(LoadStage::Instantiate),
            LoadStage::Derive
        );
        assert_eq!(LoadStage::Boot.earliest(LoadStage::Boot), LoadStage::Boot);
    }
}

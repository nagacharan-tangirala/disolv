use hashbrown::HashMap;
use pavenet_engine::entity::Kind;
use std::hash::Hash;

pub trait Tier: Default + Copy + Clone + Hash + PartialEq + Eq + Send + Sync {}

pub trait Tiered<T>
where
    T: Tier,
{
    fn tier(&self) -> &T;
    fn set_tier(&mut self, tier: T);
}

pub struct TierRelations<T>
where
    T: Tier,
{
    tier_to_tiers: HashMap<T, Option<Vec<T>>>, // tier can talk to various tiers
}

impl<T> TierRelations<T>
where
    T: Tier,
{
    fn new() -> Self {
        Self {
            tier_to_tiers: HashMap::new(),
        }
    }

    fn add_relation(&mut self, tier: T, other_tier: T) {
        self.tier_to_tiers
            .entry(tier)
            .or_insert(Some(Vec::new()))
            .as_mut()
            .map(|v| v.push(other_tier));
    }

    fn relations_of(&self, tier: T) -> Option<&Option<Vec<T>>> {
        self.tier_to_tiers.get(&tier)
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
struct TierKindMap<T, K>
where
    T: Tier,
    K: Kind,
{
    tier_to_kind: HashMap<T, K>,
}

impl<T, K> TierKindMap<T, K>
where
    T: Tier + PartialOrd + Ord,
    K: Kind,
{
    fn new() -> Self {
        Self::default()
    }

    fn add_tier_of_kind(&mut self, tier: T, kind: K) {
        match self.tier_to_kind.insert(tier, kind) {
            Some(_) => panic!("Tier already exists"),
            None => (),
        }
    }

    fn kind_of(&self, tier: T) -> &K {
        self.tier_to_kind.get(&tier).expect("Tier does not exist")
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::tier::{Tier, TierKindMap, TierRelations, Tiered};
    use pavenet_engine::entity::Kind;
    use std::fmt::Debug;

    #[derive(Default, Debug, Copy, Clone, Hash, Ord, PartialOrd, PartialEq, Eq)]
    struct Level(u32);

    impl From<u32> for Level {
        fn from(level: u32) -> Self {
            Self(level)
        }
    }

    impl Into<i32> for Level {
        fn into(self) -> i32 {
            self.0 as i32
        }
    }

    impl Tier for Level {}

    #[derive(Default, Copy, Clone, Hash, PartialEq, Eq)]
    enum DType {
        #[default]
        A,
        B,
        C,
    }

    impl Debug for DType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                DType::A => write!(f, "A"),
                DType::B => write!(f, "B"),
                DType::C => write!(f, "C"),
            }
        }
    }

    impl Kind for DType {}

    struct TDevice {
        level: Level,
        id: u32,
        kind: DType,
    }

    impl TDevice {
        fn new(level: Level, id: u32, kind: DType) -> Self {
            Self { level, id, kind }
        }
    }

    impl Tiered<Level> for TDevice {
        fn tier(&self) -> &Level {
            &self.level
        }

        fn set_tier(&mut self, tier: Level) {
            self.level = tier;
        }
    }

    #[test]
    fn test_tiered_device() {
        let mut device = TDevice::new(0.into(), 0, DType::A);
        assert_eq!(device.tier(), &0.into());
        device.set_tier(1.into());
        assert_eq!(device.tier(), &1.into());
    }

    #[test]
    fn test_tier_kind_map() {
        let devicea = TDevice::new(0.into(), 0, DType::A);
        let deviceb = TDevice::new(1.into(), 1, DType::B);
        let devicec = TDevice::new(2.into(), 2, DType::C);
        let mut tier_kind_map: TierKindMap<Level, DType> = TierKindMap::new();
        tier_kind_map.add_tier_of_kind(0.into(), DType::A);
        tier_kind_map.add_tier_of_kind(1.into(), DType::B);
        tier_kind_map.add_tier_of_kind(2.into(), DType::C);
        assert_eq!(tier_kind_map.kind_of(0.into()), &DType::A);
        assert_eq!(tier_kind_map.kind_of(1.into()), &DType::B);
        assert_eq!(tier_kind_map.kind_of(2.into()), &DType::C);
    }

    #[test]
    fn test_tier_relations() {
        let mut tier_relations: TierRelations<Level> = TierRelations::new();
        tier_relations.add_relation(0.into(), 1.into());
        tier_relations.add_relation(0.into(), 2.into());
        tier_relations.add_relation(0.into(), 3.into());
        tier_relations.add_relation(1.into(), 2.into());
        tier_relations.add_relation(1.into(), 3.into());
        tier_relations.add_relation(2.into(), 3.into());
        assert_eq!(
            tier_relations.relations_of(0.into()),
            Some(&Some(vec![1.into(), 2.into(), 3.into()]))
        );
        assert_eq!(
            tier_relations.relations_of(1.into()),
            Some(&Some(vec![2.into(), 3.into()]))
        );
        assert_eq!(
            tier_relations.relations_of(2.into()),
            Some(&Some(vec![3.into()]))
        );
        assert_eq!(tier_relations.relations_of(3.into()), None);
    }
}
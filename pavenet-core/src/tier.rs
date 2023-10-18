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

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct TierKindMap<T, K>
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_tier_of_kind(&mut self, tier: T, kind: K) {
        match self.tier_to_kind.insert(tier, kind) {
            Some(_) => panic!("Tier already exists"),
            None => (),
        }
    }

    pub fn kind_of(&self, tier: T) -> &K {
        self.tier_to_kind.get(&tier).expect("Tier does not exist")
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::tier::{Tier, TierKindMap, Tiered};
    use pavenet_engine::entity::Kind;
    use std::fmt::{Debug, Display, Formatter};

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

    impl Display for DType {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                DType::A => write!(f, "A"),
                DType::B => write!(f, "B"),
                DType::C => write!(f, "C"),
            }
        }
    }

    impl Debug for DType {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
}

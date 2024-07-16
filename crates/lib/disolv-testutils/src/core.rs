use disolv_core::core::Core;
use disolv_core::hashbrown::HashMap;

use crate::agent::TDevice;
use crate::bucket::MyBucket;

pub fn create_core() -> Core<TDevice, MyBucket> {
    Core {
        bucket: MyBucket::default(),
        agent_cache: HashMap::new(),
        agent_stats: HashMap::new(),
    }
}

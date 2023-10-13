use crate::node::pool::NodePool;
use krabmaga::engine::schedule::Schedule;
use pavenet_core::enums::NodeType;
use pavenet_core::types::TimeStamp;

#[derive(Clone, Default)]
pub struct PoolSet<U>
where
    U: NodePool,
{
    pools_by_type: Vec<(NodeType, U)>,
}

impl<U> PoolSet<U>
where
    U: NodePool,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, node_type: NodeType, pool: U) {
        self.pools_by_type.push((node_type, pool));
    }

    pub fn pool_of(&mut self, node_type: NodeType) -> Option<&mut U> {
        self.pools_by_type
            .iter_mut()
            .find(|(n_type, _)| *n_type == node_type)
            .map(|(_, pool)| pool)
    }
}

impl<U> NodePool for PoolSet<U>
where
    U: NodePool,
{
    fn init(&mut self, schedule: &mut Schedule) {
        self.pools_by_type
            .iter_mut()
            .for_each(|(_, pool)| pool.init(schedule));
    }

    fn before_step(&mut self, step: TimeStamp) {
        self.pools_by_type
            .iter_mut()
            .for_each(|(_, pool)| pool.before_step(step));
    }

    fn update(&mut self, step: TimeStamp) {
        self.pools_by_type
            .iter_mut()
            .for_each(|(_, pool)| pool.update(step));
    }

    fn after_step(&mut self, schedule: &mut Schedule) {
        self.pools_by_type
            .iter_mut()
            .for_each(|(_, pool)| pool.after_step(schedule));
    }

    fn streaming_step(&mut self, step: TimeStamp) {
        self.pools_by_type
            .iter_mut()
            .for_each(|(_, pool)| pool.streaming_step(step));
    }
}

#[cfg(test)]
pub(crate) mod tests {
    pub(crate) mod pool_set {
        use super::super::PoolSet;
        use crate::engine::nodeimpl::tests::make_test_pool;
        use crate::engine::nodeimpl::tests::test_pool::TestPool;
        use pavenet_core::enums::NodeType::Vehicle;

        pub(crate) fn make_test_pool_set(test_pool: &TestPool) -> PoolSet<TestPool> {
            let mut pool_set = PoolSet::new();
            pool_set.add(Vehicle, test_pool.to_owned());
            pool_set
        }

        #[test]
        fn make_pool_set() {
            let test_pool = make_test_pool();
            let mut pool_set = make_test_pool_set(&test_pool);
            assert_eq!(
                Vehicle,
                pool_set.pool_of(Vehicle).unwrap().nodes[0]
                    .node_info
                    .node_type
            );
        }
    }
}

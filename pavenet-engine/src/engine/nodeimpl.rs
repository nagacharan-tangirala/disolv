use crate::engine::engine::Engine;
use crate::node::node::Node;
use crate::node::pool::NodePool;
use crate::node::power::{PowerSchedule, PowerState};
use krabmaga::engine::agent::Agent;
use krabmaga::engine::state::State;
use pavenet_core::types::NodeId;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

#[derive(Clone)]
pub struct NodeImpl<T, U>
where
    T: Node,
    U: NodePool,
{
    pub(crate) node_id: NodeId,
    pub(crate) power_schedule: PowerSchedule,
    pub(crate) node: T,
    pub(crate) _phantom_u: PhantomData<fn() -> U>,
}

impl<T, U> NodeImpl<T, U>
where
    T: Node,
    U: NodePool,
{
    pub fn new(node_id: NodeId, power_schedule: PowerSchedule, node: T) -> Self {
        Self {
            node_id,
            node,
            power_schedule,
            _phantom_u: PhantomData,
        }
    }

    fn power_off(&mut self, engine: &mut Engine<T, U>) {
        self.node.set_power_state(PowerState::Off);
        self.power_schedule.pop_time_to_off();
        engine.node_set.to_pop.push(self.node_id);

        let time_stamp = self.power_schedule.pop_time_to_on();
        if time_stamp > engine.step {
            engine.node_set.to_add.push(self.node_id);
        }
    }
}

impl<T, U> Agent for NodeImpl<T, U>
where
    T: Node,
    U: NodePool,
{
    fn step(&mut self, state: &mut dyn State) {
        self.node.set_power_state(PowerState::On);
        let engine: &mut Engine<T, U> = state.as_any_mut().downcast_mut::<Engine<T, U>>().unwrap();
        let pool = match engine.pool_set.pool_of(self.node.node_type()) {
            Some(pool) => pool,
            None => panic!("Could not find pool for type {}", self.node.node_type()),
        };
        self.node.step(pool);
        if engine.step == self.power_schedule.peek_time_to_off() {
            self.power_off(engine);
        }
    }

    fn after_step(&mut self, state: &mut dyn State) {
        let engine = state.as_any_mut().downcast_mut::<Engine<T, U>>().unwrap();
        let pool = match engine.pool_set.pool_of(self.node.node_type()) {
            Some(pool) => pool,
            None => panic!("Could not find pool for type {}", self.node.node_type()),
        };
        self.node.after_step(pool);
    }

    fn is_stopped(&self, _state: &mut dyn State) -> bool {
        self.node.power_state() == PowerState::Off
    }
}

impl<T, U> Hash for NodeImpl<T, U>
where
    T: Node,
    U: NodePool,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.node_id.hash(state);
    }
}

impl<T, U> fmt::Display for NodeImpl<T, U>
where
    T: Node,
    U: NodePool,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.node_id)
    }
}

impl<T, U> Eq for NodeImpl<T, U>
where
    T: Node,
    U: NodePool,
{
}

impl<T, U> PartialEq for NodeImpl<T, U>
where
    T: Node,
    U: NodePool,
{
    fn eq(&self, other: &NodeImpl<T, U>) -> bool {
        self.node_id == other.node_id
    }
}

#[cfg(test)]
pub(crate) mod tests {
    pub(crate) mod test_pool {
        use super::test_node::TestNode;
        use crate::node::pool::NodePool;
        use krabmaga::engine::schedule::Schedule;
        use pavenet_core::types::TimeStamp;

        #[derive(Clone, Debug, Default)]
        pub(crate) struct TestPool {
            pub(crate) nodes: Vec<TestNode>,
        }

        impl TestPool {
            pub fn add_node(&mut self, nodes: TestNode) {
                self.nodes.push(nodes);
            }
        }

        impl NodePool for TestPool {
            fn init(&mut self, schedule: &mut Schedule) {
                println!("Initializing test pool");
            }

            fn before_step(&mut self, step: TimeStamp) {
                println!("TestPool::before_step {}", step);
            }

            fn update(&mut self, step: TimeStamp) {
                println!("TestPool::update {}", step);
            }

            fn after_step(&mut self, schedule: &mut Schedule) {
                println!("TestPool::after_step {}", schedule.step);
            }

            fn streaming_step(&mut self, step: TimeStamp) {
                println!("TestPool::streaming_step {}", step);
            }
        }
    }

    pub(crate) mod test_node {
        use crate::node::node::Node;
        use crate::node::power::PowerState;
        use pavenet_core::enums::NodeType;
        use pavenet_core::structs::NodeInfo;

        #[derive(Clone, Copy, Debug, Default)]
        pub(crate) struct TestNode {
            pub node_info: NodeInfo,
            pub power_state: PowerState,
        }

        impl TestNode {
            pub fn new(node_info: NodeInfo) -> Self {
                Self {
                    node_info,
                    power_state: PowerState::Off,
                }
            }
        }

        impl Node for TestNode {
            fn node_type(&self) -> NodeType {
                self.node_info.node_type
            }

            fn power_state(&self) -> PowerState {
                self.power_state
            }

            fn node_order(&self) -> i32 {
                self.node_info.order.as_i32()
            }

            fn set_power_state(&mut self, power_state: PowerState) {
                self.power_state = power_state;
            }

            fn step<TestPool>(&mut self, _pool: &mut TestPool) {
                println!("TestNode::step");
            }

            fn after_step<TestPool>(&mut self, _pool: &mut TestPool) {
                println!("TestNode::after_step");
            }
        }
    }

    use super::tests::test_node::TestNode;
    use super::NodeImpl;
    use crate::engine::node_set::NodeSet;
    use crate::engine::nodeimpl::tests::test_pool::TestPool;
    use crate::node::node::Node;
    use crate::node::power::{PowerSchedule, SCHEDULE_SIZE};
    use hashbrown::HashMap;
    use pavenet_core::enums::NodeType;
    use pavenet_core::structs::NodeInfo;
    use pavenet_core::types::{Class, NodeId, Order, TimeStamp};

    pub(crate) fn make_test_pool() -> TestPool {
        let mut test_pool = TestPool::default();
        for i in 0..10 {
            let node_info = NodeInfo::builder()
                .node_type(NodeType::Vehicle)
                .node_class(Class::from(1))
                .id(i.into())
                .order(Order::from(1))
                .build();
            let test_node = TestNode::new(node_info);
            test_pool.add_node(test_node);
        }
        test_pool
    }

    fn make_power_schedule() -> PowerSchedule {
        let mut on_times: [Option<TimeStamp>; SCHEDULE_SIZE] = [None; SCHEDULE_SIZE];
        let mut off_times: [Option<TimeStamp>; SCHEDULE_SIZE] = [None; SCHEDULE_SIZE];
        for i in 0..SCHEDULE_SIZE {
            on_times[i] = Some(TimeStamp::from(1u64));
            off_times[i] = Some(TimeStamp::from(10u64));
        }
        PowerSchedule::new(on_times, off_times)
    }

    pub(crate) fn make_test_node_set() -> NodeSet<TestNode, TestPool> {
        let test_pool = make_test_pool();
        let mut nodes: HashMap<NodeId, NodeImpl<TestNode, TestPool>> = HashMap::new();
        for node in test_pool.nodes.into_iter() {
            let power_schedule = make_power_schedule();
            let node_id = node.node_info.id;
            let node_impl = NodeImpl::new(node_id, power_schedule, node);
            nodes.insert(node_id, node_impl);
        }
        NodeSet::new(nodes)
    }
}

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

impl fmt::Display for NodeImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.node_id)
    }
}

impl Eq for NodeImpl {}

impl PartialEq for NodeImpl {
    fn eq(&self, other: &NodeImpl) -> bool {
        self.node_id == other.node_id
    }
}

dyn_clone::clone_trait_object!(Node);
impl_downcast!(Node);

#[cfg(test)]
pub(crate) mod tests {
    mod test_node {
        use crate::engine::engine::Engine;
        use crate::node::node::Node;
        use crate::node::power::PowerState;
        use pavenet_core::structs::NodeInfo;

        #[derive(Clone, Debug)]
        pub struct TestNode {
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
            fn power_state(&self) -> PowerState {
                self.power_state
            }

            fn node_order(&self) -> i32 {
                self.node_info.order.as_i32()
            }

            fn set_power_state(&mut self, power_state: PowerState) {
                self.power_state = power_state;
            }

            fn step(&mut self, _engine: &mut Engine) {
                println!("TestNode::step");
            }

            fn after_step(&mut self, _engine: &mut Engine) {
                println!("TestNode::after_step");
            }
        }
    }

    use crate::engine::nodeimpl::tests::test_node::TestNode;
    use crate::engine::nodeimpl::NodeImpl;
    use crate::node::node::Node;
    use crate::node::power::{PowerSchedule, SCHEDULE_SIZE};
    use hashbrown::HashMap;
    use pavenet_core::enums::NodeType;
    use pavenet_core::named::class::Class;
    use pavenet_core::structs::NodeInfo;
    use pavenet_core::types::{NodeId, Order, TimeStamp};

    pub(crate) fn make_dyn_nodes() -> HashMap<NodeId, Box<dyn Node>> {
        let mut nodes: HashMap<NodeId, Box<dyn Node>> = HashMap::with_capacity(10);
        for i in 0..10 {
            let node_info = NodeInfo::builder()
                .node_type(NodeType::Vehicle)
                .node_class(Class::from(1))
                .id(i.into())
                .order(Order::from(1))
                .build();
            let test_node = TestNode::new(node_info);
            let power_schedule = make_power_schedule();
            nodes.insert(NodeId::from(i), Box::new(test_node));
        }
        nodes
    }

    pub(crate) fn make_power_schedule() -> PowerSchedule {
        let mut on_times: [Option<TimeStamp>; SCHEDULE_SIZE] = [None; SCHEDULE_SIZE];
        let mut off_times: [Option<TimeStamp>; SCHEDULE_SIZE] = [None; SCHEDULE_SIZE];
        for i in 0..SCHEDULE_SIZE {
            on_times[i] = Some(TimeStamp::from(0u64));
            off_times[i] = Some(TimeStamp::from(10u64));
        }
        PowerSchedule::new(on_times, off_times)
    }
}

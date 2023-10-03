use crate::core::core::Core;
use crate::node::info::NodeInfo;
use crate::node::power::PowerState;
use crate::node::receive::Recipient;
use crate::node::transmit::Transmitter;
use krabmaga::engine::agent::Agent;
use pavenet_config::types::ts::TimeStamp;

pub trait Node: Agent + Transmitter + Recipient + Copy + Send + Sync + Clone {
    fn as_agent(self) -> Box<dyn Agent>;
    fn node_info(&self) -> NodeInfo;
    fn power_on_time(&self) -> TimeStamp;
    fn set_power(&mut self, power_state: PowerState);
    fn update_map_state(&mut self, core: &mut Core);
    fn transmit_data(&mut self, core: &mut Core);
    fn receive_data(&mut self, core: &mut Core);
    fn collect_stats(&mut self, core: &mut Core);
}

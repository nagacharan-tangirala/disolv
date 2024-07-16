use disolv_core::agent::{Agent, AgentId};
use disolv_core::bucket::TimeMS;
use disolv_testutils::agent::{DeviceType, TDevice};
use disolv_testutils::core::create_core;

#[test]
fn test_add_agent() {
    let mut core = create_core();
    let device_a = TDevice::make_device(AgentId::from(1), DeviceType::TypeA, 1);
    let device_b = TDevice::make_device(AgentId::from(2), DeviceType::TypeB, 2);
    core.add_agent(device_a.id(), TimeMS::from(0));
    core.add_agent(device_b.id(), TimeMS::from(0));
    assert_eq!(core.agent_cache.len(), 1);
    assert_eq!(core.agent_cache.get(&TimeMS::from(0)).unwrap().len(), 2);
    let device_c = TDevice::make_device(AgentId::from(3), DeviceType::TypeA, 3);
    core.add_agent(device_c.id(), TimeMS::from(1));
    assert_eq!(core.agent_cache.len(), 2);
    assert_eq!(core.agent_cache.get(&TimeMS::from(1)).unwrap().len(), 1);
}

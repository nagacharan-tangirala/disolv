use indexmap::IndexMap;
use keyed_priority_queue::KeyedPriorityQueue;

use disolv_core::agent::{Agent, AgentId, AgentImpl};
use disolv_core::bucket::TimeMS;
use disolv_core::hashbrown::HashMap;
use disolv_testutils::agent::{DeviceType, TDevice};
use disolv_testutils::bucket::MyBucket;
use disolv_testutils::core::create_core;
use disolv_v2x::simulation::scheduler::{DefaultScheduler, MapScheduler, Scheduler};

pub fn create_map_scheduler() -> MapScheduler<TDevice, MyBucket> {
    let agents = IndexMap::with_capacity(1000000);
    let mut inactive_agents = HashMap::with_capacity(1000000);
    for i in 0..1000000 {
        let device = TDevice::make_device(AgentId::from(i), DeviceType::TypeA, i as i32);
        inactive_agents.insert(
            device.id(),
            AgentImpl::builder()
                .agent_id(device.id())
                .agent(device)
                .build(),
        );
    }
    MapScheduler {
        core: create_core(),
        active_agents: agents,
        inactive_agents,
        deactivated: Vec::with_capacity(100000),
        duration: TimeMS::from(1000),
        streaming_interval: TimeMS::from(10),
        streaming_step: TimeMS::from(0),
        output_interval: TimeMS::from(100),
        output_step: TimeMS::from(0),
        step_size: TimeMS::from(100),
        now: TimeMS::from(0),
    }
}

pub fn create_scheduler() -> DefaultScheduler<TDevice, MyBucket> {
    let mut agents = HashMap::new();
    let mut agent_queue = KeyedPriorityQueue::new();
    for i in 0..100000 {
        let device = TDevice::make_device(AgentId::from(i), DeviceType::TypeA, i as i32);
        agent_queue.push(device.id, device.order);
        agents.insert(
            device.id(),
            AgentImpl::builder()
                .agent_id(device.id())
                .agent(device)
                .build(),
        );
    }
    DefaultScheduler {
        core: create_core(),
        agents,
        agent_queue,
        duration: TimeMS::from(1000),
        streaming_interval: TimeMS::from(10),
        streaming_step: TimeMS::from(0),
        output_interval: TimeMS::from(100),
        output_step: TimeMS::from(0),
        step_size: TimeMS::from(100),
        now: TimeMS::from(0),
    }
}

#[test]
fn test_map_activate() {
    let mut scheduler = create_map_scheduler();
    scheduler.activate();
    assert_eq!(scheduler.active_agents.len(), 1000000);
}

#[test]
fn test_map_collect_stats() {
    let mut scheduler = create_map_scheduler();
    scheduler.collect_stats();
    assert_eq!(scheduler.core.agent_stats.len(), 1000000);
}

#[test]
fn test_map_trigger() {
    let mut scheduler = create_map_scheduler();
    scheduler.activate();
    scheduler.trigger();
    assert_eq!(scheduler.now, TimeMS::from(100));
}

#[test]
fn test_activate() {
    let mut scheduler = create_scheduler();
    scheduler.activate();
    assert_eq!(scheduler.agent_queue.len(), 100000);
}

#[test]
fn test_collect_stats() {
    let mut scheduler = create_scheduler();
    scheduler.collect_stats();
    assert_eq!(scheduler.core.agent_stats.len(), 100000);
}

#[test]
fn test_trigger() {
    let mut scheduler = create_scheduler();
    scheduler.activate();
    scheduler.trigger();
    assert_eq!(scheduler.now, TimeMS::from(100));
}

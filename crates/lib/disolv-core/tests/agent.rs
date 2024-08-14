use disolv_core::agent::{Agent, AgentId, AgentKind, AgentOrder};
use disolv_core::bucket::TimeMS;
use disolv_testutils::agent::TDevice;
use disolv_testutils::bucket::MyBucket;

#[test]
fn test_device_creation() {
    let device_a = TDevice::make_device(AgentId::from(1), AgentKind::Vehicle, 1);
    assert_eq!(device_a.id(), AgentId::from(1));
}

#[test]
fn test_device_comparison() {
    let device_a = TDevice::make_device(AgentId::from(1), AgentKind::Vehicle, 1);
    let device_b = TDevice::make_device(AgentId::from(2), AgentKind::Vehicle, 2);
    assert_ne!(device_a.id(), device_b.id());
    assert_eq!(
        device_a.device_info.device_type,
        device_b.device_info.device_type
    );
    assert_eq!(device_a.device_info.order, AgentOrder::from(1));
    assert_eq!(device_b.device_info.order, AgentOrder::from(2));
}

#[test]
fn test_device_step() {
    let mut device_a = TDevice::make_device(AgentId::from(1), AgentKind::Vehicle, 1);
    let mut bucket = MyBucket::default();
    assert_eq!(bucket.step, TimeMS::default());
}

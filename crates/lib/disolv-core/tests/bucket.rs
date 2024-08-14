use disolv_core::bucket::{Bucket, TimeMS};
use disolv_testutils::bucket::MyBucket;

#[test]
fn test_bucket_update() {
    let mut bucket = MyBucket::default();
    let step0 = TimeMS::from(0i64);
    bucket.initialize(step0);
    assert_eq!(bucket.step, TimeMS::from(0i64));
    let step1 = TimeMS::from(1i64);
    bucket.before_agents(step1);
    assert_eq!(bucket.step, TimeMS::from(1i64));
    let step2 = TimeMS::from(2i64);
    bucket.before_agents(step2);
    assert_eq!(bucket.step, TimeMS::from(2i64));
}

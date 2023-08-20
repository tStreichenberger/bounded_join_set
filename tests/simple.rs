use bounded_join_set::JoinSet;

use tokio::time::{self, Duration};

#[tokio::test]
async fn simple() {
    async fn send_request() {
        time::sleep(Duration::from_millis(5)).await;
    }

    let mut join_set = JoinSet::new(4);

    for _ in 0..64 {
        join_set.spawn(send_request());
    }

    time::sleep(Duration::from_millis(1)).await;

    assert_eq!(join_set.num_active(), 4);
    assert_eq!(join_set.num_queued(), 60);
    assert_eq!(join_set.num_completed(), 0);

    time::sleep(Duration::from_millis(50)).await;

    assert_eq!(join_set.num_active(), 4);

    time::sleep(Duration::from_millis(100)).await;

    assert_eq!(join_set.num_completed(), 64);
}

#[tokio::test]
async fn test_len() {
    let mut join_set = JoinSet::new(16);

    for _ in 0..100 {
        join_set.spawn(async {});
    }

    time::sleep(Duration::from_millis(10)).await;

    assert_eq!(join_set.len(), 100);

    for _ in 0..5 {
        join_set.join_next().await;
    }

    assert_eq!(join_set.len(), 95);

    while join_set.join_next().await.is_some() {}

    assert_eq!(join_set.len(), 0);
}

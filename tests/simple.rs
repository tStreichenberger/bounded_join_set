use bounded_join_set::JoinSet;

use rand::Rng;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::sync::Arc;

#[tokio::test]
async fn simple() {
    let mut join_set = JoinSet::new(16);
    let num_active = Arc::new(AtomicUsize::new(0));

    let mut rng = rand::thread_rng();

    for _ in 0..100 {
        let num_active = num_active.clone();
        let jitter_ms = rng.gen_range(0..=1000);
        join_set.spawn(async move {
            num_active.fetch_add(1, Relaxed);
            println!("Starting Task: num_active {}", num_active.load(Relaxed));
            tokio::time::sleep(tokio::time::Duration::from_millis(1500 + jitter_ms)).await;
            num_active.fetch_sub(1, Relaxed);
            println!("Finished Task: num_active {}", num_active.load(Relaxed));
        });
    }

    // sleep for a little before calling join next to confirm that tasks are still processed even if join_next is not waited od
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    println!("Starting Join Next Loop");

    while let Some(r) = join_set.join_next().await {
        r.unwrap();
    }
    println!("Finished Awaiting All");

    join_set.spawn(async {
        println!("Processing one more after main loop");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    });

    join_set.join_next().await.unwrap().unwrap();
    println!("Done!");
}

use tokio::time::{Duration};
use tokio_adjustable_timeout::{adjustable_timeout, Elapsed};

async fn sleep(ms: u64) {
    tokio::time::sleep(Duration::from_millis(ms)).await
}

const BIAS: Duration = Duration::from_millis(10);

#[tokio::test]
async fn it_expires() {
    let timeout = adjustable_timeout(Duration::from_millis(50), async {
        sleep(100).await;
    });

    assert!(matches!(timeout.await, Err(Elapsed)));
}

#[tokio::test]
async fn it_expires_after_increment() {
    let timeout = adjustable_timeout(Duration::from_millis(50), async {
        sleep(100).await;
    });

    let handle = timeout.handle();

    assert!(handle.increment(Duration::from_millis(25)).is_ok());
    assert!(matches!(timeout.await, Err(Elapsed)));
}

#[tokio::test]
async fn it_expires_after_increment_in_task() {
    let timeout = adjustable_timeout(Duration::from_millis(50), async {
        sleep(100).await;
        1337
    });

    let handle = timeout.handle();

    tokio::spawn(async move {
        sleep(25).await;
        assert!(handle.increment(Duration::from_millis(25)).is_ok());
    });

    assert!(matches!(timeout.await, Err(Elapsed)));
}

#[tokio::test]
async fn it_expires_after_decrement() {
    let timeout = adjustable_timeout(Duration::from_millis(100), async {
        sleep(50).await;
        1337
    });

    let handle = timeout.handle();

    assert!(handle.decrement(Duration::from_millis(50) + BIAS).is_ok());
    assert!(matches!(timeout.await, Err(Elapsed)));
}

#[tokio::test]
async fn it_expires_after_decrement_in_task() {
    let timeout = adjustable_timeout(Duration::from_millis(100), async {
        sleep(50).await;
        1337
    });

    let handle = timeout.handle();

    tokio::spawn(async move {
        sleep(25).await;
        assert!(handle.decrement(Duration::from_millis(50) + BIAS).is_ok());
    });

    assert!(matches!(timeout.await, Err(Elapsed)));
}

#[tokio::test]
async fn it_succeeds() {
    let timeout = adjustable_timeout(Duration::from_millis(100), async {
        sleep(50).await;
        1337
    });

    assert!(matches!(timeout.await, Ok(1337)));
}

#[tokio::test]
async fn it_succeeds_after_increment() {
    let timeout = adjustable_timeout(Duration::from_millis(50), async {
        sleep(100).await;
        1337
    });

    let handle = timeout.handle();

    assert!(handle.increment(Duration::from_millis(50) + BIAS).is_ok());
    assert!(matches!(timeout.await, Ok(1337)));
}

#[tokio::test]
async fn it_succeeds_after_increment_in_task() {
    let timeout = adjustable_timeout(Duration::from_millis(50), async {
        sleep(100).await;
        1337
    });

    let handle = timeout.handle();

    tokio::spawn(async move {
        sleep(25).await;
        assert!(handle.increment(Duration::from_millis(50) + BIAS).is_ok());
    });

    assert!(matches!(timeout.await, Ok(1337)));
}

#[tokio::test]
async fn it_succeeds_after_several_increments_in_task() {
    let timeout = adjustable_timeout(Duration::from_millis(50), async {
        sleep(300).await;
        1337
    });

    let handle = timeout.handle();

    tokio::spawn(async move {
        sleep(25).await;
        // at 75, new deadline = 200
        assert!(handle.increment(Duration::from_millis(150)).is_ok());
        sleep(150).await;
        // at 175, new deadline = 250
        assert!(handle.increment(Duration::from_millis(50)).is_ok());
        sleep(50).await;
        // at 225, new deadline = 300 + BIAS
        assert!(handle.increment(Duration::from_millis(50) + BIAS).is_ok());
    });

    assert!(matches!(timeout.await, Ok(1337)));
}

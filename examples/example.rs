use std::time::Duration;
use tokio::time::sleep;
use tokio_adjustable_timeout::adjustable_timeout;

#[tokio::main]
async fn main() {
    let timeout = adjustable_timeout(Duration::from_millis(1500), async {
        println!("worker-begin");
        sleep(Duration::from_millis(1000)).await;
        println!("worker-end");
    });

    let handle = timeout.handle();

    tokio::spawn(async move {
        sleep(Duration::from_millis(100)).await;
        handle.decrement(Duration::from_millis(505)).unwrap();
    });

    match timeout.await {
        Ok(_) => println!("success!"),
        Err(_) => println!("timeout!"),
    }
}

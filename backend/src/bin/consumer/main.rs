use backend::utils::mq::*;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    backend::log_init();
    let scheduler = vec![tokio::spawn(generate_trending())];
    let handles = vec![
        tokio::spawn(pulsar_relation()),
        tokio::spawn(pulsar_typesense()),
    ];
    futures::future::join_all(handles).await;
    futures::future::join_all(scheduler).await;
    std::thread::sleep(Duration::from_millis(1000));
}

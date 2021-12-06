use backend::utils::mq::*;
use parking_lot::Mutex;
use rocket::local::asynchronous::Client;
use tokio::sync::OnceCell;

pub async fn get_client() -> &'static Mutex<Client> {
    static INSTANCE: OnceCell<Mutex<Client>> = OnceCell::const_new();
    INSTANCE
        .get_or_init(|| async {
            let _ = vec![tokio::spawn(generate_trending())];
            let _ = vec![
                tokio::spawn(pulsar_relation()),
                tokio::spawn(pulsar_typesense()),
            ];
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            let rocket = backend::rocket_init();
            let client = Client::tracked(rocket)
                .await
                .expect("valid rocket instance");
            let r = Mutex::new(client);
            r
        })
        .await
}

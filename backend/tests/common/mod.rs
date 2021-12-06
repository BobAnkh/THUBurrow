use backend::utils::mq::*;
// use once_cell::sync::OnceCell;
use parking_lot::Mutex;
// use rocket::local::blocking::Client;

// pub fn get_client() -> &'static Mutex<Client> {
//     static INSTANCE: OnceCell<Mutex<Client>> = OnceCell::new();
//     INSTANCE.get_or_init(|| {
//         let rocket = backend::rocket_init();
//         let client = Client::tracked(rocket).expect("valid rocket instance");
//         Mutex::new(client)
//     })
// }

use rocket::local::asynchronous::Client;
use tokio::sync::OnceCell;

pub async fn get_client() -> &'static Mutex<Client> {
    static INSTANCE: OnceCell<Mutex<Client>> = OnceCell::const_new();
    INSTANCE
        .get_or_init(|| async {
            let scheduler = vec![tokio::spawn(generate_trending())];
            let handles = vec![
                tokio::spawn(pulsar_relation()),
                tokio::spawn(pulsar_typesense()),
            ];
            let rocket = backend::rocket_init();
            let client = Client::tracked(rocket).await.expect("valid rocket instance");
            let r = Mutex::new(client);
            r
        })
        .await
}

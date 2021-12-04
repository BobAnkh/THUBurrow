// use once_cell::sync::OnceCell;
// use rocket::local::blocking::Client;
// use std::sync::Mutex;
use rocket::local::asynchronous::Client;
use tokio::sync::OnceCell;

// pub fn get_client() -> &'static Mutex<Client> {
//     static INSTANCE: OnceCell<Mutex<Client>> = OnceCell::new();
//     INSTANCE.get_or_init(|| {
//         let rocket = backend::rocket_init();
//         let client = Client::tracked(rocket).expect("valid rocket instance");
//         Mutex::new(client)
//     })
// }

pub async fn get_client() -> &'static Client {
    static INSTANCE: OnceCell<Client> = OnceCell::const_new();
    INSTANCE
        .get_or_init(|| async {
            let rocket = backend::rocket_init();
            let client = Client::tracked(rocket)
                .await
                .expect("valid rocket instance");
            client
        })
        .await
}

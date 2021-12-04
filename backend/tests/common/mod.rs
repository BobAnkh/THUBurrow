use rocket::local::asynchronous::Client;
use tokio::sync::OnceCell;

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

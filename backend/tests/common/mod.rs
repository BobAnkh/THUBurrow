use backend::utils::mq::*;
use parking_lot::Mutex;
use rocket::local::blocking::Client;
// use tokio::sync::OnceCell;
use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;

pub fn get_client() -> &'static Mutex<Client> {
    static INSTANCE: OnceCell<Mutex<Client>> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        let rt = Runtime::new().unwrap();
        rt.spawn(generate_trending());
        rt.spawn(pulsar_relation());
        rt.spawn(pulsar_typesense());
        // let _ = vec![
        //     tokio::spawn(pulsar_relation()),
        //     tokio::spawn(pulsar_typesense()),
        // ];
        // tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        let rocket = backend::rocket_init();
        let client = Client::tracked(rocket).expect("valid rocket instance");
        
        Mutex::new(client)
    })
}

use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use rocket::local::blocking::Client;

pub fn get_client() -> &'static Mutex<Client> {
    static INSTANCE: OnceCell<Mutex<Client>> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        let rocket = backend::rocket_init();
        let client = Client::tracked(rocket).expect("valid rocket instance");
        Mutex::new(client)
    })
}

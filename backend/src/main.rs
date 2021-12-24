#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
    backend::rocket_init()
}

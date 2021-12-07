#[macro_use]
extern crate rocket;
extern crate rustc_serialize;

#[launch]
fn rocket() -> _ {
    backend::rocket_init()
}

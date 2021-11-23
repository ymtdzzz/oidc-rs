#![feature(decl_macro)]
#![macro_use]
extern crate rocket;

use oidc_rs::server;

#[rocket::launch]
fn rocket() -> _ {
    server::run()
}

#![feature(proc_macro_hygiene)]
#![feature(decl_macro)]

#[macro_use]
extern crate rocket;
extern crate dotenv;
extern crate serde_derive;
pub mod models;

mod api;
mod oauth;

use std::path::PathBuf;

use rocket::fs::{FileServer, NamedFile};

#[get("/<path..>", rank = 5)]
async fn svelte(path: PathBuf) -> Option<NamedFile> {
    if !path.starts_with("api/") {
        NamedFile::open("public/index.html").await.ok()
    } else {
        None
    }
}

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();
    rocket::build()
        .mount(
            "/",
            FileServer::from(concat!(env!("CARGO_MANIFEST_DIR"), "/public")).rank(4),
        )
        .mount("/", routes![svelte])
        .mount("/oauth", oauth::routes())
        .mount("/api", api::routes())
}

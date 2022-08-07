#[macro_use]
extern crate rocket;
#[macro_use]
extern crate bson;
#[macro_use]
extern crate lazy_static;
extern crate dotenv;
extern crate mongodb;
extern crate rocket_db_pools;
extern crate serde_derive;

pub mod db;
pub mod http;
pub mod models;

mod database;
mod oauth;
mod user;

use db::{MongoDB, Redis};
use rocket::fs::{relative, FileServer};
use rocket_db_pools::Database;

use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{routes, Request, Response, Rocket};

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new(
            "Access-Control-Allow-Origin",
            "dashboard.squid.pink",
        ));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new(
            "Access-Control-Allow-Headers",
            "dashboard.squid.pink",
        ));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[options("/<_..>", rank = 7)]
async fn cors() -> String {
    "".to_string()
}

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();

    let production =
        std::env::var("ROCKET_ENV").unwrap_or("production".to_string()) == "production";
    println!("Running in production: {}", production);
    let r: Rocket<_> = rocket::build()
        .attach(CORS)
        .attach(MongoDB::init())
        .attach(Redis::init())
        .mount("/static", FileServer::from(relative!("static")).rank(9))
        .mount("/static", routes![cors])
        .mount("/api/oauth", oauth::routes())
        .mount("/api/", database::routes())
        .mount("/api/user", user::routes());

    let r = if !production {
        println!("mounting frontend dist");
        r.mount("/", FileServer::from(relative!("../client/dist")).rank(8))
            .mount("/api", routes![cors])
    } else {
        r
    };
    r
}

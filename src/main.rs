#[macro_use]
extern crate rocket;
use std::fs;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[rocket::main]
async fn main() {
    fs::create_dir_all("storage/").unwrap_or_else(|err| {
        if err.kind() != std::io::ErrorKind::AlreadyExists {
            panic!("Failed to create storage directory: {}", err);
        }
    });

    rocket::build()
        .mount("/", routes![index])
        .launch()
        .await
        .unwrap_or_else(|e| panic!("Failed to launch the rocket: {}", e));
}

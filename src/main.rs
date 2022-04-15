#[macro_use]
extern crate rocket;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::data::{Data, ToByteUnit};
use rocket::http::{Status, ContentType};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::tokio;
use std::fs::{self, File};
use std::path::Path;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/create", data = "<data>")]
async fn create(data: Data<'_>, content_length: ContentLength) -> Result<String, std::io::Error> {
    let name = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();

    let ext = "txt";

    let file = tokio::fs::File::create(Path::new(format!("storage/{}.{}", name, ext))).await?;
    let stream = data.open(512.kibibytes());

    return Ok("".into());
}

#[rocket::main]
async fn main() {
    fs::create_dir_all("storage/").unwrap_or_else(|err| {
        if err.kind() != std::io::ErrorKind::AlreadyExists {
            panic!("Failed to create storage directory: {}", err);
        }
    });

    rocket::build()
        .mount("/", routes![index, create])
        .launch()
        .await
        .unwrap_or_else(|e| panic!("Failed to launch the rocket: {}", e));
}

struct ContentLength(u64);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ContentLength {
    type Error = String;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let length = request
            .headers()
            .get_one("content-length")
            .unwrap_or_else(|| "".into());
        let length_as_num = length.parse::<u64>();
        match length_as_num {
            Ok(length_as_num) => Outcome::Success(ContentLength(length_as_num)),
            _ => Outcome::Failure((
                Status::InternalServerError,
                "Failed to parse content length".into(),
            )),
        }
    }
}

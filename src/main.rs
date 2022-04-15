#[macro_use]
extern crate rocket;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::data::{Data, ToByteUnit};
use rocket::http::ContentType;
use rocket::tokio;
use rocket_multipart_form_data::{
    mime, MultipartFormData, MultipartFormDataField, MultipartFormDataOptions, Repetition,
};
use std::fs;
use std::path::Path;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/create", data = "<data>")]
async fn create(data: Data<'_>, content_type: &ContentType) -> Result<String, std::io::Error> {
    let mut options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::file("file")
            .content_type_by_string(Some(mime::STAR))
            .unwrap(),
        MultipartFormDataField::text("ext")
            .content_type_by_string(Some(mime::TEXT_PLAIN))
            .unwrap(),
    ]);

    let mut form_data = MultipartFormData::parse(content_type, data, options)
        .await
        .unwrap();

    let file_field = form_data.files.get("file");
    let ext = form_data.texts.remove("ext");

    if let Some(file_fields) = file_field {
        let file = &file_fields[0];

        let f_content_type = &file.content_type;
        let f_file_name = &file.file_name;
        let f_path = &file.path;

        let salt = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        

        let file = tokio::fs::File::create(Path::new(format!("storage/{}.{}", name, ext))).await?;
        let stream = data.open(512.kibibytes());

        return Ok("".into());
    }
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

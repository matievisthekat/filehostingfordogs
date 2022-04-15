#[macro_use]
extern crate rocket;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::data::{Data, ToByteUnit};
use rocket::http::ContentType;
use rocket::tokio;
use rocket_multipart_form_data::{
    mime, MultipartFormData, MultipartFormDataField, MultipartFormDataOptions, TextField,
};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

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
    let ext_field = form_data.texts.remove("ext");

    if let Some(ext_fields) = ext_field {
        // ! start defining ext ! \\
        // sorry this is so long
        let ext = ext_field.unwrap_or(vec![TextField {
            content_type: None,
            file_name: None,
            text: "".to_string(),
        }])[0]
            .text;
        // ! end defining ext ! \\

        if let Some(file_fields) = file_field {
            let file = file_fields[0];

            let f_content_type = file.content_type.unwrap_or("*".parse().unwrap());
            let f_file_name = file.file_name.unwrap_or("unknown".into());
            let f_path = file.path;

            println!("{}", f_path.display());

            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards?")
                .as_secs();
            let salt = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(10)
                .map(char::from)
                .collect();
            let encoded_name = base64::encode(format!(
                "{original_name}::{content_type}::{timestamp}::{salt}",
                original_name = f_file_name[0..10].to_string(),
                content_type = f_content_type,
                timestamp = timestamp,
                salt = salt
            ));

            let new_file_path = Path::new(format!("storage/{}.{}", &encoded_name, &ext));
            let file = tokio::fs::File::create(new_file_path).await?;
            let stream = data.open(512.kibibytes());

            return Ok("".into());
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Missing 'file' input field",
            ));
        }
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

#[macro_use]
extern crate rocket;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::data::Data;
use rocket::http::ContentType;
use rocket::response::status::{BadRequest, Created};
use rocket_multipart_form_data::{
    MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/create", data = "<data>")]
async fn create(
    data: Data<'_>,
    content_type: &ContentType,
) -> Result<Created<String>, BadRequest<String>> {
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::file("file")
            .content_type_by_string(Some("*/*"))
            .unwrap(),
        MultipartFormDataField::text("ext")
            .content_type_by_string(Some("text/plain"))
            .unwrap(),
    ]);
    let mut form_data = MultipartFormData::parse(content_type, data, options)
        .await
        .unwrap_or_else(|e| panic!("{}", e));

    let file_field = form_data.files.get("file");
    let ext_field = form_data.texts.remove("ext");

    if let Some(ext_fields) = ext_field {
        let ext = &ext_fields[0].text;

        if let Some(file_fields) = file_field {
            let file = &file_fields[0];

            let f_content_type = &file.content_type;
            let f_file_name = &file.file_name;
            let f_path = &file.path;

            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards?")
                .as_secs();
            let salt = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(10)
                .map(char::from)
                .collect::<String>();
            let encoded_name = base64::encode_config(
                format!(
                    "{original_name}::{cnt_type}::{timestamp}::{salt}",
                    original_name =
                        f_file_name.as_ref().unwrap_or(&String::from("unknown"))[0..10].to_string(),
                    cnt_type = f_content_type.as_ref().unwrap().to_string(), // should default to application/octet-stream
                    timestamp = timestamp,
                    salt = salt
                ),
                base64::URL_SAFE_NO_PAD,
            );

            let new_file_path = PathBuf::from(format!("storage/{}.{}", &encoded_name, &ext));
            fs::copy(f_path, new_file_path).expect("Failed to copy file into `storage/` folder");

            return Ok(Created::new(format!("/f/{}", &encoded_name)));
        } else {
            return Err(BadRequest(Some("Missing 'file' input field".into())));
        }
    } else {
        return Err(BadRequest(Some("Missing 'ext' input field".into())));
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

#[macro_use]
extern crate rocket;
use rocket::data::ToByteUnit;
use rocket::http::{ContentType, Status};
use rocket::response::status::Custom;
use rocket::Data;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[get("/")]
fn index() -> &'static str {
  "Hello, world!"
}

#[post("/create?<ext>&<og_name>", data = "<data>", format = "*/*")]
async fn create(
  data: Option<Data<'_>>,
  ext: Option<String>,
  og_name: Option<String>,
  content_type: &ContentType,
) -> Result<String, Custom<String>> {
  let content_type_string = content_type.to_string();
  let split_string = content_type_string.split(";").collect::<Vec<&str>>();
  let clean_content_type = split_string.get(0).unwrap();

  if let Some(ext) = ext {
    if let Some(data) = data {
      let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards?")
        .as_secs();
      let encoded_name = base64::encode_config(
        format!(
          "{original_name}::{cnt_type}::{timestamp}",
          original_name = og_name
            .unwrap_or("unknown".into())
            .get(0..7)
            .unwrap_or("unknown".into()) // this is quite easily the ugliest piece of code i've ever written
            .to_string(),
          cnt_type = clean_content_type.to_string(),
          timestamp = timestamp,
          // salt = salt
        ),
        base64::URL_SAFE_NO_PAD,
      );
      let new_file_path = PathBuf::from(format!("storage/{}.{}", &encoded_name, &ext));
      let file = rocket::tokio::fs::File::create(&new_file_path).await;

      if let Ok(file) = file {
        let res = data.open(5.gigabytes()).stream_to(file).await;

        if let Ok(_) = res {
          // return Ok(Created::new(format!("/f/{}", &encoded_name)));
          return Ok(format!("/f/{}", &encoded_name));
        } else if let Err(e) = res {
          return Err(Custom(
            Status::InternalServerError,
            format!("Failed to stream data to file: {}", e),
          ));
        } else {
          return Err(Custom(
            Status::InternalServerError,
            "Unreachable error was reached. Please report this".to_string(),
          ));
        }
      } else {
        return Err(Custom(
          rocket::http::Status::InternalServerError,
          "Could not create file".into(),
        ));
      }
    } else {
      return Err(Custom(
        Status::BadRequest,
        "Missing 'file' input field".into(),
      ));
    }
  } else {
    return Err(Custom(
      Status::BadRequest,
      "Missing '?ext=<file_extension>' query param".into(),
    ));
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

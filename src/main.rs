#[macro_use]
extern crate rocket;
use rocket::data::{Limits, ToByteUnit};
use rocket::fs::NamedFile;
use rocket::http::{ContentType, Status};
use rocket::response::status::Custom;
use rocket::serde::{json::Json, Serialize};
use rocket::Data;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct File {}

#[get("/")]
fn index() -> Json<Vec<File>> {
  let files = vec![];


  
  return Json(files);
}

#[get("/f/<file>")]
async fn get_file(file: String) -> Result<NamedFile, Custom<String>> {
  let path = PathBuf::from(format!("storage/{}", file));
  let named_file = NamedFile::open(path).await;

  match named_file {
    Ok(named_file) => Ok(named_file),
    Err(_) => Err(Custom(Status::NotFound, "File not found".to_string())),
  }
}

#[post("/create?<ext>&<og_name>", data = "<data>", format = "*/*")]
async fn create(
  data: Option<Data<'_>>,
  ext: Option<String>,
  og_name: Option<String>,
  content_type: &ContentType,
  limits: &Limits,
) -> Result<String, Custom<String>> {
  let content_type_string = content_type.to_string();
  let split_string = content_type_string.split(";").collect::<Vec<&str>>();
  let clean_content_type = split_string.get(0).unwrap();

  let limit = limits.get("data").unwrap_or(1.gibibytes());

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
        ),
        base64::URL_SAFE_NO_PAD,
      );
      let new_file_path = PathBuf::from(format!("storage/{}.{}", &encoded_name, &ext));

      let res = data.open(limit).into_file(new_file_path).await;

      if let Ok(_) = res {
        return Ok(format!("/f/{}.{}", &encoded_name, &ext));
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
    .mount("/", routes![index, create, get_file])
    .launch()
    .await
    .unwrap_or_else(|e| panic!("Failed to launch the rocket: {}", e));
}

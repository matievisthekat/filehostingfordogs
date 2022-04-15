#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Who's a good boi?"
}

#[launch]
fn main() {
    rocket::build().mount("/", routes![index])
}

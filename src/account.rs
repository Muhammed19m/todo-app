use rocket::{http::ContentType, response::Responder, FromForm, Response};
use serde::{Deserialize, Serialize};

#[derive(Clone, FromForm, Serialize, Deserialize, Hash, PartialEq, Eq)]

pub struct Account {
    pub login: String,
    pub password: String,
}

impl Account {
    pub fn new(login: String, password: String) -> Account {
        Account { login, password }
    }
}

impl<'r> Responder<'r, 'static> for Account {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let string = format!("login: {}\npassword: {}", self.login, self.password);
        Response::build_from(string.respond_to(request)?)
            .raw_header("Login", self.login)
            .raw_header("Password", self.password)
            .header(ContentType::JSON)
            .ok()
    }
}

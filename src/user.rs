use crate::*;
use account::Account;
use data::*;
use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, Message,
    Tokio1Executor,
};
use rocket::{serde::json::Json, FromForm, State};
use uuid::Uuid;

use sqlx::{self, MySql, Pool};

#[post("/login", data = "<user>")]
pub async fn login(
    user: Json<UserInput>,
    conf_users: &State<Mutex<HashMap<String, User>>>,
) -> String {
    /*отправляем на письмо ссылку http://localhost:8000/confirmed/some_code */
    let code = Uuid::new_v4().simple();
    let email = if let Ok(a) = format!("{} <{}>", user.name, user.login).parse() {
        a
    } else {
        return "не верная почта".into();
    };
    let email = Message::builder()
        .from("TODOAPP <nobody@domain.tld>".parse().unwrap())
        .to(email)
        .subject("login")
        .body(format!("{}/confirmed/{}\n", HOST, code))
        .unwrap();

    let creds = Credentials::new(EMAIL.to_string(), PASSWORD.to_string());
    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay("smtp.gmail.com")
            .unwrap()
            .credentials(creds)
            .build();
    match mailer.send(email).await {
        Ok(_) => {
            conf_users
                .lock()
                .await
                .insert(code.to_string(), User::from(user.into_inner()));
            "письмо на почту отправлено".into()
        }
        Err(_) => "ошибка при отправке письма".into(),
    }
}

#[get("/confirmed/<code>")]
pub async fn confirm(
    code: String,
    conf_users: &State<Mutex<HashMap<String, User>>>,
    db: &State<Pool<MySql>>,
) -> String {
    match conf_users.lock().await.remove(&code) {
        Some(User { name, account }) => {
            let Account {
                login, password, ..
            } = account;
            let db = db.inner();

            let sql = format!(
                "insert into todo_app.account (name, login, password) values ('{}', '{}', '{}')",
                name, login, password
            );
            match sqlx::query(&sql).execute(db).await {
                Ok(_) => "зарегестрировано".into(),
                Err(_) => "ошибка регистрации или аккаунт с этой почтой уже зарегестрирован, попробуйте зайти".into(),
            }
        }
        None => "не верный код".into(),
    }
}

// #[post("/signin", data = "<account>")]
// pub async fn signin(account: Form<Account>) -> String {
//     unimplemented!()
// }

#[derive(FromForm, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct User {
    pub name: String,
    pub account: Account,
}
impl User {
    fn new(name: String, account: Account) -> User {
        User { name, account }
    }
}
impl From<UserInput> for User {
    fn from(usin: UserInput) -> Self {
        User::new(usin.name, Account::new(usin.login, usin.password))
    }
}

#[derive(FromForm, Serialize, Deserialize)]
pub struct UserInput {
    name: String,
    login: String,
    password: String,
}

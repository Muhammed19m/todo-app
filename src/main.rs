use rocket::{
    self, get, post, routes,
    serde::{Deserialize, Serialize},
};
mod account;
mod task;
mod user;
use sqlx::mysql::MySqlPoolOptions;
use task::{add_task, delete_task, edit_task, get_tasks};
use user::{confirm, login, User};
mod data;

use rocket::tokio::sync::Mutex;
use std::collections::HashMap;

#[rocket::launch]
async fn rocket() -> _ {
    let conf_users = Mutex::new(HashMap::<String, User>::new());

    let db = MySqlPoolOptions::new()
        .max_connections(100)
        .connect(data::HOST_DB)
        .await
        .expect("Ошибка подключеник к база данных");

    rocket::build()
        .mount(
            "/",
            routes![login, confirm, add_task, get_tasks, edit_task, delete_task],
        )
        .manage(conf_users)
        .manage(db)
}

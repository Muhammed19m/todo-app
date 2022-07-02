#![allow(unused)]

use rocket::{post, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use sqlx::{self, FromRow, MySql, Pool};

use crate::{account::Account, user::User};

#[post("/task", data = "<new_task>")]
pub async fn add_task(new_task: Json<InputUserTask>, db: &State<Pool<MySql>>) -> String {
    let InputUserTask {
        user: User {
            name,
            account: Account { login, password },
        },
        task,
    } = new_task.into_inner();
    let InputTask {
        task_id,
        name,
        description,
        date,
        performed,
    } = task;
    let description = description.map(|value| format!("'{}'", value));
    let date = date.map(|value| format!("'{}'", value));

    let sql = format!("insert into todo_app.task(id, name, description, date, performed) values((select id from todo_app.account where login = '{}' and password = '{}'), '{}', {}, {}, {}); ",login,password,name,description.unwrap_or("null".into()), date.unwrap_or("null".into()), performed);
    match sqlx::query(&sql).execute(db.inner()).await {
        Ok(_) => "задача добавлена".into(),
        Err(e) => {
            dbg!(e);
            "ошибка при добавлении задачи, проверьте верность данных".into()
        }
    }
}

#[post("/tasks", data = "<user>")]
pub async fn get_tasks(user: Json<User>, db: &State<Pool<MySql>>) -> Json<Vec<Task>> {
    let User { name, account } = user.into_inner();
    let Account { login, password } = account;
    let sql = format!(
        "SELECT task.* from todo_app.task, todo_app.account 
    where account.login = '{login}' and 
    account.password = '{password}' and 
    task.id = account.id"
    );
    match sqlx::query_as::<_, Task>(&sql).fetch_all(db.inner()).await {
        Ok(tasks) => Json::from(tasks),
        Err(_) => Json::from(Vec::<Task>::with_capacity(0)),
    }
}

#[post("/edittask", data = "<ustk>")]
pub async fn edit_task(ustk: Json<InputUserTask>, db: &State<Pool<MySql>>) -> String {
    let InputUserTask {
        user: User {
            name,
            account: Account { login, password },
        },
        task,
    } = ustk.into_inner();
    let InputTask {
        task_id,
        name,
        description,
        date,
        performed,
    } = task;
    let description = description.map(|value| format!("'{}'", value));
    let date = date.map(|value| format!("'{}'", value));

    let sql = format!(
        "update todo_app.task
    set name = '{}',
        description = {},
        date = {},
        performed = {}
    where (select id from todo_app.account where account.login = '{}' 
    and account.password = '{}'
    and account.id) = id and task_id = {}",
        name,
        description.unwrap_or("null".into()),
        date.unwrap_or("null".into()),
        performed,
        login,
        password,
        task_id
    );

    match sqlx::query(&sql).execute(db.inner()).await {
        Ok(res) => {
            if res.rows_affected() == 1 {
                "успешно изменено".into()
            } else {
                "ошибка изменения".into()
            }
        }
        Err(_) => "ошибка изменения".into(),
    }
}

#[post("/taskdel", data = "<user>")]
pub async fn delete_task(user: Json<InputDeleteTask>, db: &State<Pool<MySql>>) -> String {
    let InputDeleteTask {
        user: User {
            name,
            account: Account { login, password },
        },
        id_task,
    } = user.into_inner();
    let sql = format!(
        "delete from todo_app.task
    where (select id from todo_app.account where account.login = '{login}' 
    and account.password = '{password}'
    and account.id) = id
    and task_id = {id_task}"
    );
    match sqlx::query(&sql).execute(db.inner()).await {
        Ok(res) => {
            if res.rows_affected() == 1 {
                "успешно удалено".into()
            } else {
                "ошибка удаления".into()
            }
        }
        Err(_er) => "ошибка удаления".into(),
    }
}

#[derive(Deserialize, Serialize)]
pub struct InputDeleteTask {
    user: User,
    id_task: u32,
}

#[derive(FromRow, Deserialize, Serialize)]
pub struct Task {
    id: u32,
    task_id: u32,
    name: String,
    description: Option<String>,
    date: Option<String>,
    performed: bool,
}

#[derive(Deserialize, Serialize)]
pub struct InputTask {
    task_id: u32,
    name: String,
    description: Option<String>,
    date: Option<String>,
    performed: bool,
}

#[derive(Deserialize, Serialize)]
pub struct InputUserTask {
    user: User,
    task: InputTask,
}

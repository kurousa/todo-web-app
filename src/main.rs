use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use askama::Template;
use askama_actix::TemplateToResponse;
use sqlx::{Row, SqlitePool};

#[derive(Template)]
#[template(path = "todo.html")]
struct TodoTemplate {
    tasks: Vec<String>,
}

#[derive(serde::Deserialize)]
struct Task {
    id: Option<String>,
    task: Option<String>,
}

#[get("/")]
async fn todo(pool: web::Data<SqlitePool>) -> HttpResponse {
    let rows = sqlx::query("SELECT task FROM tasks")
        .fetch_all(pool.as_ref())
        .await
        .unwrap();
    let tasks: Vec<String> = rows
        .iter()
        .map(|row| row.get::<String, _>("task"))
        .collect();
    let todo = TodoTemplate { tasks };
    todo.to_response()
}
#[post("/update")]
async fn update(pool: web::Data<SqlitePool>, form: web::Form<Task>) -> HttpResponse {
    let task = form.into_inner();

    if let Some(id) = task.id {
        sqlx::query("DELETE FROM tasks WHERE task = ?")
            .bind(id)
            .execute(pool.as_ref())
            .await
            .unwrap();
    }
    match task.task {
        Some(task) if !task.is_empty() => {
            sqlx::query("INSERT INTO tasks (task) VALUES (?)")
                .bind(task)
                .execute(pool.as_ref())
                .await
                .unwrap();
        }
        _ => {}
    }
    // Redirect to the main page
    HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    sqlx::query("CREATE TABLE tasks (task TEXT)")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO tasks (task) VALUES ('Task 1')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO tasks (task) VALUES ('Task 2')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO tasks (task) VALUES ('Task 3')")
        .execute(&pool)
        .await
        .unwrap();

    println!("Server running at localhost:8080");
    HttpServer::new(move || {
        App::new()
            .service(todo)
            .service(update)
            .app_data(web::Data::new(pool.clone()))
    })
    .bind("localhost:8080")?
    .run()
    .await
}

use std::env::var;

use actix_web::{
    get, middleware, post,
    web::{self, Data, Json},
    App, HttpServer, Responder,
};
use env_logger::Env;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
struct DataRow {
    pub id: i32,
    pub value: f32,
}

#[get("/data")]
async fn get_all_data(data: Data<PgPool>) -> impl Responder {
    let result = sqlx::query_as!(DataRow, "SELECT * FROM data")
        .fetch_all(&**data)
        .await
        .unwrap();

    Json(result)
}

#[post("/data/{id}/{value}")]
async fn post_data(data: Data<PgPool>, info: web::Path<DataRow>) -> impl Responder {
    sqlx::query!(
        "INSERT INTO data (id, value) VALUES ($1, $2)",
        info.id,
        info.value
    )
    .execute(&**data)
    .await
    .unwrap();

    format!("Inserted data with id: {}, value: {}", info.id, info.value)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let database_url = var("DATABASE_URL").unwrap();
    let port = var("PORT").unwrap();
    let pool = PgPool::connect(&database_url).await.unwrap();
    let data = Data::new(pool);

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(middleware::Logger::default())
            .service(web::resource("/").to(|| async { "Hello world!" }))
            .service(get_all_data)
            .service(post_data)
    })
    .bind(format!("0.0.0.0:{port}"))?
    .run()
    .await
}

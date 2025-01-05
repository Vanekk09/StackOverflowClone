use dotenvy::dotenv;
use sqlx::postgres::{PgPoolOptions};
use sqlx::{Row};
use std::error::Error;
use axum::{
    routing::{delete, get, post},
    Router,
};

extern crate pretty_env_logger;
#[macro_use] extern crate log;

mod handlers;
mod models;
mod persistance;

use handlers::*;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    dotenv().ok();

    let connection_string = std::env::var("DATABASE_URL").unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&connection_string)
        .await
        .expect("Failed to create Postgres connection pool!");

    let app = Router::new()
        .route("/question", post(create_question))
        .route("/questions", get(read_questions))
        .route("/question", delete(delete_question))
        .route("/answer", post(create_answer))
        .route("/answers", get(read_answers))
        .route("/answer", delete(delete_answer));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
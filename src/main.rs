use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use tokio_postgres::{Error, row};
use tokio_postgres::NoTls;
use actix_web::{get, post, web, web::Data, App, HttpResponse, HttpServer};
use actix_web::{Responder, HttpRequest, http, middleware, middleware::Logger, ResponseError};
use futures::future;
use futures::Future;
use uuid::Uuid;
use deadpool_postgres::{Config, Client, Pool};
use openssl::ssl::{SslConnector, SslMethod};
use postgres_openssl::*;
use tokio::runtime::Runtime;
use ::config::ConfigError;
use dotenv::dotenv;
use tokio_postgres::tls::MakeTlsConnect;
use derive_more::{Display,From};
use tokio_postgres::types::Json;
use regex::Regex;
use serde_json::{json, to_string, Value};
use dirs;
extern crate emoji_logger;

use async_std::task;
use std::time::Duration;
use actix_web::web::Query;
use futures::{join, select, StreamExt};
use futures::future::FutureExt;
use futures::stream;
use futures::pin_mut;
use futures::try_join;
use async_std;
use crate::app_error::CustomError;
use crate::app_utils::sanitize_string;
use crate::data_structs::{Data1, QueryParams};
use crate::db_crud::http_ops_data::{http_delete_record_data, http_fetch_record_data, http_upsert_record_data};
use crate::db_ops::make_db_pool;
use crate::pg_ops_advanced::get_data_with_advanced_query;
use crate::pg_ops_basic::get_data_with_basic_query;

mod app_error;
mod data_structs;
mod app_utils;
mod db_ops;

mod pg_ops_basic;
mod pg_ops_advanced;

mod db_crud;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_env_file = "app.rust.env".to_string();

    let file_path = format!("/etc/secrets/{}", app_env_file);

    dotenv::from_filename(file_path).ok();

    std::env::set_var("RUST_LOG", "actix_web=debug");
    emoji_logger::init();

    let pool = make_db_pool().await;

    let result = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(Data::new(pool.clone())) // example of passing app-data
            .service(api_check)
            .route("/api/v1/fetch_with_basic_query", web::get().to(get_data_with_basic_query))
            .route("/api/v1/fetch_with_query_advanced", web::get().to(get_data_with_advanced_query))
            .route("/api/v1/{backend_data}", web::post().to(http_upsert_record_data))
            .route("/api/v1/{backend_data}/{record_identifier}", web::delete().to(http_delete_record_data))
            .route("/api/v1/{backend_data}/{record_identifier}", web::get().to(http_fetch_record_data))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await;
    println!("🚨 Sample Service Stopping");
    result
}

#[get("/api/health")]
async fn api_check() -> impl Responder {
    HttpResponse::Ok().body("{\"status\": \"ok\"}")
}


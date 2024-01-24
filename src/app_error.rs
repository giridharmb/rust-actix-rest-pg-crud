use serde::{Deserialize, Serialize};
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


#[derive(Debug)]
pub enum GenericError {
    InternalServerError,
    SerializationFailed,
    HttpRequestFailed,
    EndpointNotFound,
    TokenNotFound,
    TenantQueryFailed,
    HypervisorQueryFailed,
}

#[derive(Debug)]
pub struct CustomErrorType {
    pub err_type: GenericError,
    pub err_msg: String
}

#[derive(Display, From, Debug)]
pub enum CustomError {
    DatabaseError,
    InvalidData,
    QueryError,
    InvalidTable,
}

impl std::error::Error for CustomError {}

impl ResponseError for CustomError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            CustomError::DatabaseError => HttpResponse::InternalServerError().finish(),
            CustomError::InvalidData => HttpResponse::BadRequest().finish(),
            CustomError::QueryError => HttpResponse::BadRequest().finish(),
            CustomError::InvalidTable => HttpResponse::BadRequest().finish(),
        }
    }
}

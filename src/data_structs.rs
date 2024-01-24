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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Data1 {
    // Define your data structure
    pub random_num: Option<i32>,
    pub random_float: Option<f64>,
    pub md5: Option<String>,
    pub record_id: Option<String>, // primary-key on PG
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Data2 {
    // Define your data structure
    pub data_1: Option<String>,
    pub data_2: Option<String>,
    pub record_id: Option<String>, // primary-key on PG
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum UserData {
    Data1(Data1),
    Data2(Data2),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PathInfoV1 {
    pub record_identifier: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PathInfoV2 {
    pub backend_data: String,
    pub record_identifier: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PathInfoV3 {
    pub backend_data: String,
}

#[derive(Deserialize)]
pub struct QueryParams {
    pub string_match: String, // 'like' or 'exact'
    pub search_string: String, // 'xyz' or '123-xyz' > basically any search string
}

#[derive(Deserialize)]
pub struct QueryParamsAdvanced {
    //----------------------------------------------------
    // examples of value for (string_match)
    // 'like' or 'exact'
    pub string_match: String,
    //----------------------------------------------------
    // examples of value for (search_type)
    // this can be 'or' (OR) 'and'
    // 'or' means , search for 'abc' | 'xyz' etc
    // 'and' means , search for 'abc' + 'xyz'
    pub search_type: String,
    //----------------------------------------------------
    // examples of value for (search_string) > basically any search string
    // 'xyz'
    // '123-xyz'
    // 'xyz,555,abc-123' (',' delimited values : for multiple values)
    // if you gave (search_type=and), then search the table for all columns where : 'xyz' AND '555' AND 'abc-123' is present
    // if you gave (search_type=or), then search the table for all columns where : 'xyz' OR '555' OR 'abc-123' is present
    pub search_string: String,
    //----------------------------------------------------
    pub source_table: String,
}

#[derive(Deserialize)]
pub struct QueryParamsBasic {
    pub source_table: String,
}

impl QueryParamsAdvanced {
    pub fn validate(&self) -> bool {
        return !self.search_type.is_empty() && !self.string_match.is_empty() && !self.search_string.is_empty()
    }
}


#[derive(Serialize, Deserialize)]
pub struct ServerResponse {
    pub result: String,
    pub message: String,
    pub server_data: Value,
}
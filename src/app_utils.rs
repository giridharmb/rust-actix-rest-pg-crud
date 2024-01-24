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

/*
Testing sanitize_string(...)

fn main() {
    let original_string = "Hello! This is a test string with various characters: #$%^&*()";
    let sanitized = sanitize_string(original_string);
    println!("Sanitized string: {}", sanitized);
}
*/

pub async fn sanitize_string(input: &str) -> String {
    input.chars()
        .filter(|&c| c.is_ascii_alphanumeric() || "_./-@,#:;".contains(c))
        .collect()
}

pub async fn remove_leading_trailing_characters(input: &str) -> String {
    input.trim_start_matches(",").trim_end_matches(",").to_string()
}

pub async fn replace_multiple_characters(input: &str) -> String {
    let re = Regex::new(r",+").unwrap();
    re.replace_all(input, ",").to_string()
}

pub async fn split_string(input: &str) -> Vec<String> {
    input.split(",").map(|s| s.to_string()).collect()
}

/*
What get_items(...) Does >>

let original_string = "###abc#555#xyz-123###";
let my_items = get_items(original_string);

let original_string = "abcd";
let my_items = get_items(original_string);

let original_string = "###abc##555####xyz-123#3-2-1#a-b-c###";
let my_items = get_items(original_string);

Output:

updated_str : abc#555#xyz-123
validated_updated_str : abc#555#xyz-123
my_list : ["abc", "555", "xyz-123"]


updated_str : abcd
validated_updated_str : abcd
my_list : ["abcd"]


updated_str : abc##555####xyz-123#3-2-1#a-b-c
validated_updated_str : abc#555#xyz-123#3-2-1#a-b-c
my_list : ["abc", "555", "xyz-123", "3-2-1", "a-b-c"]
*/
pub async fn get_items(my_str: &str) -> Vec<String> {
    let sanitized_str = sanitize_string(my_str).await;

    let updated_str = remove_leading_trailing_characters(sanitized_str.as_str()).await;
    println!("updated_str : {}", updated_str);

    let validated_updated_str = replace_multiple_characters(updated_str.as_str()).await;
    println!("validated_updated_str : {}", validated_updated_str);

    let my_list = split_string(validated_updated_str.as_str()).await;
    println!("my_list : {:?}", my_list);

    println!("\n");

    my_list
}

pub async fn is_only_spaces(my_str: &str) -> bool {
    my_str.trim().is_empty()
}
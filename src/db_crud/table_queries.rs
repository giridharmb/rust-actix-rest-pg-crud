use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use tokio_postgres::{Error, row, Row};
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
use crate::data_structs::{Data1, Data2, UserData};

pub async fn get_insert_query_for_backend_type(backend_data: &str) -> Result<String, Box<dyn std::error::Error>> {
    return match backend_data {
        "data_1" => {
            let my_query = r#"
                INSERT INTO table_v1 (
                    random_num,
                    random_float,
                    md5,
                    record_id
                )
                VALUES (
                    $1,
                    $2,
                    $3,
                    $4
                ) ON CONFLICT (record_id)
                DO UPDATE SET
                    random_num = EXCLUDED.random_num,
                    random_float = EXCLUDED.random_float,
                    md5 = EXCLUDED.md5,
                    record_id = EXCLUDED.record_id;
            "#;
            Ok(my_query.to_string())
        },
        "data_2" => {
            let my_query = r#"
                INSERT INTO table_v2 (
                    data_1,
                    data_2,
                    record_id
                )
                VALUES (
                    $1,
                    $2,
                    $3
                ) ON CONFLICT (record_id)
                DO UPDATE SET
                    data_1 = EXCLUDED.data_1,
                    data_2 = EXCLUDED.data_2,
                    record_id = EXCLUDED.record_id;
            "#;
            Ok(my_query.to_string())
        },
        _ => {
            let msg = format!("invalid backend : {}", backend_data);
            return Err(Box::from(msg))
        }
    };
}

pub async fn get_delete_query_for_backend_type(backend_data: &str) -> Result<String, Box<dyn std::error::Error>> {
    return match backend_data {
        "data_1" => {
            let my_query = r#"
                DELETE FROM table_v1 WHERE record_id = $1 RETURNING random_num, random_float, md5, record_id
            "#;
            Ok(my_query.to_string())
        },
        "data_2" => {
            let my_query = r#"
                DELETE FROM table_v2 WHERE record_id = $1 RETURNING data_1, data_2, record_id
            "#;
            Ok(my_query.to_string())
        },
        _ => {
            let msg = format!("invalid backend : {}", backend_data);
            return Err(Box::from(msg))
        },
    };
}

pub async fn get_select_query_for_backend_type(backend_data: &str) -> Result<String, Box<dyn std::error::Error>> {
    return match backend_data {
        "data_1" => {
            let my_query = r#"
                SELECT
                    random_num,
                    random_float,
                    md5,
                    record_id
                FROM
                    table_v1
                WHERE record_id = $1
            "#;
            Ok(my_query.to_string())
        },
        "data_2" => {
            let my_query = r#"
                SELECT
                    data_1,
                    data_2,
                    record_id
                FROM
                    table_v2
                WHERE record_id = $1
            "#;
            Ok(my_query.to_string())
        },
        _ => {
            let msg = format!("invalid backend : {}", backend_data);
            return Err(Box::from(msg))
        }
    };
}

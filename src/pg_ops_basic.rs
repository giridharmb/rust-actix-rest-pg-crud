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
use crate::data_structs::{Data1, Data2, QueryParamsBasic};
use crate::db_ops::get_db_pool_for_table;

pub async fn get_data_with_basic_query(query: web::Query<QueryParamsBasic>) -> Result<HttpResponse, CustomError> {

    let actual_db_table = match query.source_table.as_str() {
        "data_1" => {
            "table_v1".to_string() // actual backend table on postgresql
        },
        "data_2" => {
            "table_v2".to_string() // actual backend table on postgresql
        },
        _ => {
            return Err(CustomError::QueryError)
        }
    };
    let my_db_pool = get_db_pool_for_table(query.source_table.as_str()).await.unwrap();

    let client: Client = my_db_pool.get().await.unwrap();

    if query.source_table.is_empty() {
        return Err(CustomError::QueryError);
    }

    let mut structs_1:Vec<Data1> = Vec::new();
    let mut structs_2:Vec<Data2> = Vec::new();

    let complete_query = format!("SELECT * from {}", actual_db_table);

    println!("complete_query >> \n\n{}\n\n",complete_query);

    let rows = client.query(complete_query.as_str(), &[]).await.map_err(|_| CustomError::DatabaseError).unwrap();

    match query.source_table.as_str() {
        "data_1" => {
            for row in rows {
                let random_num: i32 = row.try_get("random_num").unwrap_or_else(|_| 0);
                let random_float: f64 = row.try_get("random_float").unwrap_or_else(|_| 0.0);
                let md5: String = row.try_get("md5").unwrap_or_else(|_| "missing_random_num".to_string());
                let record_id: String = row.try_get("record_id").unwrap_or_else(|_| "missing_record_id".to_string());

                let my_struct = Data1 {
                    random_num: Some(random_num),
                    random_float: Some(random_float),
                    md5: Some(md5),
                    record_id: Some(record_id),
                };
                structs_1.push(my_struct)
            }
        },
        "data_2" => {
            for row in rows {
                let data_1: String = row.try_get("data_1").unwrap_or_else(|_| "missing_data_1".to_string());
                let data_2: String = row.try_get("data_2").unwrap_or_else(|_| "missing_data_2".to_string());
                let record_id: String = row.try_get("record_id").unwrap_or_else(|_| "missing_record_id".to_string());


                let my_struct = Data2 {
                    data_1: Some(data_1),
                    data_2: Some(data_2),
                    record_id: Some(record_id),
                };

                structs_2.push(my_struct)
            }
        },
        _ => {
            return Err(CustomError::InvalidTable);
        }
    };

    return match query.source_table.as_str() {
        "data_1" => {
            Ok(HttpResponse::Ok().content_type("application/json").body(serde_json::to_string(&structs_1).unwrap()))
        },
        "data_2" => {
            Ok(HttpResponse::Ok().content_type("application/json").body(serde_json::to_string(&structs_2).unwrap()))
        },
        _ => {
            return Err(CustomError::InvalidTable);
        }
    };
}

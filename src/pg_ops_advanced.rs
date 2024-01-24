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
use crate::app_utils::get_items;
use crate::data_structs::{Data1, Data2, QueryParamsAdvanced};
use crate::db_ops::{get_db_pool_for_table, get_inner_query};

async fn get_data_with_advanced_single_query(query: QueryParamsAdvanced) -> Result<String, CustomError> {
    if !query.validate() {
        return return Err(CustomError::QueryError);
    }

    let mut actual_db_table = "".to_string();

    let query_table = query.source_table.to_owned();

    actual_db_table = match query_table.as_str() {
        "data_1" => "table_v1".to_string(), // actual backend table on PG
        "data_2" => "table_v2".to_string(), // actual backend table on PG
        _ => return Err(CustomError::QueryError)
    };

    let my_db_pool = get_db_pool_for_table(query.source_table.as_str()).await.unwrap();

    let client: Client = my_db_pool.get().await.unwrap();

    println!("query >> \n\nstring_match => [ {} ] , \n\nsearch_string => [ {} ]\n\n", query.string_match, query.search_string);

    if query.source_table.is_empty() {
        return Err(CustomError::QueryError);
    }

    if !(query.string_match.to_string() == "exact" || query.string_match.to_string() == "like") {
        return Err(CustomError::QueryError);
    }

    if !(query.search_type.to_string() == "and" || query.search_type.to_string() == "or") {
        return Err(CustomError::QueryError);
    }

    let search_strings = get_items(query.search_string.as_str()).await;

    let length_of_search_strings = search_strings.len() as i32;

    if length_of_search_strings < 1 {
        return Err(CustomError::QueryError);
    }

    let mut table_columns = vec![];

    match query_table.as_str() {
        "data_1" => {
            table_columns.push("random_num".to_string());
            table_columns.push("random_float".to_string());
            table_columns.push("md5".to_string());
            table_columns.push("record_id".to_string());
        },
        "data_2" => {
            table_columns.push("data_1".to_string());
            table_columns.push("data_2".to_string());
            table_columns.push("record_id".to_string());
        },
        _ => {
            return Err(CustomError::InvalidTable);
        }
    };

    println!("table_columns : {:#?}", table_columns);

    let mut inner_query = "".to_string();

    let total_combinations = table_columns.len() as i32 * search_strings.len() as i32;

    println!("total_combinations : {}", total_combinations);

    println!("string_match : {}", query.string_match.to_string());

    println!("search_type : {}", query.search_type.to_string());

    let length_of_columns = table_columns.len() as i32;

    if length_of_columns < 1 {
        return Err(CustomError::QueryError);
    }

    inner_query = get_inner_query(table_columns, search_strings, query.string_match, query.search_type).await.unwrap();

    println!("inner_query >> \n\n{}\n\n", inner_query);

    let complete_query = format!("SELECT * from {} WHERE {}", actual_db_table, inner_query);

    println!("complete_query >> \n\n{}\n\n", complete_query);

    let rows = client.query(complete_query.as_str(), &[]).await.map_err(|_| CustomError::DatabaseError)?;

    let mut structs_1: Vec<Data1> = Vec::new();
    let mut structs_2: Vec<Data2> = Vec::new();

    match query_table.as_str() {
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
        }
        _ => {
            return Err(CustomError::InvalidTable);
        }
    };

    return match query_table.as_str() {
        "data_1" => {
            Ok(serde_json::to_string(&structs_1).unwrap())
        },
        "data_2" => {
            Ok(serde_json::to_string(&structs_2).unwrap())
        },
        _ => {
            return Err(CustomError::InvalidTable);
        }
    };
}

pub async fn get_data_with_advanced_query(query: web::Query<QueryParamsAdvanced>) -> Result<HttpResponse, CustomError>  {

    let my_query1 = QueryParamsAdvanced{
        string_match: query.string_match.to_string(),
        search_type: query.search_type.to_string(),
        search_string: query.search_string.to_string(),
        source_table: "data_1".to_string(),
    };

    let my_query2 = QueryParamsAdvanced{
        string_match: query.string_match.to_string(),
        search_type: query.search_type.to_string(),
        search_string: query.search_string.to_string(),
        source_table: "data_2".to_string(),
    };

    let mut jobs = vec![];
    jobs.push(my_query1);
    jobs.push(my_query2);

    let concurrency = 10;

    let results: Vec<Result<String, CustomError>> = stream::iter(jobs).map(get_data_with_advanced_single_query).buffer_unordered(concurrency).collect().await;

    let mut my_results = vec![];

    for item in results {
        let my_result = match item {
            Ok(d) => {
                let json_items: Vec<Value> = serde_json::from_str(d.as_str()).unwrap();
                for item in json_items {
                    my_results.push(item);
                }
            },
            Err(e) => {
                println!("ERROR : {:#?}", e);
            },
        };
    }

    let json_body = json!(my_results);

    Ok(HttpResponse::Ok().content_type("application/json").body(json_body.to_string()))
}
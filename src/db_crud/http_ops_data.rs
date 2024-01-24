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
use crate::app_error::CustomError;
use crate::app_utils::is_only_spaces;
use crate::data_structs;
use crate::data_structs::{Data1, PathInfoV1, PathInfoV2, PathInfoV3, UserData};
use crate::db_crud::db_common::get_http_response_object;
use crate::db_crud::resource_ops_data::{delete_record, fetch_record_for_backend_type, upsert_record_data};
use crate::db_ops::get_db_pool_for_table;


pub async fn http_fetch_record_data(db_pool: web::Data<Pool>, mut params: web::Path<PathInfoV2>) -> Result<HttpResponse, CustomError> {

    let my_db_pool = get_db_pool_for_table(params.backend_data.as_str()).await.unwrap();
    let client: Client = my_db_pool.get().await.unwrap();

    if is_only_spaces(params.record_identifier.as_str()).await {
        let msg = "invalid  record_id ! please specify valid record_id".to_string();
        println!("{}", msg);
        let http_response = get_http_response_object(
            "failed",
            msg.as_str(),
            json!({}),
        ).await;
        return Ok(http_response)
    }

    if params.record_identifier.is_empty() {
        let http_response = get_http_response_object(
            "failed",
            "record_identifier (primary key) is missing",
            json!({}),
        ).await;
        return Ok(http_response)
    }

    params.record_identifier = params.record_identifier.as_str().trim().parse().unwrap();

    return match fetch_record_for_backend_type(&client, params.clone().backend_data.as_str(), params.clone().record_identifier).await {
        Ok(d) => {
            let record_id = params.record_identifier.to_string();
            // let record_id = d.clone().record_id.unwrap();
            let msg = format!("successfully fetched record with record_identifier : {}", record_id);
            println!("{}", msg);
            let http_response = get_http_response_object(
                "successful",
                msg.as_str(),
                serde_json::to_value(&d).unwrap(),
            ).await;
            Ok(http_response)
        },
        Err(e) => {
            let msg= format!("could not fetch data with record_identifier : {}", params.clone().record_identifier.to_string());
            println!("{}", msg);
            let http_response = get_http_response_object(
                "failed",
                msg.as_str(),
                json!({}),
            ).await;
            Ok(http_response)
        }
    };
}


pub async fn http_upsert_record_data(db_pool: web::Data<Pool>, params: web::Path<PathInfoV3>, mut data: web::Json<Value>) -> Result<HttpResponse, CustomError> {
    let mut my_db_pool: Pool;
    let mut user_data: UserData;

    println!("http_upsert_record_data : backend_data : \n\n{:#?}\n\n", params.clone().backend_data.as_str());

    match params.clone().backend_data.as_str() {
        "data_1" => {
            my_db_pool = get_db_pool_for_table(params.backend_data.as_str()).await.unwrap();
            user_data = UserData::Data1(serde_json::from_value(data.into_inner()).unwrap());
        },
        "data_2" => {
            my_db_pool = get_db_pool_for_table(params.backend_data.as_str()).await.unwrap();
            user_data = UserData::Data2(serde_json::from_value(data.into_inner()).unwrap());
        },
        _ => {
            let msg = "unknown table source for database pool".to_string();
            let http_response = get_http_response_object(
                "failed",
                msg.as_str(),
                json!({}),
            ).await;
            return Ok(http_response)
        }
    };

    let client: Client = my_db_pool.get().await.unwrap();

    // let user_data: UserData = serde_json::from_value(data.into_inner()).unwrap();

    println!("http_upsert_record_data : user_data : \n\n{:#?}\n\n", user_data);

    let result = return match upsert_record_data(&client, params.clone().backend_data.as_str(), &user_data).await {
        Ok(d) => {
            let msg = format!("inserted record into the database with record_identifier : {}", d.to_string());
            println!("{}", msg);
            let http_response = get_http_response_object(
                "successful",
                msg.as_str(),
                json!({}),
            ).await;
            Ok(http_response)
        },
        Err(e) => {
            let msg = format!("could not insert data into the database : {}", e.to_string());
            println!("{}", msg);
            let http_response = get_http_response_object(
                "failed",
                msg.as_str(),
                json!({}),
            ).await;
            Ok(http_response)
        },
    };
}


pub async fn http_delete_record_data(pool: web::Data<Pool>, params: web::Path<PathInfoV2>) -> Result<HttpResponse, CustomError> {
    let my_db_pool = get_db_pool_for_table(params.backend_data.as_str()).await.unwrap();
    let client: Client = my_db_pool.get().await.unwrap();

    if params.clone().record_identifier.is_empty() {
        let server_response = data_structs::ServerResponse {
            result: "failed".to_string(),
            message: "record_identifier (primary key) is missing".to_string(),
            server_data: json!({}),
        };
        return Ok(HttpResponse::Ok().content_type("application/json").body(serde_json::to_string(&server_response).unwrap()))
    }

    match delete_record(&client, params.clone().backend_data.as_str(), params.clone().record_identifier.to_string()).await {
        Ok(d) => {
            let msg = format!("deleted record with record_identifier : {}", params.clone().record_identifier.to_string());
            println!("{}", msg);
            let server_response = data_structs::ServerResponse {
                result: "successful".to_string(),
                message: msg.to_string(),
                server_data: serde_json::to_value(d).unwrap(),
            };
            Ok(HttpResponse::Ok().content_type("application/json").body(serde_json::to_string(&server_response).unwrap()))
        },
        Err(e) => {
            let msg = format!("could not delete record with record_identifier : {}", params.clone().record_identifier.to_string());
            println!("{}", msg);
            let server_response = data_structs::ServerResponse {
                result: "failed".to_string(),
                message: msg.to_string(),
                server_data: json!({}),
            };
            Ok(HttpResponse::Ok().content_type("application/json").body(serde_json::to_string(&server_response).unwrap()))
        },
    }
}


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
use serde_json::{from_value, json, to_string, Value};
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
use crate::data_structs;
// use crate::data_structs::{Data1, Data2, PathInfoV1, PathInfoV2, PathInfoV3, UserData};
use crate::data_structs::{UserData};
use crate::data_structs::UserData::Data1;
use crate::data_structs::UserData::Data2;
use crate::db_crud::db_common::get_http_response_object;
use crate::db_crud::resource_ops_data::{delete_record, fetch_record_for_backend_type, upsert_record_data};
use crate::db_ops::get_db_pool_for_table;

impl UserData {
    pub async fn validate_data_fields(&self) -> Result<(), Box<dyn std::error::Error>> {
        return match &self {
            UserData::Data1(d) => {
                println!("checking Data1 ...");
                if Option::is_none(&d.random_num) {
                    return Err(Box::from("missing field/value : random_num"))
                }

                if Option::is_none(&d.random_float) {
                    return Err(Box::from("missing field/value : random_float"))
                }

                if Option::is_none(&d.md5) {
                    return Err(Box::from("missing field/value : md5"))
                }

                if Option::is_none(&d.record_id) {
                    return Err(Box::from("missing field/value : record_id"))
                }

                println!("Data1 fields are valid :)");
                Ok(())
            },
            UserData::Data2(d) => {
                println!("checking Data2 ...");
                if Option::is_none(&d.data_1) {
                    return Err(Box::from("missing field/value : data_1"))
                }

                if Option::is_none(&d.data_2) {
                    return Err(Box::from("missing field/value : data_2"))
                }

                if Option::is_none(&d.record_id) {
                    return Err(Box::from("missing field/value : record_id"))
                }

                println!("Data2 fields are valid :)");
                Ok(())
            }
        };
    }

    pub async fn get_json_value(&self) -> Result<Value, Box<dyn std::error::Error>> {
        return match serde_json::to_value(&self) {
            Ok(d) => {
                Ok(d)
            },
            Err(e) => {
                return Err(Box::from("could not convert UserData to JSON VALUE !"))
            },
        }
    }

    pub async fn get_record_id(&self) -> Result<String, Box<dyn std::error::Error>> {
        return match &self {
            UserData::Data1(d) => {
                match &d.record_id {
                    None => {
                        return Err(Box::from("record_id is missing for UserData !"))
                    }
                    Some(d) => {
                        Ok(d.to_string())
                    }
                }
            },
            UserData::Data2(d) => {
                match &d.record_id {
                    None => {
                        return Err(Box::from("record_id is missing for UserData !"))
                    }
                    Some(d) => {
                        Ok(d.to_string())
                    }
                }
            },
        }
    }

    pub async fn validate_fields_if_record_is_absent_in_db(&self, client: &Client, backend_data: &str, record_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        return match &self.validate_data_fields().await {
            Ok(d) => {
                Ok(())
            },
            Err(e) => {
                return Err(Box::from(e.to_string()))
            },
        };
    }
}
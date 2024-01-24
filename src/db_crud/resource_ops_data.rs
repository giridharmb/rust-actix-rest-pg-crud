use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::{clone, env};
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
use crate::app_utils::is_only_spaces;
use crate::data_structs::{Data1, Data2, UserData};
use crate::db_crud::merging_data::update_missing_fields_data;
use crate::db_crud::table_queries::{get_delete_query_for_backend_type, get_insert_query_for_backend_type, get_select_query_for_backend_type};


async fn fetch_and_update_record_data(client: &Client, backend_type: &str, user_data: &UserData, record_identified: &str) -> Result<Value, Box<dyn std::error::Error>> {

    return match user_data {
        UserData::Data1(data) => {

            println!("fetch_and_update_record_data : UserData::Data1 : data : \n\n{:#?}\n\n", data);

            let mut my_data = Data1 {
                random_num: data.clone().random_num,
                random_float: data.clone().random_float,
                md5: data.clone().md5,
                record_id: data.clone().record_id,
            };

            println!("(Y1) my_data : {:#?}", my_data);

            let my_data_value = serde_json::to_value(my_data).unwrap();

            match insert_or_update_value_onto_db(client, backend_type, record_identified, my_data_value).await {
                Ok(d) => {
                    Ok(d)
                },
                Err(e) => {
                    return Err(e)
                },
            }
        },
        UserData::Data2(data) => {

            println!("fetch_and_update_record_data : UserData::Data2 : data : \n\n{:#?}\n\n", data);

            let mut my_data = Data2 {
                data_1: data.clone().data_1,
                data_2: data.clone().data_2,
                record_id: data.clone().record_id,
            };

            println!("(Y1) my_data : {:#?}", my_data);

            let my_data_value = serde_json::to_value(my_data).unwrap();

            match insert_or_update_value_onto_db(client, backend_type, record_identified, my_data_value).await {
                Ok(d) => {
                    Ok(d)
                },
                Err(e) => {
                    return Err(e)
                },
            }
        },
    };
}

pub async fn insert_or_update_value_onto_db(client: &Client, backend_type: &str, record_identified: &str, my_data_value: Value) -> Result<Value, Box<dyn std::error::Error>> {
    let fetched_record = fetch_record_for_backend_type(&client, backend_type, record_identified.to_string()).await;
    println!("(X1) fetched_record : {:#?}", fetched_record);
    if let Ok(mut fetched_record_value) = fetched_record {
        // record found from backend db
        // which ever field is missing in the incoming json payload
        // merge the missing fields from POST request
        // with the ones found in the db
        let updated_data_set = update_missing_fields_data(my_data_value, fetched_record_value, backend_type).await;
        println!("(X2) updated_data_set : {:#?}", updated_data_set);
        Ok(updated_data_set)
    } else {
        // record (not) found from backend db
        // it is a totally new record
        // isert it into the db
        println!("(X3) my_data_value : {:#?}", my_data_value);
        Ok(my_data_value.clone())
    }
}


pub async fn upsert_record_data(client: &Client, backend_data: &str, user_data: &UserData) -> Result<String, Box<dyn std::error::Error>> {

    println!("upsert_record_data : user_data : \n\n{:#?}\n\n", user_data);

    println!("upsert_record_data : backend_data : {}", backend_data);

    let my_query = get_insert_query_for_backend_type(backend_data).await?;

    println!("upsert_record_data : my_query : {}", my_query);

    let stmt = client.prepare(my_query.as_str()).await.unwrap();

    let mut record_id = user_data.get_record_id().await?;

    if is_only_spaces(record_id.as_str()).await {
        return Err(Box::from("invalid record_id ! please provide valid record_id in the payload !"))
    }

    record_id = record_id.as_str().trim().parse().unwrap();

    println!("upsert_record_data : record_id : {}", record_id);

    match fetch_record_for_backend_type(client, backend_data, record_id.clone()).await {
        Ok(d) => {
            // record exists on the backend
        },
        Err(e) => {
            // no record exists on the back
            // -> which means, it is a new record
            // -> so validate all the input fields
            user_data.validate_data_fields().await?;
        }
    };



    return match user_data {
        UserData::Data1(data) => {

            println!("{}", "upsert_record_data : Data1 >>>".to_string());

            match fetch_and_update_record_data(client, backend_data, &user_data, record_id.as_str()).await {
                Ok(d) => {

                    let merged_result: Result<Data1, serde_json::Error> = from_value(d);
                    let merged_data = merged_result.unwrap();

                    println!("merged_data : {:#?}", merged_data);

                    client.execute(&stmt, &[
                        &merged_data.random_num.clone().unwrap(),
                        &merged_data.random_float.clone().unwrap(),
                        &merged_data.md5.clone().unwrap(),
                        &record_id.to_string(),
                    ]).await.unwrap();
                    Ok(record_id.to_string())
                },
                Err(e) => {
                    println!("error : {:#?}", e);
                    Err(e)
                },
            }
        },
        UserData::Data2(data) => {

            println!("{}", "upsert_record_data : Data2 >>>".to_string());

            match fetch_and_update_record_data(client, backend_data, &user_data, record_id.as_str()).await {
                Ok(d) => {
                    let merged_result: Result<Data2, serde_json::Error> = from_value(d);
                    let merged_data = merged_result.unwrap();

                    println!("merged_data : {:#?}", merged_data);

                    client.execute(&stmt, &[
                        &merged_data.data_1.clone().unwrap(),
                        &merged_data.data_2.clone().unwrap(),
                        &record_id.clone().to_string(),
                    ]).await.unwrap();
                    Ok(record_id.to_string())
                },
                Err(e) => {
                    println!("error : {:#?}", e);
                    Err(e)
                },
            }
        },
    };
}

pub async fn fetch_record_for_backend_type(client: &Client, backend_type: &str, record_identifier: String) -> Result<Value, Box<dyn std::error::Error>> {

    let my_query = get_select_query_for_backend_type(backend_type).await?;

    let stmt = client.prepare(my_query.as_str()).await.unwrap();

    let rows = client.query(&stmt, &[&record_identifier]).await.unwrap();

    return get_first_record_for_backend_type(&rows, backend_type, record_identifier).await;
}

pub async fn delete_record(client: &Client, backend_type: &str, record_identifier: String) -> Result<Value, Box<dyn std::error::Error>> {

    let my_query = get_delete_query_for_backend_type(backend_type).await?;

    let stmt = client.prepare(my_query.as_str()).await.unwrap();

    let rows = client.query(&stmt, &[&record_identifier]).await.unwrap();

    return get_first_record_for_backend_type(&rows, backend_type.clone(), record_identifier).await;
}

pub async fn get_first_record_for_backend_type(rows: &Vec<Row>, backend_type: &str, record_identifier: String) -> Result<Value, Box<dyn std::error::Error>> {
    match backend_type {
        "data_1" => {
            if let Some(row) = rows.first() {
                let my_record = Data1 {
                    random_num: Option::from(row.try_get(0).unwrap_or_else(|_| 0)),
                    random_float: Option::from(row.try_get(1).unwrap_or_else(|_| 0.0)),
                    md5: Option::from(row.try_get(2).unwrap_or_else(|_| "missing_md5".to_string())),
                    record_id: Option::from(row.try_get(3).unwrap_or_else(|_| "missing_record_id".to_string())),
                };
                Ok(serde_json::to_value(my_record).unwrap())
            } else {
                let msg = format!("no record found for record_identifier : {}", record_identifier.to_string());
                println!("{}", msg);
                return Err(Box::from(msg))
            }
        },
        "data_2" => {
            if let Some(row) = rows.first() {
                let my_record = Data2 {
                    data_1: Option::from(row.try_get(0).unwrap_or_else(|_| "missing_data_1".to_string())),
                    data_2: Option::from(row.try_get(1).unwrap_or_else(|_| "missing_data_2".to_string())),
                    record_id: Option::from(row.try_get(2).unwrap_or_else(|_| "missing_record_id".to_string())),
                };
                Ok(serde_json::to_value(my_record).unwrap())
            } else {
                let msg = format!("no record found for record_identifier : {}", record_identifier.to_string());
                println!("{}", msg);
                return Err(Box::from(msg))
            }
        },
        _ => {
            let msg = "no record found , invalid backend_type !".to_string();
            println!("{}", msg);
            return Err(Box::from(msg));
        },
    }
}

// functions called by HTTP Handlers --------------------------------- | end | -------------

pub async fn get_struct_from_value(backend_data: &str, value: Value) -> Result<UserData, Box<dyn std::error::Error>>  {
        return match backend_data {
            "data_1" => {
                let result: Result<Data1, serde_json::Error> = return match from_value(value) {
                    Ok(d) => {
                        Ok(UserData::Data1(d))
                    },
                    Err(e) => {
                        Err(Box::from("could not convert value to struct"))
                    }
                };
            },
            "data_2" => {
                let result: Result<Data2, serde_json::Error> = return match from_value(value) {
                    Ok(d) => {
                        Ok(UserData::Data2(d))
                    },
                    Err(e) => {
                        return Err(Box::from("could not convert value to struct"))
                    }
                };
            },
            _ => {
                return Err(Box::from("could not find valid backend_data"))
            }
        }
    }
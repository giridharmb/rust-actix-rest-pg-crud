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
use crate::data_structs;
use crate::data_structs::{Data1, Data2, UserData};
use crate::db_crud::table_queries::{get_delete_query_for_backend_type, get_insert_query_for_backend_type, get_select_query_for_backend_type};


pub async fn merge_data_data1(original_that_needs_to_be_updated: &mut Data1, fetched_data_from_db: &Data1) -> Value {
    if Option::is_none(&original_that_needs_to_be_updated.random_float) {
        original_that_needs_to_be_updated.random_float = fetched_data_from_db.random_float;
    }

    if Option::is_none(&original_that_needs_to_be_updated.random_num) {
        original_that_needs_to_be_updated.random_num = fetched_data_from_db.random_num;
    }

    if Option::is_none(&original_that_needs_to_be_updated.md5) {
        original_that_needs_to_be_updated.md5 = fetched_data_from_db.clone().md5;
    }

    serde_json::to_value(original_that_needs_to_be_updated).unwrap()
}

pub async fn merge_data_data2(original_that_needs_to_be_updated: &mut Data2, fetched_data_from_db: &Data2) -> Value {
    if Option::is_none(&original_that_needs_to_be_updated.data_1) {
        original_that_needs_to_be_updated.data_1 = fetched_data_from_db.clone().data_1;
    }

    if Option::is_none(&original_that_needs_to_be_updated.data_2) {
        original_that_needs_to_be_updated.data_2 = fetched_data_from_db.clone().data_2;
    }
    serde_json::to_value(original_that_needs_to_be_updated).unwrap()
}

async fn get_blank_data1() -> Data1 {
    let my_data = Data1 {
        random_num: None,
        random_float: None,
        md5: None,
        record_id: None,
    };
    my_data
}

async fn get_blank_data2() -> Data2 {
    let my_data = Data2 {
        data_1: None,
        data_2: None,
        record_id: None,
    };
    my_data
}

async fn set_data1(needs_update: &mut Data1, copy_from_this: &Data1) {
    needs_update.random_num = copy_from_this.clone().random_num;
    needs_update.random_float = copy_from_this.clone().random_float;
    needs_update.md5 = copy_from_this.clone().md5;
}

async fn set_data2(needs_update: &mut Data2, copy_from_this: &Data2) {
    needs_update.data_1 = copy_from_this.clone().data_1;
    needs_update.data_2 = copy_from_this.clone().data_2;
}

pub async fn update_missing_fields_data(original_that_needs_to_be_updated: Value, fetched_data_from_db: Value, backend_type: &str) -> Value {

    match backend_type {
        "data_1" => {
            let mut data_that_needs_updating: data_structs::Data1 = from_value(original_that_needs_to_be_updated).unwrap();
            let data_fetched_from_database: data_structs::Data1 = from_value(fetched_data_from_db).unwrap();

            println!("update_missing_fields_data : backend_type : {} , data_that_needs_updating : \n\n{:#?}\n\n", backend_type, data_that_needs_updating);
            println!("update_missing_fields_data : backend_type : {} , data_fetched_from_database : \n\n{:#?}\n\n", backend_type, data_fetched_from_database);

            if Option::is_none(&data_that_needs_updating.record_id) {
                data_that_needs_updating.record_id = data_fetched_from_database.record_id;
            }

            if Option::is_none(&data_that_needs_updating.random_num) {
                data_that_needs_updating.random_num = data_fetched_from_database.random_num;
            }

            if Option::is_none(&data_that_needs_updating.random_float) {
                data_that_needs_updating.random_float = data_fetched_from_database.random_float;
            }

            if Option::is_none(&data_that_needs_updating.md5) {
                data_that_needs_updating.md5 = data_fetched_from_database.md5;
            }

            serde_json::to_value(data_that_needs_updating).unwrap()
        },
        "data_2" => {

            let mut data_that_needs_updating: data_structs::Data2 = from_value(original_that_needs_to_be_updated).unwrap();
            let data_fetched_from_database: data_structs::Data2 = from_value(fetched_data_from_db).unwrap();

            println!("update_missing_fields_data : backend_type : {} , data_that_needs_updating : \n\n{:#?}\n\n", backend_type, data_that_needs_updating);
            println!("update_missing_fields_data : backend_type : {} , data_fetched_from_database : \n\n{:#?}\n\n", backend_type, data_fetched_from_database);

            if Option::is_none(&data_that_needs_updating.record_id) {
                data_that_needs_updating.record_id = data_fetched_from_database.record_id;
            }

            if Option::is_none(&data_that_needs_updating.data_1) {
                data_that_needs_updating.data_1 = data_fetched_from_database.data_1;
            }

            if Option::is_none(&data_that_needs_updating.data_2) {
                data_that_needs_updating.data_2 = data_fetched_from_database.data_2;
            }
            serde_json::to_value(data_that_needs_updating).unwrap()
        },
        _ => {
            json!({})
        }
    }

}

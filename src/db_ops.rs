use std::collections::HashMap;
use std::env;
use deadpool_postgres::{Config, Pool};
use dotenv::dotenv;
use tokio_postgres::{NoTls, Error, Row};
use std::fs::File;
use std::path::Path;
use tokio::time::Instant;
use uuid::Uuid;
use crate::app_error::CustomError;
use async_trait::async_trait;

pub async fn make_db_pool() -> Pool {
    dotenv().ok();
    dotenv::from_filename("app.rust.env").ok();

    let mut cfg = Config::new();
    cfg.host = Option::from(env::var("PG.HOST").unwrap());
    cfg.user = Option::from(env::var("PG.USER").unwrap());
    cfg.password = Option::from(env::var("PG.PASSWORD").unwrap());
    cfg.dbname = Option::from(env::var("PG.DBNAME").unwrap());
    let pool: Pool = cfg.create_pool(None, tokio_postgres::NoTls).unwrap();
    pool
}

pub async fn make_db_pool_v2() -> Pool {
    dotenv().ok();
    dotenv::from_filename("app.rust.env").ok();

    let mut cfg = Config::new();
    cfg.host = Option::from(env::var("PG.OS.HOST").unwrap());
    cfg.user = Option::from(env::var("PG.OS.USER").unwrap());
    cfg.password = Option::from(env::var("PG.OS.PASSWORD").unwrap());
    cfg.dbname = Option::from(env::var("PG.OS.DBNAME").unwrap());
    let pool: Pool = cfg.create_pool(None, tokio_postgres::NoTls).unwrap();
    pool
}

pub async fn get_db_pool_for_table(source_table: &str) -> Result<Pool,CustomError> {
    println!("source_table : {}", source_table.to_string());
    return match source_table {
        "data_1" => {
            Ok(make_db_pool().await)
        },
        "data_2" => {
            Ok(make_db_pool().await)
        },
        _ => {
            return Err(CustomError::InvalidTable)
        }
    };
}


#[async_trait]
pub trait FromRow: Sized {
    async fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error>
    where Self: Sized;
}

pub async fn get_inner_query(table_columns: Vec<String>, search_strings: Vec<String>, pattern_match: String, search_type: String) -> Result<String, CustomError> {
    let mut inner_query = "".to_string();

    if !(search_type == "and" || search_type == "or") {
        println!("error : search_type is neither 'and' nor 'or' !");
        return Err(CustomError::QueryError)
    }

    if !(pattern_match == "like" || pattern_match == "exact") {
        println!("error : pattern_match is neither 'like' nor 'exact' !");
        return Err(CustomError::QueryError)
    }

    if table_columns.len() == 0 {
        println!("error : table_columns length is ZERO !");
        return Err(CustomError::QueryError)
    }

    if search_strings.len() == 0 {
        println!("error : search_strings length is ZERO !");
        return Err(CustomError::QueryError)
    }

    if pattern_match.as_str() == "exact" { // exact string match
        // search all JSON fields for possible match
        // this can also be applied if there are other columns
        if search_type == "and" {
            inner_query = inner_query + " ( ";
            let mut search_string_counter = 1;
            for my_search_str in search_strings.to_owned() {
                inner_query = inner_query + " ( ";
                let mut column_counter = 1;
                // --------------------------------------
                for my_column in table_columns.to_owned() {
                    if column_counter == table_columns.len() as i32 {
                        inner_query = inner_query + format!(" lower({}::text) = lower('{}') ", my_column.to_string(), my_search_str.to_lowercase()).as_str();
                    } else {
                        inner_query = inner_query + format!(" lower({}::text) = lower('{}') OR ", my_column.to_string(), my_search_str.to_lowercase()).as_str();
                    }
                    column_counter += 1;
                }
                if search_string_counter == search_strings.len() as i32 {
                    inner_query = inner_query + " ) ";
                } else {
                    inner_query = inner_query + " ) AND ";
                }
                search_string_counter += 1;
                // --------------------------------------
            }
            inner_query = inner_query + " ) ";
        } else if search_type == "or" {
            inner_query = inner_query + " ( ";
            let mut search_string_counter = 1;
            for my_search_str in search_strings.to_owned() {
                inner_query = inner_query + " ( ";
                let mut column_counter = 1;
                // --------------------------------------
                for my_column in table_columns.to_owned() {
                    if column_counter == table_columns.len() as i32 {
                        inner_query = inner_query + format!(" lower({}::text) = lower('{}') ", my_column.to_string(), my_search_str.to_lowercase()).as_str();
                    } else {
                        inner_query = inner_query + format!(" lower({}::text) = lower('{}') OR ", my_column.to_string(), my_search_str.to_lowercase()).as_str();
                    }
                    column_counter += 1;
                }
                if search_string_counter == search_strings.len() as i32 {
                    inner_query = inner_query + " ) ";
                } else {
                    inner_query = inner_query + " ) OR ";
                }
                search_string_counter += 1;
                // --------------------------------------
            }
            inner_query = inner_query + " ) ";
        } else {
            return Err(CustomError::QueryError)
        }

    } else if pattern_match.as_str() == "like" { // pattern match
        // search all JSON fields for possible match
        // this can also be applied if there are other columns
        if search_type == "and" {
            inner_query = inner_query + " ( ";
            let mut search_string_counter = 1;
            for my_search_str in search_strings.to_owned() {
                inner_query = inner_query + " ( ";
                let mut column_counter = 1;
                // --------------------------------------
                for my_column in table_columns.to_owned() {
                    if column_counter == table_columns.len() as i32 {
                        inner_query = inner_query + format!(" lower({}::text) like lower('%{}%') ", my_column.to_string(), my_search_str.to_lowercase()).as_str();
                    } else {
                        inner_query = inner_query + format!(" lower({}::text) like lower('%{}%') OR ", my_column.to_string(), my_search_str.to_lowercase()).as_str();
                    }
                    column_counter += 1;
                }
                if search_string_counter == search_strings.len() as i32 {
                    inner_query = inner_query + " ) ";
                } else {
                    inner_query = inner_query + " ) AND ";
                }
                search_string_counter += 1;
                // --------------------------------------
            }
            inner_query = inner_query + " ) ";
        } else if search_type == "or" {
            inner_query = inner_query + " ( ";
            let mut search_string_counter = 1;
            for my_search_str in search_strings.to_owned() {
                inner_query = inner_query + " ( ";
                let mut column_counter = 1;
                // --------------------------------------
                for my_column in table_columns.to_owned() {
                    if column_counter == table_columns.len() as i32 {
                        inner_query = inner_query + format!(" lower({}::text) like lower('%{}%') ", my_column.to_string(), my_search_str.to_lowercase()).as_str();
                    } else {
                        inner_query = inner_query + format!(" lower({}::text) like lower('%{}%') OR ", my_column.to_string(), my_search_str.to_lowercase()).as_str();
                    }
                    column_counter += 1;
                }
                if search_string_counter == search_strings.len() as i32 {
                    inner_query = inner_query + " ) ";
                } else {
                    inner_query = inner_query + " ) OR ";
                }
                search_string_counter += 1;
                // --------------------------------------
            }
            inner_query = inner_query + " ) ";
        } else {
            return Err(CustomError::QueryError)
        }
    } else {
        return Err(CustomError::QueryError)
    }
    Ok(inner_query)
}


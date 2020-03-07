#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::prelude::*;
use dotenv::dotenv;
use diesel::r2d2;
use std::env;

pub mod schema;
pub mod model;

pub type PoolConnection = r2d2::Pool<r2d2::ConnectionManager<MysqlConnection>>;

pub fn new_connection_pool() ->  PoolConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let manager = r2d2::ConnectionManager::<MysqlConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .max_size(15)
        .build(manager)
        .expect("Failed to create pool.");
    pool
}

#![feature(decl_macro, proc_macro_hygiene)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket_sync_db_pools;

pub mod internal;
pub mod models;
pub mod repository;
pub mod schema;
pub mod server;

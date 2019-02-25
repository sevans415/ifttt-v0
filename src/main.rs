#![feature(proc_macro_hygiene, decl_macro)]
#![feature(plugin)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate rocket_contrib;
extern crate chrono;
extern crate reqwest;

// extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;

use r2d2_postgres::{PostgresConnectionManager, TlsMode};

// use postgres::{Connection, TlsMode};
use rocket_contrib::databases::postgres;
use rocket_contrib::json::Json;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;
use std::thread;

use tokio::prelude::*;
use tokio::timer::Interval;

use std::time::{Duration, Instant};

mod resources;
mod storage;

use self::resources::{NbaPlayer, SleepSession};
use self::storage::interactions::{
  add_nba_player, delete_sleep_session, execute_txn, get_all_sleep_sessions, save_sleep,
  transfer_funds,
};

#[database("postgres_logs")]
struct LogsDbConn(postgres::Connection);

#[post("/sleep", data = "<sleep_sesh>")]
fn add_sleep(conn: LogsDbConn, sleep_sesh: Json<SleepSession>) -> Json<Value> {
  println!("adding sleep: {:?}", sleep_sesh);
  execute_txn(&conn, |txn| save_sleep(txn, &sleep_sesh)).unwrap();
  // Json(json!(sleep_sesh))
  Json(json!({"status": "ok"}).0)
}

#[get("/sleep")]
fn get_sleep(conn: LogsDbConn) -> Json<Value> {
  match get_all_sleep_sessions(&conn) {
    Ok(search_results) => {
      println!("DB Search results {:?}", search_results);
      Json(json!(search_results).0)
    }
    Err(err) => Json(json!("ERROR!").0),
  }
  // let db_search = execute_txn(&conn, |txn| get_all_sleep_sessions(txn)).unwrap();
  // println!("search results {:?}", db_search);
  // Json(json!(search_results).0)
}

#[put("/sleep/<id>", data = "<sleep_sesh>")]
fn update_sleep(conn: LogsDbConn, id: i64, sleep_sesh: Json<SleepSession>) -> Json<SleepSession> {
  println!("updating sleep sesh with id{:?} to: {:?}", id, sleep_sesh);
  sleep_sesh
}

#[delete("/sleep/<id>")]
fn delete_sleep(conn: LogsDbConn, id: i64) -> Json<Value> {
  match delete_sleep_session(&conn, id) {
    Ok(rows_deleted) => {
      Json(json!({"status": "ok", "message": format!("Deleted {} rows", rows_deleted)}).0)
    }
    Err(_err) => Json(json!("ERROR!").0),
  }
}

#[get("/")]
fn index(conn: LogsDbConn) -> &'static str {
  execute_txn(&conn, |txn| transfer_funds(txn, 1, 2, 100)).unwrap();
  "Hello, world!"
}

fn main() {
  // let conn =
  //   Connection::connect("postgresql://maxroach@localhost:26257/bank", TlsMode::None).unwrap();

  static N_THREADS: i32 = 10;
  let mut children = Vec::new();

  for i in 0..N_THREADS {
    children.push(thread::spawn(move || {
      println!("This is thread {}", i);
    }))
  }

  let manager =
    PostgresConnectionManager::new("postgresql://maxroach@localhost:26257/bank", TlsMode::None)
      .unwrap();
  let pool = r2d2::Pool::new(manager).unwrap();
  let tokio_pool = pool.clone();

  let player_id_counter = Mutex::new(1);
  children.push(thread::spawn(move || {
    // build task for tokio to run once for every interval of X time
    let conn = tokio_pool.get().unwrap();
    let task = Interval::new(Instant::now(), Duration::new(5, 0))
      .for_each(move |instant| {
        let mut num = player_id_counter.lock().unwrap();
        *num += 1;
        let client = reqwest::Client::new();
        let request_url = &format!("https://free-nba.p.rapidapi.com/players/{}", num);
        let res: NbaPlayer = client
          .get(request_url)
          .header(
            "X-RapidAPI-Key",
            "ee470e72cbmshe44d34c6b9ef25bp124a54jsn423b49b2183a",
          )
          .send()
          .unwrap()
          .json()
          .unwrap();

        let db_result = add_nba_player(&conn, &res).unwrap();
        // match db_result {
        //   Ok(num) => println!("Successfully added {} players", num),
        //   Err(err) => println!("ERROR saving nba player"),
        // }
        // let mut body = String::new();
        // res.read_to_string(&mut body).unwrap();

        println!("{:#?}", res);
        Ok(())
      })
      .map_err(|e| panic!("interval errored; err={:?}", e));
    // start tokio task
    tokio::run(task);
  }));
  children.push(thread::spawn(|| {
    // start rocket web service
    rocket::ignite()
      .attach(LogsDbConn::fairing())
      .mount(
        "/",
        routes![index, add_sleep, get_sleep, update_sleep, delete_sleep],
      )
      .launch();
  }));

  for child in children {
    let _ = child.join();
  }
}

// fn main() {
//   let manager =
//     PostgresConnectionManager::new("postgres://postgres@localhost", TlsMode::None).unwrap();
//   let pool = r2d2::Pool::new(manager).unwrap();

//   for i in 0..10i32 {
//     let pool = pool.clone();
//     thread::spawn(move || {
//       let conn = pool.get().unwrap();
//       conn
//         .execute("INSERT INTO foo (bar) VALUES ($1)", &[&i])
//         .unwrap();
//     });
//   }
// }

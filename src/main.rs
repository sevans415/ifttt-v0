#![feature(proc_macro_hygiene, decl_macro)]
#![feature(plugin)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate rocket_contrib;
extern crate chrono;

use postgres::{Connection, TlsMode};
use rocket_contrib::databases::postgres;
use rocket_contrib::json::Json;
use serde_json::Value;

mod resources;
mod storage;

use self::resources::SleepSession;
use self::storage::interactions::{
  delete_sleep_session, execute_txn, get_all_sleep_sessions, save_sleep, transfer_funds,
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
  let conn =
    Connection::connect("postgresql://maxroach@localhost:26257/bank", TlsMode::None).unwrap();

  // Check account balances after the transaction.
  for row in &conn.query("SELECT id, balance FROM accounts", &[]).unwrap() {
    let id: i64 = row.get(0);
    let balance: i64 = row.get(1);
    println!("{} {}", id, balance);
  }

  rocket::ignite()
    .attach(LogsDbConn::fairing())
    .mount(
      "/",
      routes![index, add_sleep, get_sleep, update_sleep, delete_sleep],
    )
    .launch();
}

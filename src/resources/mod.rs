#[derive(Serialize, Deserialize, Debug)]
pub struct SleepSession {
  pub id: Option<i64>,
  // pub date: chrono::Date<chrono::Local>,
  pub hours: i64,
  pub quality: i64,
  pub note: Option<String>,
  // pub startTime: chrono::DateTime<chrono::Local>,
  // pub endTime: chrono::Date<chrono::Local>,
}

#[derive(Deserialize, Debug)]
pub struct NbaPlayer {
  pub id: Option<i64>,
  pub first_name: String,
  pub last_name: String,
  pub position: String,
}

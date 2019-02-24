use crate::resources::SleepSession;
use postgres::error::T_R_SERIALIZATION_FAILURE;
use postgres::transaction::Transaction;
use postgres::{Connection, Result};

/// Runs op inside a transaction and retries it as needed.
/// On non-retryable failures, the transaction is aborted and
/// rolled back; on success, the transaction is committed.
pub fn execute_txn<T, F>(conn: &Connection, op: F) -> Result<T>
where
  F: Fn(&Transaction) -> Result<T>,
{
  let txn = conn.transaction()?;
  loop {
    let sp = txn.savepoint("cockroach_restart")?;
    match op(&sp).and_then(|t| sp.commit().map(|_| t)) {
      Err(ref err)
        if err
          .as_db()
          .map(|e| e.code == T_R_SERIALIZATION_FAILURE)
          .unwrap_or(false) => {}
      r => break r,
    }
  }
  .and_then(|t| txn.commit().map(|_| t))
}

pub fn transfer_funds(txn: &Transaction, from: i64, to: i64, amount: i64) -> Result<()> {
  // Read the balance.
  let from_balance: i64 = txn
    .query("SELECT balance FROM accounts WHERE id = $1", &[&from])?
    .get(0)
    .get(0);

  assert!(from_balance >= amount);

  // Perform the transfer.
  txn.execute(
    "UPDATE accounts SET balance = balance - $1 WHERE id = $2",
    &[&amount, &from],
  )?;
  txn.execute(
    "UPDATE accounts SET balance = balance + $1 WHERE id = $2",
    &[&amount, &to],
  )?;
  Ok(())
}

pub fn save_sleep(txn: &Transaction, sleep_session: &SleepSession) -> Result<()> {
  txn.execute(
    "INSERT INTO sleep (hours, quality, note) VALUES ($1, $2, $3)",
    &[
      &sleep_session.hours,
      &sleep_session.quality,
      &sleep_session.note,
    ],
  )?;
  Ok(())
}

pub fn get_all_sleep_sessions(conn: &Connection) -> Result<Vec<SleepSession>> {
  let mut results = Vec::new();
  for row in conn.query("SELECT * FROM sleep", &[])?.into_iter() {
    let note: String = row.get("note");
    let hours: i64 = row.get("hours");
    let id: i64 = row.get("id");
    let quality: i64 = row.get("quality");
    results.push(SleepSession {
      id: Some(id),
      hours,
      quality,
      note: Some(note),
    })
  }
  Ok(results)
}

pub fn delete_sleep_session(conn: &Connection, id: i64) -> Result<u64> {
  conn.execute("DELETE FROM sleep WHERE id = $1", &[&id])
}

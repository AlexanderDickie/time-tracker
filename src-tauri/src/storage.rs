use rusqlite::{Connection};
use chrono::{NaiveDate, Local};
use std::io::Error;

pub struct Storage {
    db: Connection,
}

impl Storage {
    // create table if not existing, then load it 
    pub fn build(path: &str) -> Storage {
        let con = Connection::open(path).unwrap();

        con.execute(
            "CREATE TABLE IF NOT EXISTS Storage(
             Date TEXT PRIMARY KEY,
             Blocks INTEGER DEFAULT 0
         );", []).unwrap();

        Storage { db: con }
    }

    pub fn increment_or_insert_date(&self, date: NaiveDate) {
        let _date = date.format("%Y-%m-%d").to_string();

        if let 0 = self.db.execute(
            "UPDATE Storage
            SET blocks = blocks + 1
            WHERE date = ?",
            [&_date]
            ).unwrap() {
                self.db.execute("
                            INSERT INTO Storage
                            Values (?, 1);
                            ", [&_date]).unwrap();
        }
    }
}


// dont run these in parallel
#[cfg(test)]
mod tests {
    #[derive(Debug, PartialEq)]
    struct Line {
        date: String,
        blocks: i32,
    }
    use super::*;
    use std::fs;

    #[test]
    fn build_storage() {
        let storage = Storage::build("./test.db");

        fs::remove_file("./test.db").unwrap();
    }

    #[test]
    fn increment_value_existing() {
        fs::remove_file("./test.db");
        let mut storage = Storage::build("./test.db");

        storage.db.execute("
                           INSERT OR IGNORE INTO Storage
                           VALUES ('2022-11-11', 10);
                           ", []).unwrap();


        let date = NaiveDate::from_ymd(2022, 11, 11);
        storage.increment_or_insert_date(date);

        let mut stmt = storage.db.prepare(
            "SELECT * FROM Storage;"
            ).unwrap();

        let row = stmt.query_row([], |row| {
            Ok(Line {
                date: row.get(0).unwrap(),
                blocks: row.get(1).unwrap(),
            })
        }).unwrap();

        assert_eq!(row, Line {date: "2022-11-11".to_string(), blocks: 11});
        fs::remove_file("./test.db");
    }

    #[test]
    fn increment_value_non_existing() {
        fs::remove_file("./test.db");
        let mut storage = Storage::build("./test.db");

        storage.db.execute("
                           INSERT OR IGNORE INTO Storage
                           VALUES ('2022-11-11', 10);
                           ", []).unwrap();


        let date = NaiveDate::from_ymd(2022, 11, 12);
        storage.increment_or_insert_date(date);

        let mut stmt = storage.db.prepare(
            "SELECT * FROM Storage
            WHERE Date = '2022-11-12';"
            ).unwrap();

        let row = stmt.query_row([], |row| {
            Ok(Line {
                date: row.get(0).unwrap(),
                blocks: row.get(1).unwrap(),
            })
        }).unwrap();

        assert_eq!(row, Line {date: "2022-11-12".to_string(), blocks: 1});
        fs::remove_file("./test.db");
    }
}



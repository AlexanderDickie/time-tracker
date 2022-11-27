use rusqlite::{Connection, types::ValueRef};
use chrono::NaiveDate;
use std::io::Error;
use std::collections::HashMap;
use std::path::Path;
use serde_json::to_string;

pub struct Storage {
    db: Connection,
}

impl Storage {
    pub fn init<P: AsRef<Path>>(path: P) -> Storage {
        //create if not exists, then load database
        let con = Connection::open(path).unwrap();
        //create if not exists, then load table
        con.execute(
            "CREATE TABLE IF NOT EXISTS Storage(
             Date DATE PRIMARY KEY,
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
    // gets previous n inputs ending at end
    pub fn get_previous(&self, end: NaiveDate, n: usize) -> Vec<(NaiveDate, usize)>{
        let mut stmt = self.db.prepare(&format!("
                                              SELECT Date, Blocks
                                              FROM Storage
                                              ORDER BY Date DESC
                                              LIMIT {};
                                              ", n)).unwrap();
        let block_map = stmt.query_map([], |row| Ok((row.get_unwrap::<usize, NaiveDate>(0), row.get_unwrap::<usize, usize>(1))))
            .unwrap().
            map(|tup| tup.unwrap())
            .collect::<HashMap<NaiveDate, usize>>();
        let mut out = Vec::new();
        let mut cur = end;
        for _ in 0..n {
            if block_map.contains_key(&cur) {
                out.push((cur, block_map[&cur]));
            } else {
                out.push((cur, 0));
            }
            cur = cur.pred();
        }
        // earliest date at the top
        out.into_iter().rev().collect()
    }
}

// dont run these in parallel
#[cfg(test)]
mod tests {
    #[derive(Debug, PartialEq)]
    struct Line {
        date: NaiveDate,
        blocks: i32,
    }
    use super::*;
    use std::fs;

    #[test]
    fn serde() {
        fs::remove_file("./test.db");

        let storage = Storage::build("./test.db");

        let date = NaiveDate::from_ymd(2022, 11, 11);

        let past = to_string(&storage.get_previous(date, 30));
        println!("the test is {}", past.unwrap());
        panic!();

    }


    #[test] 
    fn get_previous_generic() {
        fs::remove_file("./test.db");

        let storage = Storage::build("./test.db");
        storage.db.execute("
                           INSERT INTO Storage
                           VALUES 
                           ('2022-11-11', 10),
                           ('2022-10-21', 8);
                           ", []).unwrap();

        let end = NaiveDate::from_ymd(2022, 11, 11);
        let previous = storage.get_previous(end, 30);
       
        //correct len
        assert_eq!(previous.len(), 30);

        // correct ordering
        previous
            .windows(2)
            .for_each(|p| assert!(p[0] < p[1]));

        //correct input
        let mid = NaiveDate::from_ymd(2022, 10, 21);
        for (date, blocks) in previous {
            if date == end {
                assert_eq!(blocks, 10);
            } else if date == mid {
                assert_eq!(blocks, 8);
            } else {
                assert_eq!(blocks, 0);
            }
        }

    }


    #[test]
    fn fetch_line() {
        fs::remove_file("./test.db");
        let storage = Storage::build("./test.db");
        storage.db.execute("
                           INSERT INTO Storage
                           VALUES ('2022-11-11', 10);
                           ", []);
        let mut stmt = storage.db.prepare("
                                          SELECT * FROM Storage;
                                          ").unwrap();

        let row = stmt.query_row([], |row| {
            Ok(Line {
                date: row.get(0).unwrap(),
                blocks: row.get(1).unwrap(),
            })
        }).unwrap();
        assert_eq!(row, Line{ date: NaiveDate::from_ymd(2022, 11, 11), blocks: 10});


        fs::remove_file("./test.db").unwrap();
    }

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

        assert_eq!(row, Line {date, blocks: 11});
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

        assert_eq!(row, Line {date, blocks: 1});
        fs::remove_file("./test.db");
    }
}



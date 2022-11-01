use std::fs::{self, read_to_string};
use std::collections::HashMap;
use std::fmt::{self, Formatter, Display};

use chrono::{NaiveDate, Datelike};


/* csv in the form of:
 * 29-10-2022,16
 * .....
 * which represents date and number of blocks timed on that date
 */

pub struct Storage {
    file_path: String,
    data: Vec<InputLine>,
}

impl Storage {
    pub fn build(file_path: &str) -> Storage {
        let data: Vec<InputLine> = read_to_string(file_path)
            .unwrap()
            .lines()
            .map(InputLine::parse_from_str)
            .collect();
        Storage{ file_path: file_path.to_string(), data} 
        }

    // reads the past n days ending at from
    pub fn read_previous_days(&self, from: NaiveDate, n: usize) -> Vec<InputLine> {
        let input_lines_map: HashMap<NaiveDate, usize> = self.data
            .iter()
            .rev()
            .take(n)
            .map(|il| (il.date, il.blocks))
            .collect();
        let mut out = Vec::new();

        let mut cur = from;
        let mut blocks: usize; 
        for _ in 0..n {
            if input_lines_map.contains_key(&cur) {
                blocks = input_lines_map[&cur];
            } else {
                blocks = 0;
            }
            out.push(InputLine{ date: cur, blocks });
            cur = cur.pred();
        }
        // oldest date at top of vector
        out.into_iter().rev().collect()
    }


    // update file and data field given new data
    pub fn alter_line(&mut self, date: NaiveDate, change: i32) {
        match self.data.binary_search_by_key(&date, |il| il.date) {
            // date exists in data
            Ok(pos) => {
                let blocks = self.data[pos].blocks as i32 + change;
                assert!(blocks >= 0);
                self.data[pos] = InputLine{ date, blocks: blocks as usize};
            },
            // date doesnt exist in data
            Err(pos) => {
                assert!(change >= 0);
                self.data.insert(pos, InputLine{ date, blocks: change as usize });
            }
        }

        // update file on disk
        self.update_disk();
    }

    // writes the data in self.data to the file at file_path 
    // creates temp file then renames to avoid loss of data in eg power loss
    fn update_disk(&self) {
        let mut string = self.data
            .iter()
            .map(|il| il.to_string() + "\n")
            .collect::<String>();
        // remove last \n
        string.pop();
        let bytes = string.into_bytes();

        // write new file to disk
        let temp_file_path = format!("{}.new", self.file_path);
        fs::write(&temp_file_path, bytes).unwrap();
        fs::rename(&temp_file_path, &self.file_path).unwrap();
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InputLine {
    date: NaiveDate,
    blocks: usize,
}

impl InputLine {
    pub fn parse_from_str(s: &str) -> InputLine {
        let mut iter = s.split(',');
        let date = NaiveDate::parse_from_str(iter.next().unwrap(), "%d-%m-%Y").unwrap();
        let blocks: usize = iter
            .next()
            .unwrap()
            .parse()
            .unwrap();
        InputLine{ date, blocks }
    }
}

impl Display for InputLine {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let day = self.date.day();
        let month = self.date.month();
        let year = self.date.year();
        write!(f, "{}-{}-{},{}", day, month, year, self.blocks)
    }
}

// dont run these tests in parallel!
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_line_impl_display() {
        let date = NaiveDate::from_ymd(2022, 1, 1);
        let input_line = InputLine { date, blocks: 21};

        assert_eq!(input_line.to_string(), "1-1-2022,21");
    }
    #[test]
    fn read_previous_days_generic() {
        let file_path = "test_files/alter_line.csv";
        let content = b"10-10-2022,2\n20-10-2022,5\n11-11-2022,3";
        fs::write(file_path, content).unwrap();

        let storage = Storage::build(file_path);

        let date = NaiveDate::from_ymd(2022, 11, 1);
        let prev_days = storage.read_previous_days(date, 30);

        // check for correct input 
        for input_line in prev_days {
            let blocks = input_line.blocks;
            match input_line.date.day() {
                10 => assert_eq!(2, blocks),
                20 => assert_eq!(5, blocks),
                _ => assert_eq!(0, blocks),
            }
        }

        // check input is in valid order
        storage.data
            .windows(2)
            .for_each(|p| assert!(p[0].date < p[1].date));

    }

    #[test]
    fn update_disk_generic() {
        let file_path = "test_files/alter_line.csv";
        let content = b"10-10-2022,2\n20-10-2022,5\n11-11-2022,3";
        fs::write(file_path, content).unwrap();

        let mut storage = Storage::build(file_path);

        let input_line1 = InputLine{date: NaiveDate::from_ymd(22,11,5), blocks: 5};
        let input_line2 = InputLine{date: NaiveDate::from_ymd(22,10,25), blocks: 7};

        storage.data = vec![input_line1.clone(), input_line2.clone()];

        storage.update_disk();

        let new_content: Vec<InputLine> = fs::read_to_string(file_path)
            .unwrap()
            .lines()
            .map(InputLine::parse_from_str)
            .collect();

        fs::remove_file(file_path).unwrap();
        assert_eq!(new_content, vec![input_line1, input_line2]);
    }

    #[test]
    fn alter_line_newinput() {
        let file_path = "test_files/alter_line.csv";
        let content = b"10-10-2022,2\n20-10-2022,5\n11-11-2022,3";
        fs::write(file_path, content).unwrap();

        let mut storage = Storage::build(file_path);

        let date = NaiveDate::from_ymd(2022, 11, 1);
        storage.alter_line(date, 5);

        // check for correct input 
        for input_line in storage.data.iter() {
            let blocks = input_line.blocks;
            match input_line.date.day() {
                10 => assert_eq!(2, blocks),
                20 => assert_eq!(5, blocks),
                11 => assert_eq!(3, blocks),
                1 => assert_eq!(5, blocks),
                _ => panic!(),
            }
        }

        // check input is in valid order
        storage.data
            .windows(2)
            .for_each(|p| assert!(p[0].date < p[1].date));

    }

    #[test]
    fn alter_line_changeinput() {
        let file_path = "test_files/alter_line.csv";
        let content = b"10-10-2022,2\n20-10-2022,5\n11-11-2022,3";
        fs::write(file_path, content).unwrap();

        let mut storage = Storage::build(file_path);

        let date = NaiveDate::from_ymd(2022, 10, 20);
        println!("{:?} {:?}", storage.data, date);
        storage.alter_line(date, -5);

        // check for correct input 
        for input_line in storage.data.iter() {
            let blocks = input_line.blocks;
            match input_line.date.day() {
                10 => assert_eq!(2, blocks),
                20 => assert_eq!(0, blocks),
                11 => assert_eq!(3, blocks),
                _ => panic!(),
            }
        }

        // check input is in valid order
        storage.data
            .windows(2)
            .for_each(|p| assert!(p[0].date < p[1].date));

    }


}

use tokio::{self,
    time::{interval, Duration},
    sync::mpsc,
    task,
};
use chrono::{Local, NaiveDate};
use storage::{InputLine, Storage};

use super::DURATION;

mod storage;
mod alert;

pub enum Window {
    Main,
}

pub struct Timing {
    rx: mpsc::Receiver<TimerMessage>,
    handle: task::JoinHandle<()>,
    pub cur_time: usize,
}

#[derive(Debug)]
enum TimerMessage {
    Time(usize),
    Finished
}

pub enum TimeFrame {
    Past30,
}

pub struct DisplayData {
    time_frame : TimeFrame,
    pub data: Vec<(String, u64)>,
}

impl DisplayData {
    pub fn build(time_frame: TimeFrame, input_lines: &Vec<InputLine>) -> DisplayData {
        let data:Vec<(String, u64)> = input_lines
            .iter()
            .map(|il| {
                let s = il.to_string();
                let v: Vec<&str> = s.split(',').collect();
                (v[0].to_owned(), v[1].to_owned().parse::<u64>().unwrap().to_owned())
            })
            .collect();

        DisplayData{time_frame, data}
    }
}

pub struct App {
    pub window: Window,
    pub in_progress: Option<Timing>,
    pub storage: Storage,
    pub display_data: DisplayData,
    pub should_quit: bool,
}

impl App {

    // initialize app struct with past 30 day's data
    pub fn build(file_path: &str) -> App {
        let storage = Storage::build(file_path);

        let today: NaiveDate = Local::today().naive_local();
        let display_data = DisplayData::build(TimeFrame::Past30, &storage.read_previous_days(today, 30));
        App {
            window: Window::Main,
            in_progress: None,
            storage,
            display_data,
            should_quit: false,

        }
    }
    // update out app's timer field if we are timing
    pub fn on_tick(&mut self) {
        match &mut self.in_progress {
            // no timer in progress
            None => (),
            // timer is in progress
            Some(Timing{rx, ..}) => {
                if let Ok(timer_message) = rx.try_recv() {
                    match timer_message {

                        // update curTime field
                        TimerMessage::Time(time) => self.in_progress.as_mut().unwrap().cur_time = time,

                        // update curTime field, increment today
                        TimerMessage::Finished => {
                            self.in_progress = None;
                            self.increment_today();
                            self.refresh_display_data();
                            // play alert

                        } 
                    }
                }
            }
        }
    }

    fn increment_today(&mut self) {
        let today = Local::today().naive_local();
        self.storage.alter_line(today, 1);
    }

    fn refresh_display_data(&mut self) {
        let today: NaiveDate = Local::today().naive_local();

        match self.display_data.time_frame {
            TimeFrame::Past30 => self.display_data = DisplayData::build(TimeFrame::Past30, &self.storage.read_previous_days(today, 30)),
        };
    }
    pub fn on_key(&mut self, c: char) {
        match c {
            'n' => self.on_n(),
            'q' => self.on_q(),
            'c' => self.on_c(),
            _ => (),
        }
    }

    fn on_q(&mut self) {
        self.should_quit = true;
    }

    fn on_n(&mut self) {
        // begin timer if not in progress
        if self.in_progress.is_none() {
            let (tx, rx) = mpsc::channel(16);

            let handle = tokio::spawn(async move {
                let mut interval = interval(Duration::from_secs(1));

                for i in 0..(DURATION) {
                    interval.tick().await;
                    let tx1 = tx.clone();
                    tx1.send(TimerMessage::Time(i)).await.unwrap();
                }
                //send finished message
                tx.send(TimerMessage::Finished).await.unwrap();
                alert::show_notification();
                alert::play_sound();
            });

            self.in_progress = Some(Timing{rx, handle, cur_time: 0});

            // alert use that we have finished 
        }
    }

    fn on_c(&mut self) {
        if let Some(Timing{handle, ..}) = &mut self.in_progress {
            handle.abort();
            self.in_progress = None;
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FILE_PATH;
    use tokio::time::sleep;
    #[tokio::test]
    #[ignore]
    async fn async_timer_updates_state() {
        let mut app = App::build(FILE_PATH);
        app.on_n();

        sleep(Duration::from_secs(5)).await;
        for _ in 0..10 {
            app.on_tick();
        }
        let time = app.in_progress.unwrap().cur_time;
        assert!((4..=5).contains(&time));
    }
}




use tauri::{
    AppHandle,
    async_runtime::Receiver,
    CustomMenuItem, 
    SystemTrayMenu, 
};
use tokio::{self,
    time::{
        self,
        interval
    },
};
use chrono::{
    self,
    Local,
};
use async_trait::async_trait;
use super::utils;

#[derive(Debug)]
pub enum EventMessage {
    TimerLeft,
    CountDownLeft,

    Pause,
    Resume,
    Stop,
}

#[async_trait]
pub trait Activate {
    async fn activate(mut self) -> State;
}

pub enum State {
    Inactive(Inactive),
    Active(Active),
}

impl State {
    pub fn new(app_handle: AppHandle, rx: Receiver<EventMessage>) -> State {
        State::Inactive ( 
            Inactive {
                app_handle,
                rx,
            }
        )
    }
}

#[async_trait]
impl Activate for State {
    async fn activate(mut self) -> State {
        match self {
            State::Inactive(inactive) => inactive.activate().await,
            State::Active(active) => active.activate().await,
        }
    }
}

pub struct Inactive {
    app_handle: tauri::AppHandle,
    rx: Receiver<EventMessage>,
}

pub struct Active {
    timing: Timing,
    app_handle: tauri::AppHandle,
    rx: Receiver<EventMessage>,
}

enum Timing {
    CountUp,
    CountDown(chrono::Duration),
}

#[async_trait]
impl Activate for Inactive {
    async fn activate(mut self) -> State {
        self.app_handle.tray_handle().set_title("Inactive").unwrap();
        self.app_handle.tray_handle().set_menu(inactive_menu()).unwrap();

        match self.rx.recv().await.unwrap() {
            EventMessage::TimerLeft => 
                State::Active(Active {
                    timing: Timing::CountUp,
                    app_handle: self.app_handle,
                    rx: self.rx,
                }),

            EventMessage::CountDownLeft => 
                State::Active(Active {
                    timing: Timing::CountDown(chrono::Duration::minutes(60)), // todo: change to user's choice
                    app_handle: self.app_handle,
                    rx: self.rx,
                }),

            _ => panic!("invalid event message"),
        }
    }
}


#[async_trait]
impl Activate for Active {
    async fn activate(mut self) -> State {
        // set system tray menu for active state
        if let Timing::CountUp = self.timing {
            self.app_handle.tray_handle().set_menu(timer_menu_active()).unwrap();
        } else {

            self.app_handle.tray_handle().set_menu(timer_menu_active()).unwrap();
        }

        let mut start = Local::now();
        let mut elapsed = chrono::Duration::zero();
        let mut paused = false;

        let mut interval = interval(time::Duration::from_secs(1));

        'outer: loop {
            // time has elapsed on countdown
            if let Timing::CountDown(total_duration) = (&self).timing {
                if elapsed >= total_duration {
                    break 'outer;
                }
            } 

            // paused
            if paused {
                let now = Local::now();
                elapsed = elapsed + (now - start);
                self.app_handle.tray_handle().set_menu(timer_menu_paused()).unwrap();

                match self.rx.recv().await.unwrap() {
                    EventMessage::Resume => {
                        self.app_handle.tray_handle().set_menu(timer_menu_active()).unwrap();
                        paused = false;
                        interval.reset();
                        start = Local::now();
                    },
                    EventMessage::Stop => break 'outer,
                    _ => panic!("invalid event message"),
                }
            }
            // active
            tokio::select! {
                // time one second, update title menu
                _ = interval.tick() => {
                    let now = Local::now();
                    let total = elapsed + (now - start);

                    // set system tray display to seconds elapsed/remaining
                    if let Timing::CountDown(total_duration) = self.timing {
                    self.app_handle
                        .tray_handle()                                                   
                        .set_title(&utils::format_time_countdown(total.num_seconds() as u32, 
                                total_duration.num_seconds() as u32))
                        .unwrap();
                    } else {
                        self.app_handle
                            .tray_handle()                                                   
                            .set_title(&utils::format_time_timer(total.num_seconds() as u32))
                            .unwrap();
                    }

                },
                // await event message and react
                event_message = self.rx.recv() => {
                    use EventMessage::*;
                    match event_message.unwrap() {
                        Pause => {
                            paused = true;
                        },
                        Stop => break 'outer,
                        _ => panic!("invalid event message"),
                    }
                },
            }
        }

        // return Inactive state
        let Active {timing: _, app_handle, rx} = self;
        State::Inactive(Inactive {app_handle, rx})
    }
}

pub fn inactive_menu() -> SystemTrayMenu {
    SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("Timer".to_string(), "Timer"))
        .add_item(CustomMenuItem::new("Countdown".to_string(), "Countdown"))
        .add_item(CustomMenuItem::new("Quit".to_string(), "Quit"))
}

fn timer_menu_active() -> SystemTrayMenu {
    SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("Pause".to_string(), "Pause"))
        .add_item(CustomMenuItem::new("Stop".to_string(), "Stop"))
}

fn timer_menu_paused() -> SystemTrayMenu {
    SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("Resume".to_string(), "Resume"))
        .add_item(CustomMenuItem::new("Stop".to_string(), "Stop"))
}

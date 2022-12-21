use tauri::{
    AppHandle,
    async_runtime::Receiver,
    CustomMenuItem, 
    SystemTrayMenu, 
};
use tokio::{self,
    time::{interval, Duration},
};

use super::utils;

#[derive(Debug)]
pub enum EventMessage {
    TimerLeft,
    // CountDownLeft(u32),

    Pause,
    Resume,
    Stop,
}

pub enum State {
    Timer(Timer),
    // Countdown(Countdown),
    Inactive(Inactive)
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
    pub async fn activate(self) -> State {
        match self {
            State::Inactive(inactive) => {
                inactive.activate().await
            }, 

            State::Timer(mut timer) => {
                // begin timer logic
                timer.activate().await;

                // timer ended, revert to inactive state
                let Timer {app_handle, rx} = timer;
                State::Inactive(Inactive{app_handle, rx})
            }
            _ => self, 
        }
    }
}

pub struct Inactive {
    app_handle: tauri::AppHandle,
    rx: Receiver<EventMessage>,
}

impl Inactive {
    pub async fn activate(mut self) -> State {
        self.app_handle.tray_handle().set_title("Inactive").unwrap();
        self.app_handle.tray_handle().set_menu(inactive_menu()).unwrap();

        use EventMessage::*;
        match self.rx.recv().await.unwrap() {
            TimerLeft => State::Timer(Timer{ app_handle: self.app_handle, rx: self.rx }),
            _ => panic!(""),
        }
    }
}

pub struct Timer {
    app_handle: tauri::AppHandle,
    rx: Receiver<EventMessage>,
}

impl Timer {
    pub async fn activate(&mut self) {
        self.app_handle.tray_handle().set_menu(timer_menu_active()).unwrap();

        let mut t: u32 = 0;
        let mut interval = interval(Duration::from_secs(1));
        let mut paused = false;

        'outer: loop {
            // paused
            if paused {
                self.app_handle.tray_handle().set_menu(timer_menu_paused()).unwrap();
                use EventMessage::*;
                match self.rx.recv().await.unwrap() {
                    Resume => {
                        self.app_handle.tray_handle().set_menu(timer_menu_active()).unwrap();
                        paused = false;
                        interval.reset();
                    },
                    Stop => break 'outer,
                    _ => panic!("invalid event message"),
                }
            }
            // active
            tokio::select! {
                // time one second, update title menu
                _ = interval.tick() => {
                    t += 1;
                    self.app_handle
                        .tray_handle()                                                   
                        .set_title(&utils::format_time_timer(t))
                        .unwrap();
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

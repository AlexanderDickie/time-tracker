use tauri::{self,
    AppHandle,
    async_runtime::{self, Receiver},
    CustomMenuItem, 
    SystemTray, 
    SystemTrayMenu, 
    SystemTrayEvent,
    WindowBuilder,
    WindowUrl, Manager};
use tokio::{self,
    time::{interval, Duration},
};
use async_trait::async_trait;

#[derive(Debug)]
pub enum EventMessage {
    TimerLeft,
    CountDownLeft(u32),

    Pause,
    Resume,
    Stop,
}

#[async_trait]
pub trait State {
    async fn activate(self) -> Self;
}

pub struct InActive {
    app_handle: AppHandle,
    rx: Receiver<EventMessage>, 
}

#[async_trait]
impl State for InActive{
    async fn activate(mut self) -> Box<dyn State + Send> {
        self.app_handle.tray_handle().set_menu(inactive_menu());
        use EventMessage::*;
        match self.rx.recv().await.unwrap() {
            TimerLeft => 
                Box::new(
                    Timer {
                        app_handle: self.app_handle,
                        rx: self.rx,
                    }
                ),
            _ => panic!("invalid event message"),
        }
    }
}

struct Timer {
    app_handle: AppHandle,
    rx: Receiver<EventMessage>,
}

#[async_trait]
impl State for Timer { 
    async fn activate(mut self) -> Box<dyn State + Send> {
        // set timer menu
        self.app_handle.tray_handle().set_menu(timer_menu_active());

        let mut t: u32 = 0;
        let mut interval = interval(Duration::from_secs(1));
        loop {
            tokio::select! {
                // time one second update title menu
                _ = interval.tick() => {
                    t += 1;
                    self.app_handle
                        .tray_handle()                                                   
                        .set_title(&t.to_string())
                        .unwrap();
                },
                // await event message and react
                event_message = self.rx.recv() => {
                    use EventMessage::*;
                    match event_message.unwrap() {
                        Pause => {
                            // set paused menu, wait on next event
                            self.app_handle.tray_handle().set_menu(timer_menu_paused());
                            match self.rx.recv().await.unwrap() {
                                Resume => {
                                    self.app_handle.tray_handle().set_menu(timer_menu_active());
                                },
                                Stop => break,
                                _ => panic!("invalid event message"),
                            }
                        },
                        Stop => break,
                        _ => panic!("invalid event message"),
                    }
                },
            }
        }
        Box::new(
            InActive {
                app_handle: self.app_handle,
                rx: self.rx,
            }
        )
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

use serde::Serialize;
use tauri::{self, async_runtime, CustomMenuItem, SystemTray, SystemTrayMenu, SystemTrayEvent,
    WindowBuilder,
    WindowUrl, Manager};
use tokio::{self,
    time::{interval, Duration},
};
use chrono::Local;

mod storage;
use storage::Storage;
mod utils;
use utils::*;

static DURATION: usize = 25 * 60;

#[derive(Serialize)]
struct ChartInput {
    label: String,
    value: usize,
}

#[derive(Debug)]
enum TimerState {
    Begun(usize),
    Paused(usize),
    Finished,
}

#[derive(Debug)]
enum EventMessage {
    Begin,
    Pause,
    Resume,
    Stop,
}

#[tauri::command]
fn get_previous(app_handle: tauri::AppHandle) -> Vec<ChartInput> {
    let storage = Storage::init(&get_db_path(&app_handle));
    let today = Local::today().naive_local();
    let x = storage.get_previous(today, 30);
    x.into_iter().map(|ar| {
        let date= ar.0.to_string();
        let date_day = date.split("-").last().unwrap().to_string();
        ChartInput{label: date_day, value: ar.1}
    })
        .collect::<Vec<ChartInput>>()
}

#[tauri::command]
fn close_alert_window(window: tauri::Window) {
    window.close().unwrap();
}

fn main() {
    let (tx, mut rx) = tauri::async_runtime::channel(16);

    let start = CustomMenuItem::new("start".to_string(), "Start");
    let stop = CustomMenuItem::new("stop".to_string(), "Stop");
    let pause = CustomMenuItem::new("pause".to_string(), "Pause");
    let resume = CustomMenuItem::new("resume".to_string(), "Resume");
    let view = CustomMenuItem::new("view".to_string(), "View");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu_inactive = SystemTrayMenu::new()
        .add_item(start)
        .add_item(view.clone())
        .add_item(quit.clone());
    let _tray_menu_inactive = tray_menu_inactive.clone(); // used to setup system tray
    let tray_menu_active = SystemTrayMenu::new()
        .add_item(pause)
        .add_item(stop.clone())
        .add_item(view.clone())
        .add_item(quit.clone());
    let tray_menu_paused = SystemTrayMenu::new()
        .add_item(resume)
        .add_item(stop)
        .add_item(view)
        .add_item(quit);

    tauri::Builder::default()
        .setup(|app| {
            // dont show app icon on mac os's bottom menubar
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // spawn thread acting as timer, updating system tray ui and menu on state change
            let app_handle = app.handle();
            let mut timer_state = TimerState::Finished;
            async_runtime::spawn(async move {
                'outer: loop {
                    match timer_state {

                        TimerState::Begun(remaining) => {
                            app_handle.tray_handle().set_menu(tray_menu_active.clone()).unwrap();

                            let mut interval = interval(Duration::from_secs(1));
                            for i in 0..remaining {
                                tokio::select! {
                                    _ = interval.tick() => {
                                        app_handle
                                            .tray_handle()
                                            .set_title(&format_time_remaining(i, remaining))
                                            .unwrap();
                                    }
                                    event_message = rx.recv() => {
                                        match event_message.unwrap() {
                                            EventMessage::Pause => {
                                                timer_state = TimerState::Paused(remaining - i + 1);
                                                continue 'outer;
                                            }
                                            EventMessage::Stop => {
                                                timer_state = TimerState::Finished;
                                                continue 'outer;
                                            },
                                            _ => panic!("invalid state"),
                                        }
                                    }
                                }
                            }
                            let storage = Storage::init(&get_db_path(&app_handle));
                            let today = Local::today().naive_local();
                            storage.increment_or_insert_date(today);
                            WindowBuilder::new(
                                &app_handle,
                                "alert",
                                WindowUrl::App("alert".into())
                            )
                                .always_on_top(true)
                                .decorations(false)
                                .inner_size(400.0, 200.0)
                                .build().unwrap();
                            timer_state = TimerState::Finished;
                        }

                        TimerState::Paused(remaining) => {
                            app_handle.tray_handle().set_menu(tray_menu_paused.clone()).unwrap();

                            match rx.recv().await.unwrap() {
                                EventMessage::Resume => {
                                    timer_state = TimerState::Begun(remaining);
                                }
                                EventMessage::Stop => {
                                    timer_state = TimerState::Finished;
                                }
                                _ => panic!("invalid state"),
                            }

                        }

                        TimerState::Finished => {
                            app_handle.tray_handle().set_title("Inactive").unwrap();
                            app_handle.tray_handle().set_menu(tray_menu_inactive.clone()).unwrap();

                            match rx.recv().await.unwrap() {
                                EventMessage::Begin => {
                                    timer_state = TimerState::Begun(DURATION);
                                }
                                _ => panic!("invalid state"),
                            }
                        }

                    }
                } 

            });
            Ok(())
        })
        .system_tray(SystemTray::new()
            .with_menu(_tray_menu_inactive.clone())
            .with_title("Inactive"))
        .on_system_tray_event(move |app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => {
                match id.as_str() {
                    "start" => {
                        async_runtime::block_on(tx.send(EventMessage::Begin)).unwrap();
                    },
                    "stop" => {
                        async_runtime::block_on(tx.send(EventMessage::Stop)).unwrap();
                    },
                    "pause" => {
                        async_runtime::block_on(tx.send(EventMessage::Pause)).unwrap();
                    },
                    "resume" => {
                        async_runtime::block_on(tx.send(EventMessage::Resume)).unwrap();
                    },
                    "view" => {
                        if let Some(window) = app.get_window("view") {
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        } else {
                            WindowBuilder::new(
                                app,
                                "view",
                                WindowUrl::App("index.html".into())
                            )
                                .title("past blocks")
                                .center()
                                .inner_size(1200.0, 700.0)
                                .build().unwrap();
                        }
                    },
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {},
                } 
            }
            SystemTrayEvent::LeftClick {..} => (),
            _ => (),
        })
        .invoke_handler(tauri::generate_handler![get_previous, close_alert_window])
        .build(tauri::generate_context!())
        .expect("error while building application")
        .run(|_app_handle, event| match event {
            // prevent app shutdown on window close
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}


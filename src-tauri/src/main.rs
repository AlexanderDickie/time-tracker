use tauri::{
    self,
    async_runtime,
    SystemTray, 
    SystemTrayEvent,
    WindowBuilder,
    WindowUrl,
    Manager
};
use chrono::Local;
use serde::Serialize;

mod storage;
use storage::Storage;
mod utils;
use utils::get_db_path;
mod timer;
use timer::{
    State,
    EventMessage
};

#[derive(Serialize)]
struct ChartInput {
    label: String,
    value: usize,
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
    let (tx, rx) = tauri::async_runtime::channel(16);
    tauri::Builder::default()
        .setup(|app| {
            // dont show app icon on mac os's bottom menubar
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let mut state = State::new(app.handle(), rx);
            tauri::async_runtime::spawn(async move {
                loop {
                    state = state.activate().await;
                }
            });
            Ok(())
        })
        .system_tray(SystemTray::new()
            .with_menu(timer::inactive_menu().clone())
            .with_title("Inactive"))
        .on_system_tray_event(move |app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => {
                use async_runtime::block_on;
                match id.as_str() {
                    // send message to our state
                    "Timer" => {
                        block_on(tx.send(EventMessage::TimerLeft)).unwrap();
                    },
                    // "Countdown" => {
                    //     block_on(tx.send(EventMessage::CountDownLeft(DURATION as u32))).unwrap();
                    // },
                    "Pause" => {
                        block_on(tx.send(EventMessage::Pause)).unwrap();
                    },
                    "Resume" => {
                        block_on(tx.send(EventMessage::Resume)).unwrap();
                    },
                    "Stop" => {
                        block_on(tx.send(EventMessage::Stop)).unwrap();
                    }

                    "View" => {
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


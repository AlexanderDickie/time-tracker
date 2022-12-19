use serde::Serialize;
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
use chrono::Local;
use async_trait::async_trait;

mod timer;
use timer::{
    State,
    EventMessage,
    InActive,
    inactive_menu,
};
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
    let (tx, mut rx) = async_runtime::channel(16);
    tauri::Builder::default()
        .setup(|app| {
            // dont show app icon on mac os's bottom menubar
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // spawn thread that manages system tray timing functionality
            let initial_state = InActive {
                app_handle: app.handle(),
                rx,
            };
            async_runtime::spawn(async move {
                let mut state: Box<dyn State + Send> = Box::new(initial_state);
                // let st = *state;
                state = (*state).activate();
                // loop {
                //     st = st.activate();
                //     // state = (*state).activate().await;
                // }
            });
            Ok(())
        })
        .system_tray(SystemTray::new()
            .with_menu(inactive_menu())
            .with_title("Inactive"))
        .on_system_tray_event(move |app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => {
                use async_runtime::block_on;
                match id.as_str() {
                    "Timer" => {
                        block_on(tx.send(EventMessage::TimerLeft)).unwrap();
                    },
                    "Countdown" => {
                        block_on(tx.send(EventMessage::CountDownLeft(DURATION as u32))).unwrap();
                    },
                    "pause" => {
                        block_on(tx.send(EventMessage::Pause)).unwrap();
                    },
                    "resume" => {
                        block_on(tx.send(EventMessage::Resume)).unwrap();
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


use std::sync::{Arc, Mutex};
use serde::Serialize;
use tauri::{self, async_runtime, CustomMenuItem, SystemTray, SystemTrayMenu, SystemTrayEvent, App, Manager,
    WindowBuilder,
    WindowUrl};
use tokio::{self,
    time::{interval, Duration},
};
use chrono::{Local, NaiveDate};
use crate::storage::Storage;

mod storage;
mod utils;

static DB_PATH: &str = "./storage.db";
static DURATION: usize = 25 * 60;

type AppState = Arc<Mutex<Storage>>;
#[derive(Debug)]
enum TimeMessage {
    Time(usize),
    Finished,
}
struct Timing {
    in_progress: Mutex<Option<async_runtime::JoinHandle<()>>>,
}

#[derive(Serialize)]
struct ChartInput {
    name: String,
    value: usize,
}

#[tauri::command]
fn get_previous(state: tauri::State<AppState>) -> Vec<ChartInput> {
    let data = state.lock().unwrap();
    let today = Local::today().naive_local();
    let x = data.get_previous(today, 30);
    x.into_iter().map(|ar| ChartInput{name: ar.0.to_string(), value: ar.1}).collect::<Vec<ChartInput>>()
}

#[tauri::command]
fn close_alert_window(window: tauri::Window) {
    window.close().unwrap();
}

fn main() {
    let state = Arc::new(Mutex::new(Storage::build(DB_PATH)));
    let timing = Timing{ in_progress: Mutex::new(None) };
    let (tx, mut rx) = tauri::async_runtime::channel(16);

    let start = CustomMenuItem::new("start".to_string(), "Start");
    let pause = CustomMenuItem::new("pause".to_string(), "Pause");
    let stop = CustomMenuItem::new("stop".to_string(), "Stop");
    let view = CustomMenuItem::new("view".to_string(), "View");
    let tray_menu_inactive = SystemTrayMenu::new()
        .add_item(start)
        .add_item(view.clone());
    // below is the tray menu that is moved into the timer ui changing/ db writing thread
    let _tray_menu_inactive = tray_menu_inactive.clone();
    let tray_menu_active = SystemTrayMenu::new()
        .add_item(pause)
        .add_item(stop)
        .add_item(view);

    tauri::Builder::default()
        .manage(Arc::clone(&state))
        .setup(|app| {
            // dont show app icon on mac os's bottom menubar
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            // thread to receive messages from timer thread, updates timer ui, increments blocks
            // and shows alert window 
            // time
            let _app = app.handle();
            async_runtime::spawn(async move {
                loop {
                    match rx.recv().await.unwrap() {
                        TimeMessage::Time(time) => _app
                            .tray_handle()
                            .set_title(&utils::format_time_remaining(time, DURATION)).unwrap(),
                        TimeMessage::Finished => {
                            _app.tray_handle().set_title("Inactive").unwrap();
                            _app.tray_handle().set_menu(_tray_menu_inactive.clone()).unwrap();

                            let today = Local::today().naive_local();
                            let data = state.lock().unwrap();
                            data.increment_or_insert_date(today);

                            let alert_window = WindowBuilder::new(
                                &_app,
                                "alert",
                                WindowUrl::App("alert".into())
                            )
                                .inner_size(400.0, 200.0)
                                .always_on_top(true)
                                .center()
                                .decorations(false)
                                .build().unwrap();

                        },
                    }
                }
            });
            Ok(())
        })
        .system_tray(SystemTray::new()
            .with_menu(tray_menu_inactive.clone())
            .with_title("Inactive"))
        .on_system_tray_event(move |app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => {
                match id.as_str() {
                    "start" => {
                        let tx1 = tx.clone();
                        // spawn timer thread
                        let join_handle = async_runtime::spawn(async move {
                            let mut interval = interval(Duration::from_secs(1));
                            for i in 0..(DURATION) {
                                interval.tick().await;
                                tx1.send(TimeMessage::Time(i)).await.unwrap();
                            }
                            tx1.send(TimeMessage::Finished).await.unwrap();
                        });
                        let mut data = timing.in_progress.lock().unwrap();
                        *data = Some(join_handle);
                        app.tray_handle().set_menu(tray_menu_active.clone()).unwrap();
                    },
                    "stop" => {
                        // abort timer thread, update menu to inactive state
                        let mut data = timing.in_progress.lock().unwrap();
                        data.as_ref().unwrap().abort();
                        *data = None;
                        app.tray_handle().set_menu(tray_menu_inactive.clone()).unwrap();
                        app.tray_handle().set_title("Inactive").unwrap();
                    },
                    "pause" => {
                    },
                    "view" => {
                        let alert_window = WindowBuilder::new(
                            app,
                            "view",
                            WindowUrl::App("index.html".into())
                        ).build().unwrap();
                    },
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



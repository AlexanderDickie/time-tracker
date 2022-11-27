use std::sync::Mutex;
use serde::Serialize;
use tauri::{self, async_runtime, CustomMenuItem, SystemTray, SystemTrayMenu, SystemTrayEvent,
    WindowBuilder,
    WindowUrl};
use tokio::{self,
    time::{interval, Duration},
};
use chrono::Local;

mod storage;
use storage::Storage;
mod utils;

static DURATION: usize = 2;

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
fn get_previous(_app: tauri::AppHandle) -> Vec<ChartInput> {
    let resource_path = _app
        .path_resolver()
        .resolve_resource("storage.db")
        .expect("failed to resolve resource");
    let _storage = Storage::init(&resource_path);
    let today = Local::today().naive_local();
    let x = _storage.get_previous(today, 30);
    x.into_iter().map(|ar| ChartInput{name: ar.0.to_string(), value: ar.1}).collect::<Vec<ChartInput>>()
}

#[tauri::command]
fn close_alert_window(window: tauri::Window) {
    window.close().unwrap();
}

fn main() {
    let timing = Timing{ in_progress: Mutex::new(None) };
    let (tx, mut rx) = tauri::async_runtime::channel(16);

    let start = CustomMenuItem::new("start".to_string(), "Start");
    let pause = CustomMenuItem::new("pause".to_string(), "Pause");
    let stop = CustomMenuItem::new("stop".to_string(), "Stop");
    let view = CustomMenuItem::new("view".to_string(), "View");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu_inactive = SystemTrayMenu::new()
        .add_item(start)
        .add_item(view.clone())
        .add_item(quit.clone());
    // below is the tray menu that is moved into the timer ui changing/ db writing thread
    let _tray_menu_inactive = tray_menu_inactive.clone();
    let tray_menu_active = SystemTrayMenu::new()
        .add_item(pause)
        .add_item(stop)
        .add_item(view)
        .add_item(quit);

    tauri::Builder::default()
        .setup(|app| {
            // dont show app icon on mac os's bottom menubar
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            // thread to receive messages from timer thread, updates timer ui, increments blocks
            // and shows alert window 
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

                            let resource_path = _app.path_resolver()
                                .resolve_resource("storage.db")
                                .expect("failed to resolve resouce");

                            let _storage = Storage::init(resource_path);
                            let today = Local::today().naive_local();
                            _storage.increment_or_insert_date(today);

                            WindowBuilder::new(
                                &_app,
                                "alert.html",
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
                        // spawn timer thread, add join handle to 'in_progress' state
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
                        // abort timer thread, update 'in_progress' state, update menu to inactive state
                        let mut data = timing.in_progress.lock().unwrap();
                        data.as_ref().unwrap().abort();
                        *data = None;
                        app.tray_handle().set_menu(tray_menu_inactive.clone()).unwrap();
                        app.tray_handle().set_title("Inactive").unwrap();
                    },
                    "pause" => {
                    },
                    "view" => {
                        WindowBuilder::new(
                            app,
                            "view",
                            WindowUrl::App("index.html".into())
                        ).build().unwrap();
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



use std::sync::{Arc, Mutex};
use tauri::{self, async_runtime, CustomMenuItem, SystemTray, SystemTrayMenu, SystemTrayEvent, App};
use tokio::{self,
    time::{interval, Duration},
};
use chrono::{Local};
use storage::Storage;

mod storage;

static DB_PATH: &str = "./storage.rs";
static DURATION: usize = 60 * 25;

enum TimeMessage {
    Time(usize),
    Finished,
}

struct Timing {
    in_progress: Mutex<Option<async_runtime::JoinHandle<()>>>,
}

fn main() {
    let state = Arc::new(Mutex::new(Storage::build(DB_PATH)));
    
    let timing = Timing{ in_progress: Mutex::new(None) };
    let (tx, mut rx) = tauri::async_runtime::channel(16);

    let start = CustomMenuItem::new("start".to_string(), "Start");
    let pause = CustomMenuItem::new("pause".to_string(), "Pause");
    let stop = CustomMenuItem::new("stop".to_string(), "Stop");

    let tray_menu_inactive = SystemTrayMenu::new()
        .add_item(start);
    let tray_menu_active = SystemTrayMenu::new()
        .add_item(pause)
        .add_item(stop);

    tauri::Builder::default()
    .manage(Arc::clone(&state))
    .setup(|app| {
           let _app = app.handle();
           // manages ui of system tray, increments date upon timer completion
           async_runtime::spawn(async move {
               loop {
                   match rx.recv().await.unwrap() {
                       TimeMessage::Time(time) => _app.tray_handle().set_title(&format!("{}", time)).unwrap(),
                       TimeMessage::Finished => {
                           _app.tray_handle().set_title("Inactive").unwrap();

                           let today = Local::today().naive_local();
                           let data = state.lock().unwrap();
                           data.increment_or_insert_date(today);
                       },
                   }
               }
           });
           Ok(())
    })
    .system_tray(SystemTray::new().with_menu(tray_menu_inactive.clone()))
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
                            tx1.send(TimeMessage::Time(i)).await;
                        }

                        tx1.send(TimeMessage::Finished).await;
                    });

                    let mut data = timing.in_progress.lock().unwrap();
                    *data = Some(join_handle);
                    
                    // update menu to active state 
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

                }

                _ => (),
            }
        }
        SystemTrayEvent::LeftClick {..} => (),
        _ => (),
    })
    .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


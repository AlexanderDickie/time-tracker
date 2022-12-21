use std::path::PathBuf;
use tauri::AppHandle;

pub fn format_time_timer(elapsed: u32) -> String {
    let _hours = elapsed / 3600;
    let _minutes = (elapsed % 3600) / 60;
    let _secs = elapsed % 60;
    format!("{:0>2}:{:0>2}:{:0>2}", _hours,  _minutes, _secs)
}

pub fn format_time_countdown(elapsed: usize, total: usize) -> String {
    let remaining = total - elapsed;
    let _hours = remaining / 3600;
    let _minutes = (remaining % 3600) / 60;
    let _secs = remaining % 60;
    format!("{:0>2}:{:0>2}:{:0>2}", _hours,  _minutes, _secs)
}

pub fn get_db_path(app_handle: &AppHandle) -> PathBuf {
    app_handle
        .path_resolver()
        .resolve_resource("storage.db")
        .expect("failed to resolve resource")
} 



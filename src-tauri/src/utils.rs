pub fn format_time_remaining(elapsed: usize, total: usize) -> String {
    let remaining = total - elapsed;
    let _hours = remaining / 3600;
    let _minutes = (remaining % 3600) / 60;
    let _secs = remaining % 60;
    format!("{:0>2}:{:0>2}:{:0>2}", _hours,  _minutes, _secs)
}



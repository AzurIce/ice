use chrono::Local;

pub fn get_cur_time_str() -> String {
    Local::now().format("%Y-%m-%d %H-%M-%S").to_string()
}

#[macro_export]
macro_rules! log {
    ($($t:tt)*) => {
        #[cfg(debug_assertions)]
        {
            console_log(format!($($t)*));
        }
    }
}

pub fn console_log(s: String) {
    web_sys::console::log_1(&s.into());
}

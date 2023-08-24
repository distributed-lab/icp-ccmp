#[macro_export]
macro_rules! storage_get {
    ($field:ident) => {{
        $crate::STORAGE.with(|state| state.borrow().$field.clone())
    }};
}

#[macro_export]
macro_rules! storage_set {
    ($field:ident, $value:expr) => {{
        $crate::STORAGE.with(|state| state.borrow_mut().$field = $value)
    }};
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        ic_cdk::println!($($arg)*);
        ic_utils::logger::log_message(format!($($arg)*));
        ic_utils::monitor::collect_metrics();
    }};
}

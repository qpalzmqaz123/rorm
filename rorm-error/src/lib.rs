pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Connection: `{0}`")]
    Connection(String),
    #[error("Timeout: `{0}`")]
    Timeout(String),
    #[error("Database: `{0}`")]
    Database(String),
    #[error("Runtime: `{0}`")]
    Runtime(String),
    #[error("FromValue: `{0}`")]
    FromValue(String),
    #[error("OutOfRange: `{0}`")]
    OutOfRange(String),
}

#[macro_export]
macro_rules! connection {
    ($($arg:tt)*) => { $crate::Error::Connection(format!($($arg)*)) };
}

#[macro_export]
macro_rules! timeout {
    ($($arg:tt)*) => { $crate::Error::Timeout(format!($($arg)*)) };
}

#[macro_export]
macro_rules! database {
    ($($arg:tt)*) => { $crate::Error::Database(format!($($arg)*)) };
}

#[macro_export]
macro_rules! runtime {
    ($($arg:tt)*) => { $crate::Error::Runtime(format!($($arg)*)) };
}

#[macro_export]
macro_rules! from_value {
    ($($arg:tt)*) => { $crate::Error::FromValue(format!($($arg)*)) };
}

#[macro_export]
macro_rules! out_of_range {
    ($($arg:tt)*) => { $crate::Error::OutOfRange(format!($($arg)*)) };
}

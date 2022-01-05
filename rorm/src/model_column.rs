use crate::pool::{ToValue, Value};

#[derive(Debug)]
pub enum ModelColumn<T> {
    NotSet,
    Set(T),
}

impl<T: ToValue> ToValue for ModelColumn<T> {
    fn to_value(&self) -> Value {
        match &self {
            Self::NotSet => Value::Null,
            Self::Set(v) => v.to_value(),
        }
    }
}

impl<T> Default for ModelColumn<T> {
    fn default() -> Self {
        Self::NotSet
    }
}

impl<T: ToValue> From<T> for ModelColumn<T> {
    fn from(v: T) -> Self {
        Self::Set(v)
    }
}

impl From<&str> for ModelColumn<String> {
    fn from(v: &str) -> Self {
        Self::Set(v.to_string())
    }
}

impl From<Option<&str>> for ModelColumn<Option<String>> {
    fn from(v: Option<&str>) -> Self {
        Self::Set(v.map(|s| s.into()))
    }
}

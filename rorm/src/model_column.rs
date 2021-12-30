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

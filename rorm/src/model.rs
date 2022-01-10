use crate::{
    query::{and, eq, Where},
    ToValue, Value,
};

pub trait Model<K>: Sized {
    fn into_set_pairs(self) -> Vec<(&'static str, Value)>; // (column, value)

    fn to_primary_key(id: u64) -> K;

    fn gen_where_and_params(self) -> (Option<Where>, Vec<Value>) {
        let pairs = self.into_set_pairs();
        if pairs.is_empty() {
            (None, vec![])
        } else {
            let mut params = vec![];
            let mut cond = None;

            for (col, value) in pairs {
                let c = eq!(col, "?");
                cond = if let Some(cond) = cond {
                    Some(and!(cond, c))
                } else {
                    Some(c)
                };

                params.push(value);
            }

            (cond, params)
        }
    }
}

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

impl<T> From<T> for ModelColumn<T> {
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

macro_rules! impl_from_for_signedint {
    ($($ty:ty),+) => {
        $(
            impl From<$ty> for Value {
                fn from(v: $ty) -> Self {
                    Self::SignedInt(v as i64)
                }
            }
        )+
    };
}

macro_rules! impl_from_for_unsignedint {
    ($($ty:ty),+) => {
        $(
            impl From<$ty> for Value {
                fn from(v: $ty) -> Self {
                    Self::UnsignedInt(v as u64)
                }
            }
        )+
    };
}

macro_rules! impl_from_for_float {
    ($($ty:ty),+) => {
        $(
            impl From<$ty> for Value {
                fn from(v: $ty) -> Self {
                    Self::Float(v as f64)
                }
            }
        )+
    };
}

macro_rules! impl_from_for_column {
    ($($ty:ty),+) => {
        $(
            impl From<$ty> for Value {
                fn from(v: $ty) -> Self {
                    Self::Column(v.to_string())
                }
            }
        )+
    };
}

#[derive(Debug)]
pub enum Value {
    Bool(bool),
    SignedInt(i64),
    UnsignedInt(u64),
    Float(f64),
    Column(String),
    Str(String),
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match &self {
            Self::Bool(v) => v.to_string(),
            Self::SignedInt(v) => v.to_string(),
            Self::UnsignedInt(v) => v.to_string(),
            Self::Float(v) => v.to_string(),
            Self::Column(v) => v.to_string(),
            Self::Str(v) => format!("'{}'", v),
        }
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl_from_for_signedint! {i8, i64, i32}
impl_from_for_unsignedint! {u8, u64, u32}
impl_from_for_float! {f32, f64}
impl_from_for_column! {&str, String}

pub fn sql_str(v: impl ToString) -> Value {
    Value::Str(v.to_string())
}

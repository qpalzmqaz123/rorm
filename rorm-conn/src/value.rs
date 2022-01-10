use rorm_error::Result;

macro_rules! impl_to_value_base {
    ($ty:ty, $enum_field:ident) => {
        impl ToValue for $ty {
            fn to_value(&self) -> Value {
                Value::$enum_field(self.clone())
            }
        }
    };
}

macro_rules! impl_from_value_integer {
    ($ty:ty) => {
        impl FromValue for $ty {
            type Output = $ty;

            fn from_value(v: &Value) -> Result<Self::Output> {
                match v {
                    Value::U8(v) => Ok(*v as $ty),
                    Value::I8(v) => Ok(*v as $ty),
                    Value::U16(v) => Ok(*v as $ty),
                    Value::I16(v) => Ok(*v as $ty),
                    Value::U32(v) => Ok(*v as $ty),
                    Value::I32(v) => Ok(*v as $ty),
                    Value::U64(v) => Ok(*v as $ty),
                    Value::I64(v) => Ok(*v as $ty),
                    _ => Err(rorm_error::from_value!(
                        "Invalid value: {:?}, output type: {}",
                        v,
                        stringify!($ty)
                    )),
                }
            }
        }
    };
}

macro_rules! impl_from_value_float {
    ($ty:ty) => {
        impl FromValue for $ty {
            type Output = $ty;

            fn from_value(v: &Value) -> Result<Self::Output> {
                match v {
                    Value::F32(v) => Ok(*v as $ty),
                    Value::F64(v) => Ok(*v as $ty),
                    _ => Err(rorm_error::from_value!(
                        "Invalid value: {:?}, output type: {}",
                        v,
                        stringify!($ty)
                    )),
                }
            }
        }
    };
}

macro_rules! impl_from_value_base {
    ($ty:ty, $enum_field:ident) => {
        impl FromValue for $ty {
            type Output = $ty;

            fn from_value(v: &Value) -> Result<Self::Output> {
                match v {
                    Value::$enum_field(v) => Ok(v.clone()),
                    _ => Err(rorm_error::from_value!(
                        "Invalid value: {:?}, output type: {}",
                        v,
                        stringify!($ty)
                    )),
                }
            }
        }
    };
}

#[derive(Debug)]
pub enum Value {
    Null,
    Bool(bool),
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    F32(f32),
    F64(f64),
    Str(String),
    Bytes(Vec<u8>),
}

pub trait ToValue {
    fn to_value(&self) -> Value;
}

impl_to_value_base! {bool, Bool}
impl_to_value_base! {u8, U8}
impl_to_value_base! {i8, I8}
impl_to_value_base! {u16, U16}
impl_to_value_base! {i16, I16}
impl_to_value_base! {u32, U32}
impl_to_value_base! {i32, I32}
impl_to_value_base! {u64, U64}
impl_to_value_base! {i64, I64}
impl_to_value_base! {f32, F32}
impl_to_value_base! {f64, F64}
impl_to_value_base! {String, Str}
impl_to_value_base! {Vec<u8>, Bytes}

impl<T: ToValue> ToValue for Option<T> {
    fn to_value(&self) -> Value {
        if let Some(v) = self {
            T::to_value(v)
        } else {
            Value::Null
        }
    }
}

pub trait FromValue {
    type Output;

    fn from_value(v: &Value) -> Result<Self::Output>;
}

impl<T: FromValue> FromValue for Option<T> {
    type Output = Option<<T as FromValue>::Output>;

    fn from_value(v: &Value) -> Result<Self::Output> {
        match v {
            Value::Null => Ok(None),
            _ => Ok(Some(T::from_value(v)?)),
        }
    }
}

impl FromValue for bool {
    type Output = bool;

    fn from_value(v: &Value) -> Result<Self::Output> {
        match v {
            Value::Bool(v) => Ok(*v),
            Value::U8(v) => Ok(if *v != 0 { true } else { false }),
            Value::I8(v) => Ok(if *v != 0 { true } else { false }),
            Value::U16(v) => Ok(if *v != 0 { true } else { false }),
            Value::I16(v) => Ok(if *v != 0 { true } else { false }),
            Value::U32(v) => Ok(if *v != 0 { true } else { false }),
            Value::I32(v) => Ok(if *v != 0 { true } else { false }),
            Value::U64(v) => Ok(if *v != 0 { true } else { false }),
            Value::I64(v) => Ok(if *v != 0 { true } else { false }),
            _ => Err(rorm_error::from_value!(
                "Invalid value: {:?}, output type: {}",
                v,
                stringify!(bool)
            )),
        }
    }
}

impl_from_value_integer! {u8}
impl_from_value_integer! {i8}
impl_from_value_integer! {u16}
impl_from_value_integer! {i16}
impl_from_value_integer! {u32}
impl_from_value_integer! {i32}
impl_from_value_integer! {u64}
impl_from_value_integer! {i64}

impl_from_value_float! {f32}
impl_from_value_float! {f64}

impl_from_value_base! {String, Str}
impl_from_value_base! {Vec<u8>, Bytes}

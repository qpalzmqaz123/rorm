macro_rules! impl_condition_for_unsigned_int {
    ($ty:ty) => {
        impl From<$ty> for Where {
            fn from(v: $ty) -> Self {
                Self::UnsignedInt(v as u64)
            }
        }
    };
}

macro_rules! impl_condition_for_signed_int {
    ($ty:ty) => {
        impl From<$ty> for Where {
            fn from(v: $ty) -> Self {
                Self::SignedInt(v as i64)
            }
        }
    };
}

macro_rules! impl_condition_for_float {
    ($ty:ty) => {
        impl From<$ty> for Where {
            fn from(v: $ty) -> Self {
                Self::Float(v as f64)
            }
        }
    };
}

#[derive(Debug)]
pub enum Where {
    And(Box<Where>, Box<Where>),
    Or(Box<Where>, Box<Where>),
    Not(Box<Where>),
    Eq(Box<Where>, Box<Where>),
    Ne(Box<Where>, Box<Where>),
    Lt(Box<Where>, Box<Where>),
    Le(Box<Where>, Box<Where>),
    Gt(Box<Where>, Box<Where>),
    Ge(Box<Where>, Box<Where>),
    Between(Box<Where>, Box<Where>, Box<Where>),
    In(Box<Where>, Vec<Where>),
    Like(Box<Where>, Box<Where>),
    Quote(Box<Where>),
    SignedInt(i64),
    UnsignedInt(u64),
    Float(f64),
    Bool(bool),
    Str(String),
}

impl_condition_for_unsigned_int! {u8}
impl_condition_for_unsigned_int! {u16}
impl_condition_for_unsigned_int! {u32}
impl_condition_for_signed_int! {i8}
impl_condition_for_signed_int! {i16}
impl_condition_for_signed_int! {i32}
impl_condition_for_float! {f32}
impl_condition_for_float! {f64}

impl From<bool> for Where {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<String> for Where {
    fn from(v: String) -> Self {
        Self::Str(v)
    }
}

impl From<&str> for Where {
    fn from(v: &str) -> Self {
        Self::Str(v.into())
    }
}

impl ToString for Where {
    fn to_string(&self) -> String {
        match &self {
            Self::And(l, r) => format!("({} AND {})", l.to_string(), r.to_string()),
            Self::Or(l, r) => format!("({} OR {})", l.to_string(), r.to_string()),
            Self::Not(v) => format!("(NOT {})", v.to_string()),
            Self::Eq(l, r) => format!("({} = {})", l.to_string(), r.to_string()),
            Self::Ne(l, r) => format!("({} <> {})", l.to_string(), r.to_string()),
            Self::Lt(l, r) => format!("({} < {})", l.to_string(), r.to_string()),
            Self::Le(l, r) => format!("({} <= {})", l.to_string(), r.to_string()),
            Self::Gt(l, r) => format!("({} > {})", l.to_string(), r.to_string()),
            Self::Ge(l, r) => format!("({} >= {})", l.to_string(), r.to_string()),
            Self::Between(var, l, r) => format!(
                "({} BETWEEN {} AND {})",
                var.to_string(),
                l.to_string(),
                r.to_string()
            ),
            Self::In(var, list) => format!(
                "({} IN ({}))",
                var.to_string(),
                list.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Like(var, lik) => format!("({} LIKE {})", var.to_string(), lik.to_string()),
            Self::Quote(cond) => format!("'{}'", cond.to_string()),
            Self::SignedInt(n) => n.to_string(),
            Self::UnsignedInt(n) => n.to_string(),
            Self::Float(f) => f.to_string(),
            Self::Bool(v) => v.to_string(),
            Self::Str(v) => v.to_string(),
        }
    }
}

#[macro_export]
macro_rules! and {
    ($left:expr, $right:expr) => {
        $crate::Where::And(
            Box::new($crate::literal!($left)),
            Box::new($crate::literal!($right)),
        )
    };
}

#[macro_export]
macro_rules! or {
    ($left:expr, $right:expr) => {
        $crate::Where::Or(
            Box::new($crate::literal!($left)),
            Box::new($crate::literal!($right)),
        )
    };
}

#[macro_export]
macro_rules! literal {
    ($lit:expr) => {
        $crate::Where::from($lit)
    };
}

#[macro_export]
macro_rules! not {
    ($expr:expr) => {
        $crate::Where::Not(Box::new($crate::literal!($expr)))
    };
}

#[macro_export]
macro_rules! quote {
    ($expr:expr) => {
        $crate::Where::Quote(Box::new($crate::literal!($expr)))
    };
}

#[macro_export]
macro_rules! eq {
    ($left:expr, $right:expr) => {
        $crate::Where::Eq(
            Box::new($crate::literal!($left)),
            Box::new($crate::literal!($right)),
        )
    };
}

#[macro_export]
macro_rules! ne {
    ($left:expr, $right:expr) => {
        $crate::Where::Ne(
            Box::new($crate::literal!($left)),
            Box::new($crate::literal!($right)),
        )
    };
}

#[macro_export]
macro_rules! lt {
    ($left:expr, $right:expr) => {
        $crate::Where::Lt(
            Box::new($crate::literal!($left)),
            Box::new($crate::literal!($right)),
        )
    };
}

#[macro_export]
macro_rules! le {
    ($left:expr, $right:expr) => {
        $crate::Where::Le(
            Box::new($crate::literal!($left)),
            Box::new($crate::literal!($right)),
        )
    };
}

#[macro_export]
macro_rules! gt {
    ($left:expr, $right:expr) => {
        $crate::Where::Gt(
            Box::new($crate::literal!($left)),
            Box::new($crate::literal!($right)),
        )
    };
}

#[macro_export]
macro_rules! ge {
    ($left:expr, $right:expr) => {
        $crate::Where::Ge(
            Box::new($crate::literal!($left)),
            Box::new($crate::literal!($right)),
        )
    };
}

#[macro_export]
macro_rules! between {
    ($var:expr, [$left:expr, $right:expr]) => {
        $crate::Where::Between(
            Box::new($crate::literal!($var)),
            Box::new($crate::literal!($left)),
            Box::new($crate::literal!($right)),
        )
    };
}

#[macro_export]
macro_rules! r#in {
    ($var:expr, [$($expr:expr),+]) => {
        $crate::Where::In(Box::new($crate::literal!($var)), vec![$($crate::literal!($expr)),+])
    };
}

#[macro_export]
macro_rules! like {
    ($left:expr, $right:expr) => {
        $crate::Where::Like(
            Box::new($crate::literal!($left)),
            Box::new($crate::literal!($right)),
        )
    };
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn test1() {
        assert_eq!(&and!(1, 2).to_string(), "(1 AND 2)");

        assert_eq!(&or!(1, 2).to_string(), "(1 OR 2)");

        assert_eq!(&eq!(1, 2).to_string(), "(1 = 2)");

        let n = 2;
        assert_eq!(&eq!(1, n).to_string(), "(1 = 2)");

        assert_eq!(&eq!("a", 1).to_string(), "(a = 1)");

        assert_eq!(&quote!("1").to_string(), "'1'");

        assert_eq!(&quote!("abc").to_string(), "'abc'");

        assert_eq!(&eq!("a", quote!("abc")).to_string(), "(a = 'abc')");

        assert_eq!(&eq!(eq!(1, 2), true).to_string(), "((1 = 2) = true)");

        assert_eq!(&ne!(eq!(1, 2), true).to_string(), "((1 = 2) <> true)");

        assert_eq!(&lt!(1, 2).to_string(), "(1 < 2)");

        assert_eq!(&le!(1, 2).to_string(), "(1 <= 2)");

        assert_eq!(&gt!(1, 2).to_string(), "(1 > 2)");

        assert_eq!(&ge!(1, 2).to_string(), "(1 >= 2)");

        assert_eq!(&(ge!(1, 2)).to_string(), "(1 >= 2)");

        assert_eq!(&not!(ge!(1, 2)).to_string(), "(NOT (1 >= 2))");

        assert_eq!(&(between!("a", [1, 2])).to_string(), "(a BETWEEN 1 AND 2)");

        assert_eq!(&r#in!("a", [1, 2, 3]).to_string(), "(a IN (1, 2, 3))");

        assert_eq!(&like!("a", quote!("abc")).to_string(), "(a LIKE 'abc')");
    }
}

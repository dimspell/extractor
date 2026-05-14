use serde::{Deserialize, Serialize};

/// Scalar value carried in a [`FieldDelta`](super::change::ChangeOp::FieldDelta).
///
/// Kept deliberately small so deltas serialise cleanly and do not leak
/// `serde_json::Value` into the rest of the codebase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value")]
pub enum Value {
    String(String),
    I64(i64),
    F64(f64),
    Bool(bool),
    Bytes(Vec<u8>),
    Null,
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::String(_) => "string",
            Value::I64(_) => "i64",
            Value::F64(_) => "f64",
            Value::Bool(_) => "bool",
            Value::Bytes(_) => "bytes",
            Value::Null => "null",
        }
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(v)
    }
}
impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(v.to_owned())
    }
}
impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::I64(v)
    }
}
impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::I64(v as i64)
    }
}
impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Value::I64(v as i64)
    }
}
impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::F64(v)
    }
}
impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}
impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        Value::Bytes(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_each_variant() {
        let values = vec![
            Value::String("Helmet".into()),
            Value::I64(-12),
            Value::F64(3.5),
            Value::Bool(true),
            Value::Bytes(vec![0xDE, 0xAD, 0xBE, 0xEF]),
            Value::Null,
        ];
        for v in values {
            let s = serde_json::to_string(&v).unwrap();
            let back: Value = serde_json::from_str(&s).unwrap();
            assert_eq!(v, back, "round trip failed for {}", v.type_name());
        }
    }

    #[test]
    fn json_shape_is_tag_value() {
        let v = Value::String("Helmet".into());
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!(s, r#"{"kind":"String","value":"Helmet"}"#);
    }
}

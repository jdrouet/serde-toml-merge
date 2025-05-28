use std::fmt;
use toml::map::Map;
use toml::Value;

#[derive(Debug, PartialEq)]
pub struct Error {
    pub path: String,
    pub expected: &'static str,
    pub existing: &'static str,
}

impl Error {
    pub fn new(path: String, expected: &'static str, existing: &'static str) -> Self {
        Self {
            path,
            expected,
            existing,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "Incompatible types at path \"{}\", expected \"{}\" received \"{}\".",
            self.path, self.expected, self.existing
        )
    }
}

fn merge_into_table_inner(
    value: &mut Map<String, Value>,
    other: Map<String, Value>,
    path: &str,
    replace_arrays: bool,
) -> Result<(), Error> {
    for (name, inner) in other {
        if let Some(existing) = value.remove(&name) {
            let inner_path = format!("{path}.{name}");
            value.insert(
                name,
                merge_inner(existing, inner, &inner_path, replace_arrays)?,
            );
        } else {
            value.insert(name, inner);
        }
    }
    Ok(())
}

/// Merges two toml tables into a single one.
pub fn merge_tables(
    mut value: Map<String, Value>,
    other: Map<String, Value>,
    replace_arrays: bool,
) -> Result<Map<String, Value>, Error> {
    merge_into_table_inner(&mut value, other, "$", replace_arrays)?;
    Ok(value)
}

/// Merges two toml tables into a single one.
pub fn merge_into_table(
    value: &mut Map<String, Value>,
    other: Map<String, Value>,
    replace_arrays: bool,
) -> Result<(), Error> {
    merge_into_table_inner(value, other, "$", replace_arrays)
}

fn merge_inner(
    value: Value,
    other: Value,
    path: &str,
    replace_arrays: bool,
) -> Result<Value, Error> {
    match (value, other) {
        (Value::String(_), Value::String(inner)) => Ok(Value::String(inner)),
        (Value::Integer(_), Value::Integer(inner)) => Ok(Value::Integer(inner)),
        (Value::Float(_), Value::Float(inner)) => Ok(Value::Float(inner)),
        (Value::Boolean(_), Value::Boolean(inner)) => Ok(Value::Boolean(inner)),
        (Value::Datetime(_), Value::Datetime(inner)) => Ok(Value::Datetime(inner)),
        (Value::Array(_), Value::Array(inner)) if replace_arrays => Ok(Value::Array(inner)),
        (Value::Array(mut existing), Value::Array(inner)) if !replace_arrays => {
            existing.extend(inner);
            Ok(Value::Array(existing))
        }
        (Value::Table(mut existing), Value::Table(inner)) => {
            merge_into_table_inner(&mut existing, inner, path, replace_arrays)?;
            Ok(Value::Table(existing))
        }
        (v, o) => Err(Error::new(path.to_owned(), v.type_str(), o.type_str())),
    }
}

/// Merges two toml values into a single one.
pub fn merge(value: Value, other: Value, replace_arrays: bool) -> Result<Value, Error> {
    merge_inner(value, other, "$", replace_arrays)
}

#[cfg(test)]
mod tests {
    use crate::{merge, Error};
    use toml::Value;

    macro_rules! should_fail {
        ($first: expr, $second: expr) => {
            should_fail!($first, $second,)
        };
        ($first: expr, $second: expr,) => {{
            let first = $first.parse::<Value>().unwrap();
            let second = $second.parse::<Value>().unwrap();
            merge(first, second, false).unwrap_err()
        }};
    }

    macro_rules! should_match {
        // 4-argument form with replace_arrays flag
        ($first:expr, $second:expr, $result:expr, $replace_arrays:expr) => {{
            let first = $first.parse::<Value>().unwrap();
            let second = $second.parse::<Value>().unwrap();
            let result = $result.parse::<Value>().unwrap();
            assert_eq!(merge(first, second, ($replace_arrays)).unwrap(), result);
        }};
        // 3-argument fallback: default replace_arrays = false
        ($first:expr, $second:expr, $result:expr) => {
            should_match!($first, $second, $result, false)
        };
    }
    #[test]
    fn with_basic() {
        should_match!(
            r#"
        string = "foo"
        integer = 42
        float = 42.24
        boolean = true
        keep_me = true
        "#,
            r#"
        string = "bar"
        integer = 43
        float = 24.42
        boolean = false
        missing = true
        "#,
            r#"
        string = "bar"
        integer = 43
        float = 24.42
        boolean = false
        keep_me = true
        missing = true
        "#
        );
    }

    #[test]
    fn with_array_merged() {
        should_match!(
            r#"foo = ["a", "b"]"#,
            r#"foo = ["c", "d"]"#,
            r#"foo = ["a", "b", "c", "d"]"#
        );
    }

    #[test]
    fn with_array_replaced() {
        should_match!(
            r#"foo = ["a", "b"]"#,
            r#"foo = ["c", "d"]"#,
            r#"foo = ["c", "d"]"#,
            true
        );
    }

    #[test]
    fn with_table() {
        should_match!(
            r#"
            [foo]
            bar = "baz"
        "#,
            r#"
            [foo]
            hello = "world"
        "#,
            r#"
            [foo]
            bar = "baz"
            hello = "world"
        "#
        );
    }

    #[test]
    fn invalid_kinds() {
        assert_eq!(
            should_fail!("foo = true", "foo = 42"),
            Error::new("$.foo".to_owned(), "boolean", "integer")
        );
        assert_eq!(
            should_fail!("foo = \"true\"", "foo = 42.5"),
            Error::new("$.foo".to_owned(), "string", "float")
        );
    }
}

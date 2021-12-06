# Serde Toml Merge

[![codecov](https://codecov.io/gh/jdrouet/serde-toml-merge/branch/main/graph/badge.svg?token=VR6M1YHRFA)](https://codecov.io/gh/jdrouet/serde-toml-merge)

Just like [serde_merge](https://crates.io/crates/serde_merge), this crate allows you to merge [`toml`](https://crates.io/crates/toml) values.

## How to use

```rust
use serde_toml_merge::merge;
use toml::Value;

let first = r#"
string = "foo"
integer = 42
float = 42.24
boolean = true
keep_me = true
"#.parse::<Value>().unwrap();

let second = r#"
string = "bar"
integer = 43
float = 24.42
boolean = false
missing = true
"#.parse::<Value>().unwrap();

let expected = r#"
integer = 43
float = 24.42
boolean = false
keep_me = true
missing = true
"#.pare::<Value>().unwrap();

assert_eq!(merge(first, second).unwrap(), expected);
```

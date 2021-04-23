Entity Tag
====================

[![CI](https://github.com/magiclen/entity-tag/actions/workflows/ci.yml/badge.svg)](https://github.com/magiclen/entity-tag/actions/workflows/ci.yml)

This crate provides a `EntityTag` structure and functions to deal with the ETag header field of HTTP.

## Examples

```rust
extern crate entity_tag;

use entity_tag::EntityTag;

let etag1 = EntityTag::with_str(true, "foo").unwrap();
let etag2 = EntityTag::from_str("\"foo\"").unwrap();

assert_eq!(true, etag1.weak);
assert_eq!(false, etag2.weak);

assert!(etag1.weak_eq(&etag2));
assert!(etag1.strong_ne(&etag2));

let etag3 = EntityTag::from_data(true, &[102, 111, 111]).unwrap();
assert_eq!("W/\"j4VF2Hjg0No\"", etag3.to_string());

let etag4 = EntityTag::from_file_meta(&std::fs::File::open("tests/data/P1060382.JPG").unwrap().metadata().unwrap());
assert_eq!("W/\"CmgjkoKAfwQ\"", etag4.to_string());
```

## No Std

Disable the default features to compile this crate without std.

```toml
[dependencies.entity-tag]
version = "*"
default-features = false
```

## Crates.io

https://crates.io/crates/entity-tag

## Documentation

https://docs.rs/entity-tag

## License

[MIT](LICENSE)
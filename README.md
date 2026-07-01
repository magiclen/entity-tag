Entity Tag
====================

[![CI](https://github.com/magiclen/entity-tag/actions/workflows/ci.yml/badge.svg)](https://github.com/magiclen/entity-tag/actions/workflows/ci.yml)

This crate provides a `EntityTag` structure and functions to deal with the ETag header field of HTTP.

## Examples

```rust
use entity_tag::EntityTag;

let etag1 = EntityTag::with_str(true, "foo").unwrap();
let etag2 = EntityTag::from_str("\"foo\"").unwrap();

assert_eq!(true, etag1.weak);
assert_eq!(false, etag2.weak);

assert!(etag1.weak_eq(&etag2));
assert!(etag1.strong_ne(&etag2));

let etag3 = EntityTag::from_data(&[102, 111, 111]);
assert_eq!(r#"W/"ea75LoNFQSGrbl9kB359ig""#, etag3.to_string());

let etag4 = EntityTag::from_data_strong(&[102, 111, 111]);
assert_eq!(r#""BOC7OfMLGj/rifU2yTvhUFVILfdIZ0sA0m5adXd3Auk""#, etag4.to_string());

let etag5 = EntityTag::from_file_meta(&std::fs::File::open("tests/data/P1060382.JPG").unwrap().metadata().unwrap());
assert_eq!(r#"W/"wMRGbc5gr2L/keSkynH4KQ""#, etag5.to_string());

let etag6 = EntityTag::from_file_meta_strong(&std::fs::File::open("tests/data/P1060382.JPG").unwrap().metadata().unwrap());
assert_eq!(r#""mmkAoOwMqROVvws3tlxJ9tIInflE3JGuOCn1REERmPo""#, etag6.to_string());
```

## No Std

Disable the default features to compile this crate without std.

```toml
[dependencies.entity-tag]
version = "*"
default-features = false
```

## Hashers

Generated ETag constructors are optional. Enable `weak-hasher` to use XXH3-128 weak ETag generation and `strong-hasher` to use BLAKE3-256 strong ETag generation.

## Crates.io

https://crates.io/crates/entity-tag

## Documentation

https://docs.rs/entity-tag

## License

[MIT](LICENSE)
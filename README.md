entity-tag
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
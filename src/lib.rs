/*!
# Entity Tag

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

# #[cfg(feature = "weak-hasher")]
# {
let etag3 = EntityTag::from_data(&[102, 111, 111]);
assert_eq!(r#"W/"ea75LoNFQSGrbl9kB359ig""#, etag3.to_string());
# }

# #[cfg(feature = "strong-hasher")]
# {
let etag4 = EntityTag::from_data_strong(&[102, 111, 111]);
assert_eq!(r#""BOC7OfMLGj/rifU2yTvhUFVILfdIZ0sA0m5adXd3Auk""#, etag4.to_string());
# }

# #[cfg(all(feature = "std", feature = "weak-hasher"))]
# {
let etag5 = EntityTag::from_file_meta(&std::fs::File::open("tests/data/P1060382.JPG").unwrap().metadata().unwrap());
assert_eq!(r#"W/"wMRGbc5gr2L/keSkynH4KQ""#, etag5.to_string());
# }

# #[cfg(all(feature = "std", feature = "strong-hasher"))]
# {
let etag6 = EntityTag::from_file_meta_strong(&std::fs::File::open("tests/data/P1060382.JPG").unwrap().metadata().unwrap());
assert_eq!(r#""mmkAoOwMqROVvws3tlxJ9tIInflE3JGuOCn1REERmPo""#, etag6.to_string());
# }
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
*/

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod entity_tag_error;

use alloc::{borrow::Cow, string::String};
use core::{
    fmt::{self, Display, Formatter, Write},
    str::FromStr,
};
#[cfg(all(feature = "std", any(feature = "weak-hasher", feature = "strong-hasher")))]
use std::fs::Metadata;
#[cfg(all(feature = "std", any(feature = "weak-hasher", feature = "strong-hasher")))]
use std::time::UNIX_EPOCH;

#[cfg(any(feature = "weak-hasher", feature = "strong-hasher"))]
use base64::{Engine, engine::general_purpose::STANDARD_NO_PAD};
pub use entity_tag_error::EntityTagError;
#[cfg(feature = "weak-hasher")]
use xxhash_rust::xxh3::xxh3_128;

#[cfg(feature = "strong-hasher")]
#[inline]
fn encode_blake3_256(data: &[u8]) -> String {
    STANDARD_NO_PAD.encode(blake3::hash(data).as_bytes())
}

#[cfg(feature = "weak-hasher")]
#[inline]
fn encode_xxh3_128_data(data: &[u8]) -> String {
    encode_xxh3_128(xxh3_128(data))
}

#[cfg(feature = "weak-hasher")]
#[inline]
fn encode_xxh3_128(hash: u128) -> String {
    STANDARD_NO_PAD.encode(hash.to_be_bytes())
}

#[cfg(all(feature = "std", any(feature = "weak-hasher", feature = "strong-hasher")))]
fn encode_file_meta<F>(metadata: &Metadata, encode: F) -> String
where
    F: Fn(&[u8]) -> String, {
    let len_bytes = metadata.len().to_le_bytes();

    if let Ok(modified_time) = metadata.modified() {
        match modified_time.duration_since(UNIX_EPOCH) {
            Ok(time) => {
                let mut bytes = [0; 24];

                bytes[..8].copy_from_slice(&len_bytes);
                bytes[8..].copy_from_slice(&time.as_nanos().to_le_bytes());

                encode(&bytes)
            },
            Err(err) => {
                let mut bytes = [0; 25];

                bytes[..8].copy_from_slice(&len_bytes);
                bytes[8] = b'-';
                bytes[9..].copy_from_slice(&err.duration().as_nanos().to_le_bytes());

                encode(&bytes)
            },
        }
    } else {
        encode(&len_bytes)
    }
}

/// An entity tag, defined in [RFC7232](https://tools.ietf.org/html/rfc7232#section-2.3).
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct EntityTag<'t> {
    /// Whether to have a weakness indicator.
    pub weak: bool,
    /// The opaque tag content without the surrounding double quotes.
    tag:      Cow<'t, str>,
}

impl<'t> EntityTag<'t> {
    /// `ETag`
    pub const HEADER_NAME: &'static str = "ETag";
}

impl<'t> EntityTag<'t> {
    /// Construct a new EntityTag without checking.
    ///
    /// # Safety
    /// The caller must ensure `tag` is a valid unquoted entity-tag value, because this function skips all validation.
    #[inline]
    pub const unsafe fn new_unchecked(weak: bool, tag: Cow<'t, str>) -> Self {
        EntityTag {
            weak,
            tag,
        }
    }

    /// Get the tag. The double quotes are not included.
    #[inline]
    pub const fn get_tag_cow(&self) -> &Cow<'t, str> {
        &self.tag
    }
}

impl<'t> EntityTag<'t> {
    /// Construct a new EntityTag without checking.
    ///
    /// # Safety
    /// The caller must ensure `tag` is a valid unquoted entity-tag value, because this function skips all validation.
    #[inline]
    pub unsafe fn with_string_unchecked<S: Into<String>>(weak: bool, tag: S) -> EntityTag<'static> {
        EntityTag {
            weak,
            tag: Cow::from(tag.into()),
        }
    }

    /// Construct a new EntityTag without checking.
    ///
    /// # Safety
    /// The caller must ensure `tag` is a valid unquoted entity-tag value, because this function skips all validation.
    #[inline]
    pub unsafe fn with_str_unchecked<S: ?Sized + AsRef<str>>(weak: bool, tag: &'t S) -> Self {
        EntityTag {
            weak,
            tag: Cow::from(tag.as_ref()),
        }
    }
}

impl<'t> EntityTag<'t> {
    #[inline]
    fn check_unquoted_tag(s: &str) -> Result<(), EntityTagError> {
        if s.bytes().all(|c| c == b'\x21' || (b'\x23'..=b'\x7e').contains(&c) || c >= b'\x80') {
            Ok(())
        } else {
            Err(EntityTagError::InvalidTag)
        }
    }

    fn check_tag(s: &str) -> Result<bool, EntityTagError> {
        let (s, quoted) =
            if let Some(stripped) = s.strip_prefix('"') { (stripped, true) } else { (s, false) };

        let s = if quoted {
            if let Some(stripped) = s.strip_suffix('"') {
                stripped
            } else {
                return Err(EntityTagError::MissingClosingDoubleQuote);
            }
        } else {
            s
        };

        // now check the ETag characters

        Self::check_unquoted_tag(s)?;

        Ok(quoted)
    }

    /// Construct a new EntityTag from an opaque tag, with or without surrounding double quotes.
    #[inline]
    pub fn with_string<S: AsRef<str> + Into<String>>(
        weak: bool,
        tag: S,
    ) -> Result<EntityTag<'static>, EntityTagError> {
        let quoted = Self::check_tag(tag.as_ref())?;

        let mut tag = tag.into();

        if quoted {
            tag.remove(tag.len() - 1);
            tag.remove(0);
        }

        Ok(EntityTag {
            weak,
            tag: Cow::from(tag),
        })
    }

    /// Construct a new EntityTag from an opaque tag, with or without surrounding double quotes.
    #[inline]
    pub fn with_str<S: ?Sized + AsRef<str>>(
        weak: bool,
        tag: &'t S,
    ) -> Result<Self, EntityTagError> {
        let tag = tag.as_ref();

        let quoted = Self::check_tag(tag)?;

        let tag = if quoted { &tag[1..(tag.len() - 1)] } else { tag };

        Ok(EntityTag {
            weak,
            tag: Cow::from(tag),
        })
    }
}

impl<'t> EntityTag<'t> {
    #[inline]
    fn check_opaque_tag(s: &str) -> Result<(), EntityTagError> {
        if let Some(s) = s.strip_prefix('"') {
            if let Some(s) = s.strip_suffix('"') {
                // now check the ETag characters
                Self::check_unquoted_tag(s)
            } else {
                Err(EntityTagError::MissingClosingDoubleQuote)
            }
        } else {
            Err(EntityTagError::MissingStartingDoubleQuote)
        }
    }

    /// Parse and construct a new EntityTag from a full ETag header value.
    pub fn from_string<S: AsRef<str> + Into<String>>(
        etag: S,
    ) -> Result<EntityTag<'static>, EntityTagError> {
        let weak = {
            let s = etag.as_ref();

            let (weak, opaque_tag) = if let Some(opaque_tag) = s.strip_prefix("W/") {
                (true, opaque_tag)
            } else {
                (false, s)
            };

            Self::check_opaque_tag(opaque_tag)?;

            weak
        };

        let mut tag = etag.into();

        tag.remove(tag.len() - 1);

        if weak {
            tag.replace_range(..3, "");
        } else {
            tag.remove(0);
        }

        Ok(EntityTag {
            weak,
            tag: Cow::from(tag),
        })
    }

    /// Parse and construct a new EntityTag from a full ETag header value.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str<S: ?Sized + AsRef<str>>(etag: &'t S) -> Result<Self, EntityTagError> {
        let s = etag.as_ref();

        let (weak, opaque_tag) = if let Some(opaque_tag) = s.strip_prefix("W/") {
            (true, opaque_tag)
        } else {
            (false, s)
        };

        Self::check_opaque_tag(opaque_tag)?;

        Ok(EntityTag {
            weak,
            tag: Cow::from(&opaque_tag[1..(opaque_tag.len() - 1)]),
        })
    }

    #[cfg(feature = "weak-hasher")]
    /// Construct a weak EntityTag from data with an XXH3-128 fingerprint.
    #[inline]
    pub fn from_data<S: ?Sized + AsRef<[u8]>>(data: &S) -> EntityTag<'static> {
        let tag = encode_xxh3_128_data(data.as_ref());

        EntityTag {
            weak: true, tag: Cow::from(tag)
        }
    }

    #[cfg(feature = "strong-hasher")]
    /// Construct a strong EntityTag from data with a BLAKE3-256 digest.
    #[inline]
    pub fn from_data_strong<S: ?Sized + AsRef<[u8]>>(data: &S) -> EntityTag<'static> {
        let tag = encode_blake3_256(data.as_ref());

        EntityTag {
            weak: false, tag: Cow::from(tag)
        }
    }

    #[cfg(all(feature = "std", feature = "weak-hasher"))]
    /// Construct a weak EntityTag from file metadata with an XXH3-128 fingerprint.
    #[inline]
    pub fn from_file_meta(metadata: &Metadata) -> EntityTag<'static> {
        let tag = encode_file_meta(metadata, encode_xxh3_128_data);

        EntityTag {
            weak: true, tag: Cow::from(tag)
        }
    }

    #[cfg(all(feature = "std", feature = "strong-hasher"))]
    /// Construct a strong EntityTag from file metadata with a BLAKE3-256 digest.
    #[inline]
    pub fn from_file_meta_strong(metadata: &Metadata) -> EntityTag<'static> {
        let tag = encode_file_meta(metadata, encode_blake3_256);

        EntityTag {
            weak: false, tag: Cow::from(tag)
        }
    }
}

impl<'t> EntityTag<'t> {
    /// Get the tag. The double quotes are not included.
    #[inline]
    pub fn get_tag(&self) -> &str {
        self.tag.as_ref()
    }

    /// Into the tag. The double quotes are not included.
    #[inline]
    pub fn into_tag(self) -> Cow<'t, str> {
        self.tag
    }

    /// Extracts the owned data.
    #[inline]
    pub fn into_owned(self) -> EntityTag<'static> {
        let tag = self.tag.into_owned();

        EntityTag {
            weak: self.weak, tag: Cow::from(tag)
        }
    }
}

impl<'t> EntityTag<'t> {
    /// For strong comparison two entity-tags are equivalent if both are not weak and their opaque-tags match character-by-character.
    #[inline]
    pub fn strong_eq(&self, other: &EntityTag) -> bool {
        !self.weak && !other.weak && self.tag == other.tag
    }

    /// For weak comparison two entity-tags are equivalent if their opaque-tags match character-by-character, regardless of either or both being tagged as "weak".
    #[inline]
    pub fn weak_eq(&self, other: &EntityTag) -> bool {
        self.tag == other.tag
    }

    /// The inverse of `strong_eq`.
    #[inline]
    pub fn strong_ne(&self, other: &EntityTag) -> bool {
        !self.strong_eq(other)
    }

    /// The inverse of `weak_eq`.
    #[inline]
    pub fn weak_ne(&self, other: &EntityTag) -> bool {
        !self.weak_eq(other)
    }
}

impl FromStr for EntityTag<'static> {
    type Err = EntityTagError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        EntityTag::from_string(s)
    }
}

impl<'t> Display for EntityTag<'t> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        if self.weak {
            f.write_str("W/")?;
        }

        f.write_char('"')?;
        f.write_str(self.tag.as_ref())?;
        f.write_char('"')
    }
}

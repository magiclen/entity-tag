/*!
# Entity Tag

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

let etag3 = EntityTag::from_data(&[102, 111, 111]);
assert_eq!("\"j4VF2Hjg0No\"", etag3.to_string());

# #[cfg(feature = "std")]
# {
let etag4 = EntityTag::from_file_meta(&std::fs::File::open("tests/data/P1060382.JPG").unwrap().metadata().unwrap());
println!("{}", etag4) // W/"CmgjkoKAfwQ"
# }
```

## No Std

Disable the default features to compile this crate without std.

```toml
[dependencies.entity-tag]
version = "*"
default-features = false
```
*/

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

extern crate base64;
extern crate wyhash;

mod entity_tag_error;

use core::fmt::{self, Display, Formatter, Write};
use core::hash::Hasher;

use alloc::borrow::Cow;
use alloc::string::String;

#[cfg(feature = "std")]
use std::fs::Metadata;

#[cfg(feature = "std")]
use std::time::UNIX_EPOCH;

pub use entity_tag_error::EntityTagError;

use wyhash::WyHash;

/// An entity tag, defined in [RFC7232](https://tools.ietf.org/html/rfc7232#section-2.3).
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EntityTag<'t> {
    /// Whether to have a weakness indicator.
    pub weak: bool,
    /// *etagc
    tag: Cow<'t, str>,
}

impl<'t> EntityTag<'t> {
    /// `ETag`
    pub const HEADER_NAME: &'static str = "ETag";
}

impl<'t> EntityTag<'t> {
    /// Construct a new EntityTag without checking.
    #[allow(clippy::missing_safety_doc)]
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
    #[allow(clippy::missing_safety_doc)]
    #[inline]
    pub unsafe fn with_string_unchecked<S: Into<String>>(weak: bool, tag: S) -> Self {
        EntityTag {
            weak,
            tag: Cow::from(tag.into()),
        }
    }

    /// Construct a new EntityTag without checking.
    #[allow(clippy::missing_safety_doc)]
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
        let (s, quoted) = if let Some(stripped) = s.strip_prefix('"') {
            (stripped, true)
        } else {
            (s, false)
        };

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

    /// Construct a new EntityTag.
    #[inline]
    pub fn with_string<S: AsRef<str> + Into<String>>(
        weak: bool,
        tag: S,
    ) -> Result<Self, EntityTagError> {
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

    /// Construct a new EntityTag.
    #[inline]
    pub fn with_str<S: ?Sized + AsRef<str>>(
        weak: bool,
        tag: &'t S,
    ) -> Result<Self, EntityTagError> {
        let tag = tag.as_ref();

        let quoted = Self::check_tag(tag)?;

        let tag = if quoted {
            &tag[1..(tag.len() - 1)]
        } else {
            tag
        };

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

    /// Parse and construct a new EntityTag from a `String`.
    pub fn from_string<S: AsRef<str> + Into<String>>(etag: S) -> Result<Self, EntityTagError> {
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
            unsafe {
                tag.as_mut_vec().drain(0..3);
            }
        } else {
            tag.remove(0);
        }

        Ok(EntityTag {
            weak,
            tag: Cow::from(tag),
        })
    }

    /// Parse and construct a new EntityTag from a `str`.
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

    /// Construct a strong EntityTag.
    #[inline]
    pub fn from_data<S: ?Sized + AsRef<[u8]>>(data: &S) -> Self {
        let mut hasher = WyHash::with_seed(3);
        hasher.write(data.as_ref());

        let tag = base64::encode_config(hasher.finish().to_le_bytes(), base64::STANDARD_NO_PAD);

        EntityTag {
            weak: false,
            tag: Cow::from(tag),
        }
    }

    #[cfg(feature = "std")]
    /// Construct a weak EntityTag.
    pub fn from_file_meta(metadata: &Metadata) -> Self {
        let mut hasher = WyHash::with_seed(4);

        hasher.write(&metadata.len().to_le_bytes());

        if let Ok(modified_time) = metadata.modified() {
            if let Ok(time) = modified_time.duration_since(UNIX_EPOCH) {
                hasher.write(&time.as_nanos().to_le_bytes());
            } else {
                hasher.write(b"-");

                let time = UNIX_EPOCH.duration_since(modified_time).unwrap();
                hasher.write(&time.as_nanos().to_le_bytes());
            }
        }

        let tag = base64::encode_config(hasher.finish().to_le_bytes(), base64::STANDARD_NO_PAD);

        EntityTag {
            weak: true,
            tag: Cow::from(tag),
        }
    }
}

impl<'t> EntityTag<'t> {
    /// Get the tag. The double quotes are not included.
    #[inline]
    pub fn get_tag(&'t self) -> &'t str {
        self.tag.as_ref()
    }

    /// Into the tag. The double quotes are not included.
    #[inline]
    pub fn into_tag(self) -> Cow<'t, str> {
        self.tag
    }
}

impl<'t> EntityTag<'t> {
    /// For strong comparison two entity-tags are equivalent if both are not weak and their opaque-tags match character-by-character.
    #[inline]
    pub fn strong_eq<'v>(&self, other: &EntityTag<'v>) -> bool {
        !self.weak && !other.weak && self.tag == other.tag
    }

    /// For weak comparison two entity-tags are equivalent if their opaque-tags match character-by-character, regardless of either or both being tagged as "weak".
    #[inline]
    pub fn weak_eq<'v>(&self, other: &EntityTag<'v>) -> bool {
        self.tag == other.tag
    }

    /// The inverse of `strong_eq`.
    #[inline]
    pub fn strong_ne<'v>(&self, other: &EntityTag<'v>) -> bool {
        !self.strong_eq(other)
    }

    /// The inverse of `weak_eq`.
    #[inline]
    pub fn weak_ne<'v>(&self, other: &EntityTag<'v>) -> bool {
        !self.weak_eq(other)
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

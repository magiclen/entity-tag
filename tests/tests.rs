extern crate entity_tag;

use entity_tag::*;

#[test]
fn cmp() {
    const FIRST: &'static str = "FIRST";
    const SECOND: &'static str = "SECOND";

    let etag1 = EntityTag::with_str(true, FIRST).unwrap();
    let etag2 = EntityTag::with_str(true, FIRST).unwrap();
    assert!(!etag1.strong_eq(&etag2));
    assert!(etag1.weak_eq(&etag2));
    assert!(etag1.strong_ne(&etag2));
    assert!(!etag1.weak_ne(&etag2));

    let etag1 = EntityTag::with_str(true, FIRST).unwrap();
    let etag2 = EntityTag::with_str(true, SECOND).unwrap();
    assert!(!etag1.strong_eq(&etag2));
    assert!(!etag1.weak_eq(&etag2));
    assert!(etag1.strong_ne(&etag2));
    assert!(etag1.weak_ne(&etag2));

    let etag1 = EntityTag::with_str(true, FIRST).unwrap();
    let etag2 = EntityTag::with_str(false, FIRST).unwrap();
    assert!(!etag1.strong_eq(&etag2));
    assert!(etag1.weak_eq(&etag2));
    assert!(etag1.strong_ne(&etag2));
    assert!(!etag1.weak_ne(&etag2));

    let etag1 = EntityTag::with_str(false, FIRST).unwrap();
    let etag2 = EntityTag::with_str(false, FIRST).unwrap();
    assert!(etag1.strong_eq(&etag2));
    assert!(etag1.weak_eq(&etag2));
    assert!(!etag1.strong_ne(&etag2));
    assert!(!etag1.weak_ne(&etag2));
}

#[test]
fn etag_fmt() {
    assert_eq!("\"foobar\"", EntityTag::with_str(false, "foobar").unwrap().to_string());
    assert_eq!("\"\"", EntityTag::with_str(false, "").unwrap().to_string());
    assert_eq!("W/\"weak-etag\"", EntityTag::with_str(true, "weak-etag").unwrap().to_string());
    assert_eq!("W/\"\"", EntityTag::with_str(true, "").unwrap().to_string());
}

#[test]
fn etag_parse_success() {
    assert_eq!(
        EntityTag::from_str("\"foobar\"").unwrap(),
        EntityTag::from_string("\"foobar\"").unwrap()
    );
    assert_eq!(
        EntityTag::from_str("\"foobar\"").unwrap(),
        EntityTag::with_string(false, "foobar").unwrap()
    );
    assert_eq!(
        EntityTag::with_str(false, "foobar").unwrap(),
        EntityTag::with_string(false, "foobar").unwrap()
    );
    assert_eq!(
        EntityTag::with_str(false, "\"foobar\"").unwrap(),
        EntityTag::with_string(false, "foobar").unwrap()
    );
    assert_eq!(
        EntityTag::with_str(false, "foobar").unwrap(),
        EntityTag::with_string(false, "\"foobar\"").unwrap()
    );

    assert_eq!(
        EntityTag::from_str("W/\"foobar\"").unwrap(),
        EntityTag::from_string("W/\"foobar\"").unwrap()
    );
    assert_eq!(
        EntityTag::from_str("W/\"foobar\"").unwrap(),
        EntityTag::with_string(true, "foobar").unwrap()
    );
    assert_eq!(
        EntityTag::with_str(true, "foobar").unwrap(),
        EntityTag::with_str(true, "foobar").unwrap()
    );
    assert_eq!(
        EntityTag::with_str(true, "foobar").unwrap(),
        EntityTag::with_str(true, "\"foobar\"").unwrap()
    );
    assert_eq!(
        EntityTag::with_str(true, "foobar").unwrap(),
        EntityTag::with_str(true, "\"foobar\"").unwrap()
    );

    assert_eq!(EntityTag::from_str("\"\"").unwrap(), EntityTag::from_string("\"\"").unwrap());
    assert_eq!(EntityTag::from_str("\"\"").unwrap(), EntityTag::with_string(false, "").unwrap());
    assert_eq!(EntityTag::with_str(false, "").unwrap(), EntityTag::with_string(false, "").unwrap());
    assert_eq!(
        EntityTag::with_str(false, "\"\"").unwrap(),
        EntityTag::with_string(false, "").unwrap()
    );
    assert_eq!(
        EntityTag::with_str(false, "").unwrap(),
        EntityTag::with_string(false, "\"\"").unwrap()
    );

    assert_eq!(EntityTag::from_str("W/\"\"").unwrap(), EntityTag::from_string("W/\"\"").unwrap());
    assert_eq!(EntityTag::from_str("W/\"\"").unwrap(), EntityTag::with_string(true, "").unwrap());
    assert_eq!(EntityTag::with_str(true, "").unwrap(), EntityTag::with_string(true, "").unwrap());
    assert_eq!(
        EntityTag::with_str(true, "\"\"").unwrap(),
        EntityTag::with_string(true, "").unwrap()
    );
    assert_eq!(
        EntityTag::with_str(true, "").unwrap(),
        EntityTag::with_string(true, "\"\"").unwrap()
    );
}

#[test]
fn etag_parse_failures() {
    assert_eq!(Err(EntityTagError::InvalidTag), EntityTag::from_str("W/\"\t\""));
    assert_eq!(Err(EntityTagError::MissingStartingDoubleQuote), EntityTag::from_str("no-dquotes"));
    assert_eq!(Err(EntityTagError::MissingClosingDoubleQuote), EntityTag::from_str("\"no-dquote"));
}

#[test]
fn from_data() {
    assert_eq!("\"oC5gwMEUN28\"", EntityTag::from_data(&[1, 2, 3, 4]).to_string());
}

#[cfg(feature = "std")]
#[test]
fn from_file_meta() {
    let file = std::fs::File::open("tests/data/P1060382.JPG").unwrap();

    let metadata = file.metadata().unwrap();

    let etag = EntityTag::from_file_meta(&metadata);

    assert_eq!(true, etag.weak);
}

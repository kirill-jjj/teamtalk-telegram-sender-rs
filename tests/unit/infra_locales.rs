use super::*;
use crate::args;

#[test]
fn get_text_supports_known_langs() {
    let text_en = get_text("en", LocaleKey::CmdDescHelp, None);
    let text_ru = get_text("ru", LocaleKey::CmdDescHelp, None);
    assert!(!text_en.is_empty());
    assert!(!text_ru.is_empty());
}

#[test]
fn args_macro_builds_map() {
    let map = args!(name = "Bob", count = 3).expect("args map");
    let name = map.get("name").expect("name");
    let count = map.get("count").expect("count");
    assert_eq!(name, &FluentValue::from("Bob"));
    assert_eq!(count, &FluentValue::from(3));
}

#[test]
fn args_macro_handles_unicode() {
    let map = args!(name = "Привет").expect("args map");
    let name = map.get("name").expect("name");
    assert_eq!(name, &FluentValue::from("Привет"));
}

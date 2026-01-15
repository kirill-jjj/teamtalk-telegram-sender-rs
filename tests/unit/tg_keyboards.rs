use super::*;
use crate::core::callbacks::MenuAction;

#[test]
fn create_main_menu_keyboard_has_expected_rows_for_user() {
    let kb = create_main_menu_keyboard(LanguageCode::En, false);
    assert_eq!(kb.inline_keyboard.len(), 4);
}

#[test]
fn create_main_menu_keyboard_has_expected_rows_for_admin() {
    let kb = create_main_menu_keyboard(LanguageCode::En, true);
    assert_eq!(kb.inline_keyboard.len(), 8);
}

#[test]
fn create_pagination_keyboard_includes_back_when_requested() {
    let kb = create_pagination_keyboard(
        0,
        3,
        |_p| CallbackAction::Menu(MenuAction::Help),
        Some((
            "back".to_string(),
            CallbackAction::Menu(MenuAction::Settings),
        )),
        LanguageCode::En,
    );
    assert_eq!(kb.inline_keyboard.len(), 2);
}

#[test]
fn create_user_list_keyboard_paginates_items() {
    let items: Vec<String> = (0..15).map(|i| format!("u{i}")).collect();
    let kb = create_user_list_keyboard(
        &items,
        0,
        |name| (name.clone(), CallbackAction::Menu(MenuAction::Help)),
        |_p| CallbackAction::Menu(MenuAction::Settings),
        Some(("back".to_string(), CallbackAction::Menu(MenuAction::Help))),
        LanguageCode::En,
    );
    assert_eq!(kb.inline_keyboard.len(), 12);
}

#[test]
fn create_user_list_keyboard_empty_list_only_shows_back() {
    let items: Vec<String> = Vec::new();
    let kb = create_user_list_keyboard(
        &items,
        0,
        |name| (name.clone(), CallbackAction::Menu(MenuAction::Help)),
        |_p| CallbackAction::Menu(MenuAction::Settings),
        Some(("back".to_string(), CallbackAction::Menu(MenuAction::Help))),
        LanguageCode::En,
    );
    assert_eq!(kb.inline_keyboard.len(), 1);
}

#[test]
fn confirm_cancel_keyboard_has_single_row_two_buttons() {
    let kb = confirm_cancel_keyboard(
        LanguageCode::En,
        "btn-yes",
        CallbackAction::Menu(MenuAction::Help),
        "btn-no",
        CallbackAction::Menu(MenuAction::Settings),
    );
    assert_eq!(kb.inline_keyboard.len(), 1);
    assert_eq!(kb.inline_keyboard[0].len(), 2);
}

#[test]
fn back_button_keyboard_has_single_row() {
    let kb = back_button_keyboard(
        LanguageCode::En,
        "btn-back",
        CallbackAction::Menu(MenuAction::Settings),
    );
    assert_eq!(kb.inline_keyboard.len(), 1);
}

#[test]
fn create_pagination_keyboard_without_back_or_nav_is_empty() {
    let kb = create_pagination_keyboard(
        0,
        1,
        |_p| CallbackAction::Menu(MenuAction::Help),
        None,
        LanguageCode::En,
    );
    assert!(kb.inline_keyboard.is_empty());
}

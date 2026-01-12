use crate::core::callbacks::CallbackAction;
use crate::core::types::LanguageCode;
use crate::infra::locales;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn callback_button<T, A>(text: T, action: A) -> InlineKeyboardButton
where
    T: Into<String>,
    A: ToString,
{
    InlineKeyboardButton::callback(text.into(), action.to_string())
}

pub fn back_button(
    lang: LanguageCode,
    back_key: &str,
    back_action: CallbackAction,
) -> InlineKeyboardButton {
    callback_button(
        locales::get_text(lang.as_str(), back_key, None),
        back_action,
    )
}

pub fn confirm_cancel_keyboard(
    lang: LanguageCode,
    yes_key: &str,
    yes_action: CallbackAction,
    no_key: &str,
    no_action: CallbackAction,
) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        callback_button(locales::get_text(lang.as_str(), yes_key, None), yes_action),
        callback_button(locales::get_text(lang.as_str(), no_key, None), no_action),
    ]])
}

pub fn back_button_keyboard(
    lang: LanguageCode,
    back_key: &str,
    back_action: CallbackAction,
) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![callback_button(
        locales::get_text(lang.as_str(), back_key, None),
        back_action,
    )]])
}

pub fn back_btn(
    lang: LanguageCode,
    back_key: &str,
    back_action: CallbackAction,
) -> (String, CallbackAction) {
    (
        locales::get_text(lang.as_str(), back_key, None),
        back_action,
    )
}

pub const USERS_PER_PAGE: usize = 10;

pub fn create_pagination_keyboard<F>(
    current_page: usize,
    total_pages: usize,
    page_builder: F,
    back_btn: Option<(String, CallbackAction)>,
    lang: LanguageCode,
) -> InlineKeyboardMarkup
where
    F: Fn(usize) -> CallbackAction,
{
    let mut buttons = vec![];
    let mut nav_row = vec![];

    if current_page > 0 {
        let data = page_builder(current_page - 1).to_string();
        nav_row.push(callback_button(
            locales::get_text(lang.as_str(), "btn-prev", None),
            data,
        ));
    }

    if total_pages > 0 && current_page < total_pages - 1 {
        let data = page_builder(current_page + 1).to_string();
        nav_row.push(callback_button(
            locales::get_text(lang.as_str(), "btn-next", None),
            data,
        ));
    }

    if !nav_row.is_empty() {
        buttons.push(nav_row);
    }

    if let Some((text, action)) = back_btn {
        buttons.push(vec![callback_button(text, action)]);
    }

    InlineKeyboardMarkup::new(buttons)
}

pub fn create_user_list_keyboard<T, FMap, FPage>(
    items: &[T],
    page: usize,
    item_mapper: FMap,
    page_builder: FPage,
    back_btn: Option<(String, CallbackAction)>,
    lang: LanguageCode,
) -> InlineKeyboardMarkup
where
    FMap: Fn(&T) -> (String, CallbackAction),
    FPage: Fn(usize) -> CallbackAction,
{
    let total_items = items.len();
    let total_pages = total_items.div_ceil(USERS_PER_PAGE);
    let page = if total_pages == 0 {
        0
    } else {
        page.min(total_pages - 1)
    };

    let start = page * USERS_PER_PAGE;
    let end = (start + USERS_PER_PAGE).min(total_items);
    let slice = if start < total_items {
        &items[start..end]
    } else {
        &[]
    };

    let mut buttons = vec![];
    for item in slice {
        let (name, action) = item_mapper(item);
        buttons.push(vec![callback_button(name, action)]);
    }

    let nav_kb = create_pagination_keyboard(page, total_pages, page_builder, back_btn, lang);

    let mut final_buttons = buttons;
    for row in nav_kb.inline_keyboard {
        final_buttons.push(row);
    }
    InlineKeyboardMarkup::new(final_buttons)
}

pub fn create_main_menu_keyboard(lang: LanguageCode, is_admin: bool) -> InlineKeyboardMarkup {
    use crate::core::callbacks::{AdminAction, MenuAction};

    let mut buttons = vec![
        vec![callback_button(
            locales::get_text(lang.as_str(), "btn-menu-who", None),
            CallbackAction::Menu(MenuAction::Who),
        )],
        vec![callback_button(
            locales::get_text(lang.as_str(), "btn-menu-settings", None),
            CallbackAction::Settings(crate::core::callbacks::SettingsAction::Main),
        )],
        vec![callback_button(
            locales::get_text(lang.as_str(), "btn-menu-unsub", None),
            CallbackAction::Menu(MenuAction::Unsub),
        )],
        vec![callback_button(
            locales::get_text(lang.as_str(), "btn-menu-help", None),
            CallbackAction::Menu(MenuAction::Help),
        )],
    ];

    if is_admin {
        buttons.push(vec![callback_button(
            locales::get_text(lang.as_str(), "btn-menu-kick", None),
            CallbackAction::Admin(AdminAction::KickList { page: 0 }),
        )]);
        buttons.push(vec![callback_button(
            locales::get_text(lang.as_str(), "btn-menu-ban", None),
            CallbackAction::Admin(AdminAction::BanList { page: 0 }),
        )]);
        buttons.push(vec![callback_button(
            locales::get_text(lang.as_str(), "btn-menu-unban", None),
            CallbackAction::Admin(AdminAction::UnbanList { page: 0 }),
        )]);
        buttons.push(vec![callback_button(
            locales::get_text(lang.as_str(), "btn-menu-subs", None),
            CallbackAction::Admin(AdminAction::SubsList { page: 0 }),
        )]);
    }

    InlineKeyboardMarkup::new(buttons)
}

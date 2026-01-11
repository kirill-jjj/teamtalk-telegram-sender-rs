use crate::locales;
use crate::tg_bot::callbacks_types::CallbackAction;
use crate::types::LanguageCode;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

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
        nav_row.push(InlineKeyboardButton::callback(
            locales::get_text(lang.as_str(), "btn-prev", None),
            data,
        ));
    }

    if total_pages > 0 && current_page < total_pages - 1 {
        let data = page_builder(current_page + 1).to_string();
        nav_row.push(InlineKeyboardButton::callback(
            locales::get_text(lang.as_str(), "btn-next", None),
            data,
        ));
    }

    if !nav_row.is_empty() {
        buttons.push(nav_row);
    }

    if let Some((text, action)) = back_btn {
        buttons.push(vec![InlineKeyboardButton::callback(
            text,
            action.to_string(),
        )]);
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
        buttons.push(vec![InlineKeyboardButton::callback(
            name,
            action.to_string(),
        )]);
    }

    let nav_kb = create_pagination_keyboard(page, total_pages, page_builder, back_btn, lang);

    let mut final_buttons = buttons;
    for row in nav_kb.inline_keyboard {
        final_buttons.push(row);
    }
    InlineKeyboardMarkup::new(final_buttons)
}

pub fn create_main_menu_keyboard(lang: LanguageCode, is_admin: bool) -> InlineKeyboardMarkup {
    use crate::tg_bot::callbacks_types::{AdminAction, MenuAction};

    let mut buttons = vec![
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang.as_str(), "btn-menu-who", None),
            CallbackAction::Menu(MenuAction::Who).to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang.as_str(), "btn-menu-settings", None),
            CallbackAction::Settings(crate::tg_bot::callbacks_types::SettingsAction::Main)
                .to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang.as_str(), "btn-menu-unsub", None),
            CallbackAction::Menu(MenuAction::Unsub).to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang.as_str(), "btn-menu-help", None),
            CallbackAction::Menu(MenuAction::Help).to_string(),
        )],
    ];

    if is_admin {
        buttons.push(vec![InlineKeyboardButton::callback(
            locales::get_text(lang.as_str(), "btn-menu-kick", None),
            CallbackAction::Admin(AdminAction::KickList { page: 0 }).to_string(),
        )]);
        buttons.push(vec![InlineKeyboardButton::callback(
            locales::get_text(lang.as_str(), "btn-menu-ban", None),
            CallbackAction::Admin(AdminAction::BanList { page: 0 }).to_string(),
        )]);
        buttons.push(vec![InlineKeyboardButton::callback(
            locales::get_text(lang.as_str(), "btn-menu-unban", None),
            CallbackAction::Admin(AdminAction::UnbanList { page: 0 }).to_string(),
        )]);
        buttons.push(vec![InlineKeyboardButton::callback(
            locales::get_text(lang.as_str(), "btn-menu-subs", None),
            CallbackAction::Admin(AdminAction::SubsList { page: 0 }).to_string(),
        )]);
    }

    InlineKeyboardMarkup::new(buttons)
}

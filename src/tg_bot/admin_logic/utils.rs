pub fn format_tg_user(chat: &teloxide::types::ChatFullInfo) -> String {
    let full_name = match (chat.first_name(), chat.last_name()) {
        (Some(f), Some(l)) => format!("{} {}", f, l),
        (Some(f), None) => f.to_string(),
        (None, Some(l)) => l.to_string(),
        (None, None) => String::new(),
    };

    let username = chat
        .username()
        .map(|u| format!("@{}", u))
        .unwrap_or_default();

    if !full_name.is_empty() {
        if !username.is_empty() {
            format!("{} ({})", full_name, username)
        } else {
            full_name
        }
    } else if !username.is_empty() {
        username
    } else {
        chat.id.0.to_string()
    }
}

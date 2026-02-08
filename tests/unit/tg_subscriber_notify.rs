use super::*;
use crate::core::types::{LanguageCode, TelegramId};

#[test]
fn render_message_with_actor_username() {
    let actor = AdminActor {
        telegram_id: TelegramId::from(1),
        full_name: "Alice Doe".to_string(),
        username: Some("alice".to_string()),
    };
    let text = render_subscriber_change_message(
        LanguageCode::En,
        &SubscriberChangeKind::AdminAdded,
        &actor,
    );
    assert!(text.contains("You were granted bot admin rights."));
    assert!(text.contains("Changed by: Alice Doe (@alice)."));
}

#[test]
fn render_message_without_actor_username_uses_id_fallback() {
    let actor = AdminActor {
        telegram_id: TelegramId::from(55),
        full_name: String::new(),
        username: None,
    };
    let text =
        render_subscriber_change_message(LanguageCode::En, &SubscriberChangeKind::Deleted, &actor);
    assert!(text.contains("Your subscription profile was deleted."));
    assert!(text.contains("Changed by: 55."));
    assert!(!text.contains("(@"));
}

#[test]
fn render_language_value_is_localized() {
    let actor = AdminActor {
        telegram_id: TelegramId::from(1),
        full_name: "Админ".to_string(),
        username: None,
    };
    let ru_text = render_subscriber_change_message(
        LanguageCode::Ru,
        &SubscriberChangeKind::Language(LanguageCode::En),
        &actor,
    );
    assert!(ru_text.contains("Английский"));

    let en_text = render_subscriber_change_message(
        LanguageCode::En,
        &SubscriberChangeKind::Language(LanguageCode::Ru),
        &actor,
    );
    assert!(en_text.contains("Russian"));
}

#[test]
fn render_notification_setting_uses_user_facing_label() {
    let actor = AdminActor {
        telegram_id: TelegramId::from(9),
        full_name: "Bob".to_string(),
        username: Some("@bob".to_string()),
    };
    let text = render_subscriber_change_message(
        LanguageCode::En,
        &SubscriberChangeKind::Notifications(crate::core::types::NotificationSetting::All),
        &actor,
    );
    assert!(text.contains("All (Join & Leave)"));
    assert!(text.contains("Changed by: Bob (@bob)."));
}

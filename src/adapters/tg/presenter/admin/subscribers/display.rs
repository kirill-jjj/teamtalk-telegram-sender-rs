use crate::adapters::tg::presenter::admin::utils::format_tg_user;
use crate::core::types::{TelegramId, TtUsername};
use crate::infra::db::types::SubscriberInfo;
use teloxide_ng::prelude::*;

#[derive(Clone)]
pub struct SubDisplayInfo {
    pub telegram_id: TelegramId,
    pub display_name: String,
    pub tt_username: Option<TtUsername>,
}

pub async fn prepare_display_list(bot: &Bot, subs: Vec<SubscriberInfo>) -> Vec<SubDisplayInfo> {
    let mut display_list = Vec::new();
    for sub in subs {
        let display_name = match bot
            .get_chat(teloxide_ng::types::ChatId(sub.telegram_id.as_i64()))
            .await
        {
            Ok(chat) => format_tg_user(&chat),
            Err(e) => {
                tracing::error!(
                    telegram_id = sub.telegram_id.as_i64(),
                    error = %e,
                    "Failed to load Telegram user"
                );
                sub.telegram_id.to_string()
            }
        };
        display_list.push(SubDisplayInfo {
            telegram_id: sub.telegram_id,
            display_name,
            tt_username: sub.teamtalk_username,
        });
    }
    display_list.sort_by(|a, b| {
        a.display_name
            .to_lowercase()
            .cmp(&b.display_name.to_lowercase())
    });
    display_list
}

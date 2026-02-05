use crate::app::services::tt_context::TtServiceContext;
use crate::core::types::{LanguageCode, TtUsername};

pub async fn resolve_reply_lang(
    ctx: &TtServiceContext,
    username: &TtUsername,
    default_lang: LanguageCode,
) -> LanguageCode {
    if username.as_str().is_empty() {
        return default_lang;
    }
    match ctx.state.get_lang_cached(username).await {
        Ok(Some(lang)) => return lang,
        Ok(None) => {}
        Err(err) => {
            tracing::error!(
                tt_username = %username,
                error = %err,
                "Failed to read language from cache"
            );
        }
    }
    let lang = ctx
        .db
        .get_user_lang_by_tt_user(username)
        .await
        .unwrap_or(default_lang);
    ctx.state.set_lang_cached(username.clone(), lang);
    lang
}

pub async fn resolve_is_admin(
    ctx: &TtServiceContext,
    username: &TtUsername,
    admin_username: Option<&TtUsername>,
) -> bool {
    if username.as_str().is_empty() {
        return false;
    }
    if admin_username.is_some_and(|u| u == username) {
        return true;
    }
    match ctx.state.get_tg_cached(username).await {
        Ok(Some(tg_id)) => {
            return match ctx.db.get_all_admins().await {
                Ok(admins) => admins.contains(&tg_id),
                Err(err) => {
                    tracing::error!(tt_username = %username, error = %err, "Failed to load admins list from cache path");
                    false
                }
            };
        }
        Ok(None) => {}
        Err(err) => {
            tracing::error!(
                tt_username = %username,
                error = %err,
                "Failed to read Telegram id from cache"
            );
        }
    }
    if let Some(tg_id) = ctx.db.get_telegram_id_by_tt_user(username).await {
        ctx.state.set_tg_cached(username.clone(), tg_id);
        return match ctx.db.get_all_admins().await {
            Ok(admins) => admins.contains(&tg_id),
            Err(err) => {
                tracing::error!(tt_username = %username, error = %err, "Failed to load admins list after DB lookup");
                false
            }
        };
    }
    false
}

pub async fn get_user_lang_by_tt_user(
    ctx: &TtServiceContext,
    username: &TtUsername,
    default_lang: LanguageCode,
) -> LanguageCode {
    if username.as_str().is_empty() {
        return default_lang;
    }
    ctx.db
        .get_user_lang_by_tt_user(username)
        .await
        .unwrap_or(default_lang)
}

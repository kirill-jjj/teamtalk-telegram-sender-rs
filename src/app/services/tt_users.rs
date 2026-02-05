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
    if let Some(lang) = ctx.state.get_lang_cached(username).await.ok().flatten() {
        return lang;
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
    if let Some(tg_id) = ctx.state.get_tg_cached(username).await.ok().flatten() {
        return ctx
            .db
            .get_all_admins()
            .await
            .map(|admins| admins.contains(&tg_id))
            .unwrap_or(false);
    }
    if let Some(tg_id) = ctx.db.get_telegram_id_by_tt_user(username).await {
        ctx.state.set_tg_cached(username.clone(), tg_id);
        return ctx
            .db
            .get_all_admins()
            .await
            .map(|admins| admins.contains(&tg_id))
            .unwrap_or(false);
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

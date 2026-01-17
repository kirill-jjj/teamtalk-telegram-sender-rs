use crate::adapters::tt::{WorkerContext, resolve_channel_name, resolve_server_name};
use crate::args;
use crate::core::types::{BridgeEvent, DeeplinkAction, LanguageCode, TtCommand};
use crate::infra::locales;
use teamtalk::Client;
use teamtalk::types::TextMessage;
use uuid::Uuid;

#[derive(Clone)]
struct TtTextCtx {
    db: crate::infra::db::Database,
    online_users: std::sync::Arc<
        std::sync::RwLock<std::collections::HashMap<i32, crate::core::types::LiteUser>>,
    >,
    tx_tt_cmd: tokio::sync::mpsc::Sender<TtCommand>,
    tx_bridge: tokio::sync::mpsc::Sender<BridgeEvent>,
    admin_username: Option<String>,
    bot_username: Option<String>,
    tt_config: crate::bootstrap::config::TeamTalkConfig,
    default_lang: LanguageCode,
    deeplink_ttl: i64,
    real_name_from_client: Option<String>,
    rt: tokio::runtime::Handle,
}

pub(super) fn handle_text_message(client: &Client, ctx: &WorkerContext, msg: TextMessage) {
    if msg.from_id == client.my_id() {
        return;
    }

    let text_ctx = TtTextCtx {
        db: ctx.db.clone(),
        online_users: ctx.online_users.clone(),
        tx_tt_cmd: ctx.tx_tt_cmd.clone(),
        tx_bridge: ctx.tx_bridge.clone(),
        admin_username: ctx.config.general.admin_username.clone(),
        bot_username: ctx.bot_username.clone(),
        tt_config: ctx.config.teamtalk.clone(),
        default_lang: LanguageCode::from_str_or_default(
            &ctx.config.general.default_lang,
            LanguageCode::En,
        ),
        deeplink_ttl: ctx.config.operational_parameters.deeplink_ttl,
        real_name_from_client: client.get_server_properties().map(|p| p.name),
        rt: ctx.rt.clone(),
    };

    match msg.msg_type {
        teamtalk::client::ffi::TextMsgType::MSGTYPE_CHANNEL => {
            handle_channel_message(client, &text_ctx, &msg);
        }
        teamtalk::client::ffi::TextMsgType::MSGTYPE_USER => {
            spawn_user_message(text_ctx, msg);
        }
        _ => {}
    }
}

fn handle_channel_message(client: &Client, ctx: &TtTextCtx, msg: &TextMessage) {
    let content = msg.text.trim();
    let cmd = content
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_lowercase();

    if cmd == "/skip" {
        handle_channel_skip(ctx, msg);
        return;
    }

    if let Some(rest) = content.strip_prefix("/pm ") {
        let pm_text = rest.trim();
        if pm_text.is_empty() {
            return;
        }
        let channel_name = resolve_channel_name(client, msg.channel_id, LanguageCode::En);
        let server_name = resolve_server_name(&ctx.tt_config, ctx.real_name_from_client.as_deref());
        if let Err(e) = ctx.tx_bridge.try_send(BridgeEvent::ToAdminChannel {
            channel_id: msg.channel_id.0,
            channel_name,
            server_name,
            msg_content: pm_text.to_string(),
        }) {
            tracing::error!(error = %e, "Failed to send channel PM event");
        }
    }
}

fn handle_channel_skip(ctx: &TtTextCtx, msg: &TextMessage) {
    let from_uid = msg.from_id.0;
    let channel_id = msg.channel_id.0;
    let db = ctx.db.clone();
    let online_users = ctx.online_users.clone();
    let tx_tt_cmd = ctx.tx_tt_cmd.clone();
    let admin_username = ctx.admin_username.clone();
    let default_lang = ctx.default_lang;

    ctx.rt.spawn(async move {
        let username = online_users.read().map_or_else(
            |_| String::new(),
            |users| {
                users
                    .get(&from_uid)
                    .map(|u| u.username.clone())
                    .unwrap_or_default()
            },
        );
        let reply_lang = resolve_user_lang(&db, &username, default_lang).await;
        let is_admin = is_tt_admin(&db, admin_username.as_deref(), &username).await;

        let text_key = if is_admin {
            if let Err(e) = tx_tt_cmd.try_send(TtCommand::SkipStream) {
                tracing::error!(
                    tt_username = %username,
                    error = %e,
                    "Failed to send TT skip command"
                );
                "tt-error-generic"
            } else {
                "tt-skip-sent"
            }
        } else {
            "cmd-unauth"
        };
        let text = locales::get_text(reply_lang.as_str(), text_key, None);
        if let Err(e) = tx_tt_cmd.try_send(TtCommand::SendToChannel { channel_id, text }) {
            tracing::error!(channel_id, error = %e, "Failed to send TT channel reply");
        }
    });
}

fn spawn_user_message(ctx: TtTextCtx, msg: TextMessage) {
    let rt = ctx.rt.clone();
    rt.spawn(async move {
        handle_user_message(ctx, msg).await;
    });
}

async fn handle_user_message(ctx: TtTextCtx, msg: TextMessage) {
    let content = msg.text.trim();
    let from_uid = msg.from_id.0;

    let (nick, username) = ctx.online_users.read().map_or_else(
        |_| ("Unknown".to_string(), String::new()),
        |users| {
            users.get(&from_uid).map_or_else(
                || ("Unknown".to_string(), String::new()),
                |u| (u.nickname.clone(), u.username.clone()),
            )
        },
    );

    tracing::info!(
        component = "tt_worker",
        nick = %nick,
        tt_username = %username,
        "Received TT message"
    );

    let reply_lang = resolve_user_lang(&ctx.db, &username, ctx.default_lang).await;
    let parts: Vec<&str> = content.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }
    let cmd = parts[0].to_lowercase();

    match cmd.as_str() {
        "/sub" => handle_sub_command(&ctx, &username, reply_lang, from_uid).await,
        "/unsub" => handle_unsub_command(&ctx, &username, reply_lang, from_uid).await,
        "/help" => handle_help_command(&ctx, &username, reply_lang, from_uid),
        "/skip" => handle_user_skip(&ctx, &username, reply_lang, from_uid).await,
        "/add_admin" => handle_add_admin(&ctx, &username, reply_lang, from_uid, &parts).await,
        "/remove_admin" => handle_remove_admin(&ctx, &username, reply_lang, from_uid, &parts).await,
        _ => handle_forward_to_admin(&ctx, &username, nick, from_uid, content).await,
    }
}

async fn handle_sub_command(ctx: &TtTextCtx, username: &str, lang: LanguageCode, user_id: i32) {
    let Some(bot_user) = &ctx.bot_username else {
        send_user_reply(
            &ctx.tx_tt_cmd,
            user_id,
            username,
            "Telegram integration is currently disabled (Event Token missing).".to_string(),
        );
        return;
    };

    let is_guest = username.is_empty()
        || ctx
            .tt_config
            .guest_username
            .as_ref()
            .is_some_and(|g| g == username);

    let payload = if is_guest { None } else { Some(username) };
    let token = Uuid::now_v7().to_string().replace('-', "");
    let expected_telegram_id = if username.is_empty() {
        None
    } else {
        ctx.db.get_telegram_id_by_tt_user(username).await
    };

    let res = ctx
        .db
        .create_deeplink(
            &token,
            DeeplinkAction::Subscribe,
            payload,
            expected_telegram_id,
            ctx.deeplink_ttl,
        )
        .await;

    if res.is_ok() {
        let link = format!("https://t.me/{bot_user}?start={token}");
        let text = locales::get_text(lang.as_str(), "tt-sub-link", args!(link = link).as_ref());
        send_user_reply(&ctx.tx_tt_cmd, user_id, username, text);
    } else {
        let text = locales::get_text(lang.as_str(), "tt-error-generic", None);
        send_user_reply(&ctx.tx_tt_cmd, user_id, username, text);
    }
}

async fn handle_unsub_command(ctx: &TtTextCtx, username: &str, lang: LanguageCode, user_id: i32) {
    let Some(bot_user) = &ctx.bot_username else {
        send_user_reply(
            &ctx.tx_tt_cmd,
            user_id,
            username,
            "Telegram integration is currently disabled (Event Token missing).".to_string(),
        );
        return;
    };

    let token = Uuid::now_v7().to_string().replace('-', "");
    let expected_telegram_id = if username.is_empty() {
        None
    } else {
        ctx.db.get_telegram_id_by_tt_user(username).await
    };
    let res = ctx
        .db
        .create_deeplink(
            &token,
            DeeplinkAction::Unsubscribe,
            None,
            expected_telegram_id,
            ctx.deeplink_ttl,
        )
        .await;

    if res.is_ok() {
        let link = format!("https://t.me/{bot_user}?start={token}");
        let text = locales::get_text(lang.as_str(), "tt-unsub-link", args!(link = link).as_ref());
        send_user_reply(&ctx.tx_tt_cmd, user_id, username, text);
    } else {
        let text = locales::get_text(lang.as_str(), "tt-error-generic", None);
        send_user_reply(&ctx.tx_tt_cmd, user_id, username, text);
    }
}

fn handle_help_command(ctx: &TtTextCtx, username: &str, lang: LanguageCode, user_id: i32) {
    let is_main_admin = ctx.admin_username.as_ref().is_some_and(|u| u == username);
    let mut help_msg = locales::get_text(lang.as_str(), "help-text", None);
    if is_main_admin {
        let header = locales::get_text(lang.as_str(), "tt-admin-help-header", None);
        let cmds = locales::get_text(lang.as_str(), "tt-admin-help-cmds", None);
        help_msg.push_str(&header);
        help_msg.push_str(&cmds);
    }
    send_user_reply(&ctx.tx_tt_cmd, user_id, username, help_msg);
}

async fn handle_user_skip(ctx: &TtTextCtx, username: &str, lang: LanguageCode, user_id: i32) {
    let is_admin = is_tt_admin(&ctx.db, ctx.admin_username.as_deref(), username).await;
    if !is_admin {
        let text = locales::get_text(lang.as_str(), "cmd-unauth", None);
        send_user_reply(&ctx.tx_tt_cmd, user_id, username, text);
        return;
    }
    if let Err(e) = ctx.tx_tt_cmd.try_send(TtCommand::SkipStream) {
        tracing::error!(tt_username = %username, error = %e, "Failed to send TT skip command");
        let text = locales::get_text(lang.as_str(), "tt-error-generic", None);
        send_user_reply(&ctx.tx_tt_cmd, user_id, username, text);
        return;
    }
    let text = locales::get_text(lang.as_str(), "tt-skip-sent", None);
    send_user_reply(&ctx.tx_tt_cmd, user_id, username, text);
}

async fn handle_add_admin(
    ctx: &TtTextCtx,
    username: &str,
    lang: LanguageCode,
    user_id: i32,
    parts: &[&str],
) {
    if ctx.admin_username.as_deref() != Some(username) {
        let text = locales::get_text(lang.as_str(), "cmd-unauth", None);
        send_user_reply(&ctx.tx_tt_cmd, user_id, username, text);
        return;
    }
    if parts.len() < 2 {
        let text = locales::get_text(lang.as_str(), "tt-admin-no-ids", None);
        send_user_reply(&ctx.tx_tt_cmd, user_id, username, text);
        return;
    }

    let (added_count, failed_count) = update_admins(&ctx.db, parts, true).await;
    if added_count > 0 {
        let args = args!(count = added_count);
        let text = locales::get_text(lang.as_str(), "tt-admin-added", args.as_ref());
        send_user_reply(&ctx.tx_tt_cmd, user_id, username, text);
    }
    if failed_count > 0 {
        let args = args!(count = failed_count);
        let text = locales::get_text(lang.as_str(), "tt-admin-add-fail", args.as_ref());
        send_user_reply(&ctx.tx_tt_cmd, user_id, username, text);
    }
}

async fn handle_remove_admin(
    ctx: &TtTextCtx,
    username: &str,
    lang: LanguageCode,
    user_id: i32,
    parts: &[&str],
) {
    if ctx.admin_username.as_deref() != Some(username) {
        let text = locales::get_text(lang.as_str(), "cmd-unauth", None);
        send_user_reply(&ctx.tx_tt_cmd, user_id, username, text);
        return;
    }
    if parts.len() < 2 {
        let text = locales::get_text(lang.as_str(), "tt-admin-no-ids", None);
        send_user_reply(&ctx.tx_tt_cmd, user_id, username, text);
        return;
    }

    let (removed_count, failed_count) = update_admins(&ctx.db, parts, false).await;
    if removed_count > 0 {
        let args = args!(count = removed_count);
        let text = locales::get_text(lang.as_str(), "tt-admin-removed", args.as_ref());
        send_user_reply(&ctx.tx_tt_cmd, user_id, username, text);
    }
    if failed_count > 0 {
        let args = args!(count = failed_count);
        let text = locales::get_text(lang.as_str(), "tt-admin-remove-fail", args.as_ref());
        send_user_reply(&ctx.tx_tt_cmd, user_id, username, text);
    }
}

async fn update_admins(db: &crate::infra::db::Database, parts: &[&str], add: bool) -> (i64, i64) {
    let mut ok = 0;
    let mut failed = 0;
    for id_str in &parts[1..] {
        if let Ok(tg_id) = id_str.parse::<i64>() {
            let res = if add {
                db.add_admin(tg_id).await
            } else {
                db.remove_admin(tg_id).await
            };
            match res {
                Ok(val) if val => ok += 1,
                Ok(_) => failed += 1,
                Err(e) => {
                    tracing::error!(telegram_id = tg_id, error = %e, "DB error updating admin");
                    failed += 1;
                }
            }
        } else {
            failed += 1;
        }
    }
    (ok, failed)
}

async fn handle_forward_to_admin(
    ctx: &TtTextCtx,
    username: &str,
    nick: String,
    user_id: i32,
    content: &str,
) {
    let server_name = resolve_server_name(&ctx.tt_config, ctx.real_name_from_client.as_deref());
    if let Err(e) = ctx
        .tx_bridge
        .send(BridgeEvent::ToAdmin {
            user_id,
            nick,
            tt_username: username.to_string(),
            msg_content: content.to_string(),
            server_name,
        })
        .await
    {
        tracing::error!(error = %e, "Failed to send admin bridge event");
    }
}

async fn resolve_user_lang(
    db: &crate::infra::db::Database,
    username: &str,
    default_lang: LanguageCode,
) -> LanguageCode {
    if username.is_empty() {
        default_lang
    } else {
        db.get_user_lang_by_tt_user(username)
            .await
            .unwrap_or(default_lang)
    }
}

async fn is_tt_admin(
    db: &crate::infra::db::Database,
    admin_username: Option<&str>,
    username: &str,
) -> bool {
    if username.is_empty() {
        return false;
    }
    if admin_username.is_some_and(|u| u == username) {
        return true;
    }
    if let Some(tg_id) = db.get_telegram_id_by_tt_user(username).await {
        db.get_all_admins()
            .await
            .is_ok_and(|admins| admins.contains(&tg_id))
    } else {
        false
    }
}

fn send_user_reply(
    tx_tt_cmd: &tokio::sync::mpsc::Sender<TtCommand>,
    user_id: i32,
    username: &str,
    text: String,
) {
    if let Err(e) = tx_tt_cmd.try_send(TtCommand::ReplyToUser { user_id, text }) {
        tracing::error!(
            user_id,
            tt_username = %username,
            error = %e,
            "Failed to send TT reply command"
        );
    }
}

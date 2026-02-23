use crate::adapters::tg::utils::ensure_subscribed;
use crate::core::types::{AfkListMode, TtUsername};
use crate::infra::db::app_settings::AppSettingKey;
use teloxide_ng::prelude::*;
use teloxide_ng::sugar::request::RequestReplyExt;

use super::CommandCtx;

const HELP_TEXT: &str = "AFK commands:\n/afk\n/afk on|off\n/afk threshold <minutes>\n/afk mode none|blacklist|whitelist\n/afk cooldown <seconds>\n/afk list add|del <tt_username>\n/afk override set <tt_username> <minutes>\n/afk override del <tt_username>\n/afk global show|enabled <true|false>|threshold <minutes>|mode <none|blacklist|whitelist>|cooldown <seconds> (admin)";

#[allow(clippy::too_many_lines)]
pub(super) async fn handle_afk(ctx: &CommandCtx<'_>, text: String) -> ResponseResult<()> {
    if !ensure_subscribed(ctx.bot, ctx.msg, ctx.db, ctx.config, ctx.lang).await {
        return Ok(());
    }

    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.is_empty() {
        return send_status(ctx).await;
    }

    match parts[0].to_ascii_lowercase().as_str() {
        "global" => handle_global(ctx, &parts[1..]).await,
        "on" => {
            ctx.db.set_afk_enabled(ctx.telegram_id, true).await.ok();
            reply(ctx, "AFK notifications enabled.").await
        }
        "off" => {
            ctx.db.set_afk_enabled(ctx.telegram_id, false).await.ok();
            reply(ctx, "AFK notifications disabled.").await
        }
        "threshold" => {
            let Some(raw) = parts.get(1) else {
                return reply(ctx, HELP_TEXT).await;
            };
            let Ok(minutes) = raw.parse::<i64>() else {
                return reply(ctx, "Threshold must be an integer.").await;
            };
            if !(1..=1440).contains(&minutes) {
                return reply(ctx, "Threshold must be between 1 and 1440 minutes.").await;
            }
            ctx.db
                .set_afk_threshold_minutes(ctx.telegram_id, minutes)
                .await
                .ok();
            reply(ctx, &format!("AFK threshold set to {minutes} min.")).await
        }
        "mode" => {
            let Some(raw_mode) = parts.get(1) else {
                return reply(ctx, HELP_TEXT).await;
            };
            let Ok(mode) = AfkListMode::try_from((*raw_mode).to_ascii_lowercase().as_str()) else {
                return reply(ctx, "Mode must be one of: none, blacklist, whitelist.").await;
            };
            ctx.db.set_afk_list_mode(ctx.telegram_id, mode).await.ok();
            reply(ctx, &format!("AFK list mode set to {mode}.")).await
        }
        "cooldown" => {
            let Some(raw) = parts.get(1) else {
                return reply(ctx, HELP_TEXT).await;
            };
            let Ok(seconds) = raw.parse::<i64>() else {
                return reply(ctx, "Cooldown must be an integer.").await;
            };
            if seconds < 0 {
                return reply(ctx, "Cooldown must be >= 0 seconds.").await;
            }
            ctx.db
                .set_afk_cooldown_seconds(ctx.telegram_id, seconds)
                .await
                .ok();
            reply(ctx, &format!("AFK cooldown set to {seconds} sec.")).await
        }
        "list" => {
            let Some(action) = parts.get(1) else {
                return reply(ctx, HELP_TEXT).await;
            };
            let Some(username) = parts.get(2) else {
                return reply(ctx, HELP_TEXT).await;
            };
            let settings = ctx
                .db
                .resolve_afk_settings_for_user(ctx.telegram_id)
                .await
                .ok();
            let Some(mode) = settings.map(|s| s.list_mode) else {
                return reply(ctx, "Failed to resolve AFK settings.").await;
            };
            if matches!(mode, AfkListMode::None) {
                return reply(
                    ctx,
                    "AFK list mode is 'none'. Switch to blacklist/whitelist first.",
                )
                .await;
            }
            match action.to_ascii_lowercase().as_str() {
                "add" => {
                    let username = TtUsername::from(*username);
                    let tracked = ctx
                        .db
                        .get_afk_tracked_users(ctx.telegram_id, mode)
                        .await
                        .unwrap_or_default();
                    if tracked.contains(&username) {
                        return reply(ctx, &format!("{username} is already in AFK {mode}.")).await;
                    }
                    ctx.db
                        .toggle_afk_tracked_user(ctx.telegram_id, mode, &username)
                        .await
                        .ok();
                    reply(ctx, &format!("{username} added to AFK {mode}.")).await
                }
                "del" => {
                    let username = TtUsername::from(*username);
                    let tracked = ctx
                        .db
                        .get_afk_tracked_users(ctx.telegram_id, mode)
                        .await
                        .unwrap_or_default();
                    if !tracked.contains(&username) {
                        return reply(ctx, &format!("{username} is not in AFK {mode}.")).await;
                    }
                    ctx.db
                        .toggle_afk_tracked_user(ctx.telegram_id, mode, &username)
                        .await
                        .ok();
                    reply(ctx, &format!("{username} removed from AFK {mode}.")).await
                }
                _ => reply(ctx, HELP_TEXT).await,
            }
        }
        "override" => {
            let Some(action) = parts.get(1) else {
                return reply(ctx, HELP_TEXT).await;
            };
            let Some(username) = parts.get(2) else {
                return reply(ctx, HELP_TEXT).await;
            };
            match action.to_ascii_lowercase().as_str() {
                "set" => {
                    let Some(raw) = parts.get(3) else {
                        return reply(ctx, HELP_TEXT).await;
                    };
                    let Ok(minutes) = raw.parse::<i64>() else {
                        return reply(ctx, "Override minutes must be integer.").await;
                    };
                    if !(1..=1440).contains(&minutes) {
                        return reply(ctx, "Override minutes must be 1..1440.").await;
                    }
                    ctx.db
                        .set_afk_threshold_override(
                            ctx.telegram_id,
                            &TtUsername::from(*username),
                            minutes,
                        )
                        .await
                        .ok();
                    reply(
                        ctx,
                        &format!("AFK override for {username} set to {minutes} min."),
                    )
                    .await
                }
                "del" => {
                    ctx.db
                        .delete_afk_threshold_override(
                            ctx.telegram_id,
                            &TtUsername::from(*username),
                        )
                        .await
                        .ok();
                    reply(ctx, &format!("AFK override for {username} removed.")).await
                }
                _ => reply(ctx, HELP_TEXT).await,
            }
        }
        _ => reply(ctx, HELP_TEXT).await,
    }
}

async fn send_status(ctx: &CommandCtx<'_>) -> ResponseResult<()> {
    let Ok(settings) = ctx.db.resolve_afk_settings_for_user(ctx.telegram_id).await else {
        return reply(ctx, "Failed to load AFK settings.").await;
    };

    let tracked_blacklist = ctx
        .db
        .get_afk_tracked_users(ctx.telegram_id, AfkListMode::Blacklist)
        .await
        .unwrap_or_default()
        .len();
    let tracked_whitelist = ctx
        .db
        .get_afk_tracked_users(ctx.telegram_id, AfkListMode::Whitelist)
        .await
        .unwrap_or_default()
        .len();
    let overrides = ctx
        .db
        .list_afk_threshold_overrides(ctx.telegram_id)
        .await
        .unwrap_or_default()
        .len();

    let status = if settings.enabled { "on" } else { "off" };
    let text = format!(
        "AFK status: {status}\nThreshold: {} min\nMode: {}\nCooldown: {} sec\nBlacklist entries: {}\nWhitelist entries: {}\nOverrides: {}\n\n{}",
        settings.threshold_minutes,
        settings.list_mode,
        settings.cooldown_seconds,
        tracked_blacklist,
        tracked_whitelist,
        overrides,
        HELP_TEXT,
    );
    reply(ctx, &text).await
}

async fn reply(ctx: &CommandCtx<'_>, text: &str) -> ResponseResult<()> {
    ctx.bot
        .send_message(ctx.msg.chat.id, text.to_string())
        .reply_to(ctx.msg.id)
        .await?;
    Ok(())
}

#[allow(clippy::too_many_lines)]
async fn handle_global(ctx: &CommandCtx<'_>, args: &[&str]) -> ResponseResult<()> {
    if !ctx.is_admin {
        return reply(ctx, "Admin only.").await;
    }
    let Some(sub) = args.first() else {
        return reply(ctx, HELP_TEXT).await;
    };
    match sub.to_ascii_lowercase().as_str() {
        "show" => {
            let enabled = ctx
                .db
                .get_app_setting(AppSettingKey::AfkDefaultEnabled)
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| "false".to_string());
            let threshold = ctx
                .db
                .get_app_setting(AppSettingKey::AfkDefaultThresholdMinutes)
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| "10".to_string());
            let mode = ctx
                .db
                .get_app_setting(AppSettingKey::AfkDefaultListMode)
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| "none".to_string());
            let cooldown = ctx
                .db
                .get_app_setting(AppSettingKey::AfkDefaultCooldownSeconds)
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| "0".to_string());
            reply(
                ctx,
                &format!(
                    "AFK global defaults\nenabled={enabled}\nthreshold={threshold}\nmode={mode}\ncooldown={cooldown}"
                ),
            )
            .await
        }
        "enabled" => {
            let Some(raw) = args.get(1) else {
                return reply(ctx, HELP_TEXT).await;
            };
            let v = raw.eq_ignore_ascii_case("true");
            let _ = ctx
                .db
                .set_app_setting(
                    AppSettingKey::AfkDefaultEnabled,
                    if v { "true" } else { "false" },
                )
                .await;
            reply(ctx, "AFK global enabled updated.").await
        }
        "threshold" => {
            let Some(raw) = args.get(1) else {
                return reply(ctx, HELP_TEXT).await;
            };
            let Ok(minutes) = raw.parse::<i64>() else {
                return reply(ctx, "Threshold must be an integer.").await;
            };
            if !(1..=1440).contains(&minutes) {
                return reply(ctx, "Threshold must be 1..1440.").await;
            }
            let _ = ctx
                .db
                .set_app_setting(
                    AppSettingKey::AfkDefaultThresholdMinutes,
                    &minutes.to_string(),
                )
                .await;
            reply(ctx, "AFK global threshold updated.").await
        }
        "mode" => {
            let Some(raw_mode) = args.get(1) else {
                return reply(ctx, HELP_TEXT).await;
            };
            let Ok(mode) = AfkListMode::try_from((*raw_mode).to_ascii_lowercase().as_str()) else {
                return reply(ctx, "Mode must be none|blacklist|whitelist.").await;
            };
            let _ = ctx
                .db
                .set_app_setting(AppSettingKey::AfkDefaultListMode, &mode.to_string())
                .await;
            reply(ctx, "AFK global mode updated.").await
        }
        "cooldown" => {
            let Some(raw) = args.get(1) else {
                return reply(ctx, HELP_TEXT).await;
            };
            let Ok(seconds) = raw.parse::<i64>() else {
                return reply(ctx, "Cooldown must be integer.").await;
            };
            if seconds < 0 {
                return reply(ctx, "Cooldown must be >= 0.").await;
            }
            let _ = ctx
                .db
                .set_app_setting(
                    AppSettingKey::AfkDefaultCooldownSeconds,
                    &seconds.to_string(),
                )
                .await;
            reply(ctx, "AFK global cooldown updated.").await
        }
        _ => reply(ctx, HELP_TEXT).await,
    }
}

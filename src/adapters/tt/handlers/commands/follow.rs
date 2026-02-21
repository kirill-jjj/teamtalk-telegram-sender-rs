use crate::app::services::tt_follow::{self, FollowOverride, ValueOverride};
use crate::bootstrap::config::FollowOfflinePolicy;
use crate::core::types::{TtChannelName, TtChannelPassword};
use crate::infra::locales;

use super::user::UserCtx;

pub(super) async fn handle_follow(ctx: &UserCtx) {
    if !is_main_admin(ctx) {
        let text = locales::get_text(ctx.reply_lang.as_str(), locales::LocaleKey::CmdUnauth, None);
        ctx.send_reply(text).await;
        return;
    }

    let parts: Vec<&str> = ctx.content.split_whitespace().collect();
    if parts.len() < 2 {
        ctx.send_reply(follow_help()).await;
        return;
    }

    let command = parts[1].to_ascii_lowercase();
    match command.as_str() {
        "status" => handle_status(ctx).await,
        "on" | "off" => handle_toggle(ctx, command == "on", &parts[2..]).await,
        "policy" => handle_policy(ctx, &parts[2..]).await,
        "fallback" => handle_fallback(ctx, &parts[2..]).await,
        _ => {
            ctx.send_reply(follow_help()).await;
        }
    }
}

fn is_main_admin(ctx: &UserCtx) -> bool {
    ctx.admin_username
        .as_ref()
        .is_some_and(|u| u == &ctx.username)
}

async fn handle_status(ctx: &UserCtx) {
    let (session, permanent) = {
        let follow = ctx
            .follow_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        (
            follow.session_override.clone(),
            follow.permanent_override.clone(),
        )
    };
    let effective = tt_follow::resolve_effective_config(&ctx.tt_config, &permanent, &session);
    let text = format!(
        "follow status: enabled={}, policy={}, fallback={}, source={}",
        effective.enabled,
        tt_follow::policy_to_str(effective.offline_policy),
        effective
            .fallback_channel
            .as_ref()
            .map_or("-", TtChannelName::as_str),
        tt_follow::source_to_str(effective.source)
    );
    ctx.send_reply(text).await;
}

async fn handle_toggle(ctx: &UserCtx, enabled: bool, args: &[&str]) {
    let scope = parse_scope(args.last().copied());
    let patch = FollowOverride {
        enabled: Some(enabled),
        ..FollowOverride::default()
    };
    if let Err(err) = apply_patch(ctx, scope, patch).await {
        ctx.send_reply(format!("follow update failed: {err}")).await;
        return;
    }
    let scope_text = scope.as_str();
    let state = if enabled { "on" } else { "off" };
    ctx.send_reply(format!("follow {state} ({scope_text})"))
        .await;
}

async fn handle_policy(ctx: &UserCtx, args: &[&str]) {
    if args.is_empty() {
        ctx.send_reply(
            "usage: /follow policy <leave_root|stay|fallback_channel> [session|permanent]"
                .to_string(),
        )
        .await;
        return;
    }
    let Some(policy) = parse_policy(args[0]) else {
        ctx.send_reply("invalid policy. use leave_root|stay|fallback_channel".to_string())
            .await;
        return;
    };
    let scope = parse_scope(args.get(1).copied());
    let patch = FollowOverride {
        offline_policy: Some(policy),
        ..FollowOverride::default()
    };
    if let Err(err) = apply_patch(ctx, scope, patch).await {
        ctx.send_reply(format!("follow update failed: {err}")).await;
        return;
    }
    ctx.send_reply(format!(
        "follow policy={} ({})",
        tt_follow::policy_to_str(policy),
        scope.as_str()
    ))
    .await;
}

async fn handle_fallback(ctx: &UserCtx, args: &[&str]) {
    if args.is_empty() {
        ctx.send_reply(
            "usage: /follow fallback <channel_path> [password] [session|permanent]".to_string(),
        )
        .await;
        return;
    }
    let mut channel = args[0];
    if channel.eq_ignore_ascii_case("none") || channel == "-" {
        channel = "";
    }

    let mut password: Option<&str> = None;
    let mut scope = Scope::Session;
    if let Some(last) = args.last().copied() {
        if let Some(parsed) = Scope::parse(last) {
            scope = parsed;
            if args.len() >= 3 {
                password = Some(args[1]);
            }
        } else if args.len() >= 2 {
            password = Some(args[1]);
        }
    }

    let fallback_channel = if channel.trim().is_empty() {
        ValueOverride::Clear
    } else {
        ValueOverride::Set(TtChannelName::from(channel.to_string()))
    };
    let fallback_password = match password {
        Some(pwd) if !pwd.is_empty() => {
            ValueOverride::Set(TtChannelPassword::from(pwd.to_string()))
        }
        Some(_) => ValueOverride::Clear,
        None => ValueOverride::Unchanged,
    };

    let patch = FollowOverride {
        fallback_channel,
        fallback_channel_password: fallback_password,
        ..FollowOverride::default()
    };
    if let Err(err) = apply_patch(ctx, scope, patch).await {
        ctx.send_reply(format!("follow update failed: {err}")).await;
        return;
    }
    ctx.send_reply(format!("follow fallback updated ({})", scope.as_str()))
        .await;
}

async fn apply_patch(ctx: &UserCtx, scope: Scope, patch: FollowOverride) -> anyhow::Result<()> {
    match scope {
        Scope::Session => {
            let mut follow = ctx
                .follow_state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            merge_override(&mut follow.session_override, patch);
            drop(follow);
            Ok(())
        }
        Scope::Permanent => {
            tt_follow::save_permanent_override(&ctx.services.db, &patch).await?;
            let mut follow = ctx
                .follow_state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            merge_override(&mut follow.permanent_override, patch);
            drop(follow);
            Ok(())
        }
    }
}

fn merge_override(target: &mut FollowOverride, patch: FollowOverride) {
    if let Some(enabled) = patch.enabled {
        target.enabled = Some(enabled);
    }
    if let Some(policy) = patch.offline_policy {
        target.offline_policy = Some(policy);
    }
    if !matches!(patch.fallback_channel, ValueOverride::Unchanged) {
        target.fallback_channel = patch.fallback_channel;
    }
    if !matches!(patch.fallback_channel_password, ValueOverride::Unchanged) {
        target.fallback_channel_password = patch.fallback_channel_password;
    }
}

fn parse_policy(raw: &str) -> Option<FollowOfflinePolicy> {
    match raw.to_ascii_lowercase().as_str() {
        "leave_root" => Some(FollowOfflinePolicy::LeaveRoot),
        "stay" => Some(FollowOfflinePolicy::Stay),
        "fallback_channel" => Some(FollowOfflinePolicy::FallbackChannel),
        _ => None,
    }
}

fn parse_scope(raw: Option<&str>) -> Scope {
    raw.and_then(Scope::parse).unwrap_or(Scope::Session)
}

fn follow_help() -> String {
    String::from(
        "usage: /follow status | /follow on|off [session|permanent] | /follow policy <leave_root|stay|fallback_channel> [session|permanent] | /follow fallback <channel> [password] [session|permanent]",
    )
}

#[derive(Clone, Copy)]
enum Scope {
    Session,
    Permanent,
}

impl Scope {
    fn parse(raw: &str) -> Option<Self> {
        match raw.to_ascii_lowercase().as_str() {
            "session" => Some(Self::Session),
            "permanent" => Some(Self::Permanent),
            _ => None,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Session => "session",
            Self::Permanent => "permanent",
        }
    }
}

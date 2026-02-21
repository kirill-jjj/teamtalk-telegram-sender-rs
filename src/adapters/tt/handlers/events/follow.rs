use crate::adapters::tt::context::{PendingFollowCmd, WorkerContext};
use crate::app::services::tt_follow;
use crate::bootstrap::config::FollowOfflinePolicy;
use crate::core::types::{LanguageCode, TtChannelName, TtUserId};
use teamtalk::types::{ChannelId, User};
use teamtalk::{Client, Message};

fn is_main_admin_username(ctx: &WorkerContext, username: &str) -> bool {
    ctx.config
        .general
        .admin_username
        .as_ref()
        .is_some_and(|u| !username.is_empty() && u.as_str() == username)
}

fn track_pending_cmd(ctx: &WorkerContext, cmd_id: i32, pending: PendingFollowCmd) {
    if cmd_id <= 0 {
        tracing::warn!(
            action = pending.action,
            channel = pending.target_channel.as_ref().map(TtChannelName::as_str),
            "Follow command rejected in current TeamTalk state"
        );
        return;
    }
    let mut follow = ctx
        .follow_state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    follow.pending_cmds.insert(cmd_id, pending);
}

fn join_admin_channel(client: &Client, ctx: &WorkerContext, channel_id: ChannelId) {
    if channel_id.0 <= 0 {
        tracing::debug!("Follow skip: admin is not in a channel");
        return;
    }
    if client.my_channel_id() == channel_id {
        return;
    }
    let target = crate::adapters::tt::resolve_channel_name(client, channel_id, LanguageCode::En);
    let cmd_id = client.join_channel(channel_id, "");
    track_pending_cmd(
        ctx,
        cmd_id,
        PendingFollowCmd {
            action: "join_admin_channel",
            target_channel: Some(target),
        },
    );
}

fn join_path_with_password(
    client: &Client,
    ctx: &WorkerContext,
    channel: &TtChannelName,
    password: Option<&crate::core::types::TtChannelPassword>,
    action: &'static str,
) {
    let channel_id = client.get_channel_id_from_path(channel.as_str());
    if channel_id.0 <= 0 {
        tracing::warn!(
            action,
            channel = %channel,
            "Follow fallback channel path not found"
        );
        return;
    }
    if client.my_channel_id() == channel_id {
        return;
    }
    let cmd_id = client.join_channel(
        channel_id,
        password.map_or("", crate::core::types::TtChannelPassword::as_str),
    );
    track_pending_cmd(
        ctx,
        cmd_id,
        PendingFollowCmd {
            action,
            target_channel: Some(channel.clone()),
        },
    );
}

pub(super) fn handle_admin_join_or_login(client: &Client, ctx: &WorkerContext, user: &User) {
    if !is_main_admin_username(ctx, &user.username) {
        return;
    }
    let mut follow = ctx
        .follow_state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    follow.admin_sessions.insert(TtUserId::from(user.id.0));
    let effective = tt_follow::resolve_effective_config(
        &ctx.config.teamtalk,
        &follow.permanent_override,
        &follow.session_override,
    );
    drop(follow);
    if !effective.enabled {
        return;
    }
    join_admin_channel(client, ctx, user.channel_id);
}

pub(super) fn handle_admin_logout(client: &Client, ctx: &WorkerContext, user: &User) {
    if !is_main_admin_username(ctx, &user.username) {
        return;
    }
    let mut follow = ctx
        .follow_state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    follow.admin_sessions.remove(&TtUserId::from(user.id.0));
    let has_admin_sessions = !follow.admin_sessions.is_empty();
    let effective = tt_follow::resolve_effective_config(
        &ctx.config.teamtalk,
        &follow.permanent_override,
        &follow.session_override,
    );
    drop(follow);
    if has_admin_sessions || !effective.enabled {
        return;
    }
    match effective.offline_policy {
        FollowOfflinePolicy::Stay => {}
        FollowOfflinePolicy::LeaveRoot => {
            let cmd_id = client.leave_to_root();
            track_pending_cmd(
                ctx,
                cmd_id,
                PendingFollowCmd {
                    action: "leave_to_root",
                    target_channel: Some(TtChannelName::from(String::from("/"))),
                },
            );
        }
        FollowOfflinePolicy::FallbackChannel => {
            if let Some(channel) = effective.fallback_channel.as_ref() {
                join_path_with_password(
                    client,
                    ctx,
                    channel,
                    effective.fallback_channel_password.as_ref(),
                    "join_fallback_channel",
                );
            } else {
                tracing::warn!(
                    "Follow policy is fallback_channel but no fallback channel configured; leaving to root"
                );
                let cmd_id = client.leave_to_root();
                track_pending_cmd(
                    ctx,
                    cmd_id,
                    PendingFollowCmd {
                        action: "leave_to_root",
                        target_channel: Some(TtChannelName::from(String::from("/"))),
                    },
                );
            }
        }
    }
}

pub(super) fn handle_command_result(event: teamtalk::Event, ctx: &WorkerContext, msg: &Message) {
    let cmd_id = msg.source();
    let pending = {
        let mut follow = ctx
            .follow_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        follow.pending_cmds.remove(&cmd_id)
    };
    let Some(pending) = pending else {
        return;
    };

    match event {
        teamtalk::Event::CmdSuccess => {
            tracing::debug!(
                cmd_id,
                action = pending.action,
                channel = pending.target_channel.as_ref().map(TtChannelName::as_str),
                "Follow command completed successfully"
            );
        }
        teamtalk::Event::CmdError => {
            let err = msg.error_message();
            let (code, raw_message, reason) = match err {
                Some(err) => (err.code, err.message, classify_cmd_error(err.code)),
                None => (0, String::new(), "unknown"),
            };
            tracing::warn!(
                cmd_id,
                action = pending.action,
                channel = pending.target_channel.as_ref().map(TtChannelName::as_str),
                error_code = code,
                error_message = %raw_message,
                reason,
                "Follow command failed"
            );
        }
        _ => {}
    }
}

const fn classify_cmd_error(code: i32) -> &'static str {
    match code {
        2001 => "incorrect_channel_password",
        2003 => "max_server_users_exceeded",
        2004 => "max_channel_users_exceeded",
        2006 => "not_authorized",
        2014 => "command_flood",
        2015 => "channel_banned",
        3000 => "not_logged_in",
        3003 => "already_in_channel",
        3005 => "channel_not_found",
        _ => "unknown",
    }
}

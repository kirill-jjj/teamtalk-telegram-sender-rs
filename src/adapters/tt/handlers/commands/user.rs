use crate::adapters::tt::{WorkerContext, resolve_server_name};
use crate::app::plugins::{TtCommandContext, parse_command_text};
use crate::app::services::tt_context::TtServiceContext;
use crate::core::types::{TtCommand, TtNickname, TtUserId, TtUsername};
use teamtalk::Client;
use teamtalk::types::TextMessage;
use tokio::task::spawn_local;

use super::{admin, bridge, help, plugins, queue, skip, sub};
use crate::app::services::tt_users as tt_users_service;

pub(super) fn handle_user_message(client: &Client, ctx: &WorkerContext, msg: &TextMessage) {
    let real_name_from_client = client.get_server_properties().map(|p| p.name);
    let msg_type = msg.msg_type;
    let msg_text = msg.text.clone();
    let msg_from_id = msg.from_id;
    let tx_tt_cmd = ctx.tx_tt_cmd.clone();
    let tx_bridge = ctx.tx_bridge.clone();
    let state_handle = ctx.state.clone();
    let services = ctx.services();
    let default_lang = ctx.config.general.default_lang;
    let admin_username = ctx.config.general.admin_username.clone();
    let tt_config = ctx.config.teamtalk.clone();
    let deeplink_ttl = ctx.config.operational_parameters.deeplink_ttl;
    let bot_username = ctx.bot_username.clone();
    let tt_msg_sem = ctx.tt_msg_sem.clone();
    let plugins = ctx.plugins.clone();
    let plugins_disabled = ctx.config.plugins.disabled.clone();

    spawn_local(async move {
        if msg_type != teamtalk::client::ffi::TextMsgType::MSGTYPE_USER {
            return;
        }
        let content = msg_text.trim().to_string();
        let from_uid = TtUserId::from(msg_from_id.0);

        let (nick, username): (TtNickname, TtUsername) =
            match state_handle.online_user_by_id(from_uid).await {
                Ok(Some(user)) => (user.nickname, user.username),
                Ok(None) => (TtNickname::from("Unknown"), TtUsername::new(String::new())),
                Err(err) => {
                    tracing::error!(
                        user_id = from_uid.as_i32(),
                        error = %err,
                        "Failed to resolve user for private message"
                    );
                    (TtNickname::from("Unknown"), TtUsername::new(String::new()))
                }
            };

        tracing::info!(
            component = "tt_worker",
            nick = %nick,
            tt_username = %username,
            "Received TT message"
        );

        let reply_lang =
            tt_users_service::resolve_reply_lang(&services, &username, default_lang).await;

        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }
        let cmd = parts[0].to_lowercase();
        let cmd_args = parts
            .iter()
            .skip(1)
            .map(|item| (*item).to_string())
            .collect::<Vec<_>>();
        let needs_heavy = matches!(
            cmd.as_str(),
            "/sub" | "/unsub" | "/skip" | "/help" | "/start"
        );
        let _permit = if needs_heavy {
            Some(tt_msg_sem.acquire_owned().await)
        } else {
            None
        };

        let ctx = UserCtx {
            tx_tt_cmd,
            tx_bridge,
            services,
            reply_lang,
            admin_username,
            tt_config,
            deeplink_ttl,
            bot_username,
            from_uid,
            nick,
            username,
            content,
            real_name_from_client,
            plugins,
            plugins_disabled,
        };
        if handle_plugin_command(&ctx, &cmd, &cmd_args).await {
            return;
        }

        match cmd.as_str() {
            "/sub" => sub::handle_sub(&ctx).await,
            "/unsub" => sub::handle_unsub(&ctx).await,
            "/help" => help::handle_help(&ctx).await,
            "/skip" => skip::handle_skip(&ctx).await,
            "/queue" => queue::handle_queue(&ctx).await,
            "/plugins" => plugins::handle_plugins(&ctx).await,
            "/add_admin" => admin::handle_add_admin(&ctx).await,
            "/remove_admin" => admin::handle_remove_admin(&ctx).await,
            _ => bridge::handle_admin_bridge(&ctx).await,
        }
    });
}

pub(super) struct UserCtx {
    pub tx_tt_cmd: tokio::sync::mpsc::Sender<TtCommand>,
    pub tx_bridge: tokio::sync::mpsc::Sender<crate::core::types::BridgeEvent>,
    pub services: TtServiceContext,
    pub reply_lang: crate::core::types::LanguageCode,
    pub admin_username: Option<TtUsername>,
    pub tt_config: crate::bootstrap::config::TeamTalkConfig,
    pub deeplink_ttl: i64,
    pub bot_username: Option<TtUsername>,
    pub from_uid: TtUserId,
    pub nick: TtNickname,
    pub username: TtUsername,
    pub content: String,
    pub real_name_from_client: Option<String>,
    pub plugins: crate::app::plugins::PluginManagerHandle,
    pub plugins_disabled: Vec<String>,
}

impl UserCtx {
    pub async fn send_reply(&self, text: String) {
        if let Err(e) = self
            .tx_tt_cmd
            .send(TtCommand::ReplyToUser {
                user_id: self.from_uid,
                text,
            })
            .await
        {
            tracing::error!(
                user_id = self.from_uid.as_i32(),
                tt_username = %self.username,
                error = %e,
                "Failed to send TT reply command"
            );
        }
    }

    pub async fn is_admin(&self) -> bool {
        tt_users_service::resolve_is_admin(
            &self.services,
            &self.username,
            self.admin_username.as_ref(),
        )
        .await
    }

    pub fn server_name(&self) -> crate::core::types::TtServerName {
        resolve_server_name(&self.tt_config, self.real_name_from_client.as_deref())
    }
}

async fn handle_plugin_command(ctx: &UserCtx, command: &str, args: &[String]) -> bool {
    if !command.starts_with('/') {
        return false;
    }
    let is_admin = ctx.is_admin().await;
    let Some((plugin_command, _)) = parse_command_text(command) else {
        return false;
    };
    ctx.plugins
        .dispatch_tt_command(
            &plugin_command,
            args,
            TtCommandContext {
                user_id: ctx.from_uid,
                username: ctx.username.clone(),
                nickname: ctx.nick.as_str().to_string(),
                is_admin,
                text: ctx.content.clone(),
            },
        )
        .await
}

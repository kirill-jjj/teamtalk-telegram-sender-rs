use crate::core::types::{TgChatId, TtChannelId, TtCommand, TtUserId};
use mlua::{Function, Lua, LuaSerdeExt, Value};
use serde::Deserialize;
use serde_json::{Value as JsonValue, json};
use std::collections::VecDeque;
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedSender;

pub enum PluginAction {
    TgSend {
        chat_id: i64,
        text: String,
        reply_to: Option<i32>,
    },
    Tt(TtCommand),
}

#[derive(Deserialize, Clone)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub entry: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

const fn default_enabled() -> bool {
    true
}

pub struct PluginRuntime {
    lua: Lua,
    call_timeout: Duration,
}

impl PluginRuntime {
    pub fn load(
        plugin_dir: &Path,
        manifest: &PluginManifest,
        action_tx: UnboundedSender<PluginAction>,
        call_timeout: Duration,
    ) -> anyhow::Result<Self> {
        let lua = Lua::new();
        let runtime = Self { lua, call_timeout };
        runtime.register_api(action_tx)?;
        let entry_path = plugin_dir.join(&manifest.entry);
        let source = std::fs::read_to_string(&entry_path)?;
        runtime.lua.load(&source).set_name(&manifest.name).exec()?;
        Ok(runtime)
    }

    pub fn dispatch_command(
        &self,
        command: &str,
        args: &[String],
        context: &JsonValue,
    ) -> anyhow::Result<bool> {
        let globals = self.lua.globals();
        let handlers: Value = globals.get("commands")?;
        let Value::Table(handlers) = handlers else {
            return Ok(false);
        };
        let func: Value = handlers.get(command)?;
        let Value::Function(func) = func else {
            return Ok(false);
        };

        let args_table = self.lua.create_table()?;
        for (idx, arg) in args.iter().enumerate() {
            args_table.set(idx + 1, arg.clone())?;
        }
        let context_value = self.lua.to_value(context)?;
        self.call_bool(&func, (args_table, context_value))
    }

    pub fn dispatch_event(&self, event: &JsonValue) -> anyhow::Result<bool> {
        let globals = self.lua.globals();
        let event_value = self.lua.to_value(event)?;
        let mut handled = false;

        if let Ok(Value::Function(func)) = globals.get::<Value>("on_event") {
            handled |= self.call_bool(&func, event_value.clone())?;
        }
        if let Ok(Value::Table(table)) = globals.get::<Value>("events")
            && let Ok(Value::Function(func)) =
                table.get::<Value>(event["name"].as_str().unwrap_or_default())
        {
            handled |= self.call_bool(&func, event_value)?;
        }
        Ok(handled)
    }

    fn call_bool<A>(&self, func: &Function, args: A) -> anyhow::Result<bool>
    where
        A: mlua::IntoLuaMulti,
    {
        let started = Instant::now();
        let result: Value = func.call(args)?;
        if started.elapsed() > self.call_timeout {
            anyhow::bail!("plugin call timeout exceeded");
        }
        Ok(matches!(result, Value::Boolean(true)))
    }

    fn register_api(&self, action_tx: UnboundedSender<PluginAction>) -> anyhow::Result<()> {
        let globals = self.lua.globals();

        let register_command =
            self.lua
                .create_function(|lua, (name, func): (String, Function)| {
                    let commands =
                        if let Value::Table(table) = lua.globals().get::<Value>("commands")? {
                            table
                        } else {
                            let table = lua.create_table()?;
                            lua.globals().set("commands", table.clone())?;
                            table
                        };
                    commands.set(name.trim().trim_start_matches('/').to_lowercase(), func)?;
                    Ok(())
                })?;
        globals.set("register_command", register_command)?;

        let tg = self.lua.create_table()?;
        let tx = action_tx.clone();
        tg.set(
            "send",
            self.lua
                .create_function(move |_lua, (chat_id, text): (i64, String)| {
                    let _ = tx.send(PluginAction::TgSend {
                        chat_id,
                        text,
                        reply_to: None,
                    });
                    Ok(())
                })?,
        )?;
        let tx = action_tx.clone();
        tg.set(
            "reply",
            self.lua.create_function(
                move |_lua, (chat_id, message_id, text): (i64, i32, String)| {
                    let _ = tx.send(PluginAction::TgSend {
                        chat_id,
                        text,
                        reply_to: Some(message_id),
                    });
                    Ok(())
                },
            )?,
        )?;
        globals.set("tg", tg)?;

        let tt = self.lua.create_table()?;
        let tx = action_tx.clone();
        tt.set(
            "send_user",
            self.lua
                .create_function(move |_lua, (user_id, text): (i32, String)| {
                    let _ = tx.send(PluginAction::Tt(TtCommand::ReplyToUser {
                        user_id: TtUserId::from(user_id),
                        text,
                    }));
                    Ok(())
                })?,
        )?;
        let tx = action_tx.clone();
        tt.set(
            "send_channel",
            self.lua
                .create_function(move |_lua, (channel_id, text): (i32, String)| {
                    let _ = tx.send(PluginAction::Tt(TtCommand::SendToChannel {
                        channel_id: TtChannelId::from(channel_id),
                        text,
                    }));
                    Ok(())
                })?,
        )?;
        let tx = action_tx;
        tt.set(
            "command",
            self.lua.create_function(
                move |_lua, (name, args): (String, Option<Vec<String>>)| {
                    let args = args.unwrap_or_default();
                    if let Some(cmd) = parse_tt_command(&name, &args) {
                        let _ = tx.send(PluginAction::Tt(cmd));
                    }
                    Ok(())
                },
            )?,
        )?;
        globals.set("tt", tt)?;

        let bot = self.lua.create_table()?;
        bot.set(
            "now_unix",
            self.lua
                .create_function(|_lua, ()| Ok(chrono::Utc::now().timestamp()))?,
        )?;
        bot.set(
            "log",
            self.lua
                .create_function(|_lua, (level, message): (String, String)| {
                    tracing::info!(target: "plugin", level = %level, %message);
                    Ok(())
                })?,
        )?;
        globals.set("bot", bot)?;

        Ok(())
    }
}

fn parse_tt_command(name: &str, args: &[String]) -> Option<TtCommand> {
    match name.trim().to_lowercase().as_str() {
        "broadcast" => Some(TtCommand::Broadcast {
            text: args.join(" "),
        }),
        "reply_user" => {
            let user_id = args.first()?.parse::<i32>().ok()?;
            let text = args.get(1..)?.join(" ");
            Some(TtCommand::ReplyToUser {
                user_id: TtUserId::from(user_id),
                text,
            })
        }
        "send_channel" => {
            let channel_id = args.first()?.parse::<i32>().ok()?;
            let text = args.get(1..)?.join(" ");
            Some(TtCommand::SendToChannel {
                channel_id: TtChannelId::from(channel_id),
                text,
            })
        }
        "who" => {
            let chat_id = args.first()?.parse::<i64>().ok()?;
            Some(TtCommand::Who {
                chat_id: TgChatId::from(chat_id),
                lang: crate::core::types::LanguageCode::En,
                reply_to: None,
            })
        }
        "kick" => {
            let user_id = args.first()?.parse::<i32>().ok()?;
            Some(TtCommand::KickUser {
                user_id: TtUserId::from(user_id),
            })
        }
        "ban" => {
            let user_id = args.first()?.parse::<i32>().ok()?;
            Some(TtCommand::BanUser {
                user_id: TtUserId::from(user_id),
            })
        }
        "load_accounts" => Some(TtCommand::LoadAccounts),
        "skip_stream" => Some(TtCommand::SkipStream),
        _ => None,
    }
}

pub fn event_envelope(
    name: &str,
    source: &str,
    normalized: &JsonValue,
    raw: &JsonValue,
) -> JsonValue {
    json!({
        "name": name,
        "source": source,
        "normalized": normalized,
        "raw": raw
    })
}

pub fn normalized_tg_context(ctx: &crate::app::plugins::TgCommandContext) -> JsonValue {
    json!({
        "source": "tg",
        "chat_id": ctx.chat_id,
        "user_id": ctx.user_id,
        "is_admin": ctx.is_admin,
        "text": ctx.text,
    })
}

pub fn normalized_tt_context(ctx: &crate::app::plugins::TtCommandContext) -> JsonValue {
    json!({
        "source": "tt",
        "user_id": ctx.user_id.as_i32(),
        "username": ctx.username.as_str(),
        "nickname": ctx.nickname,
        "is_admin": ctx.is_admin,
        "text": ctx.text,
    })
}

pub fn should_disable(errors: &mut VecDeque<Instant>, window: Duration, threshold: u32) -> bool {
    let now = Instant::now();
    errors.push_back(now);
    while let Some(front) = errors.front() {
        if now.duration_since(*front) > window {
            let _ = errors.pop_front();
        } else {
            break;
        }
    }
    u32::try_from(errors.len()).unwrap_or(u32::MAX) >= threshold
}

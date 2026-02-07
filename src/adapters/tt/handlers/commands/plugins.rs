use super::user::UserCtx;

pub(super) async fn handle_plugins(ctx: &UserCtx) {
    if !ctx.is_admin().await {
        ctx.send_reply("Unauthorized".to_string()).await;
        return;
    }
    let parts = ctx.content.split_whitespace().collect::<Vec<_>>();
    let sub = parts.get(1).copied().unwrap_or("status");

    let result = match sub {
        "status" => Ok(ctx.plugins.status_text().await),
        "reload" => match parts.get(2).copied() {
            Some(name) => ctx
                .plugins
                .reload_named(name, &ctx.plugins_disabled)
                .await
                .map(|()| format!("Plugin reloaded: {name}"))
                .map_err(|error| error.to_string()),
            None => Err("Usage: /plugins reload <name>".to_string()),
        },
        "enable" => match parts.get(2).copied() {
            Some(name) => ctx
                .plugins
                .set_enabled(name, true)
                .await
                .map(|()| format!("Plugin enabled: {name}"))
                .map_err(|error| error.to_string()),
            None => Err("Usage: /plugins enable <name>".to_string()),
        },
        "disable" => match parts.get(2).copied() {
            Some(name) => ctx
                .plugins
                .set_enabled(name, false)
                .await
                .map(|()| format!("Plugin disabled: {name}"))
                .map_err(|error| error.to_string()),
            None => Err("Usage: /plugins disable <name>".to_string()),
        },
        _ => Err("Usage: /plugins status|reload|enable|disable".to_string()),
    };

    match result {
        Ok(text) => ctx.send_reply(text).await,
        Err(error) => {
            ctx.send_reply(format!("Plugin command failed: {error}"))
                .await;
        }
    }
}

# Plugins Guide

This document explains how to create, enable, disable, and operate Lua plugins.

## What a plugin can do

Plugins run inside the bot process through `mlua` and can:

- Register custom slash commands (for Telegram and TeamTalk private text flow).
- React to Telegram and TeamTalk events.
- Send Telegram messages.
- Send TeamTalk commands via bot command channel.
- Use helper API exposed by the bot (`register_command`, `tg.*`, `tt.*`, `bot.*`).

The project is open-source and plugins are treated as user-provided code.  
If plugin code is unsafe or buggy, it can break plugin behavior and may affect runtime quality.

## Folder layout

All plugins are loaded from the root `plugins/` directory.

Expected structure:

```text
plugins/
  example/
    plugin.toml
    main.lua
  my_plugin/
    plugin.toml
    index.lua
```

Each plugin is one directory with:

- `plugin.toml` (manifest)
- one entry Lua file from manifest `entry`

## Manifest format (`plugin.toml`)

Required fields:

```toml
name = "my_plugin"
version = "0.1.0"
entry = "index.lua"
enabled = true
```

Field meaning:

- `name`: unique plugin id used by loader.
- `version`: free-form version string.
- `entry`: Lua file path relative to plugin folder.
- `enabled`: plugin default enabled/disabled state.

If `plugin.toml` is invalid, plugin is not loaded and error is logged.

## Global plugin config (`config.toml`)

Use `[plugins]` section:

```toml
[plugins]
enabled = true
dir = "plugins"
auto_reload = true
call_timeout_ms = 500
error_window_seconds = 60
error_threshold = 10
disabled = []
```

Meaning:

- `enabled`: master switch for plugin system.
- `dir`: root plugin directory.
- `auto_reload`: watch filesystem and reload on file changes.
- `call_timeout_ms`: maximum single Lua handler call time.
- `error_window_seconds`: error accounting time window.
- `error_threshold`: if exceeded, plugin auto-disables.
- `disabled`: list of plugin names forcibly disabled by config.

Priority of enable state:

1. plugin manifest `enabled = true/false`
2. global `disabled = ["name"]` list

Final effective state is both checks combined.

## Hot reload behavior

When `auto_reload = true`:

- any change under `plugins/` is watched;
- changed plugin is reloaded;
- if new version fails to load, previous working version stays active;
- loader logs detailed error.

This gives safe reload without service interruption.

## Command registration

Use `register_command` in Lua:

```lua
register_command("ping", function(args, ctx)
    tg.send(ctx.chat_id, "pong")
    return true
end)
```

Optional source filter:

```lua
register_command("ping", function(args, ctx)
    tg.send(ctx.chat_id, "pong")
    return true
end, { tg = true, tt = false })
```

Rules:

- command is registered without leading `/` (`"ping"`).
- returning `true` means command handled.
- plugin command priority is higher than built-in command handlers.
- third argument `{ tg = bool, tt = bool }` is optional; defaults to both `true`.

## Context object for command handlers

Handler gets `(args, ctx)`.

Telegram command context fields:

- `ctx.source = "tg"`
- `ctx.chat_id`
- `ctx.user_id`
- `ctx.is_admin`
- `ctx.text`

TeamTalk command context fields:

- `ctx.source = "tt"`
- `ctx.user_id`
- `ctx.username`
- `ctx.nickname`
- `ctx.is_admin`
- `ctx.text`

`args` is array-like table with command arguments.

TeamTalk event normalized payload also includes:

- `normalized.is_linked` - `true` when TT user is linked to Telegram.
- `normalized.linked_telegram_id` - linked Telegram id or `null`.

## Event handling

Plugins can handle events through:

- `on_event(event)` generic hook
- `events["EventName"] = function(event) ... end` specific hook

Event object contains:

- `event.name` - event name
- `event.source` - `tg` or `tt`
- `event.normalized` - normalized payload for convenient handling
- `event.raw` - raw payload fields collected from source message

Example:

```lua
events = {}
events["UserLoggedIn"] = function(event)
    if event.raw.user ~= nil then
        bot.log("info", "login: " .. event.raw.user.nickname)
    end
    return false
end
```

## Lua API exposed by bot

### `register_command(name, fn, opts?)`
- Registers command handler callable from slash command flow.
- `opts` is optional table: `{ tg = true/false, tt = true/false }`.

### `tg.*`
- `tg.send(chat_id, text)`
- `tg.reply(chat_id, message_id, text)`

### `tt.*`
- `tt.send_user(user_id, text)`
- `tt.send_channel(channel_id, text)`
- `tt.command(name, args_table)`

Current `tt.command` names:

- `broadcast`
- `reply_user`
- `send_channel`
- `who`
- `kick`
- `ban`
- `load_accounts`
- `skip_stream`

### API extension policy (for core contributors)

When adding new bot features in Core/TG/TT modules:

1. Decide if the feature should be plugin-accessible.
2. If yes, add API mapping in `src/app/plugins/runtime.rs`.
3. Add/adjust unit tests in `tests/unit/app_plugins.rs`.
4. Update this document (`PLUGINS.md`) in the same commit.
5. Update example plugin in `plugins/example/` when behavior is user-visible.

No plugin API changes should be merged without docs + tests.

## Advanced examples

### Modular plugin with multiple Lua files

`plugin.toml`
```toml
name = "rec"
version = "0.1.0"
entry = "main.lua"
enabled = true
```

`main.lua`
```lua
local commands = require("commands")
local events = require("events")

register_command("rec", commands.handle_rec)
on_event = events.on_event
```

`commands.lua`
```lua
local M = {}

function M.handle_rec(args, ctx)
    if #args == 0 then
        if ctx.source == "tg" then
            tg.send(ctx.chat_id, "usage: /rec start|stop")
        else
            tt.send_user(ctx.user_id, "usage: /rec start|stop")
        end
        return true
    end

    if args[1] == "start" then
        -- Example: map to backend command exposed by tt.command
        tt.command("broadcast", {"Recording started"})
        return true
    end

    if args[1] == "stop" then
        tt.command("broadcast", {"Recording stopped"})
        return true
    end

    return false
end

return M
```

`events.lua`
```lua
local M = {}

function M.on_event(event)
    if event.source == "tt" and event.name == "UserLoggedIn" then
        bot.log("info", "UserLoggedIn observed by rec plugin")
    end
    return false
end

return M
```

## Plugin status and operations

Admin commands:

- TG: `/plugins status`, `/plugins reload <name>`, `/plugins enable <name>`, `/plugins disable <name>`
- TT: `/plugins status`, `/plugins reload <name>`, `/plugins enable <name>`, `/plugins disable <name>`

Status includes:

- enabled/disabled state
- forced-disable flag from config
- command/event counters
- failure/timeout counters
- last error message

### `bot.*`
- `bot.now_unix()`
- `bot.log(level, message)`

## Example plugin

See `plugins/example/`.

It demonstrates:

- command registration (`ping`)
- source-sensitive reply logic (TG/TT)
- event hook for TeamTalk login event

## Core plugins in repository

You can ship "core plugins" directly in repo by placing them under `plugins/`.

Recommended approach:

- keep one folder per plugin (`plugins/<name>/...`);
- keep plugin logic isolated;
- version plugin through `plugin.toml`;
- document public behavior in this file or plugin-specific `README.md`.

## Operational notes

- Plugin errors are counted per plugin in a rolling time window.
- If errors exceed threshold, plugin is auto-disabled.
- Auto-disabled plugin can be re-enabled by fixing code and reloading.
- For production, keep `call_timeout_ms` strict and monitor logs.

## Quick start checklist

1. Add plugin folder under `plugins/`.
2. Add valid `plugin.toml`.
3. Create entry Lua file.
4. Set `[plugins].enabled = true` in `config.toml`.
5. Optional: set `[plugins].disabled = []`.
6. Start bot and verify logs.
7. Trigger command/event and confirm behavior.


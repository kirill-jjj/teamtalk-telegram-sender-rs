local function cfg(key, fallback)
    return bot.config(key, fallback)
end

local function now_stamp()
    local ts = tonumber(bot.now_unix()) or os.time()
    return os.date("!%Y%m%d_%H%M%S", ts)
end

local function build_output_path()
    local template = cfg("recording.path_template", "recordings/tt_record_{date}_{time}.wav")
    local stamp = now_stamp()
    local date_part = string.sub(stamp, 1, 8)
    local time_part = string.sub(stamp, 10, 15)

    local path = template
    path = string.gsub(path, "{datetime}", stamp)
    path = string.gsub(path, "{date}", date_part)
    path = string.gsub(path, "{time}", time_part)
    return path
end

local function msg(key, fallback)
    return cfg("messages." .. key, fallback)
end

local function reply(ctx, text)
    if ctx.source == "tg" then
        tg.send(ctx.chat_id, text)
    else
        tt.send_user(ctx.user_id, text)
    end
end

local function resolve_notify_chat(ctx)
    if ctx.source == "tg" then
        return tostring(ctx.chat_id)
    end

    local raw = cfg("recording.default_notify_chat", 0)
    local chat_id = tonumber(raw) or 0
    if chat_id <= 0 then
        return nil
    end
    return tostring(chat_id)
end

local function start_recording(ctx)
    local notify_chat = resolve_notify_chat(ctx)
    if notify_chat == nil and ctx.source == "tt" then
        reply(ctx, msg("no_target", "Не настроен чат для отправки записи в Telegram"))
        return true
    end

    local output_path = build_output_path()
    local format = cfg("recording.format", "wave")
    local auto_subscribe_audio = tostring(cfg("recording.auto_subscribe_audio", true))

    tt.command("record_start", {
        notify_chat or "",
        output_path,
        format,
        auto_subscribe_audio,
    })

    reply(ctx, msg("started", "Запись запущена"))
    return true
end

local function stop_recording(ctx)
    local notify_chat = resolve_notify_chat(ctx)
    if notify_chat == nil and ctx.source == "tt" then
        reply(ctx, msg("no_target", "Не настроен чат для отправки записи в Telegram"))
        return true
    end

    local caption = msg("caption", "Запись готова")
    local delete_after_send = tostring(cfg("recording.delete_after_send", true))

    tt.command("record_stop", {
        notify_chat or "",
        caption,
        delete_after_send,
    })

    if ctx.source == "tg" then
        reply(ctx, msg("stopped_tg", "Запись остановлена, отправляю файл..."))
    else
        reply(ctx, msg("stopped_tt", "Запись остановлена"))
    end

    return true
end

local function handle_rec(args, ctx)
    local action = args[1]
    if action == nil then
        reply(ctx, msg("usage", "Использование: /rec start|stop"))
        return true
    end

    action = string.lower(action)
    if action == "start" then
        return start_recording(ctx)
    end
    if action == "stop" then
        return stop_recording(ctx)
    end

    reply(ctx, msg("usage", "Использование: /rec start|stop"))
    return true
end

register_command("rec", handle_rec)

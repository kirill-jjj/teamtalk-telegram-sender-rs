register_command("ping", function(args, ctx)
    if ctx.source == "tg" then
        tg.send(ctx.chat_id, "pong from plugin")
    else
        tt.send_user(ctx.user_id, "pong from plugin")
    end
    return true
end)

events = {}
events["UserLoggedIn"] = function(event)
    local user = event.raw.user
    if user ~= nil and user.nickname ~= nil then
        bot.log("info", "plugin saw login: " .. user.nickname)
    end
    return false
end

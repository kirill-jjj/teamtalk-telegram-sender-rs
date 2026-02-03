commands = {
  start = function(args)
    return true
  end
}

state = {
  total_text = 0
}

function on_event(ev)
  if ev.type == "TextMessage" and ev.text ~= nil then
    state.total_text = state.total_text + 1
  end
  return false
end

events = {
  UserJoined = function(ev)
    return false
  end,
  UserLeft = function(ev)
    return false
  end
}
# General
hello-start = Hello! Use /help to see available commands.
help-text =
    <b>Available Commands:</b>
    /who - Show online users.
    /settings - Access the interactive settings menu (language, notifications, mute lists, Offline-only feature).
    /unsub - Unsubscribe from notifications.
    /help - Show this help message.
    (Note: <code>/start</code> is used to initiate the bot and process deeplinks.)

    <b>Admin Commands:</b>
    /kick - Kick a user from the server (via buttons).
    /ban - Ban a user from the server (via buttons).
    /unban - Unban a user from the server (shows a list of banned users).
    /subscribers - View and manage subscribed users.
    /broadcast - Send a broadcast to all TeamTalk users.
    /message - Send a Telegram message to all subscribers.
    /exit - Shut down the bot.

cmd-invalid-deeplink = Invalid or expired deeplink.
cmd-success-sub = You have successfully subscribed to notifications.
cmd-success-sub-guest = Subscribed as GUEST. Note: Offline-only is unavailable.
cmd-success-unsub = You have successfully unsubscribed from notifications.
cmd-relink = TeamTalk account linked successfully!
cmd-fail-account = Your TeamTalk account must have a username to subscribe.
cmd-fail-noon-guest = Offline-only requires a linked TeamTalk account.
cmd-error = An error occurred. Please try again later.
admin-error-user = User { $user_id } error in { $context }: { $error }
admin-error-context-command = command handling
admin-error-context-callback = callback handling
admin-error-context-tt-command = TeamTalk command dispatch
admin-error-context-subscription = subscription check
admin-error-context-update-listener = Telegram update listener
cmd-no-users = No users found online.
cmd-unauth = You are not authorized to perform this action.
cmd-broadcast-empty = Usage: /broadcast <text>
cmd-broadcast-sent = Broadcast sent to TeamTalk.
cmd-message-empty = Usage: /message <text>
cmd-message-sent =
    Sent to { $sent ->
        [one] { NUMBER($sent) } subscriber
       *[other] { NUMBER($sent) } subscribers
    }{ $failed ->
        [0] .
        [one] (error: { NUMBER($failed) })
       *[other] (errors: { NUMBER($failed) })
    }
cmd-not-subscribed = You are not subscribed. Request a link via <code>/sub</code> in TeamTalk.
cmd-user-banned = Your Telegram account is banned from using this service.
cmd-tt-banned = The TeamTalk username '{ $name }' is banned.
cmd-shutting-down = Shutting down...

# Unsubscribe
cmd-desc-unsub = Unsubscribe from notifications
unsub-confirm-text = Are you sure you want to unsubscribe? This will delete your settings and stop all notifications.
unsub-cancelled = Operation cancelled. You remain subscribed.
btn-yes = Yes
btn-no = No

# Notifications
event-join = { $nickname } joined server { $server }
event-leave = { $nickname } left server { $server }

# Settings Menu
settings-title = <b>Settings</b>
msg-choose-lang = Please choose your language:
btn-lang = Language
btn-sub-settings = Subscription Settings
btn-notif-settings = Notification Settings

# Notification Settings
notif-settings-title = <b>Notification Settings</b>
btn-noon = Offline-only: { $status }
btn-mute-manage = Manage Mute List
resp-noon-updated = Offline-only updated: { $status }

# Mute Management
mute-title = <b>Manage Mute List</b>

    { $mode_desc }
    { $guest_note }

mute-guest-note = ⚠️ <b>Note on Guests:</b> This server allows shared guest accounts. You cannot mute a specific guest individually. Adding a guest account to the blacklist will mute <b>ALL</b> users logged in as guests.

mute-mode-blacklist = Current mode is Blacklist. You receive notifications from everyone except those on the list.
mute-mode-whitelist = Current mode is Whitelist. You only receive notifications from users on the list.

display-guest-account = 👤 Guest Account
alert-mute-guest = ⚠️ WARNING: You are muting the shared Guest account. This will mute/unmute ALL users currently logged in as guests!

btn-mode-blacklist = { $marker } Blacklist Mode
btn-mode-whitelist = { $marker } Whitelist Mode
btn-manage-list = Manage { $mode }
btn-mute-server-list = Mute/Unmute from Server List
btn-manage-blacklist = Manage Blacklist
btn-manage-whitelist = Manage Whitelist
btn-mute-server-list-blacklist = Mute/Unmute from Server List (Blacklist)
btn-mute-server-list-whitelist = Mute/Unmute from Server List (Whitelist)


mode-blacklist = Blacklist
mode-whitelist = Whitelist

# User List Item Status
item-status-blacklist-in = 🚫 { $name } (in blacklist)
item-status-blacklist-out = ✅ { $name } (not in blacklist)
item-status-whitelist-in = ✅ { $name } (in whitelist)
item-status-whitelist-out = 🚫 { $name } (not in whitelist)

# Pagination / Lists
list-kick-title = Select a user to kick from { $server }:
list-ban-title = Select a user to ban from { $server }:
list-unban-title = Banned Users
list-subs-title = Here is the list of subscribers.
list-mute-title = Mute list
list-mute-title-for = Mute list for: { $name }
list-all-accs-title = All Server Accounts
list-link-title = Select a TeamTalk account to link to subscriber { $id }:
list-empty = The list is empty.
list-subs-empty = No subscribers found.
list-ban-empty = The ban list is empty.
list-mute-empty = The mute list is currently empty.
list-search-hint = Type a username or nickname to search.
list-search-empty = No matches for "{ $query }".
list-search-title = Search results: "{ $query }"
list-page = Page { NUMBER($current) }/{ NUMBER($total) }

btn-prev = ⬅️ Prev
btn-next = Next ➡️
btn-back = Back to { $dest }
btn-back-settings = Back to Settings
btn-back-notif = Back to Notifications
btn-back-mute = Back to Mute Management
btn-back-menu = Back to Main Menu
btn-back-subs = Back to Subscribers List
btn-back-user-actions = Back to User Actions
btn-back-manage-acc = Back to Manage Account
btn-back-search = Back to list

# Toast messages
toast-mute-mode-set = Mute list mode set to { $mode }.
toast-user-muted =
    { $action ->
        [added] { $user } was added to the list.
        [removed] { $user } was removed from the list.
       *[toggled] { $user } was toggled.
    }
toast-lang-updated = Language has been changed.
toast-command-sent = Command sent.
toast-user-banned = User was banned and their profile was deleted.
toast-user-unbanned = User has been successfully unbanned.
toast-subscriber-deleted = Subscriber deleted successfully.
toast-account-unlinked = Account { $user } has been unlinked.
toast-account-linked = Successfully linked TeamTalk account: { $user }.
toast-noon-toggled = Offline-only for subscriber { $id } set to: { $status }.
toast-lang-set = Language for subscriber { $id } changed to { $lang }.
toast-notif-set = Notification preference for subscriber { $id } set to: { $val }.
toast-mute-mode-sub-set = Mute list mode for subscriber { $id } set to: { $val }.

act-added-blacklist = added to blacklist
act-removed-blacklist = removed from blacklist
act-added-whitelist = added to whitelist
act-removed-whitelist = removed from whitelist

status-enabled = Enabled
status-disabled = Disabled

# Admin
admin-alert =
    Message from server <b>{ $server }</b>
    From <b>{ $nick }</b>:

    { $msg }
admin-channel-pm =
    Message from server <b>{ $server }</b>, channel <b>{ $channel }</b>:

    { $msg }
tg-reply-sent = Message sent.
tg-reply-failed = Failed to send message.
tg-reply-offline = User is offline.
tt-msg-sent = Message sent to Telegram.
tt-msg-failed = Failed to send message to Telegram.
tt-channel-reply = Reply to: { $msg } (duration { $duration })
tt-channel-reply-text =
    Reply to: { $msg }
    { $reply }
tt-skip-sent = Skip command sent.

# TT Commands & Responses
tt-admin-added =
    { $count ->
        [one] Successfully added { NUMBER($count) } admin.
       *[other] Successfully added { NUMBER($count) } admins.
    }
tt-admin-add-fail =
    { $count ->
        [one] Failed to add { NUMBER($count) } admin (already admin or invalid ID).
       *[other] Failed to add { NUMBER($count) } admins (already admins or invalid IDs).
    }
tt-admin-removed =
    { $count ->
        [one] Successfully removed { NUMBER($count) } admin.
       *[other] Successfully removed { NUMBER($count) } admins.
    }
tt-admin-remove-fail =
    { $count ->
        [one] Failed to remove { NUMBER($count) } admin (not admin or invalid ID).
       *[other] Failed to remove { NUMBER($count) } admins (not admins or invalid IDs).
    }
tt-admin-no-ids = No valid admin IDs provided for adding or removing.
tt-admin-help-header =

    Admin commands (MAIN_ADMIN from config only):
tt-admin-help-cmds =
    /add_admin <Telegram ID> [<Telegram ID>...] - Add bot admin.
    /remove_admin <Telegram ID> [<Telegram ID>...] - Remove bot admin.
    /skip - Skip media playback.

tt-report-header =
    There { $count ->
        [one] is { NUMBER($count) } user
       *[other] are { NUMBER($count) } users
    } on the server { $server }:
tt-report-unauth = (not in a channel)
tt-sub-fail-nouser = Your TeamTalk account must have a username to subscribe.
tt-sub-link = Click this link to subscribe: { $link }
tt-unsub-link = Click this link to unsubscribe: { $link }
tt-error-generic = Error. Try again.

# Icons & Symbols
icon-muted = 🔇
icon-unmuted = 🔊
icon-checked = ✅
icon-unchecked = ⚪️
icon-check-simple = ✅

# TeamTalk Report
tt-report-root = the root channel
tt-root-channel-name = root channel
tt-report-row = <b>{ $users }</b> in { $channel }

# Subscription Settings
btn-sub-all = { $marker } All (Join & Leave)
btn-sub-join = { $marker } Join Only
btn-sub-leave = { $marker } Leave Only
btn-sub-none = { $marker } None
resp-sub-updated = Subscription setting updated to: { $text }.

# Menu
menu-title = <b>Main Menu:</b>
btn-menu-who = ℹ️ Who is online?
btn-menu-settings = ⚙️ Settings
btn-menu-help = ❓ Help
btn-menu-kick = 👢 Kick User
btn-menu-ban = 🚫 Ban User
btn-menu-unban = ✅ Unban User
btn-menu-subs = 👥 Subscribers
btn-menu-unsub = 🚪 Unsubscribe

# Subscriber Details
sub-details-title = <b>Subscriber: { $name }</b>
    Linked TT Account: { $tt_user }
    Language: { $lang }
    Offline-only: { $noon }
    Notifications: { $notif }
    Mute Mode: { $mode }

sub-manage-tt-title = Manage TeamTalk account link for subscriber { $id }:
sub-lang-title = Select new language for subscriber { $id }:
sub-notif-title = Select notification preference for subscriber { $id }:
sub-mode-title = Select mute list mode for subscriber { $id }:

btn-sub-delete = 🗑️ Delete Subscriber
btn-sub-ban = 🚫 Ban User (TG & TT)
btn-sub-manage-tt = 🔗 Manage TeamTalk Account
btn-sub-lang = 🗣️ Change Language
btn-sub-noon = 🌞 Toggle Offline-only
btn-sub-notif = 🔔 Set Notification Prefs
btn-sub-mute-mode = 🔇 Set Mute Mode
btn-sub-view-mute = 📜 View Mute List
btn-unban = ✅ Unban
btn-unlink = ➖ Unlink { $user }
btn-link-new = ➕ Link/Change TeamTalk Account

# Bot Command Descriptions
cmd-desc-menu = Show main menu with all commands
cmd-desc-who = Show online users in TeamTalk
cmd-desc-help = Show this help message
cmd-desc-settings = Access interactive settings menu
cmd-desc-kick = Kick TT user (admin, via buttons)
cmd-desc-ban = Ban TT user (admin, via buttons)
cmd-desc-unban = Unban user (shows a list of banned users)
cmd-desc-subscribers = View and manage subscribed users
cmd-desc-exit = Shut down the bot
cmd-desc-broadcast = Send a TeamTalk broadcast message
cmd-desc-message = Send a Telegram message to subscribers
val-none = None

cmd-desc-queue = Reply queue.
cmd-queue-help = /queue on|off (admin), /queue me on|off, /queue clear [all]
cmd-queue-no-link = Link your TeamTalk account first.
resp-queue-user-enabled = Reply queue enabled.
resp-queue-user-disabled = Reply queue disabled.
resp-queue-user-already-enabled = { -queue-reply-already-enabled }
resp-queue-user-already-disabled = { -queue-reply-already-disabled }
resp-queue-global-enabled = Global reply queue enabled.
resp-queue-global-disabled = Global reply queue disabled.
resp-queue-global-already-enabled = Global reply queue is already enabled.
resp-queue-global-already-disabled = Global reply queue is already disabled.
resp-queue-global-disabled-user = { -queue-global-disabled-user }
resp-queue-cleared = { -queue-cleared($count) }
resp-queue-cleared-all = { -queue-cleared-all($count) }

queue-settings-title = Reply queue
btn-queue-settings = Reply queue
btn-queue-user-toggle = Personal queue: { $status }
btn-queue-global-toggle = Global queue: { $status }
btn-queue-clear = Clear my queue
btn-queue-clear-all = Clear all queues

tg-reply-queued = Message received and queued.

tt-queue-help = /queue on|off, /queue me on|off, /queue clear [all]
tt-queue-no-link = Link your Telegram account first.
tt-queue-user-enabled = Personal queue enabled.
tt-queue-user-disabled = Personal queue disabled.
tt-queue-user-already-enabled = { -queue-reply-already-enabled }
tt-queue-user-already-disabled = { -queue-reply-already-disabled }
tt-queue-global-enabled = Global queue enabled.
tt-queue-global-disabled = Global queue disabled.
tt-queue-global-already-enabled = Global queue is already enabled.
tt-queue-global-already-disabled = Global queue is already disabled.
tt-queue-global-disabled-user = { -queue-global-disabled-user }
tt-queue-cleared = { -queue-cleared($count) }
tt-queue-cleared-all = { -queue-cleared-all($count) }

-queue-reply-already-enabled = Reply queue is already enabled.
-queue-reply-already-disabled = Reply queue is already disabled.
-queue-global-disabled-user = Global reply queue is disabled by the admin.
-queue-cleared =
    Queue cleared ({ $count ->
        [one] { NUMBER($count) } item
       *[other] { NUMBER($count) } items
    }).
-queue-cleared-all =
    Queue cleared for all ({ $count ->
        [one] { NUMBER($count) } item
       *[other] { NUMBER($count) } items
    }).

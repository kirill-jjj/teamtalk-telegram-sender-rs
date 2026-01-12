# General
hello-start = Hello! Use /help to see available commands.
help-text =
    <b>Available Commands:</b>
    /who - Show online users.
    /settings - Access the interactive settings menu (language, notifications, mute lists, NOON feature).
    /unsub - Unsubscribe from notifications.
    /help - Show this help message.
    (Note: <code>/start</code> is used to initiate the bot and process deeplinks.)

    <b>Admin Commands:</b>
    /kick - Kick a user from the server (via buttons).
    /ban - Ban a user from the server (via buttons).
    /unban - Unban a user from the server (shows a list of banned users).
    /subscribers - View and manage subscribed users.
    /exit - Shut down the bot.

cmd-invalid-deeplink = Invalid or expired deeplink.
cmd-success-sub = You have successfully subscribed to notifications.
cmd-success-sub-guest = You have subscribed as a GUEST. Note: "NOON" mode is unavailable.
cmd-success-unsub = You have successfully unsubscribed from notifications.
cmd-relink = TeamTalk account linked successfully!
cmd-fail-account = Your TeamTalk account must have a username to subscribe.
cmd-fail-noon-guest = Feature unavailable. You must have a linked TeamTalk account to use NOON mode. Please subscribe from a registered account.
cmd-error = An error occurred. Please try again later.
admin-error-user = User {user_id} error in {context}: {error}
admin-error-context-command = command handling
admin-error-context-callback = callback handling
admin-error-context-tt-command = TeamTalk command dispatch
admin-error-context-subscription = subscription check
admin-error-context-update-listener = Telegram update listener
cmd-no-users = No users found online.
cmd-unauth = You are not authorized to perform this action.
cmd-not-subscribed = You are not subscribed. Please request a link from the bot in TeamTalk via <code>/sub</code> command.
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
btn-noon = NOON (Not on Online): { $status }
btn-mute-manage = Manage Mute List
resp-noon-updated = NOON status updated: { $status }

# Mute Management
mute-title = <b>Manage Mute List</b>

    { $mode_desc }

    ⚠️ <b>Note on Guests:</b> This server allows shared guest accounts. You cannot mute a specific guest individually. Adding a guest account to the blacklist will mute <b>ALL</b> users logged in as guests.

mute-mode-blacklist = Current mode is Blacklist. You receive notifications from everyone except those on the list.
mute-mode-whitelist = Current mode is Whitelist. You only receive notifications from users on the list.

display-guest-account = 👤 Guest Account
alert-mute-guest = ⚠️ WARNING: You are muting the shared Guest account. This will mute/unmute ALL users currently logged in as guests!

btn-mode-blacklist = { $marker } Blacklist Mode
btn-mode-whitelist = { $marker } Whitelist Mode
btn-manage-list = Manage { $mode }
btn-mute-server-list = Mute/Unmute from Server List

mode-blacklist = Blacklist
mode-whitelist = Whitelist

# User List Item Status
item-status-muted = { $name } (Status: Muted)
item-status-unmuted = { $name } (Status: Not Muted)

# Pagination / Lists
list-kick-title = Select a user to kick from { $server }:
list-ban-title = Select a user to ban from { $server }:
list-unban-title = Banned Users
list-subs-title = Here is the list of subscribers.
list-mute-title = Mute list for: { $name }
list-all-accs-title = All Server Accounts
list-link-title = Select a TeamTalk account to link to subscriber { $id }:
list-empty = The list is empty.
list-subs-empty = No subscribers found.
list-ban-empty = The ban list is empty.
list-mute-empty = The mute list is currently empty.
list-page = Page { $current }/{ $total }

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

# Toast messages
toast-mute-mode-set = Mute list mode set to { $mode }.
toast-user-muted = { $user } has been { $action }.
toast-lang-updated = Language has been changed.
toast-command-sent = Command sent.
toast-user-banned = User was banned and their profile was deleted.
toast-user-unbanned = User has been successfully unbanned.
toast-subscriber-deleted = Subscriber deleted successfully.
toast-account-unlinked = Account { $user } has been unlinked.
toast-account-linked = Successfully linked TeamTalk account: { $user }.
toast-noon-toggled = NOON for subscriber { $id } set to: { $status }.
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
tt-admin-added = Successfully added { $count } admins.
tt-admin-add-fail = Failed to add { $count } admins (already admins or invalid IDs).
tt-admin-removed = Successfully removed { $count } admins.
tt-admin-remove-fail = Failed to remove { $count } admins (not admins or invalid IDs).
tt-admin-no-ids = No valid admin IDs provided for adding or removing.
tt-admin-help-header =

    Admin commands (MAIN_ADMIN from config only):
tt-admin-help-cmds =
    /add_admin <Telegram ID> [<Telegram ID>...] - Add bot admin.
    /remove_admin <Telegram ID> [<Telegram ID>...] - Remove bot admin.
    /skip - Skip media playback.

tt-report-header = There are { $count } users on the server { $server }:
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
    NOON (Not on Online): { $noon }
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
btn-sub-noon = 🌞 Toggle NOON
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
val-none = None

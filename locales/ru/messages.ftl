# General
hello-start = Привет! Используйте /help для просмотра доступных команд.
help-text =
    <b>Доступные команды:</b>
    /who - Показать онлайн пользователей.
    /settings - Доступ к интерактивному меню настроек (язык, уведомления, списки игнора, функция «Только офлайн»).
    /unsub - Отписаться от уведомлений.
    /help - Показать это сообщение.
    (Примечание: <code>/start</code> используется для запуска бота и обработки deeplink-ссылок.)

    <b>Команды для администраторов:</b>
    /kick - Кикнуть пользователя с сервера (через кнопки).
    /ban - Забанить пользователя на сервере (через кнопки).
    /unban - Разбанить пользователя (показывает список забаненных).
    /subscribers - Просмотр и управление подписчиками.
    /broadcast - Отправить объявление всем пользователям TeamTalk.
    /message - Отправить сообщение всем подписчикам в Telegram.
    /exit - Выключить бота.

cmd-invalid-deeplink = Недействительная или истекшая ссылка.
cmd-success-sub = Вы успешно подписались на уведомления.
cmd-success-sub-guest = Вы подписались как ГОСТЬ. Примечание: режим «Только офлайн» недоступен.
cmd-success-unsub = Вы успешно отписались от уведомлений.
cmd-relink = TeamTalk аккаунт успешно привязан!
cmd-fail-account = У вашего аккаунта TeamTalk должен быть username для подписки.
cmd-fail-noon-guest = Режим «Только офлайн» доступен только с привязанным аккаунтом TeamTalk.
cmd-error = Произошла ошибка. Попробуйте позже.
admin-error-user = Ошибка у пользователя { $user_id } в { $context }: { $error }
admin-error-context-command = обработка команды
admin-error-context-callback = обработка кнопки
admin-error-context-tt-command = отправка команды TeamTalk
admin-error-context-subscription = проверка подписки
admin-error-context-update-listener = обработчик обновлений Telegram
cmd-no-users = Пользователей онлайн не найдено.
cmd-unauth = У вас нет прав для этого действия.
cmd-broadcast-empty = Использование: /broadcast <текст>
cmd-broadcast-sent = Сообщение отправлено в TeamTalk.
cmd-message-empty = Использование: /message <текст>
cmd-message-sent =
    { $sent ->
        [one] Отправлено { NUMBER($sent) } подписчику
        [few] Отправлено { NUMBER($sent) } подписчикам
       *[many] Отправлено { NUMBER($sent) } подписчикам
    }{ $failed ->
        [0] .
        [one] (ошибка: { NUMBER($failed) })
        [few] (ошибки: { NUMBER($failed) })
       *[many] (ошибок: { NUMBER($failed) })
    }
cmd-not-subscribed = Вы не подписаны. Запросите ссылку командой <code>/sub</code> в TeamTalk.
cmd-user-banned = Ваш Telegram аккаунт заблокирован и не может использовать этот сервис.
cmd-tt-banned = Имя пользователя TeamTalk '{ $name }' забанено.
cmd-shutting-down = Выключение...

# Unsubscribe
cmd-desc-unsub = Отписаться от уведомлений
unsub-confirm-text = Вы уверены, что хотите отписаться? Это удалит ваши настройки и остановит все уведомления.
unsub-cancelled = Операция отменена. Вы остаетесь подписаны.
btn-yes = Да
btn-no = Нет

# Notifications
event-join = { $nickname } присоединился к серверу { $server }
event-leave = { $nickname } покинул сервер { $server }

# Settings Menu
settings-title = <b>Настройки</b>
msg-choose-lang = Пожалуйста, выберите ваш язык:
btn-lang = Язык (Language)
btn-sub-settings = Настройки подписки
btn-notif-settings = Настройки уведомлений

# Notification Settings
notif-settings-title = <b>Настройки уведомлений</b>
btn-noon = Только офлайн: { $status }
btn-mute-manage = Управление списком игнора
resp-noon-updated = Статус «Только офлайн» обновлён: { $status }

# Mute Management
mute-title = <b>Управление списком игнора</b>

    { $mode_desc }
    { $guest_note }

mute-guest-note = ⚠️ <b>О гостевых аккаунтах:</b> На сервере разрешено использование общей гостевой учетной записи. Заглушить конкретного гостя невозможно — добавление гостя в черный список скроет уведомления от <b>ВСЕХ</b>, кто сидит с этого аккаунта.

mute-mode-blacklist = Текущий режим: Черный список. Вы получаете уведомления от всех, КРОМЕ тех, кто в списке.
mute-mode-whitelist = Текущий режим: Белый список. Вы получаете уведомления ТОЛЬКО от пользователей в списке.

display-guest-account = 👤 Гостевой аккаунт
alert-mute-guest = ⚠️ ВНИМАНИЕ: Вы глушите общую гостевую учетную запись. Это заглушит/разглушит ВСЕХ пользователей, которые сидят как гости!

btn-mode-blacklist = { $marker } Черный список
btn-mode-whitelist = { $marker } Белый список
btn-manage-list = Управлять { $mode }
btn-mute-server-list = Добавить/убрать из списка сервера
btn-manage-blacklist = Управлять черным списком
btn-manage-whitelist = Управлять белым списком
btn-mute-server-list-blacklist = Добавить/убрать из списка сервера (черный список)
btn-mute-server-list-whitelist = Добавить/убрать из списка сервера (белый список)


mode-blacklist = Черным списком
mode-whitelist = Белым списком

# User List Item Status
item-status-blacklist-in = 🚫 { $name } (в чёрном списке)
item-status-blacklist-out = ✅ { $name } (не в чёрном списке)
item-status-whitelist-in = ✅ { $name } (в белом списке)
item-status-whitelist-out = 🚫 { $name } (не в белом списке)

# Pagination / Lists
list-kick-title = Выберите пользователя для кика с сервера { $server }:
list-ban-title = Выберите пользователя для бана на сервере { $server }:
list-unban-title = Забаненные пользователи
list-subs-title = Вот список подписчиков.
list-mute-title = Список игнора
list-mute-title-for = Список игнора для: { $name }
list-all-accs-title = Все учетные записи сервера
list-link-title = Выберите учетную запись TeamTalk для привязки к подписчику { $id }:
list-empty = Список пуст.
list-subs-empty = Подписчики не найдены.
list-ban-empty = Список забаненных пуст.
list-mute-empty = Список игнора пуст.
list-search-hint = Введите имя или ник для поиска.
list-search-empty = Ничего не найдено по "{ $query }".
list-search-title = Результаты поиска: "{ $query }"
list-page = Страница { NUMBER($current) }/{ NUMBER($total) }

btn-prev = ⬅️ Назад
btn-next = Вперед ➡️
btn-back = Назад к { $dest }
btn-back-settings = Назад в Настройки
btn-back-notif = Назад в Уведомления
btn-back-mute = Назад в меню игнора
btn-back-menu = Назад в Главное меню
btn-back-subs = Назад к списку подписчиков
btn-back-user-actions = Назад к действиям пользователя
btn-back-manage-acc = Назад к управлению аккаунтом
btn-back-search = Назад к списку

# Toast messages
toast-mute-mode-set = Режим списка игнорирования изменен на { $mode }.
toast-user-muted =
    { $action ->
        [added] { $user } добавлен в список.
        [removed] { $user } удален из списка.
       *[toggled] { $user } статус изменен.
    }
toast-lang-updated = Язык был изменен.
toast-command-sent = Команда отправлена.
toast-admin-added = Пользователь добавлен в админы.
toast-admin-removed = Пользователь удален из админов.
toast-user-banned = Пользователь был забанен, а его профиль удален.
toast-user-unbanned = Пользователь успешно разбанен.
toast-subscriber-deleted = Подписчик успешно удален.
toast-account-unlinked = Аккаунт { $user } был отвязан.
toast-account-linked = Аккаунт TeamTalk успешно привязан: { $user }.
toast-noon-toggled = Статус «Только офлайн» для подписчика { $id } установлен на: { $status }.
toast-lang-set = Язык для подписчика { $id } изменен на { $lang }.
toast-notif-set = Настройка уведомлений для подписчика { $id } установлена на: { $val }.
toast-mute-mode-sub-set = Режим списка игнорирования для подписчика { $id } установлен на: { $val }.

act-added-blacklist = добавлен в чёрный список
act-removed-blacklist = удалён из чёрного списка
act-added-whitelist = добавлен в белый список
act-removed-whitelist = удалён из белого списка

status-enabled = Включено
status-disabled = Выключено

# Admin
admin-alert =
    Сообщение с сервера <b>{ $server }</b>
    От <b>{ $nick }</b>:

    { $msg }
admin-channel-pm =
    Сообщение с сервера <b>{ $server }</b>, канал <b>{ $channel }</b>:

    { $msg }
tg-reply-sent = Сообщение отправлено.
tg-reply-failed = Ошибка отправки сообщения.
tg-reply-offline = Пользователь не в сети.
tt-msg-sent = Сообщение успешно отправлено в Telegram.
tt-msg-failed = Не удалось доставить сообщение в Telegram.
tt-channel-reply = Ответ на: { $msg } (длительность { $duration })
tt-channel-reply-text =
    Ответ на: { $msg }
    { $reply }
tt-skip-sent = Команда пропуска отправлена.

# TeamTalk Admin Commands
tt-admin-added =
    { $count ->
        [one] Успешно добавлен { NUMBER($count) } администратор.
        [few] Успешно добавлены { NUMBER($count) } администратора.
       *[many] Успешно добавлено { NUMBER($count) } администраторов.
    }
tt-admin-add-fail =
    Не удалось добавить { $count ->
        [one] { NUMBER($count) } администратора
        [few] { NUMBER($count) } администратора
       *[many] { NUMBER($count) } администраторов
    } (уже администраторы или неверные ID).
tt-admin-removed =
    { $count ->
        [one] Успешно удален { NUMBER($count) } администратор.
        [few] Успешно удалены { NUMBER($count) } администратора.
       *[many] Успешно удалено { NUMBER($count) } администраторов.
    }
tt-admin-remove-fail =
    Не удалось удалить { $count ->
        [one] { NUMBER($count) } администратора
        [few] { NUMBER($count) } администратора
       *[many] { NUMBER($count) } администраторов
    } (не администраторы или неверные ID).
tt-admin-no-ids = Не указаны действительные ID администраторов для добавления или удаления.
tt-admin-help-header =

    Команды администратора (только для ГЛАВНОГО АДМИНА из конфигурации):
tt-admin-help-cmds =
    /add_admin <Telegram ID> [<Telegram ID>...] - Добавить админа бота.
    /remove_admin <Telegram ID> [<Telegram ID>...] - Удалить админа бота.
    /skip - Пропустить медиа.

tt-report-header =
    На сервере { $server } сейчас { $count ->
        [one] { NUMBER($count) } пользователь
        [few] { NUMBER($count) } пользователя
       *[many] { NUMBER($count) } пользователей
    }:
tt-report-unauth = (не в канале)
tt-sub-fail-nouser = У вашего аккаунта TeamTalk должен быть установлен username для подписки.
tt-sub-link = Нажмите на эту ссылку, чтобы подписаться на уведомления: { $link }
tt-unsub-link = Нажмите на эту ссылку, чтобы отписаться от уведомлений: { $link }
tt-error-generic = Ошибка. Попробуйте позже.

# Icons & Symbols
icon-muted = 🔇
icon-unmuted = 🔊
icon-checked = ✅
icon-unchecked = ⚪️
icon-check-simple = ✅

# TeamTalk Report
tt-report-root = корневом канале
tt-root-channel-name = корневой канал
tt-report-row = <b>{ $users }</b> в { $channel }

# Subscription Settings
btn-sub-all = { $marker } Все (Вход и выход)
btn-sub-join = { $marker } Только вход
btn-sub-leave = { $marker } Только выход
btn-sub-none = { $marker } Нет
resp-sub-updated = Настройка подписки обновлена до: { $text }.

# Menu
menu-title = <b>Главное меню:</b>
btn-menu-who = ℹ️ Кто в сети?
btn-menu-settings = ⚙️ Настройки
btn-menu-help = ❓ Помощь
btn-menu-kick = 👢 Кикнуть пользователя
btn-menu-ban = 🚫 Забанить пользователя
btn-menu-unban = ✅ Разбанить пользователя
btn-menu-subs = 👥 Подписчики
btn-menu-unsub = 🚪 Отписаться

# Subscriber Details
sub-details-title = <b>Подписчик: { $name }</b>
    Привязанный аккаунт TT: { $tt_user }
    Язык: { $lang }
    Только офлайн: { $noon }
    Уведомления: { $notif }
    Режим игнорирования: { $mode }

sub-manage-tt-title = Управление привязкой TeamTalk для подписчика { $id }:
sub-lang-title = Выберите новый язык для подписчика { $id }:
sub-notif-title = Выберите настройку уведомлений для подписчика { $id }:
sub-mode-title = Выберите режим тишины для подписчика { $id }:

btn-sub-delete = 🗑️ Удалить подписчика
btn-sub-ban = 🚫 Забанить (TG и TT)
btn-sub-manage-tt = 🔗 Управлять аккаунтом TeamTalk
btn-sub-lang = 🗣️ Сменить язык
btn-sub-noon = 🌞 Только офлайн
btn-sub-notif = 🔔 Установить настройки уведомлений
btn-sub-mute-mode = 🔇 Установить режим игнорирования
btn-sub-view-mute = 📜 Просмотреть список игнорирования
btn-unban = ✅ Разбанить
btn-unlink = ➖ Отвязать { $user }
btn-link-new = ➕ Привязать/Изменить аккаунт TeamTalk

# Bot Command Descriptions
cmd-desc-menu = Показать главное меню со всеми командами
cmd-desc-who = Показать онлайн пользователей в TeamTalk
cmd-desc-help = Показать это справочное сообщение
cmd-desc-settings = Доступ к интерактивному меню настроек
cmd-desc-kick = Кикнуть пользователя TT (админ, через кнопки)
cmd-desc-ban = Забанить пользователя TT (админ, через кнопки)
cmd-desc-unban = Разбанить пользователя (показывает список забаненных)
cmd-desc-subscribers = Просмотр и управление подписанными пользователями
cmd-desc-exit = Выключить бота
cmd-desc-broadcast = Отправить объявление в TeamTalk
cmd-desc-message = Отправить сообщение подписчикам в Telegram
val-none = Нет

cmd-desc-queue = Управление очередью.
cmd-queue-help = /queue on|off (глобально), /queue me on|off, /queue clear [all]
cmd-queue-no-link = Сначала привяжите аккаунт TeamTalk.
resp-queue-user-enabled = { -queue-reply-enabled }
resp-queue-user-disabled = { -queue-reply-disabled }
resp-queue-user-already-enabled = { -queue-reply-already-enabled }
resp-queue-user-already-disabled = { -queue-reply-already-disabled }
resp-queue-global-enabled = Глобальная очередь ответов включена.
resp-queue-global-disabled = Глобальная очередь ответов отключена.
resp-queue-global-already-enabled = Глобальная очередь ответов уже включена.
resp-queue-global-already-disabled = Глобальная очередь ответов уже отключена.
resp-queue-global-disabled-user = { -queue-global-disabled-user }
resp-queue-cleared = { -queue-cleared($count) }
resp-queue-cleared-all = { -queue-cleared-all($count) }

queue-settings-title = Очередь ответов
btn-queue-settings = Очередь ответов
btn-queue-user-toggle = Очередь ответов: { $status }
btn-queue-global-toggle = Глобальная очередь: { $status }
btn-queue-clear = Очистить очередь
btn-queue-clear-all = Очистить все очереди

tg-reply-queued = Сообщение получено и поставлено в очередь.

tt-queue-help = /queue on|off, /queue me on|off, /queue clear [all]
tt-queue-no-link = Сначала привяжите Telegram аккаунт.
tt-queue-user-enabled = { -queue-reply-enabled }
tt-queue-user-disabled = { -queue-reply-disabled }
tt-queue-user-already-enabled = { -queue-reply-already-enabled }
tt-queue-user-already-disabled = { -queue-reply-already-disabled }
tt-queue-global-enabled = Глобальная очередь включена.
tt-queue-global-disabled = Глобальная очередь отключена.
tt-queue-global-already-enabled = Глобальная очередь уже включена.
tt-queue-global-already-disabled = Глобальная очередь уже отключена.
tt-queue-global-disabled-user = { -queue-global-disabled-user }
tt-queue-cleared = { -queue-cleared($count) }
tt-queue-cleared-all = { -queue-cleared-all($count) }

-queue-reply-enabled = Очередь ответов включена.
-queue-reply-disabled = Очередь ответов отключена.
-queue-reply-already-enabled = Очередь ответов уже включена.
-queue-reply-already-disabled = Очередь ответов уже отключена.
-queue-global-disabled-user = Глобальная очередь ответов отключена администратором.
-queue-cleared =
    Очередь очищена ({ $count ->
        [one] { NUMBER($count) } элемент
        [few] { NUMBER($count) } элемента
       *[many] { NUMBER($count) } элементов
    }).
-queue-cleared-all =
    Очередь очищена для всех ({ $count ->
        [one] { NUMBER($count) } элемент
        [few] { NUMBER($count) } элемента
       *[many] { NUMBER($count) } элементов
    }).

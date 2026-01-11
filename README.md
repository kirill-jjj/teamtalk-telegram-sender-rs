# TeamTalk to Telegram Bridge (Rust)

A robust, asynchronous bridge between a **TeamTalk 5** server and **Telegram**, written in Rust.

This bot monitors your TeamTalk server for user activity (joins, leaves) and sends notifications to subscribed Telegram users. It also provides a full suite of moderation tools, allowing administrators to kick or ban users directly from Telegram.

## 🚀 Features

*   **Real-time Notifications:** Receive alerts when users join or leave the server.
*   **Two-Way Interaction:**
    *   Chat messages sent to the bot in TeamTalk are forwarded to the Telegram Admin.
    *   Admins can reply from Telegram back to the TeamTalk user.
*   **Admin Tools:** Kick and Ban users via an interactive Telegram interface (buttons).
*   **User Settings:**
    *   **Mute Lists:** Blacklist or Whitelist specific users/channels.
    *   **NOON (Not On Online):** Smart feature that mutes notifications if you are currently logged into TeamTalk yourself.
    *   **Localization:** Multi-language support (English and Russian included).
*   **Account Linking:** Securely link TeamTalk accounts to Telegram IDs via Deep Links.
*   **High Performance:** Built with `Tokio`, `Teloxide`, and `SQLx` for speed and safety.

## 🛠 Prerequisites

Before building, ensure you have the following installed:

*   **Rust** (latest stable toolchain): [Install Rust](https://www.rust-lang.org/tools/install)
*   **TeamTalk Server**: Version 5.x.
*   **Telegram Bot**: You will need a bot token from [@BotFather](https://t.me/BotFather).

## 📦 Installation & Building

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/kirill-jjj/teamtalk-telegram-sender-rs.git
    cd teamtalk-telegram-sender-rs
    ```

2.  **Prepare Configuration:**
    Copy the example configuration file:
    ```bash
    cp config.toml.example config.toml
    ```
    *See the [Configuration](#-configuration) section below for details.*

3.  **Build the project:**
    `sqlx` macros require either a live `DATABASE_URL` at build time or prepared query data (`sqlx-data.json` or `.sqlx/`).
    ```bash
    cargo build --release
    ```

    The compiled binary will be located at `target/release/teamtalk-telegram-sender-rs`.

## ⚙️ Configuration

Edit `config.toml` with your settings.

```toml
[teamtalk]
host_name = "your.server.com"
port = 10333
user_name = "bot_account"
password = "bot_password"
nick_name = "Telegram Bot"
# Channel path to join (e.g., "/" for root)
channel = "/"
# Optional channel password
channel_password = ""
# Enable encrypted connection
encrypted = false
# TeamTalk client name
client_name = "telegrambot"
# Optional override for display name
server_name = ""
# Optional: Text displayed in the bot's status field
status_text = "I bridge events to Telegram"
# Skip notifications for these usernames
global_ignore_usernames = ["admin_bot"]
# Optional guest username for filtering
guest_username = ""

[telegram]
# Token for the main interaction bot (optional, disables TG interactions if missing)
event_token = "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11"
# Token for sending admin alerts (optional)
message_token = "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11"
# The Telegram Chat ID of the main administrator
admin_chat_id = 123456789

[general]
admin_username = "MainAdminTTAccount"
default_lang = "en" # 'en' or 'ru'

[operational_parameters]
deeplink_ttl_seconds = 300
tt_reconnect_retry_seconds = 10
tt_reconnect_check_interval_seconds = 30

[database]
db_file = "bot_data.db"
```

## 🏃‍♂️ Running

Run the executable. You can optionally specify the config file path:

```bash
./target/release/teamtalk-telegram-sender-rs --config config.toml
```

On the first run, the bot will automatically create the SQLite database file (`bot_data.db`) and apply all necessary migrations.

## 🤖 Bot Commands

### User Commands
*   `/start` - Initialize the bot or process deep links.
*   `/help` - Show the help message.
*   `/menu` - Open the main interactive menu.
*   `/who` - Show a list of online users in TeamTalk grouped by channel.
*   `/settings` - Open subscription and notification settings.
*   `/unsub` - Unsubscribe from notifications.

### Admin Commands (Restricted)
*   `/kick` - Open an interactive list to kick a user.
*   `/ban` - Open an interactive list to ban a user.
*   `/unban` - Manage the ban list.
*   `/subscribers` - View and manage subscribed Telegram users.
*   `/exit` - Gracefully shut down the bot.

### TeamTalk Chat Commands
If you message the bot inside the TeamTalk client:
*   `/sub` - Generates a Deep Link to subscribe to notifications.
*   `/unsub` - Generates a link to unsubscribe.
*   `/help` - Shows available TT commands.

## 💻 Development

### Database Migrations
This project uses **SQLx** for database management. If you modify the database schema, you need `sqlx-cli`.

1.  **Install CLI:**
    ```bash
    cargo install sqlx-cli
    ```
2.  **Create a `.env` file** (do not commit this):
    ```env
    DATABASE_URL=sqlite:bot_data.db
    ```
3.  **Run migrations:**
    ```bash
    sqlx migrate run
    ```
4.  **Update cached queries** (before committing changes):
    ```bash
    cargo sqlx prepare
    ```

## 🌍 Localization

Translations are handled via **Fluent** (`fluent-templates`).
*   Language files are located in `locales/`.
*   Supported languages: **English (en)**, **Russian (ru)**.
*   The bot automatically detects the user's language preference or falls back to the default defined in `config.toml`.

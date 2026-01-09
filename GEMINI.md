# GEMINI.md - teamtalk-telegram-sender-rs

## Project Overview

This project, `teamtalk-telegram-sender-rs`, is a Rust-based application that acts as a bridge between a TeamTalk server and the Telegram messaging platform. Its primary function is to monitor a TeamTalk server for user join/leave events and broadcast these events as notifications to subscribed Telegram users. It also facilitates administrative actions on the TeamTalk server (like kicking or banning users) directly from a Telegram chat.

The application is structured into three main components:

1.  **TeamTalk Worker (`tt_worker`):** A dedicated thread that connects to the TeamTalk server using the `teamtalk` crate. It listens for server events, processes them, and sends relevant information to the bridge. It also executes commands received from the bridge, such as kicking or banning users.
2.  **Telegram Bot (`tg_bot`):** An asynchronous component built with the `teloxide` framework. It handles all interactions with users on Telegram, including processing commands, managing user settings, and sending notifications.
3.  **Bridge (`bridge`):** The central communication hub that connects the TeamTalk worker and the Telegram bot. It receives events from the `tt_worker` and dispatches them as formatted messages to the `tg_bot` to be sent to subscribed users. It also relays administrative commands from the bot to the worker.

The application uses an SQLite database (`sqlx`) to persist user data, including subscriptions, notification preferences, and administrative roles. Configuration is managed through a `config.toml` file, which specifies connection details for both the TeamTalk server and the Telegram bots (one for general events and another for admin-specific messages).

## Building and Running

### Prerequisites

*   Rust toolchain
*   A running TeamTalk server
*   Telegram Bot tokens

### Configuration

1.  Copy the `config.toml.example` to `config.toml` (if `config.toml` doesn't exist).
2.  Edit `config.toml` to provide the necessary credentials:
    *   `[teamtalk]`: Host, port, and user credentials for the TeamTalk server.
    *   `[telegram]`: API tokens for the event and message bots, and the admin's chat ID.
    *   `[database]`: Path to the SQLite database file.

### Building

To build the project in release mode, run the following command:

```sh
cargo build --release
```

The optimized executable will be located at `target/release/teamtalk-telegram-sender-rs`.

### Running

To run the application, execute the compiled binary:

```sh
./target/release/teamtalk-telegram-sender-rs --config config.toml
```

## Development Conventions

*   **Asynchronous Runtime:** The project heavily relies on `tokio` for its asynchronous operations, particularly for the Telegram bot component.
*   **Concurrency:** The TeamTalk client runs in a separate OS thread to avoid blocking the async runtime, communicating with the rest of the application via channels. The bridge and Telegram bot run as async tasks on the Tokio runtime.
*   **State Management:** Application state (like the list of online users) is shared safely across threads and tasks using `Arc<DashMap>`.
*   **Database:** `sqlx` is used for all database interactions. The database access layer is organized into several modules within the `db` package. Instead of using traits for database operations, the implementation is done using inherent `impl` blocks on the `Database` struct. Each module (e.g., `admins`, `bans`) contributes methods to the `Database` struct, promoting code organization while maintaining a single, coherent database interface.
*   **Localization:** The application uses `fluent-templates` for localization, with message files (`.ftl`) stored in the `locales` directory for different languages.
*   **Modularity:** The code is organized into distinct modules (`tt_worker`, `tg_bot`, `bridge`, `db`, `config`) with clear responsibilities, promoting separation of concerns.

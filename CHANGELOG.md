# Changelog

All notable changes to this project will be documented in this file.

## [0.4.0]
### Added
- Pending replies and streaming queue support.

### Changed
- Introduced layered architecture with app services and adapters.
- Typed callback usernames to reduce stringly typed data.

### Fixed
- Keep NOON silent notifications when another session is still online.
- Respect configured admin chat ID for permissions.
- Sync streaming status for TeamTalk worker.

## [0.3.0]

### Breaking
- Callback payloads are now serialized; old inline buttons from pre-0.3.0 messages will no longer be recognized.

### Added
- Strongly typed enums for user settings, callbacks, and language codes.
- Compact callback serialization using `postcard` + URL-safe base64.
- Admin error notifications and improved error handling across TG/TT flows.
- Deeplink hardening with `expected_telegram_id` checks and periodic cleanup of expired tokens.
- TeamTalk gender is now applied on login.
- CI now runs `cargo check`, `cargo clippy --all-targets --all-features`, and `cargo fmt --check`.

### Changed
- Shared keyboard and callback helpers to reduce UI boilerplate.
- Server name resolution centralized for TeamTalk events.
- Documentation updated to match current configuration layout and sqlx build requirements.

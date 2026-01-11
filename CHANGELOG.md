# Changelog

All notable changes to this project will be documented in this file.

## [0.3.0]

### Breaking
- Callback payloads are now serialized; old inline buttons from pre-0.3.0 messages will no longer be recognized. (#cbd8d17)

### Added
- Strongly typed enums for user settings, callbacks, and language codes. (#afe9942)
- Compact callback serialization using `postcard` + URL-safe base64. (#cbd8d17)
- Admin error notifications and improved error handling across TG/TT flows. (#83eceb6)
- Deeplink hardening with `expected_telegram_id` checks and periodic cleanup of expired tokens. (#68cd0fd)
- TeamTalk gender is now applied on login. (#0775d33)
- CI now runs `cargo check`, `cargo clippy --all-targets --all-features`, and `cargo fmt --check`. (#fd50d12)

### Changed
- Shared keyboard and callback helpers to reduce UI boilerplate. (#2e645b6)
- Server name resolution centralized for TeamTalk events. (#2e645b6)
- Documentation updated to match current configuration layout and sqlx build requirements. (#691368f)

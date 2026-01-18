# Changelog

All notable changes to this project will be documented in this file.

## [0.6.0]
### Breaking
- Callback payloads now use Z85 encoding instead of base64; old inline buttons
  from previous releases will no longer decode.

### Added
- Bootstrap application runner to centralize startup/shutdown wiring.
- Dockerfile and .dockerignore for container builds.

### Changed
- Removed proxy-only app services in favor of direct database calls.
- Simplified callback serialization helpers and keyboard usage.

## [0.7.2]
### Changed
- Reverted TeamTalk async API back to synchronous polling to avoid sporadic delays.

## [0.7.1]
### Fixed
- Avoid blocking sends in TeamTalk async runtime (prevents `/who` panic).

## [0.7.0]
### Added
- Reply-to behavior for command responses (bot replies to the command message).
- Docker Compose file for local runs.
- Expanded unit test coverage and added `cargo test` to pre-commit/CI.

### Changed
- TeamTalk worker now uses async stream API for event handling.
- Reduced TeamTalk worker spin to lower idle CPU usage.
- Docker build cache optimized for faster rebuilds.
- Clippy flags aligned with `-D warnings` in hooks/CI.
- Added `.gitattributes` to normalize line endings (LF).
- AGENTS contributor guidance refreshed.

### Fixed
- Language selection buttons normalized (fixed mojibake).

### Dependencies
- Updated `actions/checkout` to v6.
- Updated `chrono` to 0.4.43.

## [0.6.1]
### Added
- Configurable log level via `general.log_level` in `config.toml`.

### Changed
- Structured tracing fields across TG/TT/bridge/DB logs for better filtering.

### Fixed
- Corrected "Join/Leave Only" subscription label mappings in settings UI.

## [0.4.2]
### Fixed
- Route admin messages based on token presence and equality.

## [0.5.0]
### Added
- Graceful shutdown for SIGINT/SIGTERM and `/exit`, with coordinated TT/TG/bridge stop.

### Changed
- Replaced DashMap with `RwLock<HashMap<...>>` for shared in-memory state.
- Localized root channel label in admin channel notifications.
- Improved pluralization in admin action messages.

## [0.4.1]
### Fixed
- Replies now work when message and event bots use different tokens.

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

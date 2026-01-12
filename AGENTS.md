# Repository Guidelines

## Project Structure & Module Organization
- `src/` holds the Rust application source. Core areas include the Telegram bot (`src/tg_bot/`), TeamTalk worker (`src/tt_worker/`), database access (`src/db/`), and shared types/config (`src/types.rs`, `src/config.rs`).
- `migrations/` contains SQL schema migrations.
- `locales/` stores Fluent `.ftl` localization files.
- `README.md` covers high-level usage and setup.

## Build, Test, and Development Commands
- `cargo build` builds the project in debug mode.
- `cargo build --release` produces an optimized binary at `target/release/teamtalk-telegram-sender-rs`.
- `cargo run -- <args>` runs the binary locally (see `README.md` for required config).
- `cargo test` runs the test suite.
- `cargo fmt` formats Rust code with rustfmt.
- `cargo clippy --all-targets --all-features` runs lint checks.
- After changes, run `cargo check`, then `cargo clippy --all-targets --all-features` if check passes, and `cargo fmt` if clippy passes.
## Verification and Delivery Sequence
- After each significant change (or a batch of related changes), run `cargo check`, then `cargo clippy --all-targets --all-features`, then `cargo fmt`.
- Before committing, run `cargo test` (especially after major changes).
- After tests pass, commit and push.

## Coding Style & Naming Conventions
- Follow rustfmt defaults; keep diffs minimal and avoid formatting churn.
- Use `snake_case` for functions/vars, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- Prefer `Result` with contextual errors over `unwrap` in non-test code.
- Keep modules focused; avoid widening `pub` visibility unless needed.
- Completely avoid adding code comments unless explicitly requested.
- Do not commit `Cargo.lock` or `TEAMTALK_DLL/` unless explicitly requested.
- When asked to commit and push, split commits by type (e.g., docs + code), propose commit messages, and wait for confirmation before pushing. If explicit permission is given to do everything once, proceed; for later push requests, ask again.
- Cargo registry cache lives under `%USERPROFILE%\.cargo\registry\` (e.g., `src` and `cache`). Use it to inspect crate sources (example: find teloxide reply helpers in `teloxide-0.17.0\src\sugar\request.rs`); cache keeps old versions for speed and is safe to read.
- If `pre-commit` fails on `cargo sqlx prepare`, install `sqlx-cli` with `cargo install sqlx-cli --no-default-features --features sqlite`, then run `cargo sqlx prepare` with `DATABASE_URL=sqlite://data.db`.

## Testing Guidelines
- Use `cargo test` to run unit and integration tests.
- Prefer deterministic tests; avoid network calls unless required.
- Name tests by intent, e.g., `connect_retries_on_timeout`.

## Commit & Pull Request Guidelines
- Use Conventional Commits (e.g., `feat:`, `fix:`, `docs:`, `chore:`), imperative mood, <= 72 chars.
- One change type per commit; add a body when rationale is needed.
- PRs should describe the change, link related issues, and note test results.
## Commit Message Body Guidance
- Use a body only for large or multi-part changes; keep small changes title-only.
- If you are told to include or omit a body, follow the request.
## Commit Title Format and Examples
- Format: `type: short description` (English, imperative, no trailing period).
- Optional scope for clarity: `type(scope): short description`.
- Types used in this repo include `docs`, `feat`, `fix`, `refactor`, `ci`, `chore`, `build`, `style`.
- Examples based on recent commits:
  - `docs: refresh configuration guidance`
  - `feat: apply TeamTalk gender on login`
  - `fix(tg_bot): Use correct callback data for /unsub command`
  - `ci: add check/clippy/fmt`

## Security & Configuration Tips
- Do not log secrets (tokens, chat IDs, DB URLs).
- Config is read from a local file (default `config.toml`); keep real credentials out of the repo.
- Start from `config.toml.example`; required sections are `[teamtalk]`, `[telegram]`, `[database]`, plus `[general]` for defaults and optional `[operational_parameters]` overrides.

## Architecture Overview
- The TeamTalk client runs in a dedicated OS thread (via `tt_worker`) and communicates over channels to avoid blocking the Tokio runtime.
- The Telegram bot uses `teloxide` on Tokio async tasks; the `bridge` module formats events and routes messages/commands.
- Database access is via `sqlx` with a single `Database` struct implemented across `src/db/*` modules.

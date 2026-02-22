# Repository Guidelines

## Project Structure & Module Organization
- `src/core/` contains domain types and callback models.
- `src/app/` hosts application services (use-case orchestration).
- `src/adapters/` contains presentation/transport: Telegram (`src/adapters/tg/`), TeamTalk (`src/adapters/tt/`), and the bridge (`src/adapters/bridge/`).
- `src/infra/` contains infrastructure: SQLx DB (`src/infra/db/`) and localization (`src/infra/locales.rs`).
- `src/bootstrap/` contains configuration types and startup wiring.
- `migrations/` contains SQL schema migrations.
- `locales/` stores Fluent `.ftl` localization files.
- `README.md` covers high-level usage and setup.

## Build, Test, and Development Commands
- `cargo build` builds the project in debug mode.
- `cargo build --release` produces an optimized binary at `target/release/teamtalk-telegram-sender-rs`.
- `cargo run -- <args>` runs the binary locally (see `README.md` for required config).
- `cargo test` runs the test suite.
- `cargo fmt` formats Rust code with rustfmt.
- `cargo clippy --all-targets --all-features -- -D warnings` runs lint checks.
- After changes, run `cargo check`, then `cargo clippy --all-targets --all-features -- -D warnings`, then `cargo fmt`.
- For migrations, the shell does not matter (bash or PowerShell are both fine).
- When creating migration files, verify the timestamp in the filename is correct and current.

## Local Run (Typical)
- Copy `config.toml.example` to `config.toml`.
- Run `cargo run -- --config config.toml`.
## Verification and Delivery Sequence
- Pre-commit runs `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `cargo sqlx prepare`.
- In CI, the workflow runs `cargo check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `cargo fmt --check`.
- After each significant change, run `cargo check` and `cargo test` locally; use `cargo fmt` and `cargo clippy` to match pre-commit/CI.

## Coding Style & Naming Conventions
- Follow rustfmt defaults; keep diffs minimal and avoid formatting churn.
- Use `snake_case` for functions/vars, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- Prefer `Result` with contextual errors over `unwrap` in non-test code.
- Keep modules focused; avoid widening `pub` visibility unless needed.
- Completely avoid adding code comments unless explicitly requested.
- Do not change linting/tooling policy (`clippy` flags, rustfmt toolchain, lefthook, CI lint steps) unless explicitly requested.
- `Cargo.lock` is committed in this repo; avoid manual edits and prefer `cargo update` when needed.
- When asked to commit and push, split commits by type (e.g., docs + code), propose commit messages, and wait for confirmation before pushing. If explicit permission is given to do everything once, proceed; for later push requests, ask again.
- Cargo registry cache lives under `%USERPROFILE%\.cargo\registry\` (e.g., `src` and `cache`). Use it to inspect crate sources (example: find teloxide reply helpers in `teloxide-0.17.0\src\sugar\request.rs`); cache keeps old versions for speed and is safe to read.
- If `pre-commit` fails on `cargo sqlx prepare`, install `sqlx-cli` with `cargo install sqlx-cli --no-default-features --features sqlite`, then run `cargo sqlx prepare` with `DATABASE_URL=sqlite://data.db`.

## Testing Guidelines
- Unit tests live under `tests/unit/` and are wired into modules via `#[path]`.
- Use `cargo test` to run unit and integration tests.
- Prefer deterministic tests; avoid network calls unless required.
- Name tests by intent, e.g., `connect_retries_on_timeout`.

## Commit & Pull Request Guidelines
- Use Conventional Commits (e.g., `feat:`, `fix:`, `docs:`, `chore:`), imperative mood, <= 72 chars.
- One change type per commit; add a body when rationale is needed.
- PRs should describe the change, link related issues, and note test results.
- Commit each context separately. When making a fix, commit it before moving on to other topics or changes.
- After each fix: commit, then test/validate. If the fix fails, revert or amend with a new contextual commit.
- Only ask for confirmation before pushing; committing is expected once changes are ready.
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
- Start from `config.toml.example`; required sections are `[teamtalk]`, `[telegram]`, `[database]`, `[general]`.
- `[plugins]` and `[operational_parameters]` are optional (defaults are applied in code).
- Keep `gender` under `[general]` (not `[teamtalk]`).

## Architecture Overview
- The TeamTalk client runs in a dedicated OS thread (via `tt_worker`) and communicates over channels to avoid blocking the Tokio runtime.
- The Telegram bot uses `teloxide` on Tokio async tasks; the `bridge` module formats events and routes messages/commands.
- Database access is via `sqlx` with a single `Database` struct implemented across `src/infra/db/*` modules.
 - App state/cache is managed by `StateHandle` in `src/app/state/`.

## Plugin System Guidance
- Plugin runtime lives under `src/app/plugins/`.
- Plugin root directory is configured by `[plugins].dir` (default `plugins`).
- Each plugin folder must contain `plugin.toml` and one `entry` Lua file.
- Multi-file plugins should use Lua `require` from the single `entry` file.
- Keep plugin API backward-compatible: additive changes only unless explicitly approved.
- When changing plugin runtime/loader/API, add or update dedicated tests in `tests/unit/app_plugins.rs`.
- Keep `PLUGINS.md` user-facing: usage, API reference, examples, troubleshooting.
- Lifecycle changes must keep rollback behavior: broken reload must keep previous active version.
- Structured plugin logs should include plugin name and lifecycle action (`load`, `reload`, `disable`, `error`).
- Any new Core/TG/TT capability that should be available to plugins must be explicitly wired into plugin API (`src/app/plugins/runtime.rs`) and documented.
- If a Core/TG/TT feature is changed or removed, update plugin docs and tests in the same change; no API drift is allowed.
- Every plugin API extension must include:
  - mapping implementation in runtime/manager,
  - at least one deterministic unit test,
  - `PLUGINS.md` section update,
  - `plugins/example/` update when applicable.
- Prefer stable plugin contracts over exposing internal implementation details directly.
- Keep plugin command behavior deterministic: same input must produce same dispatch order and fallback rules.
- Core plugins shipped in repo must live under `plugins/<plugin_name>/` with `plugin.toml` + entry Lua file.
- If plugin behavior depends on operator values (for example TG chat id), document exact edit points in `PLUGINS.md`.
- For every new shipped core plugin, add a dedicated `PLUGINS.md` section with:
  - feature behavior,
  - required config/manual values,
  - hot-reload verification steps.

## Plugin Parity Policy
- Default rule: if bot supports a Core/TG/TT capability, plugins should support it too.
- Do not defer plugin mapping to a later PR without explicit approval.
- Every Core/TG/TT feature change must include plugin decision in the same change-set:
  - implemented mapping in plugin runtime, or
  - explicit internal exception record in `docs/internal/plugin-parity.md`.
- `PLUGINS.md` must not contain internal process/governance rules.

## Plugin Change Checklist
- For every plugin API change, include all of:
  - runtime/manager mapping updates,
  - deterministic tests (`tests/unit/app_plugins.rs` and related),
  - user docs update in `PLUGINS.md`,
  - example update in `plugins/example/` when behavior is user-visible.
- For every Core/TG/TT feature change:
  - verify plugin parity impact,
  - add mapping or record exception in `docs/internal/plugin-parity.md`,
  - keep CI green (`check`, `clippy`, `test`).

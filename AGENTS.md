# AGENTS.md

## Stack

- Rust 2024 edition, single crate at the repo root (no workspace).
- `axum` 0.8 on `tokio`, REST/JSON under `/api`, SSE at `/api/events`.
- SQLite via `sqlx` 0.9 (WAL mode, migrations embedded with `sqlx::migrate!()`).
- Time: `jiff` with bundled tzdb. Never use `chrono` or the system tz database.
- Frontend: SvelteKit 3 in SPA mode in `web/`, npm. Kit config (adapter-static) is in
  `vite.config.js`, `ssr = false` in `src/routes/+layout.ts`. Routes in `web/src/routes/`,
  everything else in `web/src/lib/`, imported as `#lib/...` with the file extension (`.js` for `.ts`).
- Release binary is static musl, shipped in a `FROM scratch` image.

## Commands

```sh
cargo build
cargo test                         # also exports ts-rs types to web/src/lib/api/
cargo test domain::schedule        # single module; append ::test_name for one test
cargo clippy --all-targets -- -D warnings
cargo fmt
# CI also checks the `demo` feature:
cargo clippy --all-targets --features demo -- -D warnings
cargo test --features demo -- demo clock

npm --prefix web ci
npm --prefix web run dev           # Vite on :5173, proxies /api and /healthz to :8080
npm --prefix web run build         # outputs web/dist, embedded by release builds
npm --prefix web run check         # svelte-kit sync + svelte-check; needs src/lib/paraglide/ from dev or build
npm --prefix web test              # Vitest + Testing Library in jsdom (TZ=Europe/Stockholm, locale en)
```

To run the app, e.g. to check a change in the browser, use `./run-demo.sh`: a backend with
fresh demo data on :8080 and Vite on :5173 (admin password `demo`). Ctrl+C stops both.

sqlx macros check queries against `.sqlx/`. After adding or changing a `query!` macro or a
migration, regenerate it against a migrated dev database (needs `cargo install sqlx-cli`).
The app itself reads `SYSSLOR_DATABASE_URL`, never `DATABASE_URL`.

```sh
DATABASE_URL=sqlite://target/sqlx-dev.db sqlx database setup
DATABASE_URL=sqlite://target/sqlx-dev.db cargo sqlx prepare -- --all-targets
```

## Code style

- Errors: `anyhow::Result` with `.context(...)` in startup/db code. Messages are lowercase
  and include the offending value: `.with_context(|| format!("invalid SYSSLOR_DATABASE_URL {url:?}"))`.
- Unit tests live in a `#[cfg(test)] mod tests` at the bottom of the same file.
- Frontend tests live next to the module as `*.test.ts`; shared fixtures and `fetch` stubs
  are in `web/src/test/fixtures.ts`. Query components by role and label, like a user would.
- Use `let ... else` for early returns instead of nested `match`/`if let`.
- Comment sparsely to explain non-obvious *why*, not what.
- Svelte: runes only (`$state`, `$derived`, `$props`, `$effect`). No `export let`, no
  `$:` statements, no stores for component state.
- Styling: plain CSS with `light-dark()` for colors. No CSS frameworks.
- Colors: Solarized only. Use the semantic tokens (`--fg`, `--surface`, `--accent`, ...) or
  the palette variables (`--base03`..`--base3`, `--blue`, ...) defined in
  `web/src/lib/components/Shell.svelte`. No raw color literals anywhere else, except
  where CSS variables can't reach.
- UI strings: Paraglide messages in `web/messages/{en,sv}.json`, used as `m.key()`. Every
  UI change updates both files; use plural variants instead of `count === 1` checks.

## Architecture rules

- `src/domain/` is pure: no I/O, no `sqlx`, no `axum`, no `Timestamp::now()`. Pass `now`
  and `tz` in as arguments. All scheduling/sorting logic goes here, test-first.
- `src/db/` holds all SQL. Handlers in `src/api/` call `db`, never write SQL themselves.
- Timestamps are stored as UTC unix milliseconds (`INTEGER`). Convert to the configured
  time zone (`SYSSLOR_TIME_ZONE`) only for calendar-day math.
- Due dates are calendar days in the configured time zone, never `now - last >= 24h * n`.
- Names (groups, task tie-breaks) are compared with the `Collator` for `SYSSLOR_LOCALE`
  (the language the family writes names in, not the UI language), never with `str::cmp`.
- Sorting happens in Rust, not in SQL.
- Every committed write must send a `changed` event on the broadcast channel.
- Completion ids are client-generated UUIDv7; `PUT /api/completions/{uuid}` must stay
  idempotent.
- Tasks are soft-deleted via `archived_at`. Never `DELETE` tasks.
- Config comes from env vars only, all prefixed `SYSSLOR_`: `SYSSLOR_DATABASE_URL`,
  `SYSSLOR_BIND_ADDR`, `SYSSLOR_TIME_ZONE`, `SYSSLOR_LOCALE`, `SYSSLOR_ADMIN_PASSWORD`,
  `SYSSLOR_COOKIE_KEY`. No config files.
- The final image is `scratch`: no shell, no `/tmp`, no CA certs, no tzdata. Don't add
  dependencies that need them at runtime (use `rustls`, never `openssl`/`native-tls`).

## Boundaries

- Never commit secrets (`SYSSLOR_ADMIN_PASSWORD`, `SYSSLOR_COOKIE_KEY`, registry credentials)
  or `*.db`, `*.db-wal`, `*.db-shm`.
- Never edit an existing file in `migrations/`. Add a new numbered migration instead.
- Do not hand-edit generated files: `.sqlx/`, `web/src/lib/api/` (ts-rs output),
  `web/src/lib/paraglide/`, `web/.svelte-kit/`, `web/dist/`, `web/static/icons/` (PNGs
  rendered from `web/static/*.svg` by `npm run icons`, which runs before `dev` and `build`),
  `Cargo.lock`/`package-lock.json` (change via cargo/npm), `docs/screenshots/` (rendered by CI).
- Don't hand-edit `CHANGELOG.md`, `.release-please-manifest.json` or the version in
  `Cargo.toml`; release-please owns them, based on conventional commits.
- Ask before adding new dependencies (crates or npm packages).

## Git

- Conventional commits, lowercase, imperative: `feat: add completion endpoint`,
  `fix: ...`, `chore: ...`, `docs: ...`, `test: ...`.
- When changes are complete, run `cargo fmt`, both clippy commands and `cargo test`, plus
  `npm --prefix web run check` and `npm --prefix web test` if `web/` changed. Commit the
  regenerated `web/src/lib/api/`; CI fails if it is out of date.
- Do not commit unless explicitly asked to.

# Implementation Plan: `hub` (Device Identity & Spotify OAuth Relay)

> See the naming note at the top of [SOLUTION_DESIGN.md](./SOLUTION_DESIGN.md) — the service is called `hub`; the folder is still physically `remote-server/` pending a manual rename.

Companion document to [SOLUTION_DESIGN.md](./SOLUTION_DESIGN.md). Breaks the design into ordered, independently shippable work packages across the four affected modules:

- **`hub`** (folder: `remote-server`) — new standalone monolithic service, internally split into a `devices` module and a `spotify_oauth` module (see design decision #11).
- **`build-root`** — device OS image; new boot-time init.d script that generates the device's bearer secret as a plain file.
- **`application/admin_interface/server`** (`admin_interface_server`) — device-side Rust backend.
- **`application/admin_interface/web_ui`** — device-side React/Vite frontend.

Each phase lists concrete files/crates to create or touch, based on existing repo conventions (BusyBox init.d scripts in `build-root/board/tinyghettobox/rootfs-overlay/etc/init.d`, actix-web `#[get]/#[post]` routes registered in `main.rs`, polling hooks in `web_ui`).

## Ordering rationale

Build bottom-up so every phase can compile/test in isolation: device identity file bootstrap → `hub` service skeleton → `hub` OAuth session endpoints → device backend integration → device frontend → cleanup/hardening. This mirrors the request flow in [Section 7](./SOLUTION_DESIGN.md#7-end-to-end-sequence) of the design. There's no shared-crypto-crate phase this time — bearer-token auth (design decision #5) needs nothing beyond a random-token generator and a hash comparison, both trivial enough to write directly in `hub` and `admin_interface_server` without a shared crate.

---

## Phase 1 — Device identity file bootstrap (`build-root`)

**Goal:** Device generates and persists an opaque bearer secret as a plain file at boot, before `admin_interface_server` ever starts — no DB, no sea-orm, no Rust involvement, no asymmetric crypto.

1. **New init.d script** — `build-root/board/tinyghettobox/rootfs-overlay/etc/init.d/S31device-identity`, ordered right after `S30wifi` (WiFi injection) and before `S55admin-interface`, following that script's exact idempotent-by-file-presence style (see [S30wifi](../build-root/board/tinyghettobox/rootfs-overlay/etc/init.d/S30wifi) as the template):
   ```sh
   IDENTITY_DIR="/var/lib/tinyghettobox/device_identity"
   SECRET="$IDENTITY_DIR/device_secret"

   case "$1" in
       start)
           [ -f "$SECRET" ] && exit 0   # already generated, no-op

           mkdir -p "$IDENTITY_DIR"
           openssl rand -base64 32 > "$SECRET" 2>/dev/null || { echo "tinyghettobox: ERROR generating device secret" > /dev/kmsg; exit 1; }
           chown -R tinyghettobox:tinyghettobox "$IDENTITY_DIR"
           chmod 700 "$IDENTITY_DIR"
           chmod 600 "$SECRET"
           sync
           ;;
       stop) ;;
   esac
   ```
2. **Confirm `openssl` CLI (not just `libopenssl`) ends up in the target rootfs** — `BR2_PACKAGE_OPENSSL=y` is already set in [tinyghettobox_defconfig](../build-root/configs/tinyghettobox_defconfig); verify the `openssl` binary itself is installed to `TARGET_DIR` (some minimal configs strip the CLI, only keeping `libssl`/`libcrypto`) — check with `find output/target -name openssl` after a build, add `BR2_PACKAGE_OPENSSL_BIN` or equivalent if it's missing. (`openssl rand` is only used here as a convenient CSPRNG source already present in the image — nothing PKI-related about it.)
3. **No package/Config.in changes needed** — this is a plain file under `rootfs-overlay/etc/init.d/`, copied automatically like `S30wifi`, `S55admin-interface`, etc. Just needs `chmod +x` before packaging (match existing script permissions in that directory).
4. **`registration.json` is deliberately NOT created by this script** — it's written later by `admin_interface_server` once `/register` succeeds (Phase 3), keeping this script's only responsibility "local secret exists."

**Exit criteria:** After a fresh image boot (or `rm -f /var/lib/tinyghettobox/device_identity/device_secret && /etc/init.d/S31device-identity start` on a running device for iteration), `device_secret` exists with mode `600`, owned by `tinyghettobox`, and a second invocation of the script is a no-op.

---

## Phase 2 — `hub` service skeleton: `devices` module (`remote-server`)

**Goal:** Stand up the actix-web service with its own SQLite store and module structure, no Spotify logic yet — just `/register` and bearer-auth plumbing.

1. **Scaffold crate**: `cargo new --bin hub` inside the `remote-server` folder (or convert in place). Dependencies mirror `admin_interface_server` where relevant: `actix-web`, `sea-orm` (`sqlx-sqlite`, `runtime-tokio`, `macros`, `chrono`), `serde`/`serde_json`, `tracing`/`tracing-subscriber`, `tokio`, `uuid` (v4, for `device_id`), `rand`, `sha2` (secret hashing), `base64`, `chrono`.
2. **Module layout** reflecting design decision #11 (module boundaries = future service boundaries):
   ```
   src/
     main.rs           # wires up both modules' routes + shared DB connection
     devices/
       mod.rs           # POST /register, bearer-auth extractor
       model.rs         # sea-orm entity for `devices`
       repository.rs
     spotify_oauth/
       mod.rs           # (Phase 3) /spotify/client_id, /oauth/start, /oauth/callback, /oauth/check
       model.rs
       repository.rs
   ```
3. **Schema** (own SQLite DB file; use `sea-orm-migration` for consistency with `application/migration`):
   - `devices(id TEXT PK, secret_hash TEXT, created_at TEXT, revoked_at TEXT NULL)`
4. **`POST /register`** handler (in `devices` module):
   - Body: `{ device_secret: String }`.
   - Hash the secret (`sha2::Sha256`), generate `device_id` (`uuid::Uuid::new_v4()`), insert row, return `{ device_id, issued_at }`.
   - Rate limiting per source IP: use `actix-governor` (or a minimal in-memory token-bucket keyed by `peer_addr()`/`X-Forwarded-For`) at 5/hour as a starting point per design Section 10.2.
5. **Bearer-auth extractor** (in `devices` module, reused by `spotify_oauth`): an actix `FromRequest` that parses `Authorization: Bearer <device_id>.<secret>`, looks up `devices.secret_hash` by `device_id`, hashes the provided secret and compares (constant-time compare via e.g. `subtle::ConstantTimeEq` or a simple `==` on hashes — hashes are already fixed-length and not human-guessable, so timing leakage risk here is low, but constant-time compare is cheap insurance), checks `revoked_at IS NULL`, and rejects with `401` on any failure. Applied to `/oauth/start` and `/oauth/check` only (`/register`, `/oauth/callback`, `/spotify/client_id` stay unauthenticated per design).

**Exit criteria:** `POST /register` works end-to-end against a real SQLite file; rate limiting and the bearer-auth extractor have unit/integration tests (actix `test::call_service`), including a rejected request with a wrong secret and one with a revoked device.

---

## Phase 3 — `hub` service: `spotify_oauth` module (`remote-server`)

**Goal:** Complete the Spotify-relay endpoints per design Section 5.4, plus the new shared-`client_id` endpoint.

1. **`GET /spotify/client_id`** (public, no auth): returns `{ client_id }` for the shared TinyGhettoBox Spotify app, read from `hub`'s own config (env var, e.g. `SHARED_SPOTIFY_CLIENT_ID`).
2. **`POST /oauth/start`** (bearer-authenticated): body `{ code_challenge, code_challenge_method }`; generate high-entropy `state` (32 random bytes, base64url via `rand` + `base64::URL_SAFE_NO_PAD`), insert `oauth_sessions` row with `status=pending`, `expires_at = now + 10min`; return `{ state, expires_at }`.
3. **`GET /oauth/callback?code&state`** (public): look up session by `state`.
   - Not found / expired / already `ready`/`consumed` → render a static "link expired, please retry" HTML page (plain `HttpResponse::Ok().content_type("text/html").body(...)`, no templating engine needed for this small a page).
   - Otherwise: set `auth_code = code`, `status = ready`; render "you can close this window" HTML page.
4. **`POST /oauth/check`** (bearer-authenticated): body `{ state }`.
   - Look up session; verify `session.device_id == authenticated_device_id` (defense in depth per design decision #5) → else `403`.
   - `ready` → return `{ status: "ready", auth_code }`, set `consumed_at = now`, delete row immediately (one-time retrieval).
   - `pending` → `{ status: "pending" }`.
   - expired/consumed/missing → `{ status: "expired" }`.
5. **Background cleanup task**: `tokio::spawn` a loop (e.g. every 60s) deleting `oauth_sessions` rows past `expires_at` regardless of status (consumed rows are already deleted eagerly in step 4; this mainly catches abandoned `pending`/`ready`-but-never-polled sessions).

**Exit criteria:** Full `hub`-side flow testable with `curl`/integration tests: start → callback (simulate Spotify redirect) → check returns `ready` once, `expired` on second call; `/spotify/client_id` reachable without any auth header.

---

## Phase 4 — Device backend integration (`admin_interface_server`)

**Goal:** Replace today's `GET /api/spotify/auth` + `/api/spotify/auth/callback` (in [routes/spotify.rs](../application/admin_interface/server/src/routes/spotify.rs)) with the `hub`-driven flow, and add the shared-vs-custom Spotify app toggle.

1. **Config**: add `hub` base URL as a compile-time/env constant (e.g. `HUB_URL` env var, defaulted to the production `hub` HTTPS URL) — needed by both the register bootstrap and the new auth routes.
2. **`hub` HTTP client module** (`admin_interface_server/src/hub_client.rs`): thin wrapper using the existing `ureq` dependency (already used for `spotify_client.rs`-adjacent code) to call `/register`, `/spotify/client_id`, `/oauth/start`, `/oauth/check`, attaching the `Authorization: Bearer <device_id>.<secret>` header for the latter two.
3. **Device identity bootstrap**: on server startup (in `main.rs`, after `connect()`), call a new `ensure_device_identity()` helper that reads from `/var/lib/tinyghettobox/device_identity/` (path also overridable via env var for local dev, since this directory won't exist off-device):
   - Read `device_secret` as a plain string — missing file is a hard startup error (the `S31device-identity` boot script guarantees it exists on real hardware; a clear panic/log message here saves debugging time in local dev where the script hasn't run).
   - Read (or create, if absent) `registration.json` (`{ device_id: Option<String>, registered_at: Option<String> }`) from the same directory.
   - If `device_id.is_none()`, call `hub`'s `/register` and write the result back to `registration.json` (atomic write: temp file + rename, since this file may be read concurrently by request handlers). Retry lazily (don't block server startup indefinitely; log a warning and continue — Spotify auth simply won't work until registration succeeds, retried next time `/api/spotify/auth/start` is hit).
4. **Shared vs. custom `client_id` resolution**: a small helper used by the new auth routes — if `spotify_config.client_id` (and `secret_key`) are set *and* the (new) `spotify_config.use_custom_app` flag is true, use those; otherwise fetch/cache `hub`'s `GET /spotify/client_id` and use no secret. Requires a small migration on the existing `application/database`/`application/migration` crates: add `use_custom_app BOOLEAN NOT NULL DEFAULT false` to `spotify_config` (follow `m20260610_000002_add_ap_password.rs` as a template).
5. **New route file additions** in `routes/spotify.rs` (or split into `routes/spotify_auth.rs` if it keeps the file more readable — check current file length first):
   - `POST /api/spotify/auth/start`: generate PKCE `code_verifier` (random 43-128 char string) + `code_challenge` (`base64url(sha256(code_verifier))`); resolve the active `client_id` (step 4); call `hub`'s `/oauth/start` (bearer-authenticated); store `{state -> code_verifier}` — an in-memory `Arc<Mutex<HashMap<String, (String /*verifier*/, Instant)>>>` registered as `app_data`, pruned by TTL, is simplest and matches the existing `wifi_state` shared-state pattern in `main.rs`; build and return the Spotify authorize URL (`client_id` + `redirect_uri=<hub>/oauth/callback` + `state` + PKCE `code_challenge` + scopes, same scopes list as today's `auth()`).
   - `GET /api/spotify/auth/status?state=...`: look up `code_verifier` for `state` in the map; call `hub`'s `/oauth/check` (bearer-authenticated). On `ready`: exchange `auth_code` + `code_verifier` with Spotify directly via `rspotify`'s PKCE-capable client (confirm the exact `rspotify` 0.13 API — likely `AuthCodePkceSpotify`; no `client_secret` needed for the shared-app path, only supplied when `use_custom_app` is set), persist tokens into `spotify_config` (same fields `access_token`/`refresh_token`/`expired_at` as existing `callback()`), remove the map entry, return `{ status: "connected" }`. On `pending`: return `{ status: "pending" }`. On `expired`/`error`: return that status, remove map entry.
6. **Route registration**: update `main.rs` `App::new()` service list — remove `spotify::auth` / `spotify::callback` outright, add `spotify::start_auth` / `spotify::auth_status` (single-deployment hobby project, no need for a staged feature-flagged rollout).
7. **`rspotify` PKCE client**: switch the whole Spotify integration to `rspotify::AuthCodePkceSpotify` (rather than `AuthCodeSpotify` + hand-building the URL) so both the shared-app (no secret) and custom-app (with secret, still using PKCE per decision #10) paths go through one supported code path — pass `Credentials::new_pkce(&client_id)` for the shared path vs. `Credentials::new(&client_id, &secret)` for the custom path.

**Exit criteria:** Manual end-to-end test against a locally-run `hub` instance (point `HUB_URL` at `http://localhost:<port>` for dev, accepting the local-HTTP exception since Spotify's HTTPS requirement only applies to the real redirect URI registered in the Spotify dashboard) completes authorization and stores working tokens, for both the shared-app and custom-app paths.

---

## Phase 5 — Device frontend (`admin_interface/web_ui`)

**Goal:** Update the two existing "Authorize with Spotify" entry points to use the popup+polling flow instead of a full-page redirect, and add the "Custom Spotify Developer App" toggle.

Files to change (found via existing `window.location.href = '/api/spotify/auth'` usages):
- [Authorize.tsx](../application/admin_interface/web_ui/src/pages/SpotifyConfig/steps/Authorize.tsx)
- [SpotifyAuthorize.tsx](../application/admin_interface/web_ui/src/pages/Setup/steps/SpotifyAuthorize.tsx)
- [SpotifyCredentials.tsx](../application/admin_interface/web_ui/src/pages/Setup/steps/SpotifyCredentials.tsx) — gains the checkbox.

1. Replace `window.location.href = '/api/spotify/auth'` with:
   - `POST /api/spotify/auth/start` → get `{ spotify_authorize_url }` (route addition also needs to return this field, not just redirect).
   - `window.open(spotify_authorize_url, 'spotify-auth', '<popup dimensions>')`.
2. Add polling (mirror [useFolderSyncStatuses.ts](../application/admin_interface/web_ui/src/pages/MediaLibrary/useFolderSyncStatuses.ts)'s `setInterval`/cleanup pattern, or the inline `poll()` pattern in [WifiSetup.tsx](../application/admin_interface/web_ui/src/pages/Setup/steps/WifiSetup.tsx)): every ~2s call `GET /api/spotify/auth/status?state=...` until `connected` (close popup if still open, advance wizard step / update config page state), `expired` (show retry message), or `error`.
3. Handle the case where the user closes the popup manually without completing auth — stop polling after a max attempt count or on an explicit "cancel" action, to avoid an orphaned interval.
4. **"Custom Spotify Developer App" checkbox** (MUI `Checkbox` + react-hook-form `Controller`, matching the existing pattern in [SystemSettings.tsx](../application/admin_interface/web_ui/src/pages/SystemConfig/SystemSettings.tsx)): unchecked by default, bound to the new `spotify_config.use_custom_app` field. Unchecked hides the `client_id`/`client_secret` inputs entirely; checked reveals them (today's existing fields/behavior), saved via the existing `PUT /api/spotify/config`.
5. Update [SpotifyCredentials.tsx](../application/admin_interface/web_ui/src/pages/Setup/steps/SpotifyCredentials.tsx)'s help text (`http://tinyghettobox.local/api/spotify/auth/callback`) — the redirect URI shown (only relevant when the checkbox is checked, since only then does the user need to configure their own Spotify app) must now be `hub`'s fixed HTTPS callback URL, not the device's own address.

**Exit criteria:** Manual browser test of both the Setup wizard and the standalone SpotifyConfig page's authorize step, confirming popup closes and UI reflects `connected` without a full page navigation, for both the default (shared app, checkbox unchecked) and custom-app (checkbox checked) paths.

---

## Phase 6 — Deployment & hardening

Tracks the "Open Items" from design Section 10 that aren't strictly code:

1. **Domain + TLS for `hub`**: provision a domain, put Caddy or nginx + Let's Encrypt in front of `hub` (Caddy is simplest — auto-TLS with a `Caddyfile` reverse-proxying to the actix-web port). Add a `remote-server/Dockerfile` + deployment doc if the maintainer's hosting is container-based (check how `application/Dockerfile` is structured and mirror conventions if useful).
2. **Register the shared Spotify app and submit for Extended Quota Mode** — manual, non-technical, can be started early since it's on Spotify's review timeline, not this project's.
3. **Tune rate limits** for `/register`, `/oauth/start`, `/oauth/check` after observing real traffic (starting points already in Phase 2/3).
4. **Decide + implement** removal of the old `/api/spotify/auth` + `/api/spotify/auth/callback` routes (per Phase 4 step 6 — already planned as outright removal, not a flag).
5. **Smoke test the factory-provisioning migration path** (design Section 8) isn't blocked: confirm `/register`'s request/response shape has no device-generated-secret-specific assumptions baked into `hub`'s handler beyond "a secret comes in, a device_id goes out" — no code change needed now, just a design review checkpoint.
6. **Rename the `remote-server` folder to `hub`** once a filesystem-level rename is possible (attempted during design and blocked by a permission error in this environment — retry with elevated/different tooling, or do it manually) — update this plan's and the design doc's folder references if the physical location changes.

---

## Summary checklist by module

| Module | Key deliverables |
|---|---|
| `build-root` | `S31device-identity` init.d script generating a random `device_secret` file at boot, permissioned for the `tinyghettobox` user |
| `hub` (new service, folder `remote-server`, rename pending) | actix-web monolith with `devices` + `spotify_oauth` modules; `devices`/`oauth_sessions` tables; `/register`, `/spotify/client_id`, `/oauth/start`, `/oauth/callback`, `/oauth/check`; rate limiting; bearer-auth extractor; expiry cleanup job |
| `admin_interface_server` | `hub` HTTP client, device identity file read + `hub` registration on startup, shared-vs-custom `client_id` resolution, `/api/spotify/auth/start` + `/api/spotify/auth/status`, PKCE token exchange via `AuthCodePkceSpotify`, remove old auth routes |
| `application/database` + `application/migration` | small migration adding `spotify_config.use_custom_app` |
| `admin_interface/web_ui` | popup + polling flow in `Authorize.tsx` / `SpotifyAuthorize.tsx`, new "Custom Spotify Developer App" checkbox in `SpotifyCredentials.tsx`, updated redirect-URI help text |
| Deployment | `hub` TLS/hosting, Spotify Extended Quota Mode application, rate-limit tuning, folder rename |


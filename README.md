# Flatline

A self-hosted uptime monitor. Single binary, no external dependencies.

Flatline monitors your HTTP endpoints, tracks incidents, sends notifications when things go down, and serves a web UI — all from one Rust binary with an embedded SQLite database.

## Features

- **HTTP monitoring** — configurable intervals, timeouts, retries, expected status codes, and HTTP methods
- **Incident tracking** — automatically opens an incident when a monitor goes down, resolves it on recovery
- **Uptime and latency stats** — uptime percentage and P99 latency over 7, 30, and 90-day windows
- **Notifications** — Slack, Discord, Telegram, and arbitrary HTTP webhooks, configurable per monitor
- **OIDC authentication** — works with Google, Keycloak, Authentik, Dex, or any OIDC provider
- **Embedded web UI** — Yew/WASM frontend compiled into the binary, no separate asset server needed
- **OpenAPI docs** — Swagger UI at `/docs`


![Dashboard](/.github/screenshots/dashboard.png)
![Monitor Details](/.github/screenshots/monitor-detail.png)


## Quick Start

### 1. Build the frontend

```bash
cd crates/frontend
trunk build --release
cd ../..
```

### 2. Build the server

```bash
cargo build --release -p server
```

> Steps 1 and 2 are wrapped by `make build` (or `task build`), which builds the
> frontend and then the server in the correct order.

### 3. Configure environment

Create a `.env` file:

```env
DATABASE_URL=sqlite:flatline.db
OAUTH_ISSUER_URL=https://your-oidc-provider.example.com
OAUTH_CLIENT_ID=your-client-id
OAUTH_CLIENT_SECRET=your-client-secret
OAUTH_REDIRECT_URL=http://localhost:3000/auth/callback
JWT_SECRET=a-long-random-secret
```

Generate a JWT secret:

```bash
openssl rand -hex 32
```

### 4. Run

```bash
./target/release/server
```

The server starts on port 3000. Visit `http://localhost:3000` to log in with your OIDC provider. The first user to log in becomes the owner; subsequent logins from different accounts are rejected.

## Configuration

All configuration is via environment variables (or a `.env` file):

| Variable                        | Required | Default              | Description                                        |
|---------------------------------|----------|----------------------|----------------------------------------------------|
| `DATABASE_URL`                  | No       | `sqlite:flatline.db` | SQLite connection string                           |
| `OAUTH_ISSUER_URL`              | Yes      |                      | OIDC provider discovery URL                        |
| `OAUTH_CLIENT_ID`               | Yes      |                      | OIDC client ID                                     |
| `OAUTH_CLIENT_SECRET`           | Yes      |                      | OIDC client secret                                 |
| `OAUTH_REDIRECT_URL`            | Yes      |                      | Callback URL (must match provider config)          |
| `JWT_SECRET`                    | Yes      |                      | Secret for signing session cookies                 |
| `SWEEP_INTERVAL_SECONDS`        | No       | `60`                 | How often old monitor checks are swept             |
| `MONITOR_CHECKS_RETENTION_DAYS` | No       | `90`                 | Monitor checks older than this are deleted          |

The listen address defaults to `0.0.0.0:3000` and can be overridden with the `-a`/`--address` flag (e.g. `./target/release/server --address 127.0.0.1:8080`).

## Authentication

Flatline uses OIDC for login. Configure your provider with:

- **Redirect URI:** `http://your-host:3000/auth/callback`
- **Grant type:** Authorization Code

The login flow: `/auth/login` redirects to your OIDC provider, which redirects back to `/auth/callback`. A signed JWT session cookie is issued (HttpOnly, 30-day expiry). All API and UI routes require a valid session.

**First-user registration:** On first login, the authenticated user is stored as the owner. Any subsequent OIDC login from a different account is rejected with 403.

## Notification Channels

Channels are linked to monitors independently, so you can send different monitors to different channels.

### Slack

Posts a message to a Slack incoming webhook URL.

```json
{
  "name": "My Slack channel",
  "config": {
    "type": "slack",
    "url": "https://hooks.slack.com/services/..."
  }
}
```

### Discord

Posts a rich embed to a Discord channel via an incoming webhook URL.

```json
{
  "name": "My Discord channel",
  "config": {
    "type": "discord",
    "url": "https://discord.com/api/webhooks/..."
  }
}
```

### Telegram

Sends a message to a Telegram chat via a bot. Create a bot with [@BotFather](https://t.me/BotFather) to get the token.

```json
{
  "name": "My Telegram chat",
  "config": {
    "type": "telegram",
    "url": "https://api.telegram.org/bot<token>/sendMessage",
    "chat_id": "123456789"
  }
}
```

### Webhook

Sends an HTTP POST with a JSON payload to any URL.

```json
{
  "name": "My webhook",
  "config": {
    "type": "webhook",
    "url": "https://your-service.example.com/hook"
  }
}
```

## API

All endpoints are documented in the Swagger UI at `/docs`.

**Auth routes** (no authentication required):

```
GET  /auth/login       Redirect to OIDC provider
GET  /auth/callback    Handle OIDC callback, issue session cookie
POST /auth/logout      Clear session cookie
GET  /auth/me          Return current user (requires session)
```

**Protected routes** (require session cookie):

```
POST   /api/v1/monitors
GET    /api/v1/monitors
GET    /api/v1/monitors/:id
PATCH  /api/v1/monitors/:id
DELETE /api/v1/monitors/:id
GET    /api/v1/monitors/:id/checks
GET    /api/v1/monitors/:id/incidents
POST   /api/v1/monitors/:id/notifications
GET    /api/v1/monitors/:id/notifications
DELETE /api/v1/monitors/:id/notifications/:channel_id

POST   /api/v1/notification-channels
GET    /api/v1/notification-channels
GET    /api/v1/notification-channels/:id
PATCH  /api/v1/notification-channels/:id
DELETE /api/v1/notification-channels/:id

GET    /api/v1/stats/stream    SSE stream of MonitorStats for all monitors
```

## Development

```bash
# Check (no build)
cargo check

# Run tests
cargo test

# Run tests for a specific crate
cargo test -p server

# Lint
cargo clippy

# Run server (requires .env)
cargo run -p server

# Run a test HTTP server (for manual monitor testing)
cd testserver && go run main.go -port 1313
```

### Database migrations

```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features sqlite

# Create a new migration
sqlx migrate add <name>
# Edit the generated file in migrations/, then restart the server to apply it
```

Migrations run automatically at startup.

### Frontend development

```bash
cd crates/frontend
trunk serve
```

This runs the frontend dev server with hot reload. Point your browser at `http://localhost:8080` (the API is expected at `http://localhost:3000`).

## Architecture

Rust workspace with three crates:

- **`crates/server`** — Axum HTTP server, monitor engine, SQLite persistence, OIDC auth
- **`crates/shared`** — Serde types shared between server and frontend
- **`crates/frontend`** — Yew/WASM SPA, embedded into the server binary at compile time

The monitor engine spawns one tokio task per enabled monitor. Each task polls the endpoint at the configured interval, persists the result, detects up/down transitions, and fires notifications. Workers are managed via `CancellationToken` and can be started, stopped, or restarted at runtime through the API.

The frontend is compiled to WASM by [Trunk](https://trunkrs.dev/) and embedded via [`rust-embed`](https://github.com/pyros2097/rust-embed). The server serves it as static files with an `index.html` fallback for SPA routing — no separate web server or CDN needed.

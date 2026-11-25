# ovh-vps-watcher

Checks OVHcloud VPS stock (VPS-2, VPS-3 and VPS-4) in European datacenters every minute. When a model comes back in stock, it sends a Discord webhook message saying where.

## Configuration

Set these environment variables:

| Variable | Default | Description |
|---|---|---|
| `DISCORD_WEBHOOK_URL` | - (required) | Discord webhook that receives the alerts |
| `CHECK_INTERVAL_SECONDS` | `60` | Seconds between checks |
| `MODELS` | `2,3,4` | VPS models to watch (`1` to `6`) |
| `OVH_SUBSIDIARY` | `FR` | OVH subsidiary used for the API call |
| `SEND_STARTUP_MESSAGE` | `true` | Set to `false` to skip the message sent at startup |

In `src/ovh.rs`, `Datacenter::is_european` decides which datacenters are watched and `Datacenter::location` sets the names shown in Discord. The Discord messages are written in `src/watcher.rs`.

## Deploy (Dokploy)

1. Create an **Application** from this repo, with build type **Dockerfile**. A **Compose** project using `docker-compose.yml` also works.
2. Add `DISCORD_WEBHOOK_URL` in the **Environment** tab.
3. Deploy. It needs no port and no domain.

## Run locally

```sh
DISCORD_WEBHOOK_URL=https://discord.com/api/webhooks/... cargo run --release
```

Log level can be changed with `RUST_LOG` (default `info`).

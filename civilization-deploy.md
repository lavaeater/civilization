# Civilization web deployment plan

Goal: get `adv_civ` (the Advanced Civilization game) reachable on the public web,
served through `pingora-docker`, with a GitHub-tag push triggering an automatic
rebuild/redeploy. This is a planning document — civilization is still under active
development, so treat each phase as something to land incrementally as the
multiplayer work matures (see `docs/multiplayer.md`).

## Status (2026-08-29): phases 1–3 done, verified locally

Everything below through "Phased plan" step 3 is implemented and was verified with
real `docker build`/`docker run` on this machine (isolated test network, not the live
`proxy-network` — nothing here has touched the running home-server stack):

- `Dockerfile` now has `server` and `web` targets (`server` stays the default target,
  so a plain `docker build -t x .` still behaves like the old single-stage file).
  Both build successfully; `docker build --target server` produces the same
  `adv_civ_server` image as before, `docker build --target web` produces a `caddy:2`
  image with the trunk-built wasm client baked in.
- `deploy/Caddyfile.internal` added (sibling to `deploy/Caddyfile`, same path
  routing, no TLS, upstreams point at `civilization-server` by container name).
- Two bugs found and fixed while actually building the `web` target (the plan below
  didn't anticipate either — both `.dockerignore` gaps that only bit the new target):
  - `.dockerignore` excluded `build/` entirely, but `index.html`'s trunk directives
    copy several files straight out of it (`build/windows/icon.ico`,
    `build/web/manifest.webmanifest`, PWA icons, `styles.css`, `sound.js`) — un-excluded
    it, it's ~1MB of tracked assets, no reason to hide it from Docker.
  - `.dockerignore` excludes `.cargo/` (correctly, to keep the mold-linker rustflags
    off a container that doesn't have mold) but that also dropped
    `.cargo/config.toml`'s `[target.wasm32-unknown-unknown]` rustflags, which
    `getrandom` needs to compile for wasm at all. Fixed by writing just that one
    target's config back inside the `web-builder` stage rather than un-ignoring the
    whole file.
- End-to-end request routing verified: built both images, ran them on an isolated
  Docker network (container named `civilization-server` to match what
  `Caddyfile.internal` expects), confirmed `GET /api/health` and `POST /api/join`
  both route correctly through the internal Caddy to the game server and back,
  including the session-token response from the multiplayer-hardening work above.
- `pingora-docker` changes (that repo's own commit): `docker-compose.yml` gained
  `civilization-server` + `civilization-web` services; `config.json` gained the
  `civ.kidvhs.com` domain entry; `webhooks/services.json` gained a `civilization`
  entry with a new `compose_services` array (first repo needing more than one
  compose service per tag push); `webhooks/rebuild.sh` and `init-services.sh` both
  gained `git submodule update --init --recursive` after checkout (needed for
  `lava_ui_builder`) and now loop over `compose_services` instead of assuming
  one repo = one container. Validated with `docker compose config` (syntax only,
  did not touch the running stack) plus `jq`/`python -m json.tool`/`bash -n` on the
  JSON and shell changes.

**Not done yet** (phased-plan steps 4–5, and the "open questions" section below is
still fully open): the actual `docker compose up` rollout on the live host, the
GitHub webhook registration on the `lavaeater/civilization` repo, and all of the
production config decisions (real domain confirmation, a persisted `NETCODE_KEY`,
the saves-volume host path, `SEATS`/`NUM_PLAYERS` defaults). Deliberately left for
you to do — spinning up new containers or registering a webhook on your live
infrastructure isn't something to do unprompted.

## Original plan (as of 2026-06-12)

- `Dockerfile` (repo root) builds **only the headless server** (`adv_civ_server`,
  `--profile dist`) into a `debian:bookworm-slim` image. Exposes 5111 (game WS) and
  5112 (HTTP join API).
- `docker-compose.yml` + `deploy/Caddyfile` run that server behind a **Caddy**
  container which terminates TLS, serves the wasm client from `./dist` (built
  manually via `trunk build --release`), and path-routes `/api/*` → :5112,
  `/ws` → :5111, everything else → static files.
- This compose setup is **self-contained and TLS-terminating** — designed to run
  standalone on its own port/domain, independent of any other infrastructure.
- `dist/` is currently a build artifact checked into the working tree (not
  committed — see `.gitignore`/`.dockerignore`), produced by `trunk build --release`.
  ~33 MB wasm.

## Target architecture

`civilization` joins the home server as another domain behind
`pingora-docker`'s reverse proxy, the same way `rusty-budgets` and `oxidize-books`
already do — pingora terminates TLS for `*.kidvhs.com` (wildcard CloudFlare Origin
cert already present in `pingora-docker/certs/`), so **no new cert work is needed**.

Pingora's `DomainRouter` only does *domain → single host:port*, with no path-based
routing (see `pingora-docker/src/proxy.rs`). The current Caddy front does
path-based routing (`/`, `/api/*`, `/ws`) which pingora can't replicate. So we keep
a small **internal** Caddy as the per-domain front, just strip its TLS
responsibilities (pingora handles that) and point it at civilization's server
container by Docker service name instead of `game`.

```
Internet
  │ HTTPS (civ.kidvhs.com)
  ▼
pingora (TLS termination, *.kidvhs.com)
  │ plain HTTP, Host: civ.kidvhs.com
  ▼
civilization-web  (Caddy, no TLS — path router + static dist/)
  ├── /api/*  → civilization-server:5112
  ├── /ws     → civilization-server:5111
  └── /       → static wasm client (dist/, baked into this image)
  │
  ▼
civilization-server (adv_civ_server headless game)
```

Two new images come from the `civilization` repo's `Dockerfile`, as two build
**targets**:

- `server` (existing default target, basically unchanged) → `adv_civ_server`
  binary + assets.
- `web` (new target) → `trunk build --release` output baked into a `caddy:2`
  (or similar) image alongside a non-TLS `Caddyfile` (derived from
  `deploy/Caddyfile`, with TLS directives removed and `game:5111/5112` rewritten
  to `civilization-server:5111/5112`).

Both run as services in `pingora-docker/docker-compose.yml` on `proxy-network`,
addressed by container name — matching how `rusty-budgets` etc. are wired up.

## Changes needed in `bevy/civilization`

1. **Extend `Dockerfile` with a `web` build stage:**
   - Reuse the existing `builder` stage (or a sibling stage) to also run
     `trunk build --release` (needs `wasm32-unknown-unknown` target + `trunk`
     installed in the builder image — not currently present).
   - Final `web` stage: `FROM caddy:2`, `COPY` the `dist/` output to `/srv`,
     `COPY` a new non-TLS Caddyfile (e.g. `deploy/Caddyfile.internal`) to
     `/etc/caddy/Caddyfile`.
   - `docker build --target server -t civilization-server .` and
     `docker build --target web -t civilization-web .` should both work from one
     Dockerfile.

2. **New `deploy/Caddyfile.internal`** (sibling to the existing TLS
   `deploy/Caddyfile`, used for local `docker-compose.yml`):
   - Same path routing (`/api/*`, `/ws`, static `/`) but `:80`, no TLS/ACME
     directives, and upstreams point at `civilization-server:5111` /
     `civilization-server:5112` (the pingora-docker container name) instead of
     `game:5111/5112`.

3. **Submodule**: `lava_ui_builder` is a git submodule
   (`.gitmodules` → `git@github.com:lavaeater/lava_ui_builder.git`), and is in the
   `[workspace]` members. A plain `git clone`/`git checkout <tag>` (which is what
   `pingora-docker`'s webhook does — see below) **will not populate it**. Either:
   - have the webhook's clone/checkout step run
     `git submodule update --init --recursive` (small fix in
     `pingora-docker`, see below), or
   - vendor `lava_ui_builder` into the workspace directly if it's stable enough,
     removing the submodule dependency for deploy purposes.
   Recommend the first option — least disruptive.

4. **Production config knobs** (already mostly plumbed via `docker-compose.yml`
   env vars — `CIV_DOMAIN`, `PUBLIC_ADDR`, `PUBLIC_WS`, `NETCODE_KEY`,
   `SEATS`, `NUM_PLAYERS`): when run under pingora-docker, `CIV_DOMAIN` becomes
   the chosen subdomain (e.g. `civ.kidvhs.com`), `PUBLIC_ADDR`/`PUBLIC_WS` need
   `:443` + `wss://civ.kidvhs.com/ws` accordingly. `NETCODE_KEY` should be a
   real persisted secret (currently defaults to `random`, which would invalidate
   tokens across restarts — fine for dev, not for a long-running public game).

5. **Saves volume**: `adv_civ_server` writes to `/app/saves` (already a
   `VOLUME` in the Dockerfile). Map this to a host path under
   `pingora-docker`'s compose (pattern matches `rusty-budgets`'s `data.json`
   bind mount) so games survive container rebuilds — important since rebuilds
   now happen automatically on every tag push.

## Changes needed in `dioxus/pingora-docker`

1. **`docker-compose.yml`** — add two services, both on `proxy-network`:

   ```yaml
   civilization-server:
     build:
       context: ./repos/civilization
       dockerfile: Dockerfile
       target: server
     container_name: civilization-server
     environment:
       - SEATS=${CIV_SEATS:-0}
       - NUM_PLAYERS=${CIV_NUM_PLAYERS:-5}
       - NETCODE_KEY=${CIV_NETCODE_KEY}
       - PUBLIC_ADDR=civ.kidvhs.com:443
       - PUBLIC_WS=wss://civ.kidvhs.com/ws
     volumes:
       - /home/tommie/projects/civilization-saves:/app/saves
     restart: unless-stopped
     networks:
       - proxy-network

   civilization-web:
     build:
       context: ./repos/civilization
       dockerfile: Dockerfile
       target: web
     container_name: civilization-web
     depends_on:
       - civilization-server
     restart: unless-stopped
     networks:
       - proxy-network
   ```

2. **`config.json`** — add a domain entry routing to the front container:

   ```json
   "civ.kidvhs.com": {
       "host": "civilization-web",
       "port": 80,
       "tls": false
   }
   ```

3. **`webhooks/services.json`** — add civilization, and (since this is the first
   service that maps to *two* compose services) introduce an optional
   `compose_services` array:

   ```json
   "civilization": {
     "repo": "lavaeater/civilization",
     "url": "git@github.com:lavaeater/civilization.git",
     "compose_services": ["civilization-server", "civilization-web"]
   }
   ```

4. **`webhooks/rebuild.sh`** — two small changes:
   - After clone/checkout, run `git submodule update --init --recursive`
     (harmless no-op for services without submodules).
   - Loop over `compose_services` if present, else fall back to `[SERVICE_NAME]`,
     calling `docker compose ... up --no-deps --build -d` for each, so both
     `civilization-server` and `civilization-web` get rebuilt from the same tag
     checkout. (`civilization-web` should be rebuilt even though its image
     mostly just changed `dist/`, since the trunk output hash changes per build.)

5. **`webhooks/init-services.sh`** — same submodule fix for the initial
   clone path, and same `compose_services` loop for the `--build` pass.

6. **GitHub webhook** on `lavaeater/civilization`:
   - Settings → Webhooks → Add webhook
   - Payload URL: `https://webhooks.kidvhs.com/hooks/rebuild-service`
   - Secret: same `WEBHOOK_SECRET` already configured in `pingora-docker/.env`
   - Events: "Just the push event" (the hook already filters to `refs/tags/*`,
     matching the existing `release.yaml` tag pattern `v[0-9]+.[0-9]+.[0-9]+*`)

7. **`.ssh` keys** mounted into the `webhook` container
   (`~/.ssh/id_ed25519`) already give it access to private repos — civilization
   and its `lava_ui_builder` submodule are both private-capable via the same key,
   no extra setup needed there.

## Hosting question — is pingora-docker enough?

Yes, for the near term. The "simple hosting solution for web distribution" *is*
`civilization-web` (Caddy serving `dist/`) behind pingora — no separate static
host (S3/Cloudflare Pages/GitHub Pages) is needed, and keeping it on the same
home-server stack means:

- one TLS cert, one place to manage domains/services,
- the static client and the game server are redeployed atomically from the same
  tag (avoids client/server protocol drift, which matters a lot given
  `docs/multiplayer.md`'s move-protocol is still evolving),
- the existing `deploy-page.yaml` GitHub Pages workflow can stay as-is for a
  "stable demo" mirror if ever wanted, but isn't required for the primary
  deployment.

Revisit only if home-server bandwidth/uptime becomes a problem for a ~33 MB wasm
download per visitor — at that point a CDN in front of `civ.kidvhs.com` (e.g.
Cloudflare proxying, which the `*.kidvhs.com` Origin cert already implies) is the
likely next step, not a different hosting provider.

## Phased plan

1. **Dockerfile `web` target** + `deploy/Caddyfile.internal` in `civilization`.
   Verify locally: `docker build --target web -t civ-web .` and
   `docker build --target server -t civ-server .`, run both with a manual
   `docker network` to confirm the path routing works end to end.
2. **Submodule fix** in `pingora-docker`'s `rebuild.sh`/`init-services.sh`
   (small, low-risk, unblocks everything else).
3. **Wire into `pingora-docker`**: compose services, `config.json` domain,
   `services.json` entry with `compose_services`. Manually run
   `docker compose up --no-deps --build -d civilization-server civilization-web`
   once to prove the images build from `./repos/civilization` and the domain
   routes correctly.
4. **GitHub webhook** on the civilization repo; push a test tag (e.g.
   `v0.0.0-deploy-test`) and confirm `webhooks/rebuild.sh` clones, checks out
   the tag, builds, and restarts both containers, with the email notification
   firing.
5. **Production config**: pick `CIV_DOMAIN`, generate and persist a real
   `NETCODE_KEY`, set up the saves volume, decide `SEATS`/`NUM_PLAYERS` defaults
   for a "always-on AI game" vs. on-demand games (depends on how far
   `docs/multiplayer.md`'s "multiple games per server" question has progressed
   by then — process-per-game spawning would change the compose shape).

## Open questions / things to confirm before building

- **Domain name**: `civ.kidvhs.com` used as a placeholder above — confirm the
  actual subdomain to use.
- **Multi-game story**: `docs/multiplayer.md` notes one Bevy `World` = one game.
  If "process-per-game" lands before this deploy, `civilization-server` becomes
  a spawner rather than a single long-running game, which changes the compose
  service from "one container" to "container + dynamically spawned siblings" —
  worth sequencing this deploy *after* that decision, or accepting a single
  always-on game for v1.
- **`NETCODE_KEY` persistence**: needs a real secret in `pingora-docker/.env`
  (`CIV_NETCODE_KEY`), not the `random` default, so reconnects survive container
  restarts.
- **Image size/build time**: the `web` target adds a full `trunk build --release`
  (wasm + wasm-opt) to the build; confirm this doesn't make rebuild-on-tag too
  slow for the existing webhook flow (no timeout currently enforced in
  `rebuild.sh`, but worth checking in practice).

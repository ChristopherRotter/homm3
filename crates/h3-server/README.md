# h3-server

Hosts the engine's battles for untrusted clients over WebSocket (E16, ADR-0018, ADR-0019). The
working detail is in `docs/epics/E16-design.md`; this page is for the operator.

## Running

```sh
h3-server server.toml                       # serves until SIGINT/SIGTERM
h3-server rebind server.toml <login> <key>  # while stopped: an account's new key (a forgotten password)
```

The configuration's format, every field with its default, is in `src/config.rs`; an example is
`server.example.toml`. The data directory holds the games' journals, the finished games'
replays, the player log, each ladder's results log (`ladders/<id>.log`: its seasons and the
ratings after each rated game; a ladder whose content changed starts its next season at the next
start, every rating carried and its deviation reset) and a lock that keeps one server (or one
`rebind`) on it at a time; keep it readable by the server's user only. A ladder's scenarios are
offers of their own, `<ladder>.<n>` in the pool's order: its matches' journals and replays name
them, and an `[[offer]]` may not be named so. The first signal drains the server (no new game,
the running ones play on), a second stops it at once; the journals stay, and the next start
restores the running and paused games.

## On the internet: behind a reverse proxy

The server speaks plain WebSocket (`ws://`). On the internet it runs behind a reverse proxy that
terminates TLS (ADR-0019 point 10): logins are safe without TLS, a session is not.

1. **Listen on loopback**: `bind = "127.0.0.1:7878"` (the default).
2. **Name the address clients dial**: `public_url = "wss://h3.example.org"`. A login signs the
   server's id, and a client signs only if the id is the address it dialed: without the public
   URL every login through the proxy is refused.
3. **Trust the proxy's address**: `trusted_proxies = ["127.0.0.1"]`. Then a connection from it
   counts, in every limit per address, as the client whose address the last entry of its
   `X-Forwarded-For` header names (without a header that reads, as the proxy). An IPv6 client
   counts by its /64. A forwarded header from any other peer is ignored, so list only proxies
   that set the header themselves.
4. **Configure the proxy** to pass the WebSocket upgrade, set `X-Forwarded-For` to the client's
   address, and keep idle connections at least as long as `idle_timeout_ms` (300 s by default).
   The server pings every connection (`ping_every_ms`, 15 s) and closes one that answers
   nothing within `ping_timeout_ms` (30 s): a player whose network is gone leaves a ranked
   game's seat within that. nginx and Caddy pass the pings and pongs through; a proxy that
   answered pings itself would hide a client that is gone.

nginx:

```nginx
server {
    listen 443 ssl;
    server_name h3.example.org;
    ssl_certificate     /etc/letsencrypt/live/h3.example.org/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/h3.example.org/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:7878;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header X-Forwarded-For $remote_addr;
        proxy_read_timeout 600s;
        proxy_send_timeout 600s;
    }
}
```

Caddy (TLS certificates automatic; it sets `X-Forwarded-For` itself):

```caddyfile
h3.example.org {
    reverse_proxy 127.0.0.1:7878
}
```

The matching `server.toml`:

```toml
[server]
bind = "127.0.0.1:7878"
public_url = "wss://h3.example.org"
trusted_proxies = ["127.0.0.1"]
```

# h3-server

Hosts the engine's battles for untrusted clients over WebSocket (E16, ADR-0018, ADR-0019, ADR-0021). The
working detail is in `docs/epics/E16-design.md`; this page is for the operator.

## Running

```sh
h3-server server.toml                       # serves until SIGINT/SIGTERM
h3-server rebind server.toml <login> <key>  # while stopped: an account's new key (a forgotten password)
h3-server card server.toml                  # while it runs: its signed card, for a web page
```

The configuration's format, every field with its default, is in `src/config.rs`; an example is
`server.example.toml`. The data directory holds the games' journals, the finished games'
replays, the player log, each ladder's results log (`ladders/<id>.log`: its seasons and the
ratings after each rated game; a ladder whose content changed starts its next season at the next
start, every rating carried and its deviation reset), the server's key and a lock that keeps one
server (or one `rebind`) on it at a time; keep it readable by the server's user only.

**The server's key** (`server.key`, ADR-0021 point 1) is written at the first start and is the
server's identity: every welcome carries the public key and a signature by it, and clients pin
the key on first contact, as SSH does. It is the data directory's one secret: whoever holds it can
speak as the server (not as a player). Back it up; a lost key is a new identity, whose welcome
every client that pinned the old one refuses. A start stops at a key file it cannot read or
parse, and never writes a new one over it.

**The server's card** (ADR-0021 point 2) describes it to players who browse: its name, description
and operator contact (`[server] name`, `description`, `operator`; the name is the server's id
by default), its addresses (`public_url` or `ws://` and the bound address, then `addresses`), the
protocol and engine versions, each ruleset its offers and ladders are made of, its ladders and
whether anonymous play is allowed, signed by the server's key with its issue time and an expiry a
day later (`[discovery] card_ms`). It counts the open connections, the games waiting or running and
each ladder's players (the accounts on its current season's leaderboard); `[discovery] counts =
false` keeps the counts off it, which are otherwise public to anyone who asks. Any client gets it
with `Describe` after its hello, without a login; the server signs a new one when asked and its
last one is a minute old. `h3-server card server.toml` asks the running server for it (at `bind`,
on loopback for `0.0.0.0`) and prints it, for a web page: refresh the page's copy before it
expires. A card that would break its bounds (`h3_protocol::card`: 8 KiB, 16 rulesets and 16
ladders, names of 64 bytes, 64 peers) stops the start.

**Peers** (ADR-0021 point 3): the servers the operator vouches for, each a `[[peer]]` with its
key (its card's `key`, which its operator can read off `h3-server card`) and the address the
server fetches its card at. The card names them, and the server fetches their cards at start
and every `[discovery] refresh_ms` (an hour), one after the other, each within `timeout_ms`
(10 s), and relays them to any client that asks (`ListPeers`, after the hello, without a
login). A fetch checks that the configured key answers at the configured address (the peer's
welcome names that address as its id: its `public_url`, or one of its `addresses`) and signed
the card; a peer that fails is logged and its last good card is relayed until it expires. The
server dials only the addresses in its configuration, never one it read in a card. A peer runs
the same protocol version (before 1.0 a server greets only its own).

**Beyond the peers** (ADR-0021 point 3): a fetch also reads the cards a peer relays, and the
server relays them on with the chain of keys that vouches for each (`via`: the peer, then the
servers down to the card's voucher), which a client checks against the cards themselves.
`[discovery] depth` (2) says how far: 1 relays the peers' own cards only, 2 also the cards of
their peers, and so on up to 4; `cards` (200, at most 1000) bounds what the server relays, the
nearer first and the peers in the configuration's order, and how many of a peer's cards a fetch
reads. A card a peer relays that is forged or expired, or whose chain does not hold from the
peer's card (the peer relays what it does not vouch for), is dropped and logged; each server
appears once, by its shortest chain, and the server's own card never. A card is relayed until
the first card of its chain expires. The server still dials only its configured peers: it learns
of the servers beyond them from their cards alone. It holds up to `cards` relayed cards of each
peer (each at most 8 KiB), so up to the number of peers times `cards` in memory.

**Lists of servers** (ADR-0021 point 5): a list names servers by key and address and is signed
by the list's key; anyone may make one (a community, a tournament), and players choose which list
keys they trust. Make a key with `h3-server list-key community.key` (it prints the public key;
an existing key file is read, never replaced; keep it secret), write the servers into a TOML
file:

```toml
name = "Community"          # one line, at most 64 bytes
valid_days = 30             # the signed list holds this long (1 to 366 days)

[[server]]
key = "3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29"
address = "wss://h3.example.net"   # an address the server answers at, as its id
```

and sign it with `h3-server sign-list --key community.key community.toml > community.json`
(at most 128 servers, no key twice). A server serves the lists placed in `<data>/lists/` (files
ending in `.json`, at most 16, read at start and at each refresh in their names' order). It skips
and logs a file that does not read, a list that is forged, malformed or expired, and a second
list of the same key. It fetches the listed servers' cards as it fetches its peers' (their own
cards only, at the addresses the list names, after the peers, each server once, never one that is
a peer, within `cards`) and relays them marked with the list's key; any client gets the lists
with `Lists` and checks that a list names each such card. Re-sign a list before it expires;
removing its file drops its servers at the next refresh.

**The TLS client** (ADR-0021 point 4) fetches a `wss://` peer through `rustls` with the
pure-Rust graviola cryptography, checking the peer's certificate against the operating system's
root certificates (or those `SSL_CERT_FILE` or `SSL_CERT_DIR` name); `ws://` is plain, for a
LAN. Graviola runs on x86_64 processors with AES-NI, PCLMULQDQ, BMI1, ADX, AVX and AVX2 (about
2013 on) and on aarch64 with NEON and the AES, PMULL and SHA-2 extensions, and does not build for
other architectures; on an older processor every `wss://` fetch fails and is logged, and the
server serves on.

A ladder's scenarios are
offers of their own, `<ladder>.<n>` in the pool's order: its matches' journals and replays name
them, and an `[[offer]]` may not be named so. A ladder's bots (`[[ladder.bot]]`) are accounts in
the player log without a key: no one logs in as them, `rebind` refuses them, and a start renames
one whose display name the configuration changed. The first signal drains the server (no new game,
the running ones play on), a second stops it at once; the journals stay, and the next start
restores the running and paused games.

## On the internet: behind a reverse proxy

The server speaks plain WebSocket (`ws://`). On the internet it runs behind a reverse proxy that
terminates TLS (ADR-0019 point 10): logins are safe without TLS, a session is not.

1. **Listen on loopback**: `bind = "127.0.0.1:7878"` (the default).
2. **Name the address clients dial**: `public_url = "wss://h3.example.org"`. A login signs the
   server's id, and a client signs only if the id is the address it dialed: without the public
   URL every login through the proxy is refused. A server reached at other addresses too (a LAN
   address, a second name) lists them: `addresses = ["ws://192.168.1.5:7878"]`; a client that
   dialed one of them gets it as the id.
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

# Security policy

## What counts

The engine is server-authoritative ([ADR-0004](docs/adr/0004-state-commands-events.md),
[docs/06-security.md](docs/06-security.md)), so these are security issues, not ordinary bugs:

- a command that passes validation it should fail, acts for another player, or trusts an amount,
  price or ownership from its payload;
- a rejected command that changes the state;
- an observation, event or replay that shows a player what they must not see (hidden information:
  other players' tiles, the other side's private facts, the seed or the random streams, which
  predict rolls);
- anything that lets a client crash, stall or exhaust the server (`h3-server`, `h3-protocol`),
  or read or write outside what the protocol allows;
- memory unsafety through the C ABI (`h3-ffi`) or the Python binding.

A rule the engine plays differently from the original game is a fidelity bug: open an ordinary
issue ([CONTRIBUTING.md](CONTRIBUTING.md), "Reporting problems"). So is an exploit of a rule that
only affects your own game, unless it breaks one of the points above.

## How to report

Please do not open a public issue. Use GitHub's private reporting: the repository's **Security**
tab, **Report a vulnerability**. Say what you did, what happened and what you expected; a command
sequence or a replay file (`h3 run --record <out.toml>`) that shows it is the best evidence.

The maintainer answers in the private report. There are no releases yet, so fixes go to `master`
only.

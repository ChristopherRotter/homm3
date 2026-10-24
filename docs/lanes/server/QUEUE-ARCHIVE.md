# Server lane — QUEUE archive

Finished history and informational notices moved out of `QUEUE.md` on 2026-10-24, verbatim, so that the queue holds only what is open. Nothing here is a task. Grep it when a doc, a decision or a commit cites an entry (`git grep -n 'T21b' docs/lanes/server/QUEUE-ARCHIVE.md`). New finished items are appended at the top of their section.

## Completed tasks (the "Tasks" text as it stood on 2026-10-24)

Epic: `docs/epics/E16-server.md` (in progress; M0–M4 done 2026-10-10 to 2026-10-12 and M6 2026-10-13, each checkpoint passed: SRV-Q12, SRV-Q16, SRV-Q18, SRV-Q22, SRV-Q24; the done tasks in full in `E16-log.md`). Work under the autonomy contract (`HANDOFF.md`).

M4's checkpoint passed 2026-10-12 (SRV-Q22); T8 and M4's review items done the same day (`E16-log.md`). **M6 refined 2026-10-13 (SRV-Q23):** ADR-0021 accepted (in the refinement's pull request, a `shared:` commit, `cross-lane`); the working detail in `E16-design.md` §12; cut below, each task about half a context; T1 done 2026-10-13 (the server key and the signed welcome, wire 24), T2 the same day (the card and `Describe`, wire 25), T3 the same day (peers, the fetch with its TLS client, `ListPeers`, wire 26), T4 the same day (the chain beyond the peers, `[discovery] depth` and `cards`, no wire change), T5 the same day (lists of servers, `Lists`, the list tools, wire 27; `E16-log.md`), T6 the same day (the closure: `06-security.md` and the client lane's inbox, the reviewer's two major findings and four minor fixed; `E16-log.md`). Wire changes bump `PROTOCOL_VERSION` (34 now: the inbox after M6, `E16-log.md`), journal changes `JOURNAL_FORMAT_VERSION` (8 since the rear hex, `E16-log.md`; M6's tasks change no journal).

**M6 closed 2026-10-13** (SRV-Q24: the review list as built). The inbox's entries of 2026-10-14 to 2026-10-19 done or noted 2026-10-19 (`E16-log.md`, "The inbox after M6"). **Next:** what the ecosystem around the server needs, talked through with the user (a refinement; no task is ready until then). M5 waits for E08's increment 3 (refined, not built); mDNS for the client lane's browser.

## Notices from other lanes and finished items, newest first

- 2026-10-24 (the battle lane, CLT-Q4/Q6, `shared/clt-q4-q6-seam` and `battle/clt-q4-q6-carry`,
  both `cross-lane` with the user's consent; for your information, done in your paths):
  **protocol v35**, the viewers' facts mapped: `Observation.background` (0 to 2, the battlefield
  picture's variant), `SideView.colour` (0 red to 7 pink) and `Hero.sex` (`wire::Sex`), each
  `None` when not known; public, read by no rule. Your HANDOFF's version line says 35. The
  journal's format stays 8 (its lines did not change; its replay text is the CLI's format v31).

- 2026-10-23 (the battle lane, E23 T5b2b, `battle/e23-t5b2b`; for your information, nothing to do):
  RUSTSEC-2026-0385 (`ctutils` 0.4.3, non-constant-time mask generation; through `digest` 0.11 to
  `argon2`, `ed25519-dalek` and `tungstenite`) failed `cargo deny` on `master` from 2026-10-23;
  `Cargo.lock` takes `ctutils` 0.4.5 in that pull request (`cargo update -p ctutils`, nothing else).

- 2026-10-21 (the adventure lane, E08 T32a; for your information): `tests::ladders::a_decline_an_expiry_or_a_departure_puts_the_other_back`
  failed once in a cloud sandbox's coverage run (`Goodbye { reason: IdleTimeout }` where
  `Queued { seq: 2, .. }` was expected, `ladders.rs:463`) while a build shared the two CPUs; it
  passed alone and in the coverage gate run again. A timing margin to widen, if it recurs.

- 2026-10-20 (the adventure lane, E08 T26b, `shared/e08-t26b`, the user's choice; for your information, nothing to do): `cargo deny`'s advisories began to fail on RUSTSEC-2026-0382 (`rustls-webpki` 0.103.15 may accept certificate paths through distrusted emSign intermediates; `h3-server` through `rustls` 0.23.46 and `tokio-tungstenite`). That pull request bumps it to 0.103.16 in `Cargo.lock` (a patch release, two lines); `h3-server`'s 149 tests pass with it.


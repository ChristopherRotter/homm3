# RMG lane — QUEUE

Pre-refined tasks of the rmg lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E14-original-map-generator.md` (increments 2 to 7 cut 2026-11-24, RMG-Q2; increment 2 refined; T2a done). Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`) and the increment's exit criteria (a) to (f) (the epic, "Increments 2 to 7"). The design: ADR-0029 (`docs/adr/0029-original-map-generator.md`).

- **T2b — `cross-lane` (`h3-harvest`): the three tables.** `rmg.txt`, `objects.txt` and `rand_trn.txt` into `packs/ab/rmg.toml` (AB's) and `packs/sod/rmg.toml` (SoD's three templates and `objects.txt`'s 105 added and three changed lines), the values as read (ADR-0029 point 5, the picture names with them, RMG-Q2); `h3-pack-rmg`'s parser; the rmg part's compiler and hash (`RMG_RULESET_HASH_VERSION`); 8XM8's gated override and `quirk.content.rmg_8xm8_zone_ids` (RMG-QUIRK-22); `packs/README.md`; a test with the original data: the harvested tables equal the text tables.
- **T2c — the settings and the template.** The settings and their checks (RMG-SET-001 to -009; RMG-QUIRK-6 a typed error), `resolve_random`, the templates kept and drawn and the positions (RMG-TPL-001 to -008; RMG-QUIRK-11's flag), `generate`'s frame with its `Trace` and `RMG_GENERATOR_VERSION`; on the 21 corpus cases the kept and drawn templates (`kept.txt`'s names) and the draws before step 1 equal the original's. The increment's checkpoint follows.

## Inbox

Nothing yet.

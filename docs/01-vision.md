# 01 — Vision and goals

## Purpose

Build a backend engine for Heroes of Might and Magic III with **exceptional performance** and a **state-of-the-art AI trained by self-play**. The engine is the foundation; the AI is the main motivator. Everything in the core is designed so that millions of games (adventure map and battle) can be simulated quickly and reproducibly.

## Goals

1. **Fidelity.** Game logic matches the original so closely that any difference between the original and this engine comes down to RNG seed and rollouts, not to logic. Like-for-like replay of original games is *not* a goal (the original's RNG stream is not reproduced).
2. **Official content.** Base game (RoE) plus the two official extensions, Armageddon's Blade (AB) and Shadow of Death (SoD), are supported by the engine and implemented on top of an extensible core.
3. **Extensibility.** The core must allow further extensions (fan-made WoG, HotA, or new ones) without rewriting it.
4. **Separation of concerns.** Game logic is free of graphics, audio, i18n, assets and any UI concern. Boundaries are enforced mechanically.
5. **Domain-driven design** with sub-domains (at least: battle, adventure map, and a shared core), compromising only for performance when the trade-off clearly warrants it.
6. **Performance.** The logic core is the hot path of RL training; it must be fast, allocation-light, cheap to clone/fork, and deterministic.
7. **Configurability.** Gameplay-relevant data and parameters (stats, costs, buildings, mechanics toggles, which extensions are active) are data, not code — within the limits of ADR-0006 (architecture wins over configurability).
8. **Compatibility.** Maximum practical compatibility with original assets, maps and (lower priority, later) save files. Assets are extracted from the user's own legitimate copy of the game.
9. **Security.** The authoritative game logic resists invalid/malicious commands and information leaks (see `06-security.md`). Known original exploits (e.g., spending another player's resources) must not exist.
10. **AI.** A simulation environment wraps the engine; an RL agent learns from self-play (possibly hybrid: learned top-level policy with deterministic low-level execution, search, imitation/bootstrapping where data exist). Training is automated enough that the user gives guidance, not every detailed decision.
11. **Quality.** 100% unit-test coverage of core logic, property tests, determinism replays, later BDD acceptance tests.

## Non-goals (for now)

- Graphics, sound, UI, animations, localization, asset rendering. (Clients are separate, later, possibly several: web/WASM, Windows, Linux, maybe macOS.)
- Fan-made extensions (WoG, HotA), HD-mod features, QoL and graphical fixes — **but** nothing may be built that makes them require a massive rewrite (guardrails in ADR-0006).
- Original save-file import/export (future scope, low-ish priority).
- Campaigns (future scope).
- Bit-exact reproduction of the original's RNG or of its random map generator.

## Success criteria

- **Fidelity:** for every implemented rule there is a rule ID, a labelled source and a passing test; discrepancies against reference sources are tracked in the quirks ledger; acceptance scenarios (later BDD) pass.
- **Performance:** measured and tracked benchmark targets (initial provisional targets in `02-architecture.md`, calibrated during the walking skeleton); no regression allowed without an explicit decision.
- **Determinism:** replay fixtures produce identical state hashes across supported platforms in CI.
- **Quality gates:** `cargo xtask verify` green: format, lint, architecture checks, coverage, rule traceability, tests.
- **AI:** measurable strength against a fixed opponent ladder (scripted bots, later the original AI and other references); improving checkpoints promoted automatically by an evaluation gate.

## Decisions already taken by the user (2026-09-30)

- Configurability tiers 1–3 (data, parameters/toggles, composable behavior) suffice for base game + official extensions; do not over-invest in configurability when it complicates or slows the architecture.
- Fidelity = logic equivalence, not replay equivalence. Quirk **profiles** are welcome.
- Save files: lower priority, future scope.
- Determinism: follow Factorio's example of deterministic cross-platform floating point (custom trig functions, determinism tests; [FFF-52](https://factorio.com/blog/post/fff-52)). Floats are allowed under the discipline in ADR-0003; integers remain the default.
- The core exposes a **C ABI** so the RL/training side and clients can use any language; the RL implementation language is decided later.
- Information taken from wikis must be clearly labelled and linked. Original artifacts are supplied by the user to agents. VCMI may be consulted since it is open source.
- License should be as open and permissive as possible (see the VCMI conflict in ADR-0009 / OPEN-QUESTIONS Q1).
- Hybrid AI approaches are welcome (e.g., learned top-level + deterministic lower level).
- The user has an unfinished map-generator project that will seed the map-generation work; maps should optimize stated objectives (balanced zones, minimal teleport connections, etc.).
- Initial compute target: Ryzen 3900-class CPU + GTX 1070 on the user's PC; longer training times are acceptable.
- Testing requirements must be stated in enough detail, with a mechanism that keeps agents from forgetting them (`05-testing.md`).
- Walking-skeleton approach approved.
- Security requirement added: protect against invalid actions from clients and against reading data a player must not see.

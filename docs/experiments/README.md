# Experiments in the original game

Staged battles that the user plays in the original game to settle disputed rules and to confirm
quirks before they are implemented (OPEN-QUESTIONS Q28, Q30; E04a T2). The engine writes the maps;
agents never write into the installation. Results hold for the user's version only: **SoD,
Complete 4.0**; a rule settled here is `observation`-sourced with that version.

## Flow

1. An agent writes a **spec** (`experiments/specs/<batch>.toml`, committed) and compiles it:

   ```
   cargo run -p h3-experiment -- experiments/specs/<batch>.toml --install original-data/install \
       --map experiments/out/<batch>.h3m --template experiments/results/<batch>.md
   ```

   The map goes into the gitignored `experiments/out/`; the **results template** is committed.
   `--list heroes|creatures|skills|artifacts|spells` prints the names a spec may use (heroes with
   class and specialty).
2. The user copies the map into the game's `Maps` folder, plays it as the template says and fills
   in the template in the repo.
3. The agent evaluates the filled template against the spec's predictions and records the result
   in the rule catalog (the rule's source becomes `observation`), the quirks ledger (a confirmed
   quirk, Q30) or OPEN-QUESTIONS (if the result is unclear).

## Spec format (`h3-experiment`, `crates/h3-experiment/src/lib.rs`)

- `[map]`: `name`, `description`, `size` (36), `terrain` (dirt or grass; the tile pictures of
  other terrains are not known yet, `docs/formats/h3m.md`), `instructions` (the template's
  numbered introduction).
- `[[encounter]]`: `id` (also the hero's name in the game), `title`, `questions`, `procedure`
  (steps), `record` (`field = "…"` for a value, `table = [columns]` + `rows`), `predictions`
  (for the evaluating agent; never shown in the template).
- `[encounter.hero]`: `type` (name, or id if the name occurs twice), `specialty` (checked against
  the executable so that no hero with an interfering specialty slips in), `owner` (red/blue),
  `experience`, `primary`, `skills` (`[name, level]`), `army` (`[creature, count]`), `artifacts`
  (slot name → artifact; slots are checked against `ARTRAITS.TXT`), `spells` (`SPTRAITS.TXT`
  names, ids below 70; needs a worn `spell book`, E04b batch 2a). Skills, artifacts and spells
  replace the hero type's defaults, so heroes have no spell book unless one is listed. A hero
  starts with full mana: 10 × knowledge (no Intelligence), so knowledge sets the mana.
- `[encounter.opponent]`: `monster` + `count` (a savage wandering monster that never flees) or
  `[encounter.opponent.hero]` (a Blue hero). Encounters are laid out in a grid; a hero type may
  appear once per map, a player may have eight heroes.

Running a spec in our engine (to print predictions automatically) is not built (BACKLOG; Q37:
revisit with the next batches, E04b); the predictions are computed by hand or with a script and
written into the spec.

## Status

| Batch | Purpose | State |
|---|---|---|
| smoke (`E04a-smoke.h3m`) | prove that the game accepts our files | **works** (user, 2026-10-03) |
| 1a (`E04a-batch-1a`) | Q23 damage rounding (three encounters that separate exact, single, double and x87-style arithmetic), Q21 death stare, the hero's attack in the Ballista, Basic Artillery and First Aid control, Ballista ammunition (S-01), Elixir of Life regeneration (S-04), the Swordsman specialties (S-03) | **evaluated** (2026-10-03): `docs/epics/E04a-log.md`, "The design note at closure"; the First Aid Tent with the skill is left for 1b |
| 1b (`E04a-batch-1b`) | tactics: zone, war machines, placement rules, the disabled Tactics formation (OQ-1–OQ-3); surrender price, discounts and consent (D-1, D-2, D-5); Shackles of War against a monster (D-6); experience for war machines and with Learning, captured war machines (D-3, OQ-6, OQ-5); Necromancy rounding (D-7); the commanded First Aid Tent (T7b leftovers); war machines with creature artifacts and the Golden Bow, the Ballista's range penalty (artifacts open 1–2); Angelic Alliance mixing (artifacts D1); Xeron's speed (H-2) | **evaluated** (2026-10-04): `docs/epics/E04a-log.md`, "The design note at closure"; decisions in OPEN-QUESTIONS Q32; not staged: D8 (an E08–E10 input) |
| 2a (`E04b-batch-2a`) | Chain Lightning by screen or hex distance and its tie order (Q44, X-6), aiming Fireball and Implosion and the aimed stack's resistance roll (BAT-MAG-003/007/008, X-5), a spell specialist's rounding with Sorcery against creature levels, Golems and elementals and the Gold Dragon's immunity (X-1, X-2), the Orb with Sorcery (X-1), costs with Mages, Pegasi and Familiars and casting in the Ballista's turn (X-3, X-4), the tactics hand-over (Q40) | **evaluated** (2026-10-06): `docs/rules/surveys/E04b-magic.md` §13, `docs/epics/E04b-log.md` (T2e); Q40 answered; the costs (R6) open for T7 |

**How the user plays** (2026-10-03): the Complete edition's `Heroes3.exe` started through a HotA
installation's HD Mod launcher (HD Mod 5.6 R9). The HD Mod's always-on changes are crash fixes; its
gameplay fixes come only with the optional HD+ setting (Clone, Quick Combat, neutrals with the
Angelic Alliance, a hero without an army, AI turns, merchants;
https://www.sites.google.com/site/heroes3hd/eng/original-game-bug-fixing). Batch 1a was played with
HD+ on (no fix touches it); from batch 1b on the user plays with plain HD. Observation sources name
the setup.

Not staged: Q22 (jousting: path length or distance). Showing it needs a controlled detour and,
because Champion damage is random, many repetitions; reading the game's code is the cheaper last
resort (Q29) when T-tasks need it. Q26 and First Aid with the skill are settled by the code
reading in `docs/formats/installation.md` §7.5.

**Candidates for the next batch** (E04a T12c review, 2026-10-05): rules implemented after VCMI or
by generalisation and not observed in SoD. (1) Spirit of Oppression against a creature's minimum
morale (Minotaurs; VCMI lets the cap win, BAT-HER-003; the original's text only says it negates
all morale bonuses). (2) Luck and the Artillery double damage on one shot: added or multiplied
(VCMI adds, BAT-WAR-006). (3) The surrender price with an Ammo Cart and a First Aid Tent (only the
Ballista was observed, BAT-FLE-002). (4) The Bow of the Sharpshooter: may a shooter next to an
enemy still attack in melee (BAT-HER-006 lets it only shoot). (5) The order in which the tactics
phase hands over the stacks and what "next creature" does (OPEN-QUESTIONS Q40, with the
experiment; the user: the order is a regular round's, but "next creature" acts more like Wait).
Batch 2a staged (5) with the magic candidates (answered: Q40); (1)–(4) wait for a later batch,
as do Q-0005's single-precision resistance (a one-point difference in a chance), Q43 (the sprite
offset in Chain Lightning's jumps), a skill specialist's growth at a high level (Q45) and the
spell costs with Mages and a dead Pegasus (R6, T7).

The smoke map is generated by `cargo run -p h3-formats --example smoke_map --
original-data/extracted/H3bitmap/Objects.txt experiments/out/E04a-smoke.h3m`.

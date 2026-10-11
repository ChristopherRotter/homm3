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
| 2b (`E04b-batch-2b`) | the x87 precision of the damage stages, 53 or 64 bits (Q50), Counterstrike's extra round (Q-0008), Fire Shield's reflection base (Q-0010), Shield with Basic Armorer in one product and Armorer alone (BAT-DMG-002, Q-0005), a counter pair: Bloodlust and Weakness add up, Haste and Slow cancel (Q47), an Armorer specialist at level 10 (Q45 (b)) | **evaluated** (2026-10-07): every encounter as the engine's faithful prediction (53 bits; Q-0008, Q-0010, Q47, Q45 (b) observed); `docs/epics/E04b-log.md` (batch 2b folded in) |
| 2c (`E04b-batch-2c`) | the anchor hexes of natural obstacles (Q55: Remove Obstacle, Fire Wall and a flyer on the free hex beside an obstacle, three battlefields), Fire Wall burning on entering, Wait and Defend, the defender's Force Field and Fire Wall shapes (BAT-MAG-027/028), a walker's path among equally short ones past a Fire Wall (BAT-MOV-003), Berserk between two equally near stacks (BAT-MAG-025), Hypnotize against a wounded stack, Expert and Basic Cure on a hypnotized stack (BAT-MAG-026, Q-0012) | **evaluated** (2026-10-07): every encounter as the engine's faithful prediction (H1's round 3 after the user's correction, BAT-Q2); A2 confirms Q55's anchors; `docs/epics/E04b-log.md` (batch 2c folded in) |
| 2d (`E04b-batch-2d`) | where a clone appears, its turn, shots, rounds and death by any damage, and the surrender price with a clone (CL1; BAT-MAG-038, BAT-FLE-002); a clone on its own crowded two-hex original (CL2, BAT-QUIRK-7); Resurrection's and Sacrifice's targets (RA1, BAT-QUIRK-5, S-15) and corpses (RA4); Basic Resurrection's temporary creatures at the battle's end, with a Ballista too (RA2, RA3; BAT-QUIRK-6, S-20); Necromancy and experience from created stacks, the elementals' native terrain (NE; BAT-Q7, BAT-TER-001); Summon Elemental's place, a dead elemental's slot, the limit of 20 stacks, a side left with only elementals against an elimination, a retreat and a surrender (SU; BAT-MAG-037/039, BAT-Q7) | **evaluated** (2026-10-08): as the code reading everywhere but the elementals' native terrain (natives on grass, as the engine; the reading of 0x43D730 refuted); the engine's surrender price refuted (created stacks left out; fixed); BAT-QUIRK-7 observed (fix-optional), BAT-Q7 answered; `docs/epics/E04b-log.md` (batch 2d folded in) |
| 2e (`E04b-batch-2e`) | the code reading of the casting limits and the spell defences where the wiki or VCMI disagree (BAT-Q21, BAT-Q22): the Faerie Dragons and Ogre Magi under the Orb of Inhibition (OI), Blind at Troglodytes, Mirth and Sorrow at golems and elementals and Poison at Gargoyles under the Orb of Vulnerability (OV), the Unicorn aura under it (UA, mass Slow four times), Recanter's Cloak against Stone Gaze and the Archangel (RC), Power of the Dragon Father against Poison and the Thunderbirds' bolt with the Death Stare as a control (DF), Bless, Curse, Precision, Fortune, Misfortune and Slayer at the war machines (WM; BAT-MAG-061, 062, 064, 066, 067) | **evaluated** (2026-10-11): the code reading everywhere but the Unicorn aura, which the Orb lifts as the wiki and VCMI say (12 of 12 slowed); the Faerie Dragons are stopped by the Orb of Inhibition (the engine followed the wiki and VCMI: fixed); RC's Stone Gaze (8 attacks) and DF's bolt and Poison (3 and 4) are weak, as no save in battle allowed the planned tries, but decisive with UA's aura (the user, BAT-Q27); `docs/epics/E04b-log.md` (batch 2e folded in) |

**How the user plays** (2026-10-03): the Complete edition's `Heroes3.exe` started through a HotA
installation's HD Mod launcher (HD Mod 5.6 R9). The HD Mod's always-on changes are crash fixes; its
gameplay fixes come only with the optional HD+ setting (Clone, Quick Combat, neutrals with the
Angelic Alliance, a hero without an army, AI turns, merchants;
https://www.sites.google.com/site/heroes3hd/eng/original-game-bug-fixing). Batch 1a was played with
HD+ on (no fix touches it); from batch 1b on the user plays with plain HD. Observation sources name
the setup. **The game cannot be saved during a battle** (batch 2e): an encounter that needs many
tries replays the battle from a save made before it, so plan few, long rounds instead of reloads.

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
spell costs with Mages and a dead Pegasus (R6, T7); from T4a, a Magic Mirror sending a spell to the
caster's own war machine (VCMI never does) and Sorrow surviving a Basic Anti-Magic. Batch 2c staged
Q55 with T5a's and T4c's candidates (the user, Q56); the others above still wait. From T5c1
(Quicksand, read in the code, `docs/formats/installation.md` §7.16): the pits by mastery (4, 4, 6,
8) and none on a corpse's hex; the natives' sight fixed at the battle's start (BAT-QUIRK-1: a side
whose natives died still sees Quicksand cast later); a two-hex walker stopped by its tail; whether
the interface offers an attack from a seen pit and what happens (BAT-QUIRK-2); Fire Wall's cursor
refused on the other side's hidden pit (Q-0002); a hypnotized walker stopped by its owner's pit
without revealing it. From T5c2 (Land Mine, `installation.md` §7.17): the spell book refusing Land
Mine against an army whose only natives are immune to it (Green Dragons on grass: the code
refuses, the wiki says it is cast); a stack teleported onto an unseen enemy mine that then
defends (the mine goes off); a walker that survives a mine walking on and still attacking; a
hypnotized stack crossing its owner's mine unharmed and the caster's mine harmed; a fire-immune
stack crossing a mine that stays. From T5b (Remove Obstacle, `installation.md` §7.15): Remove
Obstacle at Basic refused on a Fire Wall and at Advanced on a Force Field; a usual obstacle's anchor
hex that a second usual obstacle covers (Remove Obstacle there takes the covering one, after which
the hex leads to no obstacle: 0x466B30 clears it); the original's random obstacles placed on
another obstacle's free anchor hex (the generator's anchor test was not read). Batch 2d staged T6a's and T6b's
candidates (the user, T6 checkpoint 2026-10-08: BAT-QUIRK-7, BAT-Q7 and the clone's shots before
they are decided) but two: a clone killed by 0 damage (the original kills a clone whatever the
damage, VCMI 1.7.5 not by 0; no hero spell deals 0 in SoD) and the game setting that keeps the
elementals' native terrain (0x43D730, not read; batch 2d showed them native in the user's setup). New from batch 2d: a reused record's losses (SU a's casualties list 2 Air Elementals of at least 4 killed) and so the Necromancy from elementals killed before their record was reused. From T7a1 (the strike's spells,
`installation.md` §7.20; the wiki was not consulted): Black Knights cursing with 25 %; a Dragon
Fly's Weakness at Advanced (−6 attack) and not renewed on a weakened stack; a Thunderbird's bolt
refused by Green Dragons (Lightning Bolt's level) and halved by Protection from Air; a retaliating
Unicorn's Blind stopping a Wolf Rider's second strike; a hypnotized Unicorn blinding its owner's
stack. From T7a2 (the statuses): a petrified stack struck for half that then retaliates (the
strike ends the petrification first); Poison's lost hit points staying after it ends and coming
back with a Cure; 9-hit-point creatures poisoned once and aged keeping 4 (Q-0005; exact 5);
Age leaving a wounded top creature at 1 hit point. From T7a3 (Bind, Acid Breath): a bound stack
freed by its Dendroid's walk or death and by Teleport on either; still bound while a second
Dendroid binds it;
Dendroids binding a Ballista (no test); Dispel ending Bind, Cure not; a raised stack that the
same Dendroid killed and strikes again (BAT-Q8); Acid Breath stopping at defense 0 and not
undone by Dispel; the Rust Dragon's extra damage (half a roll of 50 per dragon: 25 each). Batch 2e staged BAT-Q21's and BAT-Q22's points (T8b, T8b3); T8b's other candidates wait: a Unicorn's Blind against a Pendant of Second Sight, a hypnotized stack protected by its controller's Pendant, a hero's own Bless at its Dragon Father stacks.

The smoke map is generated by `cargo run -p h3-formats --example smoke_map --
original-data/extracted/H3bitmap/Objects.txt experiments/out/E04a-smoke.h3m`.

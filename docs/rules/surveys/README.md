# Source surveys

Research notes compiled at the start of an epic, before rules are written. A survey collects labelled facts (original data, VCMI, wiki) and the disagreements between them; it is **input**, not a rule. A fact becomes binding only when a task turns it into a catalog rule with its sources and tests (`docs/rules/catalog/`). Surveys are not updated afterwards; the catalog, the quirks ledger and OPEN-QUESTIONS are.

## E04 (E04a T1, 2026-10-03)

| File | Contents |
|---|---|
| [E04-skills.md](E04-skills.md) | the 28 secondary skills: numbers per level, scope (E04a/b/c, outcome, adventure), mechanics of the battle skills, how Archery/Offense/Armorer enter the damage formula, applicability matrix |
| [E04-artifacts.md](E04-artifacts.md) | the 144 artifacts: table layout and AB/SoD differences, membership by expansion, the full table with VCMI's effects, an effect-kind vocabulary with counts, combination artifacts, side-wide and enemy effects |
| [E04-heroes.md](E04-heroes.md) | hero classes, heroes, membership by expansion (from the executable), the eight specialty kinds with formulas and counts, precision of the specialty terms |
| [E04-battle.md](E04-battle.md) | tactics, retreat, surrender, the battle outcome (experience, Necromancy, Eagle Eye, artifact capture), hero-run war machines, the hero's presence, an outline of what E04b and E04c need, the seam inputs |
| [E04-quirks.md](E04-quirks.md) | 22 documented original bugs in the E04 scope with a suggested class each — **for the user to classify** |

The skill, artifact and battle surveys were compiled by sub-agents and reviewed by the session that committed them; the wiki was read through a summarising fetch tool, so every wiki fact that ends up in a rule must be re-checked against the page (the surveys mark facts checked against raw wikitext). The executable facts are documented in [`docs/formats/installation.md`](../../formats/installation.md) §6–§7.

## E04b (E04b T1, 2026-10-05)

| File | Contents |
|---|---|
| [E04b-magic.md](E04b-magic.md) | just-in-time survey for ADR-0014 (Q41): the spell table and its decoded flags, casting by a hero (book, once per round, mastery, cost, mana), effect kinds and targeting, durations, the resolution order (immunity, resistance, Magic Mirror, damage modifiers), stacks created in battle, hidden obstacles, creature magic, quirks by task, disputes M-1 to M-6, experiment candidates for batch 2a. Per-spell numbers are left to the tasks. |

## E04c (E04c T1, 2026-10-13)

| File | Contents |
|---|---|
| [E04c-battlefields.md](E04c-battlefields.md) | just in time for the battlefield seam (BAT-Q37): the twelve special battlefields of the original (how each arises, terrain, effects), their effects compared across the code reading, the game's texts, VCMI and the wiki, the obstacles by battlefield (the executable's tables equal VCMI's), what the setup receives and who derives it, disputes D-1 to D-4, an outline of the sieges, experiment candidates |
| [E04c-sieges.md](E04c-sieges.md) | just in time for the siege seam (E04c T4, BAT-Q40): what the original's battle takes from a town (fort level, moat, turrets, the defending hero's and the stacks' town bonuses, controls, experience) and gives back, the siege's rule areas compared across the code reading, the game's tables, VCMI and the wiki, what the setup receives and who derives it, disputes D-5 to D-7, quirk candidate S-29 (the map format's branch), experiments |

## E08 (E08 T1b, 2026-10-09; adventure lane)

| File | Contents |
|---|---|
| [E08-adventure-core.md](E08-adventure-core.md) | movement points by army speed, step costs by terrain and road, the diagonal exception and the last step of a day, where a hero may step; fog of war (scouting radius, the reveal, the fringe); the day and its order; quirk candidates A-1 to A-3; simultaneous-turn contact rules (HD, HotA, VCMI) captured for later; disputes D-1 to D-4 and F-1 with the code readings that settle them (`ADV-Q3`–`ADV-Q5`); proposed rule ids |
| [E08-objects.md](E08-objects.md) | increment 2 (E08 T11a, 2026-10-12): the object classes in the map file and their census, the two visit modes and the "not from above" rule (the original's per-class settings), pickups (resource piles, random resources, campfires) and when they are drawn, the Cartographer, the observatories, Keymaster's tents, border guards and gates, the treasury and the starting resources by difficulty; what each viewer learns; quirk candidate A-5; the two pending decisions; disputes D-5 to D-7; proposed rule ids; the wiki read the same day, agreeing wherever it speaks |
| [E08-battles.md](E08-battles.md) | increment 3 (E08 T17a, 2026-10-13): wandering monsters in the map file and their census, aggression, random creatures and counts, guarded tiles and the step that starts a battle, the monster's answer (fight, join, pay, flee), the split into stacks seeded by the tile, the battle's seed and obstacles, after the battle (A-6), guarded piles, the heroes' details and custom experience, battles between two players' heroes, applying an outcome, what each viewer learns, the pending decisions, the user's calls sized, disputes D-8 to D-13 with a batch plan, proposed rule ids |
| [E08-progression.md](E08-progression.md) | increment 4 (E08 T24a1, T24a2a, T24a2b1 and T24a2b2, 2026-10-20; complete, its checkpoint ADV-Q15): experience to levels (the table, the steps × 1.2, the 32-bit wrap past level 74, the level limit), the level-up (one level at a time, its seed from the hero's seed byte and the level, the primary draw, the two offers with Wisdom and school guarantees, the human's answer and the computer's), compared with VCMI; levels at load (the placed hero's setup `placed_hero_setup` 0x4D8B30, the computer's choice; 0x486590 is a campaign's carried hero), the battle's levels, the rules that read the level; disputes D-14 to D-21, quirk candidates A-7 to A-10; the wiki on all of it (D-20 a conflict: the left offer at load); the census of the 160 shipped maps (heroes with an experience and their levels, no level limit, banned skills, the maps that import whole without the refusal); batch E4's plan, the proposed rule ids, a fresh agent's second reading (confirms; D-20's code side) |
| [E08-hero-movement.md](E08-hero-movement.md) | increment 5 (E08 T34a, 2026-10-21; its checkpoint ADV-Q20; batch E5 scored in §13 by T38): the day's land points, the step's price with Pathfinding, Nomads, Cursed Ground, flying and water walking, the pathfinder's passage and where a move may end, the scouting radius and its reveal, the census, batch E5's plan, the rule ids |
| [E08-army.md](E08-army.md) | increment 6 (E08 T44a, 2026-10-23; its checkpoint ADV-Q24): the army dialog when a joining group finds no room (a garrison exchange, the rest lost), meetings of a player's own heroes (the step's price, Scholar, the exchange screen), a hero's army (move, swap, merge, split and its bounds, dismiss, the last stack; A-15's 16-bit count, A-16), the formation and the tactics formation flag, the artifacts (19 positions, the ordered and rotating backpack of 64, what never moves, combinations, Titan's Thunder's book), the census, batch E6's plan, the rule ids |
| [E08-rewards.md](E08-rewards.md) | increment 7 (E08 T55a, in four parts, decision 125; done 2026-10-26): the census of the classes that hand a visiting hero something (by the part that reads them); treasure chests (the draw at a new game, the gold or experience choice, the full backpack, the tutorial), placed and random artifacts (the pool, the visit, the guarded artifact's question, the unreachable pickup conditions), Spell Scrolls, the monsters' artifacts; disputes D-29 to D-32. T55a2 (§6): the objects that teach (the once-per-hero marks, 32 bits per class; the primary skill objects, the Schools, the Learning Stone and the Tree, the Shrines' draw, the Witch Hut, the Scholar, the University, the Corpse, Wagon, Warrior's Tomb and Lean To); disputes D-33 to D-36, quirk candidates A-17, A-18. T55a3 (§7, decision 127): Pandora's Box, events and the Pyramid (the shared reward record, an event's firing and its stop, the reward routine's parts and order, the Pyramid's draw, guards and spell; the banks, the Refugee Camp and the quests placed in later items); disputes D-37 to D-40, quirk candidates A-19, A-20. T55a4 (§8 to §14, decision 128): the common rules (what a visit asks and refuses, what changes when, what a player knows, the hidden event's stop and the blind `Reach`, the commands and decisions), batch E7's plan (23 rows in two runs), the proposed rule ids, a fresh agent's second reading, the checkpoint's ADV-Q28 |
| [E08-sea.md](E08-sea.md) | increment 8 (E08 T66a, in two parts, decision 160; T66a1 done 2026-10-28, T66a2 next): the census of boats, the coast and the sea's objects; the boats' records (64 at most, the type, owner and captain, gone with a defeated hero, none for a hero placed on water), boarding and landing (the step, the visit, the Admiral's Hat's proportion after the drop to 0, landing only on a free coastal tile, the guards), sailing (the price, Favorable Winds, the diagonal between two waters, the day's points at sea, what a boat visits, events on water, scouting, what another player sees), the writer's boat; disputes D-41 to D-44 |

## E23 (E23 T1b, 2026-10-20; battle lane)

| File | Contents |
|---|---|
| [E23-battle-ai.md](E23-battle-ai.md) | the original's battle AI for E23's seam: the reading in one place (`installation.md` §7.34 to §7.46: two decisions per activation, the shooter, the walker, the creature casts, the difficulty's branches, the draws), the wiki and VCMI compared, 18 planned batch rows (4a the neutral side, 4b a computer hero's), the original's slips in its own estimates, the ADR draft's questions |

This index is a common path (`lanes.toml`): each lane adds its epics' sections.

## Disputes: input for experiment batch 1

*Batch 1a results (2026-10-03): `docs/epics/E04a-log.md`, "The design note at closure"; Q21, Q23, D1, S-01–S-04, OQ-4 (Ballista), H-1 and the Ballista's attack are answered there.*

*Batch 1b (2026-10-04): `experiments/specs/E04a-batch-1b.toml` stages every row marked 1b except D8 (the Cloak of the Undead King, an E08–E10 input), plus the commanded tent (T7b) and the Ballista's range penalty as the baseline for the Golden Bow. Results evaluated in `docs/epics/E04a-log.md`, "The design note at closure": D-1 (half, war machines included), D-2 (additive, single precision), D-3 (wiki), D-5 (no consent), D-6 (wiki), D-7 (per stack, rounded down), OQ-1–OQ-3, OQ-5, OQ-6, artifacts D1 (as our packs), open checks 1–2 and H-2 are answered there.*

Collected from the surveys; ids point into them. "Observable in SoD" means the user can settle it with a staged battle (E04a T2). Batch 1 is split (user, Q30): **1a** for T5–T7, **1b** for T8–T10. Reading the game's code is allowed as a last resort (Q29); the First Aid reading (installation.md §7.5) settles two rows.

| # | Question | Sources disagree | Needed by | Batch |
|---|---|---|---|---|
| Q23 + S-02 + H-3 | Damage rounding: exact integers or single-precision floats (also Armorer 94 vs 95, specialty rounding) | wiki example vs VCMI and our exact rule | T5, T6, T10 | **1a** (highest value: affects every damage number) |
| Q21 | Death stare: health of the new top creature | wiki vs VCMI | done (E03) | 1a |
| Q22 | Jousting: path length or start-to-end distance | wiki vs VCMI | done (E03) | 1a |
| Q26 | First Aid Tent without the skill: 1–25 random or fixed 10 | wiki vs VCMI; the exe code says random 1–25 (§7.5) | done (E03) | settled by code reading (§7.5) |
| skills D2, battle D-12 | First Aid with the skill: fixed 50/75/100 or random 1–50/75/100 | VCMI and the skill text vs wiki; the exe code says random 1..25 × (1 + level) (§7.5) | T7 | settled by code reading (§7.5) |
| skills D1, battle D-4 | Artillery at Basic: one or two Ballista shots | skill text vs wiki and VCMI | T7 | **1a** |
| S-01, battle D-8 | Ballista without an Ammo Cart: 24 shots or unlimited | VCMI and the creature table vs wiki | T7 | **1a** |
| battle D-1, skills D4 | Surrender cost: half or full recruitment cost | wiki vs VCMI | T9 | **1b** |
| battle D-2 | How Diplomacy and the Statesman's Medal combine | VCMI additive; others silent | T9 | 1b |
| battle D-5 | Can the opponent refuse a surrender? | wiki (yes) vs VCMI (no) | T9 (seam: offer and answer) | **1b** |
| battle D-6 | Shackles of War against a neutral stack | VCMI vs the artifact text and wiki | T9 | 1b |
| battle D-3 | Experience for killed war machines | VCMI vs wiki | T9 (outcome inputs) | 1b |
| battle D-7, skills D5 | Necromancy: per stack (rounded towards 1) or one sum | wiki vs VCMI | T9 (outcome inputs) | 1b |
| battle OQ-1, OQ-2 | Tactics: does a disabled Tactics still cancel; is placement a path or free; can war machines be moved | VCMI only | T8 | 1b |
| battle OQ-4 | May a hero-controlled Ballista or Tent wait or defend | VCMI only | T7 | 1a |
| battle OQ-5 | War machines of a defeated hero: lost or captured | VCMI only | T9 | 1b |
| battle OQ-6 | Is the +500 experience for a defeated hero raised by Learning | VCMI only | T9 | 1b (optional) |
| artifacts D1 | Angelic Alliance: which factions may mix | artifact text and wiki vs VCMI | T10 | 1b |
| artifacts D8 | Cloak of the Undead King at Basic Necromancy: Zombies or Walking Dead | artifact text vs VCMI and wiki | T10 (outcome) | 1b (optional) |
| artifacts open 1–2 | War machines get HP/speed from artifacts; Golden Bow on the Ballista | VCMI only | T7, T10 | 1a (optional) |
| H-1, S-03 | Catherine's and Roland's Swordsmen specialty | wiki vs exe table vs VCMI | T10 | 1a (optional) |
| H-2 | Xeron's +1 speed | wiki and VCMI vs the exe record layout | T10 | 1b (optional) |
| skills (open) | Hero attack in the Ballista: multiplier only, or also the attack advantage | VCMI does both | T7 | 1a |

Later batches (E04b, E04c): battle D-9 to D-11, artifacts D2 and D3, skills D6 and D7, S-07, S-13, S-21.

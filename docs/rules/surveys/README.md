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

## Disputes: input for experiment batch 1

Collected from the surveys; ids point into them. "Observable in SoD" means the user can settle it with a staged battle (E04a T2). Code analysis of `Heroes3.exe` (installation.md §7.5) already answers some of them; whether such analysis may be used as a source is OPEN-QUESTIONS Q29.

| # | Question | Sources disagree | Needed by | Batch 1? |
|---|---|---|---|---|
| Q23 + S-02 + H-3 | Damage rounding: exact integers or single-precision floats (also Armorer 94 vs 95, specialty rounding) | wiki example vs VCMI and our exact rule | T5, T6, T10 | **yes** (highest value: affects every damage number) |
| Q21 | Death stare: health of the new top creature | wiki vs VCMI | done (E03) | yes |
| Q22 | Jousting: path length or start-to-end distance | wiki vs VCMI | done (E03) | yes |
| Q26 | First Aid Tent without the skill: 1–25 random or fixed 10 | wiki vs VCMI; the exe code says random 1–25 (§7.5) | done (E03) | yes (cheap confirmation) |
| skills D2, battle D-12 | First Aid with the skill: fixed 50/75/100 or random 1–50/75/100 | VCMI and the skill text vs wiki; the exe code says random 1..25 × (1 + level) (§7.5) | T7 | yes (confirms the code reading) |
| skills D1, battle D-4 | Artillery at Basic: one or two Ballista shots | skill text vs wiki and VCMI | T7 | **yes** |
| S-01, battle D-8 | Ballista without an Ammo Cart: 24 shots or unlimited | VCMI and the creature table vs wiki | T7 | **yes** |
| battle D-1, skills D4 | Surrender cost: half or full recruitment cost | wiki vs VCMI | T9 | **yes** |
| battle D-2 | How Diplomacy and the Statesman's Medal combine | VCMI additive; others silent | T9 | yes (same battle as D-1) |
| battle D-5 | Can the opponent refuse a surrender? | wiki (yes) vs VCMI (no) | T9 (seam: offer and answer) | **yes** |
| battle D-6 | Shackles of War against a neutral stack | VCMI vs the artifact text and wiki | T9 | yes |
| battle D-3 | Experience for killed war machines | VCMI vs wiki | T9 (outcome inputs) | yes |
| battle D-7, skills D5 | Necromancy: per stack (rounded towards 1) or one sum | wiki vs VCMI | T9 (outcome inputs) | yes |
| battle OQ-1, OQ-2 | Tactics: does a disabled Tactics still cancel; is placement a path or free; can war machines be moved | VCMI only | T8 | yes (screenshots) |
| battle OQ-4 | May a hero-controlled Ballista or Tent wait or defend | VCMI only | T7 | yes |
| battle OQ-5 | War machines of a defeated hero: lost or captured | VCMI only | T9 | yes |
| battle OQ-6 | Is the +500 experience for a defeated hero raised by Learning | VCMI only | T9 | optional |
| artifacts D1 | Angelic Alliance: which factions may mix | artifact text and wiki vs VCMI | T10 | yes |
| artifacts D8 | Cloak of the Undead King at Basic Necromancy: Zombies or Walking Dead | artifact text vs VCMI and wiki | T10 (outcome) | optional |
| artifacts open 1–2 | War machines get HP/speed from artifacts; Golden Bow on the Ballista | VCMI only | T7, T10 | optional |
| H-1, S-03 | Catherine's and Roland's Swordsmen specialty | wiki vs exe table vs VCMI | T10 | optional |
| H-2 | Xeron's +1 speed | wiki and VCMI vs the exe record layout | T10 | optional |
| skills (open) | Hero attack in the Ballista: multiplier only, or also the attack advantage | VCMI does both | T7 | yes (same battle as D-4) |

Later batches (E04b, E04c): battle D-9 to D-11, artifacts D2 and D3, skills D6 and D7, S-07, S-13, S-21.

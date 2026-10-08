# E04 quirk survey (E04a T1) — for the user to classify

Date: 2026-10-03. Scope: E04a (heroes in battle), E04b (battle magic), E04c (sieges and special battlefields). Known differences between the original game (SoD 4.0) and a clean design, with sources, for one classification sitting (OPEN-QUESTIONS Q6: `faithful` is the default; `exploit-fix` is always fixed). After the user's decision each kept entry becomes a ledger entry `Q-NNNN` in [`quirks-ledger.md`](../quirks-ledger.md) with a flag, written by the task that implements the rule.

Classes (04-rules-and-sources §4): **exploit-fix** (cheating, crash, soft-lock, state corruption; never toggleable), **preserve** (accepted original behaviour; flag on in `faithful`), **fix-optional** (unintended and mild; flag, on in `faithful`, off in `strict-fixed`). "Not modelled" means the behaviour lives outside the rules engine (the original's built-in AI or user interface) and no flag is proposed.

Sources: `wiki` = https://heroes.thelazy.net (retrieved 2026-10-03; pages named below, URL `https://heroes.thelazy.net/index.php/<Page>`). Many entries come from the wiki's "Bug:" notes and from the Horn of the Abyss changelog, which lists original bugs that HotA fixed; changelog items can also be HotA's own bugs, so their confidence is lower. `vcmi` = VCMI 1.7.5. Disputed *rules* (two sources disagree, nobody calls it a bug) are not here; they are in the [survey index](README.md#disputes-input-for-experiment-batch-1).

## Classification (user, 2026-10-03, OPEN-QUESTIONS Q30)

**Confirm before flagging** (`docs/04-rules-and-sources.md` §4): an entry becomes a ledger quirk with a flag only once it is confirmed for SoD; until then the clean behaviour is implemented. A confirmed entry takes its suggested class unless the user objects at the batch report.

| Status | Entries |
|---|---|
| confirmed by original data | S-05 (the exe's creature table holds the bugged levels) |
| exploit-fix, no confirmation needed (never implemented) | S-08, S-14, S-20 |
| batch 1a (E04a), evaluated 2026-10-03 | confirmed: S-01 (ledger Q-0004), S-02 with Q23 (Q-0005, all damage rates), S-03 for Roland only (Q-0006); not reproduced in SoD: S-04 (dropped) |
| to confirm in an E04b or E04c batch | S-06, S-07, S-11 (code reading incomplete, Q53), S-12, S-15, S-17 (Berserk's 255 rounds read, the data in every profile), S-18, S-21 (confirmed by code reading: S-19 as Q-0007, S-16 as Q-0009, S-10 as Q-0011, S-13 as Q-0012; S-09 the rule in every profile, Q53) |
| to confirm when the battle setup is derived from the map (E04c or E08) | S-22 |

## Overview

| ID | Title | Epic | Impact | Suggested class | Confidence it exists in SoD |
|---|---|---|---|---|---|
| S-01 | Ballista has unlimited shots | E04a | gameplay-altering (small) | fix-optional | medium (wiki; VCMI and our E03 rule use 24) |
| S-02 | Armorer rounding: one less damage on exact integers | E04a | gameplay-altering (tiny) | decide with Q23 | low–medium (one wiki example) |
| S-03 | Catherine's and Roland's specialty only affects Crusaders | E04a | gameplay-altering (campaign heroes) | fix-optional | medium (wiki; exe table supports it only for Roland) |
| S-04 | Elixir of Life: Trolls do not fully regenerate | E04a | gameplay-altering (tiny) | fix-optional | medium (HotA changelog) |
| S-05 | Ice, Magma and Energy Elementals have the wrong level | E04a/E04b, packs | gameplay-altering (small) | fix-optional | high (wiki creature pages; the exe's creature table) |
| S-06 | Arrow towers: Armorer, Air Shield and petrification *increase* their damage | E04c | gameplay-altering | fix-optional | high (wiki: Armorer, Air Shield, Arrow tower, Damage, Trivia) |
| S-07 | Arrow towers: Archery *reduces* their damage | E04c | gameplay-altering (small) | fix-optional | low (wiki pages contradict each other) |
| S-08 | Earthquake can disable the first defending stack | E04c | soft-lock of a stack | exploit-fix | medium (wiki Earthquake) |
| S-09 | Forgetfulness halves a shooter's melee damage | E04b | gameplay-altering | preserve | high (wiki Damage, Forgetfulness) |
| S-10 | Advanced Forgetfulness affects all enemy shooters | E04b | gameplay-altering | fix-optional | high (wiki Forgetfulness) |
| S-11 | Berserk: units fail to path around obstacles and pass | E04b | gameplay-altering | fix-optional | medium (wiki Berserk) |
| S-12 | Enchanters choose spells by the first stack's stats only | E04b | gameplay-altering (small) | fix-optional | high (wiki Enchanter) |
| S-13 | Expert Cure cannot remove Hypnotize, Basic and Advanced can | E04b | gameplay-altering (small) | fix-optional | low ("may/may not be a bug", wiki Cure) |
| S-14 | Harpies stay bound forever after Dendroid roots | E04b | soft-lock of a stack | exploit-fix | medium (wiki Harpy and Harpy Hag) |
| S-15 | Gold Dragons are affected by hero Dispel and Sacrifice | E04b | gameplay-altering (small) | fix-optional | medium (wiki Trivia) |
| S-16 | Frenzy counts Behemoth defense reduction in its attack bonus | E04b | gameplay-altering (small) | fix-optional | medium (HotA changelog) |
| S-17 | Berserk, Disrupting Ray and Dendroid roots expire after 255 rounds | E04b | cosmetic | preserve | medium (HotA changelog) |
| S-18 | Magic Arrow can deal 0 damage to Gold and Diamond Golems | E04b | gameplay-altering (tiny) | fix-optional | medium (HotA changelog) |
| S-19 | Magic Arrow uses the wrong protection when several apply | E04b | gameplay-altering (tiny) | fix-optional | medium (HotA changelog) |
| S-20 | A defender can win and be left without an army (Basic Resurrection) | E04b | state corruption | exploit-fix | medium (HotA changelog) |
| S-21 | Killing a multiply-cloned stack removes only one clone | E04b | gameplay-altering | fix-optional | low (HotA changelog; may be HotA's own bug) |
| S-22 | Battles underground always use the subterranean battlefield | E04c (setup) | gameplay-altering (native bonus) | preserve | high (wiki Terrain) |

Outside the E04 scope, listed so they are not lost (no classification needed now): experience overflow (level 75 jumps to 88, endless level-ups past 6424; wiki Experience) → exploit-fix with hero progression (E08–E10); primary skill reset to 0 at 129 (wiki Primary skill) → exploit-fix with progression; the siege "attack yourself" glitch (wiki Combat) → exploit-fix in the orchestrator (E10); AI-only behaviour (AI Necromancy always raises 1, wiki Necromancy; AI cannot cast Berserk, wiki Berserk; AI treats arrow-tower damage as 1, wiki Arrow tower) → not modelled; morale triggering after a creature spell ended the battle (HotA changelog) → not modelled (our battle ends at once).

## Entries

### S-01 — Ballista has unlimited shots (E04a T7)
- Original: "Contrary to what its in-game statistics indicate, a Ballista has unlimited shots (fixed in Horn of the Abyss)." (wiki War Machine). The creature table says 24 shots.
- Clean: the Ballista uses its 24 shots unless an Ammo Cart supplies it.
- Engine today: E03 BAT-WAR-002 gives the Ballista 24 shots (VCMI `config/creatures/special.json` L79). VCMI does not replicate the bug.
- Impact: a Ballista without a cart stops after 24 shots (12 rounds with Artillery's two shots). Experiment candidate (batch 1, D-8). Suggested **fix-optional** (faithful: unlimited).

### S-02 — Armorer: one less damage on exact integers (E04a T6)
- Original (wiki Armorer/Damage worked example): 100 Peasants against Basic Armorer deal 94, although 100 × 0.95 = 95 exactly. The likely cause is single-precision arithmetic (0.95 is not exact in binary), i.e. Q23.
- Clean: exact arithmetic gives 95 (VCMI and our BAT-DMG-002 give 95).
- Suggested: **decide together with Q23** (if the original uses floats everywhere, model the float behaviour as one quirk for all damage factors, not per skill). Experiment in batch 1 (D3 in the skills survey).

### S-03 — Catherine's and Roland's specialty only affects Crusaders (E04a T10)
- Original: wiki Swordsman and Crusader: both specialties are "bugged to only affect Crusaders. Fixed in Horn of the Abyss". The executable's specialty table names creature 7 (Crusader) for Roland but creature 6 (Swordsman) for Catherine, like Sorsha (installation.md §7.2), so the data explains the bug for Roland only.
- Clean: both affect Swordsmen and Crusaders. VCMI: both Swordsmen with upgrades.
- Suggested **fix-optional**. Experiment candidate (H-1).

### S-04 — Elixir of Life: Trolls do not fully regenerate (E04a T10)
- Original: HotA changelog: "Fixed a bug where Trolls with the Elixir of Life did not fully regenerate their health" (Elixir: +25 % hit points and regeneration for living creatures). Probably the regeneration restores the base hit points, not the raised maximum.
- Clean: regeneration restores the top creature to its effective maximum. VCMI: unknown.
- Suggested **fix-optional**; low priority.

### S-05 — Ice, Magma and Energy Elementals have the wrong level (E04a/E04b, packs)
- Original: wiki Water Elemental and Ice Elemental: "Though Ice Elementals are level 3 units, the game considers them as level 4 in creature level-based calculations … (fixed in Horn of the Abyss)"; the Fire/Energy and Earth/Magma pages have the same kind of note ("Energy Elementals' level bug is fixed", "Magma Elementals' level bug is fixed"). The executable's creature table (installation.md §5, 0-based levels) has exactly these values: Ice Elemental 3 (level 4 instead of 3), Magma Elemental 2 (level 3 instead of 5), Energy Elemental 4 (level 5 instead of 4); Storm Elemental is correct.
- **Our packs inherit them** (E02 harvests levels from the exe: `packs/ab` has Ice Elemental at level 4). Battle rules do not read creature levels yet; level-based calculations arrive with spell specialties (E04b) and later with growth, dwellings and random monsters.
- Clean: each upgrade has its base creature's level.
- Suggested **fix-optional**. Note: this is a quirk in *data*, so it needs either a per-profile override of three creature levels or a rule that reads a "level for calculations" field; T3 proposes the mechanism.

### S-06 — Arrow towers: Armorer, Air Shield and petrification increase the damage (E04c)
- Original: wiki Armorer ("not reduced by armorer … but actually increased the corresponding amount"), Air Shield, Arrow tower ("Troops, which are buffed by Air Shield spell or are petrified take additional damage as well (fixed in HotA)"), Damage, Trivia.
- Clean: they reduce it. VCMI: normal reduction.
- Suggested **fix-optional** (well documented; faithful keeps it).

### S-07 — Arrow towers: Archery reduces their damage (E04c)
- Original: wiki Archery: "The archery skill reduces damage done by arrow towers (fixed in HotA)". The wiki Arrow tower page says Archery does not affect tower damage at all. Resolve in E04c (experiment).
- Suggested **fix-optional** if confirmed.

### S-08 — Earthquake can disable the first defending stack (E04c)
- Original: wiki Earthquake: repeated Earthquakes against an AI defender whose towers are not all manned can make the first defending stack disappear and stop acting (it still retaliates); "(fixed in Horn of the Abyss)".
- Suggested **exploit-fix** (a stack in a broken state).

### S-09 — Forgetfulness halves a shooter's melee damage (E04b)
- Original: wiki Damage: "Due to a bug, a shooter's melee attack damage (when affected by Forgetfulness), is also decreased by half"; wiki Forgetfulness calls it "more like a bug". HotA kept it and documented it.
- Suggested **preserve**.
- **Confirmed by code reading** (E04b T4b, `installation.md` §7.14): the base damage counts `max(1, n / 2)` creatures of a forgetful shooter on every strike. **User (Q53): the rule in every profile, no flag** (BAT-MAG-023).

### S-10 — Advanced Forgetfulness affects all enemy shooters (E04b)
- Original: wiki Forgetfulness: "Contrary to description and other spells, advanced level affects all enemy ranged units (fixed in HotA)".
- Suggested **fix-optional**.
- **Confirmed by code reading** (flag `0x20`, `installation.md` §7.6, §7.14): ledger **Q-0011** (user, Q53).

### S-11 — Berserk: units fail to path around obstacles (E04b)
- Original: wiki Berserk: berserk units treat blocked hexes as unreachable and wait; two-hex units miscount positions and pass although a target is reachable (fixed in HD mod/HotA).
- Suggested **fix-optional** (the berserk unit's automatic behaviour is a rule we must specify anyway).
- **Partly read** (E04b T4, `installation.md` §7.14): the berserk turn picks its target from the AI pathfinder's map (0x4B3400); the bug would sit there and was not read. HD fixes it, so the user's setup cannot show it. **User (Q53): the engine's own pathfinding in every profile; S-11 stays unconfirmed.**

### S-12 — Enchanters choose spells by the first stack's stats only (E04b)
- Original: wiki Enchanter: "the spell relevance is calculated based on stats of the first stack only (fixed in HotA)".
- Suggested **fix-optional**.

### S-13 — Expert Cure cannot remove Hypnotize (E04b)
- Original: wiki Cure: Basic and Advanced Cure remove Hypnotize, Expert Cure cannot ("This may/may not be a bug").
- Suggested **fix-optional**; low confidence, experiment candidate for an E04b batch.
- **Confirmed by code reading** (E04b T4c, `installation.md` §7.14): Expert Cure's loop skips hypnotized stacks. Ledger **Q-0012** (user, Q53).

### S-14 — Harpies stay bound forever after Dendroid roots (E04b)
- Original: wiki Harpy and Harpy Hag: a Harpy that attacks a Dendroid stack returns and is then bound; if the Dendroids move or die the binding never lapses "(fixed in Horn of the Abyss)".
- Suggested **exploit-fix** (a stack stuck by a stale effect).

### S-15 — Gold Dragons are affected by hero Dispel and by Sacrifice (E04b)
- Original: wiki Trivia: "Gold Dragons are susceptible to Dispel cast by heroes, the Sacrificing part of Sacrifice (but not the Resurrection)", although they are immune to spells of levels 1–4.
- Suggested **fix-optional**.
- **Settled by code reading** (E04b T3b1b, T6a; `installation.md` §7.10, §7.18): Dispel tests no immunity at all (the rule, T3b1b); Sacrifice's victim takes Sacrifice's level 5, which no spell-level immunity reaches, and its raised stack Resurrection's level 4, which the Gold Dragons' immunity does: ledger **BAT-QUIRK-5** (fix-optional, the user, 2026-10-08).

### S-16 — Frenzy counts Behemoth defense reduction (E04b)
- Original: HotA changelog: "when calculating the bonus attack from the Frenzy spell, defense penalties from enemy creatures (such as Behemoths and Ancient Behemoths) were accounted for".
- Suggested **fix-optional**.
- E04b T3b2 (code reading, `installation.md` §7.11): confirmed; ledger Q-0009 (`quirk.battle.frenzy_reduced_defense`).

### S-17 — Long effects expire after 255 rounds (E04b)
- Original: HotA changelog: "Berserk, Disrupting Ray, and Dendroid Roots spells were removed from units after 255 rounds".
- Suggested **preserve** (harmless; an 8-bit duration).
- E04b T3a (code reading, `installation.md` §7.9): Berserk and Bind get a duration of 255 and end after it; Disrupting Ray gets 255 too but is never ended (the end routine returns at once for it), so in SoD 4.0 its defense loss lasts the whole battle. The quirk concerns Berserk (T4) and Bind (T7).
- E04b T4c: Berserk's 255 rounds are the spell's data (`duration = "fixed"`, `fixed_rounds = 255`) in every profile (agent decision, to confirm at the T4 checkpoint: a flag would only matter after 255 rounds).

### S-18 — Magic Arrow can deal 0 damage to Gold and Diamond Golems (E04b)
- Original: HotA changelog: "Fixed a bug with 0 damage against Gold and Diamond Golems when attacking with a Magic Arrow with low Power".
- Suggested **fix-optional**.

### S-19 — Magic Arrow uses the wrong protection (E04b)
- Original: HotA changelog: "Fixed a bug with the priority of protection against magic when casting a Magic Arrow (now the most effective of defenses is used)".
- Suggested **fix-optional**.
- **Confirmed for SoD by code reading** (`docs/formats/installation.md` §7.8, 0x5A7BB0; noted 2026-10-06, E04b T3b1 prep): the first of the spell's schools in the order earth, air, fire, water whose protection is active applies, not the strongest. Flagged with E04b T3b1.
- **Flagged** as ledger Q-0007 (`quirk.battle.first_protection`, E04b T3b1, 2026-10-06).

### S-20 — A defender wins and is left without an army (E04b)
- Original: HotA changelog: "a defending hero could be left without an army after winning a combat if all his stacks had been resurrected with a Basic-level Resurrection spell" (Basic Resurrection is temporary).
- Suggested **exploit-fix** (a hero without an army breaks an invariant).
- **Confirmed by code reading** (E04b T6a, `installation.md` §7.18): the end of the battle keeps one creature when none is left, but counts war machines and summoned stacks. Ledger **BAT-QUIRK-6**: the engine counts only army stacks (the user, 2026-10-08).

### S-21 — Killing a multiply-cloned stack removes only one clone (E04b)
- Original: HotA changelog: "upon death of a stack that had been cloned several times, only one clone would disappear". May be a HotA regression rather than an original bug.
- Suggested **fix-optional**; verify in an E04b batch.

### S-22 — Underground battles always use the subterranean battlefield (E04c / battle setup)
- Original: wiki Terrain: subterranean "is the only combat terrain for the underground layer (fixed in Horn of the Abyss)". The battlefield, and with it the native-terrain bonus (BAT-TER-001), comes from the layer, not from the tile's terrain.
- Lives in the setup derivation (the orchestrator chooses `BattleSetup.terrain`), not in the battle.
- Suggested **preserve**.

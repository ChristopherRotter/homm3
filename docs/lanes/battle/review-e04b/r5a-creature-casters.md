# Review r5a-creature-casters: BAT-MAG-048 to BAT-MAG-053, BAT-QUIRK-9, -10, -12, -13, -14

Nothing in the repository was changed and no build was run. Everything below was traced in the Rust
code and re-read in the executable with `scripts/exe.py` (SoD build `aac39e8c…`); the original's
code is described in my own words with addresses. The human player's paths were followed: the
mouse cursor (0x476490 → the click handler 0x476BD0 → the dispatcher 0x478D80) **and the keyboard
path of the Faerie Dragon** (key F, 0x475C17 → 0x59EC50), which the readings in
`installation.md` §7.21 name but did not follow to its end. Four of the findings come from that
second path.

## Coverage

| rule | text↔code | sources checked (exe addresses re-read; VCMI files; batch rows) | tests | verdict |
|---|---|---|---|---|
| BAT-MAG-048 a creature's cast | every clause matches `casters::{ready, plan, spend}`, `magic::{creature_target, plan_creature_cast, apply_cast, reach_stacks}`, `command.rs:464` | exe: 0x448260 whole (casts lowered first at 0x44852B, the two tables 0x448740/0x448768 decoded: 13, 37, 51, 91, 123, 125, 127, 129, 134; arguments spell, hex, 1, −1, 2, 6); 0x4476C0 whole (table 0x447A00 decoded); cursor 0x476651–0x476799 (owner side +0xF4 against +0x132C0; the caster itself refused at 0x4766EA); dispatcher case of action 10, 0x47936E–0x47943B (done bit, good morale roll, no 0x441F70); 0x59FE30 from its start to 0x5A0354 (kind 1: no cost, no 0x41F890, no test or roll at 0x5A02CB); 0x5A3C80; 0x5A8090 whole. VCMI `config/creatures/stronghold.json:185-207`, `conflux.json` (the four elementals), `server/battles/BattleFlowProcessor.cpp:789-811`, `BattleActionProcessor.cpp:734`. No observation exists | `tests/casters.rs` 51–297, `tests/limits.rs` 129, 349: the clauses that matter are pinned | behaviour ok; F-r5a-5 (sources) |
| BAT-MAG-049 Faerie Dragon | the code does what the summary says; the summary (and the code) are wrong against the executable in four places | exe: 0x447510 whole and table 0x63B850 (weights 22, 22, 21, 10, 10, 5, 5, 5, sum 100, `rand() mod sum`); 0x465330 0x4654ED–0x465543; 0x465080 whole; the F key 0x4758E4–0x4758FB, 0x475C17–0x475D04; 0x59EC50 0x59EC50–0x59F090 and its switch; 0x59F700 whole; 0x5A39C0 whole; 0x5A3070 whole; click handler 0x476BD0 0x476C99; dispatcher 0x479036–0x479056 (action 1) and 0x47936E (action 10); 0x5A4970 to 0x5A4B0E; 0x5A6360 0x5A6360–0x5A6676; 0x5A78E0 head; 0x4E5760 head; Magic Arrow, Ice Bolt, Lightning Bolt handlers' hero argument (0x5A09DC, 0x5A0A6B, 0x5A0B49). VCMI `lib/spells/BattleSpellMechanics.cpp:514-523`, `config/creatures/neutral.json:200-290`, `server/battles/BattleActionProcessor.cpp:487-546`, `lib/battle/CUnitState.cpp:423-427`. Batch 2e OI (rows 18–28) | `tests/casters.rs` 349–704, `creature_defences.rs` 200, `limits.rs` 188 pin the present behaviour, three of them the wrong one | F-r5a-1, F-r5a-2, F-r5a-3, F-r5a-4, F-r5a-5 |
| BAT-MAG-050 Master Genie | matches `casters::{useful, useful_spells, candidates}`, `magic::{helpful_target, draw_useful}` | exe: 0x4476C0 case 1; 0x447EE0 whole (count, `random(1, n)`, the k-th, 6 and 2); 0x447A80 whole with its table 0x447E8C decoded (27, 29 → melee enemy; 28 → shooting enemy; 30–34, 36 → enemy spell book; 37 wounded; 43, 44; 48 done bit; 55 flag 0x80); 0x4428F0 whole; the done bit's writers in the dispatcher (Defend sets it at 0x479717, Wait sets 0x2000000 only); flag word +0xC of spells 10–69 in the static table 0x685450 read: bit 0x800 on exactly the 21 spells of the pack, in the pack's order. VCMI `lib/battle/CBattleInfoCallback.cpp:2176-2316`, `config/creatures/tower.json:326-351` | `tests/casters.rs` 718–901: every usefulness case, the draw, the counts' visibility | ok; F-r5a-5 (VCMI differs, not recorded) |
| BAT-MAG-051 Archangel | matches `casters::{raise_target, raise}`, `raise::{creature_test, eligible, raisable, on_top}` | exe: 0x448260 case 0 (0x44854A–0x4485BE: 0x5A3CC0(controller's side, hex, 1), then 0x5A7560(stack, 100 × creatures, 0)); 0x4476C0 case 0; 0x4473D0 whole; 0x447330 whole; 0x5A3CC0 whole; 0x5A8467–0x5A8512 (sixth argument 1: the active stack's creatures × 100 for creature 13, × 50 otherwise; `health <= hit points`, `count < start`); 0x5A7560 head. VCMI `config/creatures/castle.json:424-452` agrees (Expert Resurrection, 100 per creature, one cast). Batch 2e RC row 98 (the Cloak bars it) | `tests/raisers.rs` 81–151, `limits.rs` 129, 349 | ok; F-r5a-7 (boundary untested) |
| BAT-MAG-052 Pit Lord | matches `casters::{raise_target, corpse_count, raise}`, `raise::corpse_raisable`, `created::{has_slot, raise_corpse, add}` | exe: 0x5A3E40 whole; 0x5A7390 whole; 0x47A100 whole; 0x447330 second half (divisor read from the creature table's record 48, +0x4C); corpse registration in 0x444120 (0x4442FA–0x4443F3) and 0x4687C0 whole (the part byte: 0xFF one hex, so a one-hex corpse has no second-hex test; position = the rear hex); 0x4466A0. VCMI `lib/spells/effects/DemonSummon.cpp` (same three-way minimum), `config/creatures/inferno.json` (50, one cast) | `tests/raisers.rs` 153–365 | ok |
| BAT-MAG-053 Enchanter | matches `casters::{round_started, periodic, periodic_spells, counts}`, `magic::periodic_cast`, `turn::{activation, begins, end_turn, berserk_turn}` | exe: 0x465330 0x4654ED–0x465537 (`counter > 2`, set to 0 only when 0x447FE0 answers 1); 0x465080 whole (no activation for blind, petrified, paralysed, bad morale, Fear); 0x447FE0 whole; table 0x6608B8 (53:15, 28:10, 54:10, 46:15, 43:5, 41:15, 37:10, 45:4); 0x4760B2–0x4760FE; 0x5A66D0 whole (hero argument null for kind 1, every stack rolls); Cure's handler 0x5A17E6–0x5A18DC; the timed handler's mass branch 0x5A107F–0x5A10C2. VCMI `server/battles/BattleFlowProcessor.cpp:976-1018`, `lib/battle/BattleInfo.cpp:651-657`, `config/creatures/neutral.json:388-435` | `tests/enchanters.rs`, `limits.rs` 235 | ok; F-r5a-5 (VCMI misquoted) |
| BAT-QUIRK-9 | the flag does what the ledger says | the deciding path re-read: the redirect never reaches the cast routine for a creature's cast | both variants tested | F-r5a-1: the described original behaviour does not exist |
| BAT-QUIRK-10 | one variant, no flag | confirmed by my re-read: 0x447B20–0x447B82 walks the other side's records for creature flag 0x80 and tests nothing else (no dead bit, no 0x100/0x200). VCMI does not agree (alive stacks, any king level), so the confirmation is the second reading alone | `casters.rs` 854 pins dead and king-2 | ok; F-r5a-6 (class) |
| BAT-QUIRK-12 | always refused (`created::has_slot`) | confirmed: 0x47A100 0x47A12B–0x47A162 answers 0 after 20 records without a free or dead-summoned one; 0x5A7390 0x5A7464–0x5A746B stores into the result unchecked | `raisers.rs` 285 | ok, class fits (crash) |
| BAT-QUIRK-13 | both variants in `casters::counts` | confirmed by my re-read: both loops of 0x447FE0 (0x448077–0x44808A and 0x448125–0x448138) reload the same record pointer at every pass; the wiki agrees (per the catalog) | `enchanters.rs` 180, 200: each variant fails the other's assertions | ok, class fits |
| BAT-QUIRK-14 | counters start at 255 | +0x132A0 has three users in the whole executable (0x465509, 0x46552C, 0x4760B2/0x4760F8); the constructor 0x462760 does not write it. Not re-read: that the manager is allocated once per session (0x4EDCF5). VCMI's counter (starts 0, casts at 0) agrees with the fixed behaviour | `enchanters.rs` 67 | ok, class fits |

## Findings

### F-r5a-1: Magic Mirror never sends a creature's spell away in the original; the engine redirects it in both profiles, and BAT-QUIRK-9 describes an original behaviour that does not exist
- rules: BAT-MAG-049 (with BAT-MAG-022, BAT-MAG-057), BAT-QUIRK-9
- kind: behaviour-bug (and source: the quirk's reading)
- severity: medium (a Faerie Dragon's Magic Arrow, Ice Bolt, Lightning Bolt or Chain Lightning aimed at a stack under Magic Mirror, or at an enemy Faerie Dragon with its own 20 %)
- confidence: high. Both entry paths and the dispatcher were read, and VCMI agrees. An observation would close it: Faerie Dragons casting with F at a stack under Expert Magic Mirror, ten casts, no redirect
- evidence:
  - The redirect is decided only in the player's targeting routine 0x59EC50 (0x59EE7A–0x59EF8B): it draws against the target's chance and stores the hex of a random own stack in the manager's **+0x48**. The hero's action (action 1, dispatcher 0x479036–0x479056) passes +0x40, +0x44 and **+0x48** to the cast routine as spell, hex and second hex; the cast routine redirects only when its second-hex argument is not −1 (0x5A0275–0x5A02C5).
  - The creature's action (action 10, dispatcher 0x47936E–0x479382) passes **only +0x44** to 0x448260, and every case of 0x448260 calls the cast routine with second hex **−1** (the Faerie Dragon's at 0x448653–0x448662; the Genie's 0x447FC4; the Enchanters' 0x448249). 0x448260 has one caller, and neither it nor the cast routine reads the manager's +0x48. So the redirect the F-key path draws is dropped, and the mouse path (click handler 0x476C99: action 10, +0x44 only) never draws one.
  - `installation.md` §7.21 ("0x59FE30 then skips the test and roll at 0x5A02CB for caster kind 1, the redirected stack included") and the catalog source of BAT-MAG-049 assume the redirect arrives; it does not. The skipped test at 0x5A02CB is real but only ever meets the aimed stack.
  - VCMI 1.7.5 `lib/spells/BattleSpellMechanics.cpp:520`: a spell is reflectable only in `Mode::HERO` (or an already mirrored cast); a unit's cast (`CREATURE_ACTIVE`) is never reflected.
  - Engine: `magic.rs:2331-2348` calls `mirror` for every cast with a stack target, `magic.rs:2155-2177` redirects whenever the target's side is not the caster's; `tests/casters.rs:650` and `tests/creature_defences.rs:202` assert a `SpellRedirected` event for a creature's cast in both profiles.
  - Scenario: attacker's Faerie Dragon draws Magic Arrow and aims it at the defender's Faerie Dragons (innate mirror 20 %). Original: the arrow always hits the aimed stack (70 from one dragon). Engine, faithful: in one cast of five it goes to a random living attacker stack and hits it untested; strict-fixed: the same redirect, tested.
- fix or test sketch: in `reach_stacks` ask `mirror` only for `cast.by.is_none()`; retire `quirk.battle.creature_mirror_untested` (neither variant is the original's; whether a "clean" profile should mirror creature casts at all is the user's call, and VCMI does not); replace the two tests by one that asserts no `SpellRedirected` and damage on the aimed stack over 40 seeds; correct §7.21, the map entry 0x5A02CB and the ledger.

### F-r5a-2: with the F key the Faerie Dragon aims its spell as a hero does: an area spell at any hex outside the side columns, an empty one included; the engine and the rule allow only hexes of affectable stacks
- rules: BAT-MAG-049
- kind: behaviour-bug (and rule-text: "never at an empty hex", "a Frost Ring must be centred on a stack")
- severity: medium (41 of 100 draws are area spells: Fireball 21, Frost Ring 10, Meteor Shower 5, Inferno 5; the engine's legal targets are a strict subset of the original's)
- confidence: high for the code path; medium for one corner (below). Raised by one observation: Faerie Dragons with a hero, Fireball or Frost Ring drawn, press F, click an empty hex
- evidence:
  - The combat-log line of the draw itself says "(press F to cast)" (strings at 0x660AD0/0x660AAC, used by 0x447510). Key code 33 (the keyboard message's switch at 0x4758FB) leads to 0x475C17: outside the tactics phase, for creature 134 with casts left, it calls 0x59EC50(drawn spell, 1). The stats dialog's cast button does the same (0x468954).
  - 0x59EC50 sends spells 20–23 to its area handler 0x59EF9B, which sets action 10 (its second argument 1 turns the hero's action 1 into 10) and opens the hero's targeting loop (callback 0x5A3070, cursor 0x59F700). 0x59F700 accepts a hex outside columns 0 and 16 that 0x5A39C0 accepts; 0x5A39C0 asks for a stack only for spells with flag bits 0x20070, has own hex tests for Remove Obstacle, Fire Wall and Force Field, and answers yes for every hex of the field for the rest, the four area spells among them (0x5A39F8, 0x5A3C59; §7.8 says the same of a hero's cast). The click stores the hex in +0x44 (0x5A30F9). `installation.md` §7.21 concludes from the mouse cursor alone that "the dragon never casts at an empty hex".
  - Action 10 then runs 0x448260, whose Faerie Dragon case checks only that the hex is on the field (0x44863E–0x44864B) and casts. The target test 0x4476C0 is called by the mouse cursor and the computer player only; nothing on this path calls it.
  - So the mouse path gives what the rule says (a hex of a stack the spell can affect, never the caster's), and the F path gives every hex outside the side columns, the dragon's own hex too. For the single-target spells both paths ask the same stack test; for Chain Lightning the F path refuses own stacks (0x59F7BF) and the mouse path does not, so the union is the rule's.
  - VCMI `server/battles/BattleActionProcessor.cpp:487-546` casts the unit's spell at the action's target with the spell's ordinary targeting (a location for an area spell).
  - Engine: `magic.rs:1369-1388` (`drawn_target`) and `1393-1421`; `tests/casters.rs:509-518` asserts that `Hex(8, 5)` (empty) is refused.
  - Scenario: two Faerie Dragons draw Fireball; enemy stacks stand at (10, 4) and (10, 6), (10, 5) is empty. Original (F, click (10, 5)): both are hit. Engine: `CreatureCast { target: Hex(10, 5) }` → `InvalidSpellTarget`; only the two stacks' own hexes are legal. With Frost Ring the engine forces the ring's centre onto a stack, which the ring then spares.
  - The other way round, small: `drawn_target` accepts every hex of a target's footprint, also the rear hex of a two-hex stack in a side column (where stacks start); neither path of the original accepts columns 0 and 16 (0x47666F–0x47667D, 0x59F788–0x59F79D). A hero's cast in the engine already asks `field.is_standable`.
  - Corners I did not settle: 0x59EC50 returns at once when the acting side's hero has cast this round (+0x54B4, unless the byte +0x13D74 is set, which I did not identify), so then only the mouse path is left; and it asks the acting side's hero for his mastery without a null test (0x59ECB4 → 0x4E5080 → 0x4E5100 reads the hero record), so for a side without a hero the F path probably crashes and only the mouse path counts.
- fix or test sketch: for a drawn spell that takes a hex, accept every standable hex (at least when the controller has a hero), keep the stack rule for drawn stack spells; test: `novaling` with an enemy two hexes away, `CreatureCast { target: Hex(empty next to the enemy) }` is `Ok` and damages the enemy; a side-column hex is refused.

### F-r5a-3: a creature's area spell and its Chain Lightning take the damage bonuses of the acting side's hero (Sorcery, the school's Orb, a specialty); the engine gives a creature's cast none
- rules: BAT-MAG-049 (BAT-MAG-048's "no hero bonuses", BAT-MAG-004)
- kind: behaviour-bug; by `docs/04` §2 a conflict of sources (VCMI gives a unit's cast no bonus), so also a question for the user until observed
- severity: medium (46 of 100 Faerie Dragon draws: Fireball, Frost Ring, Chain Lightning, Meteor Shower, Inferno; only when the dragons' controller has a hero with Sorcery, an Orb or that specialty)
- confidence: medium to high on the reading (both handlers read; the hero is fetched unconditionally); needs an observation because VCMI disagrees and the wiki is unknown to me
- evidence:
  - The cast routine sets its own hero variable to null for caster kind 1 (0x59FE63–0x59FE72), and the single-target handlers pass that variable as the attacker's hero to the damage modifiers 0x5A78E0 (Magic Arrow 0x5A09DC, Ice Bolt 0x5A0A6B, Lightning Bolt 0x5A0B49): no bonus, as the rule says.
  - The area handler 0x5A4970 (called with hex, spell, mastery, power: no caster kind) fetches the hero of the acting side itself (+0x53CC[+0x132C0], 0x5A4A05–0x5A4A1F) and passes it as the attacker's hero for every stack it damages (0x5A4AB0–0x5A4AD2). The Chain Lightning handler 0x5A6360 does the same for every jump (0x5A6557–0x5A656E). 0x5A78E0 applies 0x4E5760 whenever that hero is not null (0x5A790D–0x5A7916): × 1.5 for the school's Orb, × (1 + Sorcery), then the specialty bonus per target.
  - The acting side is the casting stack's controller (set at its activation, 0x465389–0x4653AE).
  - VCMI: `lib/battle/CUnitState.cpp:423-427` returns the base value for a unit's cast ("does not have sorcery-like bonuses (yet?)").
  - Engine: `magic.rs:1467` (`bonuses: SpellBonuses::default()` for every creature cast).
  - Scenario: a hero with the Orb of Tempestuous Fire leads two Faerie Dragons; Fireball drawn. Value 10 × 10 + 30 = 130. Original by this reading: 195 to each stack in the area; with Expert Sorcery alone 149. Engine: 130. A Lightning Bolt from the same stack is 270 in both.
- fix or test sketch: when a cast with `by: Some(_)` resolves a spell with an area or a chain, take the bonuses of `state.heroes[cast.side]`; test with a hero whose `bonuses.all` is 15 %: the drawn Nova deals 138 instead of 120 per stack, the drawn Bolt is unchanged. Queue the observation first (Faerie Dragons under a hero with an Orb: Fireball against Peasants, the log's damage).

### F-r5a-4: the F-key path asks no "magic allowed" test, so the Orb of Inhibition does not stop a Faerie Dragon's cast there; batch 2e's OI row tried only the mouse cursor
- rules: BAT-MAG-049 (its last sentence), BAT-MAG-061; OPEN-QUESTIONS BAT-Q21 point 1
- kind: question (the wiki against an observation that tested one of two paths; my reading supports the wiki)
- severity: medium (Faerie Dragons under an Orb of Inhibition now; the anti-magic garrison and Cursed Ground in E04c)
- confidence: medium to high on the reading; the observation is cheap: OI's setup again, press F on the dragons' turn
- evidence:
  - 0x41F890 has six callers in the executable: 0x4476C0 (twice), 0x4473D0, the Enchanters' 0x447FE0, the cast routine for caster kind 0 only (0x59FED5 skips it for the others), and the computer player's 0x4396E0. The F path (0x475C17 → 0x59EC50 → the targeting loop → action 10 → 0x448260 → the cast routine as kind 1) passes none of them, and no other test of artifact 126 lies on it.
  - `installation.md` §7.23 says "this reading found no path around 0x41F890, neither the human player's cursor 0x47670B nor the computer player's 0x421280". The F key is such a path.
  - The wiki, as BAT-Q21 quotes it: "Faerie Dragons are unaffected by the Orb of Inhibition and Anti-Magic Garrisons", and they cast on Cursed Ground "for no damage". The F path explains both: on Cursed Ground the effect test 0x5A8090 answers 0 for every spell above level 1 (0x5A80D1–0x5A80E7), so an area spell is cast and every stack in it resists.
  - Batch 2e OI (`experiments/results/E04b-batch-2e.md` rows 19, 21, 26, 28): "move the cursor over Blue's Peasants. Does a spell cursor show": no. That is 0x4476C0's answer. F was not pressed.
  - Engine: `casters.rs:391-399`; the ability `ignores_magic_block` exists and is tested (`limits.rs:188`) but no creature has it since batch 2e (`gapfill.toml:1466-1468`).
- fix or test sketch: observe first. If F casts under the Orb: give the Faerie Dragon `ignores_magic_block` back, with the F path's targets while the block holds (F-r5a-2), and correct §7.23 and BAT-Q21's record.

### F-r5a-5: VCMI is misquoted in two entries, and its contradictions of the readings are not recorded
- rules: BAT-MAG-048, BAT-MAG-049, BAT-MAG-050, BAT-MAG-051, BAT-MAG-053
- kind: source
- severity: low
- confidence: high (files opened)
- evidence:
  - BAT-MAG-053: "config/creatures/neutral.json (the eight spells at level 3)". VCMI 1.7.5 lists six `ENCHANTER` bonuses (Haste, Slow, Stone Skin, Bless, Weakness, Air Shield; `neutral.json:393-435`): no Bloodlust, no Cure. The counter's description is right (`BattleFlowProcessor.cpp:996-1016`, `BattleInfo.cpp:656`).
  - BAT-MAG-049: "fairieDragon: RANDOM_SPELLCASTER". In 1.7.5 the dragon has eight `SPELLCASTER` bonuses with weights in `addInfo` and `CREATURE_SPELL_POWER` 500 (`neutral.json:219-284`); `RANDOM_SPELLCASTER` is the Master Genie's (`tower.json:347`). The Magic Arrow weight 22 is as the entry says.
  - BAT-MAG-048 cites VCMI as agreeing. It also contradicts the reading in three places the entry does not name (`docs/04` §2 asks for a re-read and a record): the Ogre Mage's `CREATURE_ENCHANT_POWER` is 3, the reading's power and duration 6 (`stronghold.json:203-207`; my re-read: 6 is pushed at 0x4486A7); obstacles act on the unit after a `MONSTER_SPELL` (`BattleActionProcessor.cpp:734`), the reading says the caster does not re-enter its hexes (my re-read: no 0x441F70 in the case at 0x47936E); every unit rolls magic resistance against a negative spell whatever the mode (`BattleSpellMechanics.cpp:455-463`), the reading says the aimed stack of a creature's cast never rolls (my re-read: 0x5A02CB). My re-reads agree with the catalog on all three; the entry has no wiki source at all.
  - BAT-MAG-050 has no VCMI source. VCMI differs: 20 spells without Frenzy (the executable's flag 0x800 is set on Frenzy too, read from the table), `CREATURE_ENCHANT_POWER` 5 against 6, Prayer without a condition, Slayer for a living enemy of any king level (`CBattleInfoCallback.cpp:2181-2203, 2296-2306`; `tower.json:342-346`).
  - BAT-MAG-051 rests on the reading alone in the catalog; VCMI agrees and could be cited (`castle.json:436-452`).
- fix or test sketch: correct the two refs; add a note to BAT-MAG-048 and BAT-MAG-050 naming the three and four differences and that the code was re-read; cite VCMI on BAT-MAG-051.

### F-r5a-6: BAT-QUIRK-10 is classed `preserve` but has no flag, so `strict-fixed` keeps it
- rules: BAT-QUIRK-10 (BAT-MAG-050)
- kind: rule-text (ledger against `docs/04` §4)
- severity: low
- confidence: high
- evidence: `docs/04` §4: `preserve` is "controlled by a quirk flag" and `strict-fixed` is "all quirks fixed". The ledger: "Class: preserve, no flag: the rule in every profile"; `casters.rs:465-470` has one variant. The user decided it (BAT-Q12); the two documents do not say that such a class exists.
- fix or test sketch: one sentence in `docs/04` §4 for a user-decided quirk without a flag (BAT-QUIRK-11 is the same case), or a flag.

### F-r5a-7: two clauses with a boundary have no test of their own
- rules: BAT-MAG-051, BAT-MAG-049
- kind: test-gap
- severity: low
- confidence: high
- evidence: (a) the Archangel's test `health <= 100 × creatures` (0x5A850C, a `<=`; `raise.rs:136`): `tests/raisers.rs:120-151` uses healths 10, 40 and 200 against 100, never the equal case. (b) "an area's stacks do [roll]" for a creature's cast: `tests/casters.rs:492` has no resisting stack in the area, and no test shows a creature's Chain Lightning hitting its first stack without a roll while a later jump rolls.
- fix or test sketch: (a) one herald, a stack of health 100 that lost a creature: a target; health 101: none. (b) `novaling` at a hex between a `fixed` and a `dwarf` over 40 seeds: the dwarves resist some casts, the aimed-at stack's own roll is the area's.

## Wiki checks wanted
- Faerie Dragon: does Magic Mirror reflect the dragons' spells (F-r5a-1)? Do the hero's Sorcery, Orbs or spell specialties raise their Fireball, Frost Ring, Inferno, Meteor Shower or Chain Lightning (F-r5a-3)? Can the area spells be aimed at an empty hex (F-r5a-2)? The exact wording on the Orb of Inhibition, the anti-magic garrison and Cursed Ground (F-r5a-4).
- Magic Mirror: "spells cast by creatures are not reflected"?
- Ogre Mage: the duration of its Bloodlust (the reading 6 rounds, VCMI 3).
- Master Genie: the duration (6 against VCMI's 5), whether the list has Frenzy (the executable's flag says yes, VCMI no), Prayer's and Slayer's conditions.
- Enchanter: the list of eight spells and the round of the first cast.

## Not reviewed, left open
- No `cargo` run; the three sketches above describe changed behaviour, so none needs a run to show the present one (the present tests assert it).
- The computer player's paths (0x420E15, 0x420FD7, 0x421280, 0x4396E0), the tactics-phase callers, network and hidden-battle branches.
- 0x41F890 itself, 0x44A4D0 (immunities and resistance), 0x4448F0 (a timed effect on a stack), the chain's next-target search 0x5A61F0, the raise 0x5A7560 beyond its head: other slices' functions, used here as the map describes them.
- That the original's record 0 of a side is its first army stack (BAT-QUIRK-13's "lowest stack id"): the battle's setup was not read.
- That the battle manager is created once per session (0x4EDCF5), the premise of BAT-QUIRK-14's description; the fix does not depend on it.
- The byte +0x13D74 and the null-hero corner of the F path (F-r5a-2).
- Magic Plains, Cursed Ground and the anti-magic garrison (E04c).

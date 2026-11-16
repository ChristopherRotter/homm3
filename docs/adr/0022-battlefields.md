# ADR-0022: Battlefields — special battlefields as content, the battlefield on the setup

- Status: Accepted (user, 2026-10-13, BAT-Q38; drafted at E04c T1)
- Date: 2026-10-13
- Deciders: user; battle lane session (E04c T1)
- Working detail: the survey [`docs/rules/surveys/E04c-battlefields.md`](../rules/surveys/E04c-battlefields.md),
  the code reading `docs/formats/installation.md` §7.26; the epic `docs/epics/E04c-sieges-and-battlefields.md`

## Context

E04c's first increment adds the original's special battlefields (the user, BAT-Q37): the shore,
nine magic fields, a ship's deck and two boats, and the anti-magic garrison. ADR-0013 point 5
reserved "a battlefield kind" on the setup. The original decides the battlefield at the battle's
start from the map tile and the heroes (0x4643B0) and reads it in morale, luck, the native bonus,
the hero's mastery, the casting limits and the obstacle choice. The twelve battlefields differ
only in values of a few kinds of effect (survey §1), and the adventure map will start battles
soon (E08 increment 3), so the seam and its data must suit the adventure lane's derivation.

## Decision

1. **Battlefields are content of the battle part.** A battlefield is a `[[battlefield]]` of a
   pack's `battle.toml`, compiled by `h3-content-battle` (`base`: shore, Magic Plains, Cursed
   Ground, deck, boats; `sod`: the seven SoD fields). Its rules are closed typed fields, each read
   by one rule: morale and luck changes per list of factions; morale and luck set to 0; no native
   bonus; a spell level limit; no creature casts; the schools whose hero spells are cast at Expert;
   Expert for the other casts; obstacles of its own or of the terrain; fixed blocked hexes instead
   of random obstacles. An obstacle type lists its battlefields beside its terrains.
2. **The seam.** `BattleSetup` gains `battlefield: Option<ContentKey>` (the battlefield's key, as
   an obstacle's type is named: `h3-contracts` does not depend on the battle part, ADR-0015) and
   `anti_magic: bool` (the battle is at an anti-magic garrison). Every battle has a battlefield;
   the key names the special ones, and `None` is the terrain's ordinary battlefield (the
   original's −1, the user at BAT-Q38: a mandatory key would only name "ordinary" twelve times). `terrain` keeps its meaning (the
   native bonus, and the obstacles when the battlefield has none of its own). The battle resolves
   the key against its ruleset at setup (an unknown key is a setup error) and never derives the
   battlefield. Whoever starts a battle derives it (the adventure lane's orchestrator, which can
   keep the key with its object types: the original's order is the survey's §4); the sieges add
   their own field at their increment, through an amendment or their own ADR. E08 increment 13
   adds **`creature_bank: bool`**: the battle is a creature bank's, with point 8's rules; default
   false; set by whoever starts the battle from the battle's cause (the adventure lane's
   orchestrator: a bank's guards defending, never the tile).
3. **Obstacles stay before the seam** (BAT-GEO-003): the generator takes the battlefield and picks
   from the battlefield's types when it has its own, else the terrain's; for a battlefield with
   fixed hexes it returns them as one absolute obstacle without a type and nothing else. For a
   creature bank's setup it gives none (as for a town with a fort), and the setup refuses
   obstacles or a siege beside the flag (point 8).
4. **The battle applies the battlefield's effects** where the original reads the field: morale and
   luck when they are materialised (a battlefield change is part of the stack's own sum, before
   the side's caps and the creatures' minimums; set to 0 replaces everything, 0x43E000,
   0x43DF20); the native bonus at setup; its limits join the battle's magic limits (`MagicLimits`,
   with the anti-magic garrison as a ban on every cast).
5. **The mastery raise is a battle rule.** At a cast, a hero's mastery of a spell is Expert when the
   battlefield raises one of the spell's schools, else the hero's derived mastery
   (`h3_heroes::spell_masteries`, unchanged: ADR-0013 point 2 keeps the hero's own masteries
   battlefield-free); a creature's, strike's or opening spell is at Expert where the battlefield
   raises the other casts. This is the original's 0x4E5080 with the field, at cast time. The
   adventure map's raise for adventure spells on Magic Plains is the adventure lane's, through a
   shared core helper if it needs the same test.
6. **Visibility.** The battlefield and the anti-magic flag are public (the original shows the
   picture and refuses the book openly): they join the observation, its flat encoding and the wire.
   The creature bank flag is public but not in the observation: each of its effects shows in the
   battle itself (the places, no obstacles, no phase, no machines on the field, a supplied
   shooter's shots), so it joins neither the observation, its flat encoding, the C ABI nor the
   wire, unlike the anti-magic flag, which a viewer needs for the refused book.
7. **Rule ids.** Battlefield rules are `BAT-TER-002` onwards (the terrain area); BAT-GEO-004 is
   amended for the type choice. The creature bank's rules (point 8) amend BAT-SET-001,
   BAT-TAC-001, BAT-GEO-004, BAT-WAR-001 and BAT-WAR-004.
8. **The creature bank battle** (E08 increment 13). The original marks a creature bank's battle
   by one flag (the battle manager's +0x53C5, H3API's `creatureBank`), `adventure_battle`'s last
   argument, which only `bank_battle_reward` sets; four of the battle's sites read it (survey
   `E08-banks.md` §5). With `creature_bank` the battle:
   - opens **no tactics phase**, whatever the heroes' Tactics (their levels stay theirs; 0x462C5D);
   - has **no obstacles** (point 3; 0x4662B3);
   - sets up **no war machine** of either side (0x463758): the listed ones are checked, then
     neither placed nor in the outcome, as a side's without a hero, so they cannot be destroyed;
     a side whose listed machines include an ammo supply is **supplied** for the whole battle as
     by a living cart (the Ammo Cart's passive test reads the hero's artifact, not the stack,
     0x43F9E1);
   - places each side's stacks in slot order at the battle part's **bank positions** by their
     ordinal among the filled slots, the formation ignored, each its stack's position (a two-hex
     stack's rear hex; 0x4636B7, `bank_battle_hexes` 0x63D0E0). The positions are a Tier 2
     battle parameter beside the formations (seven a side, the original's table), checked by the
     compiler so that no placement can fail.

   Batch E13 (survey `E08-banks.md` §14) confirmed the four with E13-3 (Expert Tactics and no
   phase, no obstacles, the Ballista, the Ammo Cart and the Catapult worn and none on the field,
   the Archers' shots supplied, the stacks by their ordinal), the positions' cells with E13-4 and
   E13-7 (each two-hex stack on the table's hex as its rear) and the deck's with E13-5. The
   formation's part rests on the reading and VCMI (E13-3's hero fought loose, `ADV-QUIRK-8`).

   The flag is battle state (a set-up battle begins later, CLT-Q69 (a)). Everything else is any
   battle's: the battlefield, morale and luck, retreat, the computer's choices.

## Consequences

- A new battlefield or a changed value is a pack change; a new kind of effect is one typed field,
  one compiler check and one rule.
- Versions: pack format, the battle ruleset part's hash, state hash, replay format, observation
  encoding and C ABI; golden replays re-recorded. The core part (`h3-content`, `h3-pack`) does not
  change; the shared change is the two fields in `h3-contracts`. `h3-protocol` maps the two new facts (a
  `cross-lane` change). Battles without a battlefield play as before.
- The adventure lane's queue gets the derivation (survey §4, D-4), the client lane's a note that
  records now carry the battlefield (its pictures; CLT-Q4 stays for the terrain variants).
- Obstacle types are harvested from the executable's tables (equal to VCMI's today), adding the
  47 types only special battlefields use.
- Enforced by tests per rule, the compiler's checks of the battlefield data, setup errors for an
  unknown battlefield, and the observation hygiene test.
- The creature bank battle (point 8): versions: the battle state hash and the battle part's hash
  (the battle lane's) and the pack format (shared: the battle file's `bank_positions`); the
  battle lane's golden replays rehashed. The shared change is the one field in `h3-contracts`
  with every exhaustive `BattleSetup` literal's `false` (mechanical, the battle, AI and adventure
  lanes' crates). No change to the observation, its encoding, the C ABI, the battle replay format
  or the wire; the battle's scenario files and replays get a `creature_bank` key when a scenario
  first needs one (the battle lane's). Enforced by a test per rule, the compiler's checks of the
  positions, the setup's refusal and the battle's properties with bank setups.

## Alternatives considered

- A closed `Battlefield` enum in `h3-contracts` with the rules in code: twelve variants that differ
  only in values; against ADR-0006's data-first tiers.
- A core content id (`BattlefieldId`, like `TerrainId`): a shared change to `h3-content` and
  `h3-pack` for a key only the battle reads; the adventure map needs the key, not the rules.
- The battlefield's effects as hero-like `BattleEffect`s passed through the setup: the caller would
  copy data the ruleset already has, and could pass a mix no battlefield has.
- The raise in `h3-heroes` with the battlefield's schools as an argument: the battle could no longer
  check it against the setup's battlefield, and the creature and opening casts need the battle's
  raise anyway.
- The anti-magic garrison as a thirteenth battlefield: the original combines it with the shore.
- The boats' fixed hexes added by the battle at setup: a second place that places obstacles.
- For the creature bank battle: a field per rule (no tactics, no obstacles, no machines, a
  layout): a caller could pass a mix no original battle has, and every caller would derive four
  facts from one cause.
- The bank's rules left to the caller (no obstacles drawn, no machines listed, Tactics passed as
  0, a layout as desired hexes): the Ammo Cart's supply would be lost with the listed cart, the
  heroes' Tactics falsified, and the layout cannot be expressed through the formations.
- `Formation::Bank`: the formation is the army's, kept on the map, and a bank ignores it.
- The bank positions in code: against ADR-0006's data-first tiers, beside the formations' data.
- VCMI's second table for two-hex guards (its layout `creatureBankWide`: 171, 1, 86 where the one
  table has 172, 2, 87): under the footprint rule (BAT-GEO-002) a two-hex stack at the table's hex
  as its rear covers the cells VCMI names by the front hex, as batch E13's E13-4 found (survey
  `E08-banks.md` §14, D-69 dissolved), so the original's one table serves both.
- The creature bank flag in the observation, as the anti-magic flag: no viewer reads it; the
  protocol, the flat encoding and the C ABI would move for nothing.

## Revision history

- 2026-11-16 (E08 T122, `shared/`, `cross-lane`; the user at ADV-Q54 and ADV-Q55, E08 decisions
  338, 343, 344, 348): point 8 added and points 2, 3, 6, 7 amended for E08 increment 13 (the
  creature bank battle: `creature_bank`, its four rules, the bank positions); ADR-0013 point 5
  noted. Drafted in E08 T118b2 (survey `E08-banks.md` §5); batch E13's outcome (T121, survey §14:
  every row as read) folded in, the alternatives gaining VCMI's second table (D-69 dissolved).

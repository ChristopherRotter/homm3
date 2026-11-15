# ADR-0022 amendment (draft): the creature bank battle on the setup (E08 increment 13)

- Status: **Draft** (E08 T118b2, 2026-11-15), for the checkpoint after T118b2 (decision 334). The
  user gave the bank battle to the adventure lane, `cross-lane`, after a `shared/` change of the
  seam (E08 decision 338); the battle lane was told (its inbox, 2026-11-15) and reviews. Into
  ADR-0022 through the round's `shared/` pull request with batch E13's outcome.
- Amends: ADR-0022 points 2, 3, 6 and 7 and adds a **point 8**; its consequences and
  alternatives; a revision history line. Also ADR-0013 point 5's line on the setup's later fields.
  Points 1, 4 and 5 are unchanged: a bank adds no battlefield and no mastery raise.
- Detail: `docs/epics/E08-design.md` §13 (the rules), §14 (the game's side); survey
  `docs/rules/surveys/E08-banks.md` §5 (the flag's four sites in the original's battle).

## Context

The original marks a creature bank's battle by one flag (the battle manager's +0x53C5, H3API's
`creatureBank`), `adventure_battle`'s last argument, which only `bank_battle_reward` sets (14
calls read, T118b2). Four of the battle's sites read it: no tactics phase (0x462C5D), no
obstacles (0x4662B3), no war machines on the field (0x463758), and the stacks placed at a fixed
table's hexes by their ordinal among the filled slots, the formation ignored (0x4636B7,
`bank_battle_hexes` 0x63D0E0). The Ammo Cart's passive test reads the hero's artifact, not the
stack (0x43F9E1), so its supply stays though no cart stands. `BattleSetup` cannot express any of
this, and the anti-magic garrison's flag (this ADR's point 2) is the nearest precedent.

## Decision

**Point 2** (the seam), a sentence after the anti-magic flag: `BattleSetup` gains
**`creature_bank: bool`** (E08 increment 13): the battle is a creature bank's, with point 8's
rules; default false; set by whoever starts the battle from the battle's cause (the adventure
lane's orchestrator: a bank's guards defending, never the tile).

**Point 3** (obstacles), a sentence: the generator gives none for a creature bank's setup (as for
a town with a fort), and the setup refuses obstacles or a siege beside the flag.

**Point 6** (visibility), a sentence: the bank flag is public but not in the observation: each of
its effects shows in the battle itself (the places, no obstacles, no phase, no machines on the
field, a supplied shooter's shots), so it joins neither the observation, its flat encoding, the C
ABI nor the wire, unlike the anti-magic flag, which a viewer needs for the refused book.

**Point 7** (rule ids), a sentence: the bank's rules amend BAT-SET-001, BAT-TAC-001, BAT-GEO-004,
BAT-WAR-001 and BAT-WAR-004.

**Point 8** (new). **The creature bank battle.** With `creature_bank` the battle:
- opens **no tactics phase**, whatever the heroes' Tactics (their levels stay theirs);
- has **no obstacles** (point 3);
- sets up **no war machine** of either side: the listed ones are checked, then neither placed nor
  in the outcome, as a side's without a hero, so they cannot be destroyed; a side whose listed
  machines include an ammo supply is **supplied** for the whole battle as by a living cart;
- places each side's stacks in slot order at the battle part's **bank positions** by their
  ordinal among the filled slots, the formation ignored, each its stack's position (a two-hex
  stack's rear hex). The positions are a Tier 2 battle parameter beside the formations (seven a
  side, the original's table), checked by the compiler so that no placement can fail.
The flag is battle state (a set-up battle begins later, CLT-Q69 (a)). Everything else is any
battle's: the battlefield, morale and luck, retreat, the computer's choices.

**ADR-0013 point 5**, its last sentence extended: E08 increment 13 adds the setup's
`creature_bank` (ADR-0022 point 8).

**Consequences**, lines: versions: the battle state hash and the battle part's hash (the battle
lane's) and the pack format (shared: the battle file's `bank_positions`); the battle lane's golden
replays rehashed. The shared change is the one field in `h3-contracts` with every exhaustive
`BattleSetup` literal's `false` (mechanical, the battle, AI and adventure lanes' crates). No
change to the observation, its encoding, the C ABI, the battle replay format or the wire; the
battle's scenario files and replays get a `creature_bank` key when a scenario first needs one
(the battle lane's). Enforced by a test per rule, the compiler's checks of the positions, the
setup's refusal and the battle's properties with bank setups.

**Alternatives considered**, lines:
- A field per rule (no tactics, no obstacles, no machines, a layout): a caller could pass a mix no
  original battle has, and every caller would derive four facts from one cause.
- The rules left to the caller (no obstacles drawn, no machines listed, Tactics passed as 0, a
  layout as desired hexes): the Ammo Cart's supply would be lost with the listed cart, the heroes'
  Tactics falsified, and the layout cannot be expressed through the formations.
- `Formation::Bank`: the formation is the army's, kept on the map, and a bank ignores it.
- The bank positions in code: against ADR-0006's data-first tiers, beside the formations' data.
- The flag in the observation, as the anti-magic flag: no viewer reads it; the protocol, the flat
  encoding and the C ABI would move for nothing.

**Revision history**, a line: point 8 added and points 2, 3, 6, 7 amended for E08 increment 13
(the creature bank battle: `creature_bank`, its four rules, the bank positions); ADR-0013 point 5
noted. Drafted in E08 T118b2 (survey `E08-banks.md` §5); batch E13's outcome folded in.

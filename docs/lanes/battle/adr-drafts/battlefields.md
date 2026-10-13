# ADR draft: Battlefields — special battlefields as content, the battlefield on the setup

- Status: Proposed (E04c T1, 2026-10-13; for the user's review, BAT-Q38)
- Date: 2026-10-13
- Deciders: user; battle lane session (E04c T1)
- Working detail: the survey [`docs/rules/surveys/E04c-battlefields.md`](../../../rules/surveys/E04c-battlefields.md),
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

1. **Battlefields are content.** A battlefield is a core content key, `h3_content::BattlefieldId`
   (core part, like `TerrainId`; `base`: shore, Magic Plains, Cursed Ground, deck, boats; `sod`:
   the seven SoD fields). Its battle rules are data in the battle part (`h3-content-battle`), in
   closed typed fields, each read by one rule: morale and luck changes per list of factions; morale
   and luck set to 0; no native bonus; a spell level limit; no creature casts; the schools whose
   hero spells are cast at Expert; Expert for the other casts; obstacles of its own or of the
   terrain; fixed blocked hexes instead of random obstacles. An obstacle type lists its
   battlefields beside its terrains.
2. **The seam.** `BattleSetup` gains `battlefield: Option<BattlefieldId>` and `anti_magic: bool`
   (the battle is at an anti-magic garrison). `terrain` keeps its meaning (the native bonus, and
   the obstacles when the battlefield has none of its own). The battle checks the key against the
   ruleset and reads its data; it never derives the battlefield. Whoever starts a battle derives
   it (the adventure lane's orchestrator: the original's order is the survey's §4); the sieges add
   their own field at their increment, through an amendment or their own ADR.
3. **Obstacles stay before the seam** (BAT-GEO-003): the generator takes the battlefield and picks
   from the battlefield's types when it has its own, else the terrain's; for a battlefield with
   fixed hexes it returns them as one absolute obstacle without a type and nothing else.
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
7. **Rule ids.** Battlefield rules are `BAT-TER-002` onwards (the terrain area); BAT-GEO-004 is
   amended for the type choice.

## Consequences

- A new battlefield or a changed value is a pack change; a new kind of effect is one typed field,
  one compiler check and one rule.
- Versions: pack format, core and battle ruleset hashes, state hash, replay format, observation
  encoding and C ABI; golden replays re-recorded. `h3-protocol` maps the two new facts (a
  `cross-lane` change). Battles without a battlefield play as before.
- The adventure lane's queue gets the derivation (survey §4, D-4), the client lane's a note that
  records now carry the battlefield (its pictures; CLT-Q4 stays for the terrain variants).
- Obstacle types are harvested from the executable's tables (equal to VCMI's today), adding the
  47 types only special battlefields use.
- Enforced by tests per rule, the compiler's checks of the battlefield data, setup errors for an
  unknown battlefield, and the observation hygiene test.

## Alternatives considered

- A closed `Battlefield` enum in `h3-contracts` with the rules in code: twelve variants that differ
  only in values; against ADR-0006's data-first tiers.
- The battlefield's effects as hero-like `BattleEffect`s passed through the setup: the caller would
  copy data the ruleset already has, and could pass a mix no battlefield has.
- The raise in `h3-heroes` with the battlefield's schools as an argument: the battle could no longer
  check it against the setup's battlefield, and the creature and opening casts need the battle's
  raise anyway.
- The anti-magic garrison as a thirteenth battlefield: the original combines it with the shore.
- The boats' fixed hexes added by the battle at setup: a second place that places obstacles.

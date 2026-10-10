# ADR-0014: Battle magic — spells as data, casting, timed effects, created stacks, hidden objects

- Status: Accepted (user, 2026-10-05; OPEN-QUESTIONS Q42)
- Date: 2026-10-05
- Deciders: user; agent session (E04b T1)
- Working detail: [`docs/epics/E04b-design.md`](../epics/E04b-design.md); sources:
  [`docs/rules/surveys/E04b-magic.md`](../rules/surveys/E04b-magic.md)

## Context

E04b adds battle magic (Q27) in increments (Q41): about 60 hero spells, 11 ability spells, some 30
creature magic abilities, 43 magic artifacts. E04a reserved the hero's magic in the seam (ADR-0013
points 4, 5, 9, 10); the battle keeps its architecture (pure rules, validated commands, fixed-capacity
state, events with visibility, at least 100 k steps/s per core). To decide before code: how spells
are described, how casting enters the commands and the sim, where timed effects live, the order in
which a spell meets a stack, how stacks appear in a battle, how hidden objects stay hidden.

## Decision

1. **Spells are data over a closed set of effect kinds (Tier 3).** A spell row gains its targeting,
   the executable's flags (`installation.md` §7.6) and its effects per mastery from a closed
   `SpellEffect` enum (damage, timed, dispel, heal and raise, summon, clone, teleport, sacrifice,
   obstacle, remove obstacle; Earthquake with E04c), with target conditions as data. Each kind is one
   rule in code; a spell of a known kind is a data change. Creature magic is `Ability` variants that
   name a spell (ADR-0012).
2. **A hero's cast is optional and at most once per round.** While one of its side's stacks awaits a
   command (not in tactics, not on an automatic turn), the hero may cast if permitted, never must,
   and the stack still acts afterwards. Permitted: a spell book with the spell, no magic block or
   level limit against it, a valid target, mana for the cost. Mastery is the hero's mastery of the
   spell, derived once in the Heroes context (the best school level, raised by an artifact, at
   most Expert; ADR-0013 point 2) and carried per spell by the seam; a battlefield's raise is the
   battle's (E04c). It selects cost, value, area and the mass version. The
   cost includes the creature modifiers (Mages, Pegasi), never below 0. The battle keeps each hero's
   mana and casts this round and fills the reserved `HeroOutcome` fields (mana left, spells cast).
3. **A creature's cast is its stack's action** (Faerie Dragon, Archangel, Pit Lord, Ogre Mage, the AB
   elementals, Master Genie), limited by its casts per battle. Automatic magic (after-attack spells,
   the Enchanter, opening spells, triggered obstacles, Fire Shield) is applied by rules. A hero's cast
   names the spell and up to two targets (Teleport, Sacrifice). A creature's cast names only its
   target, as the original's action does: the stack decides the spell (its ability's, a Faerie
   Dragon's drawn spell, the Master Genie's draw at resolution) or raises without one (Archangel, Pit
   Lord). The actor comes from the trusted caller (ADR-0004, ADR-0008).
4. **One resolution order for a spell against a stack:** (1) valid target: target conditions,
   absolute immunities, then normal immunities unless negated (Orb of Vulnerability); a single-target
   cast must name a valid target, area and mass casts pass over invalid stacks; (2) Magic Mirror for
   single-target negative hero spells; (3) magic resistance, one draw per affected stack, for negative
   magical spells; (4) the effect with the target's reductions and the caster's bonuses; (5) events
   naming affected, resisted and reflected stacks. Draws come from the `BATTLE` stream in stack order
   (ADR-0003); factor order and rounding are per rule, checked by experiment where sources disagree.
5. **Timed effects live in one growable list per battle, not in the modifier pool.** An entry holds
   the stack, the spell, the rounds left or an end condition (until attacked, until it attacks,
   permanent) and the values computed at the cast, in canonical order (stack, then spell) for
   equality and the state hash. One entry per stack and spell: the same spell again keeps the
   longer duration and the first cast's values (the original's code, Q47); a counter spell ends
   its pair's effect, so the later one wins (Bless, then Curse: cursed; the original has the pairs
   Bless/Curse and Haste/Slow); cumulative spells (Disrupting Ray, Acid Breath) add to their entry. The list is heap storage
   reserved at the battle's start for the usual case: steady play does not allocate, a fork costs
   one allocation, and no stack ever needs room for every effect (user, Q42). Each stack's
   effective stats and statuses (Blind, Berserk, Hypnotize, Forgetfulness, petrified, paralysed,
   bound) are materialised inline when an entry changes, so hot-path reads never touch the list;
   rules read the statuses like abilities. Rounds fall for all stacks at one fixed point of the
   round (survey M-3). The side-wide pool (ADR-0012) keeps side-wide modifiers. This amends
   ADR-0004 point 8 ("fixed-capacity structures in battle") for lists whose realistic size is far
   below their worst case; spell obstacles (T5) are kept the same way.
6. **Stacks created in battle follow the original's limit.** Each side has 21 stack slots, as the
   original (`installation.md` §7.7). Clone and Summon Elemental are legal only while the caster's
   side has fewer than 20 stacks (a battle parameter), Summon Elemental only for the side's first
   elemental type. Raising (Resurrection, Animate Dead, Sacrifice) revives a dead stack in its own
   slot and is not limited. A Pit Lord's Demons are a new stack in a dead created stack's slot or
   a new one (refused without either: the original crashes), which takes the army slot of the
   dead stack they rise from. Dead stacks keep their slots and stay addressable. Summoned stacks
   and clones end with the battle, and a side left with only such stacks does not win
   (`BattleResult::Draw`, as the original). The outcome reports every stack in the side's slots
   at the end by origin (BAT-Q7, BAT-Q13): army stacks, war machines, created stacks
   (`StackOrigin::Created`, whose kills Necromancy counts) and Demons (`StackOrigin::Raised`, in
   their army slot after the battle); a slot a later stack took over reports only that one, as
   the original's records. Temporarily raised creatures (Basic Resurrection) do not count in the
   outcome's end counts; permanent ones do.
7. **Hidden battlefield objects have an owner side.** A viewer sees a spell obstacle if it is the
   owner, the obstacle is revealed or of a visible kind (Force Field, Fire Wall), or the viewer's side
   has natives: as the original by default, a side that had a stack on its native terrain at the
   battle start sees the other side's hidden objects from their placement on, whatever happens to
   its natives later; with quirk BAT-QUIRK-1 fixed, while it has a living stack on its native
   terrain (this changes during a battle). Observations project
   hidden objects away, their placement events are `Visibility::Side` (ADR-0004 §6), and legality
   never uses what the acting side cannot see. A command may therefore resolve differently from
   what its side expected (user, Q42): a move, or a move and attack, into an unseen Quicksand pit
   stops there without the attack; an unseen mine goes off. The events tell the side what happened.
   As the original, Quicksand's and Land Mine's random hexes avoid the other side's hidden objects
   (quirk BAT-QUIRK-4, fix-optional: the fix draws among the hexes the caster's side sees free).
8. **Visibility** (extends ADR-0013 point 9): mana, casts used this round, each cast with its spell,
   targets and results, and the effects on every stack are public, as in the original; the spells in
   the spell book and the hero's spell bonuses stay private to its side, and so does a Faerie
   Dragon's drawn spell until it is cast (the original's message at its activation tells only its
   controller; BAT-Q11). Whether a hero carries a spell book is public (the Master Genie's test
   reads it; BAT-Q12).
9. **A factorized casting action.** The sim keeps its dense list of other actions and adds two cast
   entries (hero, creature); choosing one opens a spell head (by spell id, sized by a content limit)
   and two target heads (none, a hex or a stack slot), each with a mask conditioned on the earlier
   choices. The creature entry's spell head allows only the spell the active stack is going to cast,
   where one is known before its resolution (point 3: the casters of one spell, a Faerie Dragon's
   drawn spell); for the Master Genie, which draws at the resolution, it allows only 0, and the
   observation shows per stack how many spells the draw could pick for it (BAT-Q11); the Archangel
   and the Pit Lord, which raise without a spell, allow only 0 too. Its second target head allows
   only none. The C ABI takes an action as
   a small struct and serves a mask per head; sizes do not depend on the ruleset (ADR-0011). Each
   task bumps the encodings it changes.
10. **Seam changes are additions only** (reviewed with this ADR): `BattleEffect` variants for the
    hero's magic (spell-damage bonuses, duration bonus, magic resistance, magic blocks and level
    limits, immunity negation, opening spells) and the use of the reserved
    `BattleMagic` and `HeroOutcome` fields. One change (the user, BAT-Q19): `BattleMagic.schools`
    gave way to `masteries`, a mastery per spell (point 2). Battle magic rules use the new area `BAT-MAG`; the
    hero's derivation continues `HER-BAT`. One exception (user, Q55): `BattleSetup.obstacles`
    becomes a list of natural obstacles, each with its blocked hexes and an optional anchor hex
    (removable by Remove Obstacle; the anchor's own effects are quirk BAT-QUIRK-3). `StackOrigin`
    gains `Created` and `Raised` for the stacks a battle creates or raises (point 6).

## Consequences

- Most spells become data once their kind exists; the tasks add kinds in order (T2 damage; T3 timed,
  dispel, heal; T4 statuses; T5 obstacles, teleport; T6 summon, clone, raise, sacrifice; T7 creatures).
- `BattleState` stays `Clone` (it is not `Copy`); today all its fields are inline, so a fork is
  one copy without allocation, and the effect list adds one allocation. The stack slots grow to 21
  per side and the dense action list's stack-indexed blocks grow once (T2c); state size, fork cost
  and steps/s are measured at T2c, T3, T6 and T9.
- Enforced by property tests (legal casts and `apply` agree; mana never negative; at most one hero
  cast per side and round; no hidden object or spell book in the other side's observation or mask;
  creatures accounted with created stacks), compiler checks and rule tests.
- Data quirks (Q-0006, S-05) need profile-level overrides: an amendment of ADR-0007 in T8.

## Alternatives considered

- Scripts or a VCMI-style bonus tree: Tier 4 by the back door, allocation and evaluation on reads.
- Timed effects as stack-bearer modifiers in the pool: capacity grows with stacks × stats × sources,
  and statuses are not stat changes. Inline slots with room for every timed spell on every stack
  (bounded by construction): about 10 KB of mostly empty state copied on every fork (Q42). One list
  per stack: a fork allocates once per stack with effects.
- An engine-chosen number of extra stack slots: the original's 21 slots and its limit of 20 are
  known from its code (Q42).
- A flat cast block (spells × targets × second targets): too large for masks and policy heads.
  Re-encoding every action as kind and target: churns E04a's encoding for no gain now (E05 may).
- Mana and costs decided outside the battle: the mask could not tell whether a cast is affordable.
- A creature's cast as the hero's cast command with a caster kind: the command would name a spell
  the Master Genie draws only at its resolution and the Archangel and the Pit Lord do not cast
  (BAT-Q10).

## Revision history

- 2026-10-05: Proposed (E04b T1). Longer than the 80-line target because, like ADR-0013, it holds the
  seam and state decisions of a whole epic; detail is in the design note.
- 2026-10-05: revised after the user's first review (Q42): point 5 keeps timed effects in one
  growable list instead of inline slots for every timed spell (amending ADR-0004 point 8); point 6
  takes the original's 21 stack slots and limit of 20 from its code (`installation.md` §7.7); point
  7 says how a command resolves when it meets an unseen obstacle.
- 2026-10-05: Accepted by the user (Q42).
- 2026-10-06: point 5 amended after reading the original's timed-spell code (E04b T3a, Q47): a
  re-cast keeps the longer duration and the first values instead of replacing the entry; the
  counter pairs are the original's (Bless/Curse, Haste/Slow).
- 2026-10-07: point 7 amended after reading the original's Quicksand code (E04b T5c1, BAT-Q1): the
  natives' sight is fixed at the battle start and given to an object when it is placed
  (`installation.md` §7.16); the living-natives rule stays as the fix of BAT-QUIRK-1.
- 2026-10-07: points 7 and 10 amended (E04b T5b; user, Q55, BAT-Q4, BAT-Q5): the natural
  obstacles cross the seam as pieces with an optional anchor, a change of a field's type rather
  than an addition; the random draws of hidden objects follow the original under BAT-QUIRK-4.
- 2026-10-08: points 6 and 10 amended (E04b T6a; the user): stacks created in battle cross no seam.
  Only army stacks and war machines enter a battle, and the orchestrator needs records only to
  update the army, so the battle leaves summoned stacks and clones out of the outcome instead of
  reporting them with new `StackOrigin` variants (VCMI also leaves temporary summons out of the
  casualties and the experience); Pit Lords' demons, which stay in the army, are T7's question.
- 2026-10-08: points 3 and 9 amended (E04b T7b1; the user, BAT-Q10): a creature's cast names only
  its target, as the original's action (the dispatcher's case 10, `installation.md` §7.21); the
  stack decides the spell. The sim's creature entry keeps its spell head, which shows the spell the
  stack will cast.
- 2026-10-08: points 8 and 9 amended (E04b T7b2; the user, BAT-Q11): a Faerie Dragon's drawn spell
  is private to its controller's side until it is cast; the Master Genie's spell head allows only 0
  and the observation counts, per stack, the spells its draw could pick. Point 8 again (the user,
  BAT-Q12): whether a hero carries a spell book is public, as the Genie's counts reveal it.
- 2026-10-08: points 6, 9 and 10 amended (E04b T6c and T7b3; the user, BAT-Q7, BAT-Q13): the
  outcome reports the stacks a battle creates (`StackOrigin::Created`; Necromancy counts their
  kills, per stack as the original's records) and a Pit Lord's Demons (`StackOrigin::Raised`, an
  army stack in the dead stack's army slot, `installation.md` §7.21); this takes back the T6a
  amendment's "no `StackOrigin` variant". The Archangel and the Pit Lord raise without a spell.
- 2026-10-10: points 2 and 10 amended (E04b T8a; the user, BAT-Q19): a spell's mastery is the
  hero's, derived once in the Heroes context like its available spells, and crosses the seam per
  spell (`BattleMagic.masteries` replaces `schools`); the battle keeps only a battlefield's raise
  (E04c). It covers every spell, as the original's mastery is a function of the hero for any
  spell (Sacrifice's test reads Resurrection's, BAT-QUIRK-5; `installation.md` §7.22).

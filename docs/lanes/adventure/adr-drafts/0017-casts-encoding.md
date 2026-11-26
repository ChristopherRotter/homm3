# ADR amendment draft: the casts' listing and the env's encoding v21 (E08 T144a1, T144a2)

Drafted by E08 T144a2 (2026-11-26) for E08 decisions 421 and 422 (the user, 2026-11-25), built in
T144a1 and T144a2; applied by increment 15's audit's `shared/` task (T145) with the increment's
other amendments, then deleted.

## ADR-0017 (the game's orchestration), point 8 (`AdventureEnv`)

Replace the paragraph **"Increment 15's action encoding"** (written before T144: three casts
without a target and two 19 × 17 windows around the hero, every target listed by
`legal_commands`) with:

**Increment 15's action encoding v21** (E08 T144a2; the AI lane's, `cross-lane`): five entries
per hero slot after the spell book's purchase (40, from `CAST_BASE` 105: slot `s`'s cast of the
behaviour of code `c`, the original's spell order Summon Boat, Scuttle Boat, Fly, Water Walk,
Dimension Door, is `105 + 5s + c`), each naming the core spell the adventure part binds to that
behaviour. Summon Boat, Fly and Water Walk take no tile (tile 0 alone) and are legal where
`legal_commands` lists the hero's cast; Scuttle Boat's boat and Dimension Door's arrival are the
**tile head's** tile (the encoding has no slot head for windows around a hero, decision 421), the
entry legal where the hero's targets query is not empty and its tile mask that query's targets
(decision 422). `legal_commands` lists the casts without a target last, per hero in the player's
list; **Scuttle Boat's and Dimension Door's targets are a per-hero query**,
`legal_cast_targets(state, rules, actor, (hero, spell))`, in row order, as the moves are
`legal_moves`' (listing every target wrote up to 646 commands per caster on every call). The
env's random agent draws each listed command, each hero's move and each hero's targeted cast
that has a target once, a move's tile and a cast's target then uniformly (the AI lane may choose
other weights). The own hero's observation carries its day spells (the door's casts made among
them); no field counts the casts left: the masks say whether a cast is legal.

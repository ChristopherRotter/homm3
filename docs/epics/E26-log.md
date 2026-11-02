# E26 — log

The record of E26's tasks: what each built and decided, beyond the one line in the epic file.

## T1 — the design (2026-11-02, increment 1)

- The user's decisions at the refinement (BAT-Q63): the port unchanged on its side's world, not
  its reads rewritten; nothing assumed of the other hero's private part now, a sampled prior
  updated by what the side has seen later; the epic's shared files on its branch with the
  `cross-lane` label. Recorded in the epic's decisions, ADR-0027 point 4's amendment,
  `BACKLOG.md`, `lanes.toml` (`docs/epics/E26*` is the battle lane's).
- Found while refining: `BattleState::determinized` (E06 T8) already is the world the fair actor
  needs, with its property `a_world_shows_its_viewer_the_battle_and_reads_nothing_hidden`. For
  the side to act the drawn spell is never redrawn: the active stack is the side's own.

## T2 — `choose_fair` (2026-11-02, increment 1)

- **Built:** `h3_battle_ai::choose_fair` (`lib.rs`, BAT-AI-025): the world from
  `determinized(ruleset, side, None, rng.clone().next_u64())`, `unchecked` on it, `checked` on
  the battle, Defend for a world the battle refuses (`map_or`: no branch of its own). Never
  inlined, as `choose`. No caller changed; `POLICY_VERSION` stays 19 (`choose`'s choices are
  unchanged).
- **Tests** (`h3-battle-ai`'s `src/tests/fair.rs`): twins whose other hero differs only in its
  book, masteries and damage skills (Archery, Offense and Armorer as `DamageBonuses`), the other
  side defending, compared while the side's observations agree (they part when the skills change
  an outcome): 70 decisions over 24 battles, the all-seeing command differed in 12, the fair one
  in none, and the stream is left the same. With no other hero the fair command and stream are
  `choose`'s (150 decisions); with one, the engine accepts the fair command.
- **Moved to T3:** the other side's hidden pit (the test ruleset has no hidden obstacle; the
  harvested packs' Quicksand and Land Mine).

## T3 — the properties (2026-11-02, increment 1)

- **Built** (BAT-AI-025, exit criteria 1 and 2): two properties as deterministic loops whose
  counts are asserted, beside E23's. At every step of random games: `unchecked` on the side to
  act's world (`determinized(.., None, seat.clone().next_u64())`) is validated on the battle (the
  fallback check) and equals `choose_fair`, which leaves the stream as it does (the same state and
  stream twice); without another hero (and, on the packs, no hidden pit of the other side) the
  fair command and stream are `choose`'s; twins that differ only in what the side may not know
  give the same fair command and stream wherever the side's observations of them agree.
  - `h3-battle-ai`'s `src/tests/fair.rs`, `the_fair_choice_is_legal_and_reads_nothing_hidden_in_random_battles`
    (test ruleset): 48 games, both profiles, both sides, every difficulty, one in three at the
    town; heroes two in three (Tactics, Ballista and Tent, half of them a book of about a quarter
    of the test spells at random masteries, power and mana; random damage bonuses and surrender
    discount), random funds and retreat facts. Twins: the battle reseeded at every step; per side
    a twin battle from a setup whose that side differs in its private part (its book a superset of
    the hero's, so that it accepts the casts played, other masteries, damage bonuses, surrender
    discount, funds, retreat facts), played on with the same commands until the observations part
    or it refuses one. Two steps in three play the fair command, the third a random legal one.
  - `h3-pack-battle`'s `harvested_computer.rs`, `the_fair_choice_reads_nothing_hidden_on_the_harvested_packs`
    (SoD packs): 20 games per profile from the E23 property's setups (`random_setup`, extracted
    with `Pools` and `cast_of`, its draws unchanged: the field, the special battlefields, sieges of
    every faction). Twins as above (the damage bonuses half the time; the masteries of Quicksand
    and Land Mine kept: their obstacles' number and value are what the world keeps, AI-STUB-1;
    the defender's Escape Tunnel toggled), plus at every step the battle's world for the side with
    the real other hero as the guess (`determinized(.., Some(real), seed)`: another stream, the
    other side's hidden pits placed again; the observations asserted equal first). Six steps in
    ten play the fair command, the others a hero's cast (Quicksand or Land Mine half the time it
    may) or a random legal command; a pit cast draws its hexes from another stream in the twin of
    the caster's side, all streams equal again after it.
- **Numbers.** Test ruleset: 2059 decisions, 796 without another hero equal to `choose`'s
  (command and stream), 2059 reseeded twins, 1080 setup twins compared (284 with another hero: the
  all-seeing command differed in 7, the fair one in none), 53 twins dropped when they parted, one
  fallback. Harvested: 1728 decisions, 623 equal to `choose`'s, 1728 worlds (114 with a hidden pit
  of the other side, the all-seeing command differing in none of them), 1456 setup twins compared
  (833 with another hero: the all-seeing command differed in 8; 31 with the other side's hidden
  pits on other hexes), 24 dropped, no fallback. Debug runtimes 8.8 s and 18.8 s, beside E23's
  4.3 s and 17.5 s.
- **Found:** no leak (every twin gave the same fair command and stream). One fallback, the
  all-seeing computer's too: a hypnotized shooter picks itself (`computer::valued_shot`'s pick
  takes the living stacks of the side its controller fights, whoever controls them), which the
  engine refuses (`NotAnEnemy`): test-ruleset game 40, the attacker's hero hypnotized the
  defender's Lich, whose area shot in round 3 aims at its own hex. Registered as BAT-STUB-18 (the
  shooter's side of BAT-STUB-10, BAT-Q58); the properties tolerate that case only. T2's tests
  validated `choose_fair`'s checked command, which cannot show a fallback; E23's properties
  validate `unchecked`, but their games never reached the case (the test ruleset's heroes there
  carry no book). No policy change (`choose`'s documentation names the fallback);
  `POLICY_VERSION` stays 19.

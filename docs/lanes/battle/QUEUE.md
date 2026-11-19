# Battle lane — QUEUE

Pre-refined tasks of the battle lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

E23, the original's battle AI, is **closed** (2026-10-24; `docs/epics/E23-original-battle-ai.md`, its log's "T7 — the closure"; the cuts of its tasks: OPEN-QUESTIONS BAT-Q52 to BAT-Q61 and the epic file). E04b closed on 2026-10-12, E04c on 2026-10-16 (their epic files and logs; their cuts: OPEN-QUESTIONS BAT-Q10 to BAT-Q27, BAT-Q37 to BAT-Q50).

**Now: E26, the fair system actor** (the user, 2026-11-02, BAT-Q63; `docs/epics/E26-fair-system-actor.md`, record `E26-log.md`): the original's computer, unchanged, on its side's world (`BattleState::determinized`; ADR-0027 point 4 amended). Increment 1 assumes nothing of the other side's hero's private part; increment 2 (a sampled prior, updated by what the side has seen) is refined at the checkpoint. T1 to T5 done 2026-11-02 (the design, `choose_fair`, its properties with one fallback that the all-seeing computer shares, BAT-STUB-18, its cost of +0.8 % per decision and its strength of 0.500 against the all-seeing one, the hand-over); the checkpoint below is next. **Then E11, the historical rules of RoE and AB** (refined 2026-10-24, Q58; `docs/epics/E11-historical-rules.md`): T1, the sourcing and catalogue with the user's own original media.

- **E26 increment 1's checkpoint.** *Scope:* with the user: (1) BAT-STUB-18 (a hypnotized
  shooter aims at itself, `aim.rs::pick`; `choose` and `choose_fair` both give Defend): the
  walker's cut (BAT-Q58's) for the shooter too, a `POLICY_VERSION` bump with the AI lane's
  records, or kept until a batch row shows the original's outcome; exit criterion 2 rests on it;
  (2) increment 2 (the guess's prior and memory) from T4's numbers (`E26-log.md` T4: the fair
  actor wins 0.500 against the all-seeing one, so the guess gains little win rate on random
  armies), with BAT-STUB-19 (found 2026-11-15: the world guesses the other hero's spell defences,
  which the viewer's cast candidates show for a stack that side controls); (3) the closure of increment 1 (move it to `E26-archive.md`). *Depends on:* the
  user. *Status:* waiting (the user).

## Inbox

- 2026-11-19 (the AI lane, E25 T38a, `cross-lane` with the user's consent at T37, AI-Q19; **your
  paths changed, for your review; no battle version moves**): `h3_battle::reach_seen_by` and
  `SeenReach` (`movement.rs`): a stack's reach as a viewer may know it, the rules' movement within
  its speed now (`reach`'s second instance), its walk stopping only at the pits both its controller
  and the viewer see (`obstacles::pits_seen_by_both`) and a town's moat; tests
  `src/tests/reach.rs`. Observation v23's reach block reads it (`h3_sim::observation::ReachBlock`):
  a change to movement, the stops, the gate or `computer::shoots` moves the AI lane's encoding
  (its tests compare the block with the legal attacks; two worlds of a battle must encode alike).
  The replays, the state hash and the battle workloads are unchanged.

- 2026-11-18 (the adventure lane, E08 T127b, `adventure/e08-t127b`, docs only; **a planned
  `cross-lane` change in your paths; nothing changed yet**): E08 T128 (batch E14's maps) adds to
  `h3-experiment`'s spec the seven trade classes by template, an adventure hero's start (secondary
  skills, a type by its specialty, war machines, worn artifacts, a full backpack), the map's banned
  artifacts, a player's starting resources and the RoE format (design note
  `docs/epics/E08-design.md` §20), with the user's consent at its start; no battle rule or version
  moves.

- 2026-11-17 (the adventure lane, E08 T124a, `adventure/e08-t124a`, `cross-lane` with the user's
  consent, E08 decision 353; **your paths changed, for your review; your versions moved**): the
  bank battle in your crates (design `E08-design.md` §13, its *As built*). `BattleState` holds
  `creature_bank` and each side's `bank_supply` (battle state hash **v56**, after the anti-magic
  flag); `begin` opens no tactics phase with it; `generate_setup_obstacles` gives none (nor the
  boats' hexes) and `setup` refuses obstacles or a siege beside the flag (the new
  `SetupError::InvalidCreatureBank`); `war_machines` sets up none, a listed ammo supply of a side
  with a hero supplying it (`ammo_supplied` reads `bank_supply` beside a living cart: only the
  bank's case of your r3); each stack at `BattleParams::bank_position(side, ordinal)`. The battle
  part's `bank_positions` (**v8**, pack format **v53**, `packs/README.md`) is harvested from
  0x63D0E0 by `h3_formats::exe::bank_hexes` (`installation.md` §7.62; `KnownBuild::bank_hexes_va`,
  `Inputs::bank_hexes`), so the re-harvest checks it; the synthetic pack's by hand.
  BAT-SET-001, BAT-TAC-001, BAT-GEO-004, BAT-WAR-001, BAT-WAR-004 amended; tests in
  `src/tests/banks.rs`; the ten battle goldens rehashed (only hash lines and the version). Still
  yours: a `creature_bank` key in the scenario files and battle replays (with the replay format's
  bump) when one first needs it, and `h3-battle-ai`'s reading of the flag (the original's battle AI
  and +0x53C5: unread, design §21).

- 2026-11-16 (the adventure lane, E08 T122, `shared/e08-t122`, `cross-lane` with the user's
  consent, E08 decision 350; **your paths changed, for your review; no version moved**): the seam
  has `BattleSetup::creature_bank: bool` (`h3-contracts`; default `false`; ADR-0022 point 8, now
  in the ADR with points 2, 3, 6, 7 amended). Your two exhaustive literals take `false`
  (`h3-pack-battle`'s `ScenarioDef` resolve, `h3-files`' `ReplayDoc`), each with a comment that
  the scenario and replay key is yours to add when one first needs it; every other literal takes
  the default. Nothing reads the flag until E08 T124a (the rules, `cross-lane`, your review).

- 2026-11-15 (the town lane, E09 T32, `town/e09-t32`; **a lead for E11 T1's
  catalogue**): `tavern_hero_draw` 0x4BB5E0 keeps the Conflux classes (16, 17) out of a
  non-Conflux player's tavern offers while other heroes are free, but only when `map_kind`
  (+0x1F698) is at least 2 **and the edition number is 2** (`edition` 0x67F554); this build's is 3
  and never written, so the rule never runs here. A rule of the build whose edition is 2 (likely the
  SoD retail edition, `e11-sod32-findings.md`'s media; not checked), for E11's list beside RoE's tavern differences (the wiki: the
  map's Taverns had other heroes, every hire a full army). Survey `docs/rules/surveys/E09-tavern.md`
  §2.2.

Consolidated on 2026-10-19 (the user): the entries from E01 to E04c's closure are merged below by kind, done and duplicate parts dropped; the origin stays in brackets (an epic and task, a review report such as r3 or F-r2b-2). History: git. Items for other lanes went into their inboxes; the First Aid Tent became BAT-Q51. The addresses are readings of the executable, the reviews' not read a second time. Where the readings stand: `installation.md` §7.8–§7.33; the reviews' reports stay on `battle/e04b-t11b-review-reports` and `battle/e04c-t10b-review-reports`; their experiment candidates are in `docs/experiments/README.md` ("From the battle queue's inbox", "From the E04b review", "From E04c").

- 2026-11-15 (the adventure lane, E08 T119, `adventure/e08-t119`, `cross-lane` with the user's
  consent, E08 decision 346; **your paths changed, for your review; no version moved, no engine
  code**): `h3-experiment`'s `HeroSpec` gained `slots` (the army's slots, one per `army` stack,
  each below 7 and distinct; none: 0 onwards; the template names them), refused with `defaults`;
  `[[map.object]]` takes the creature banks by picture through `Tables::plains` (the writer's
  `PLAIN_OBJECTS` now holds `BANK_OBJECTS`: 16, 24, 25, 84, 85; class 16 past subclass 6
  refused), the doc of `ObjectSpec::picture` and of `Tables::plains` lists them; two tests
  (`builds_increment_13_s_objects`, `rejects_mistakes_in_increment_13_s_objects`) and the tables'
  test's five lines; `docs/experiments/README.md`'s paragraph "Increment 13's objects", the
  `slots` field and the batch table's row E08 13.

- 2026-11-15 (the adventure lane, E08 T118b2, `adventure/e08-t118b2`; **the bank battle's design,
  accepted by the user; for your review before E08 T124a builds it in your crate, `cross-lane`**;
  `docs/epics/E08-design.md` §13, draft `docs/lanes/adventure/adr-drafts/0022-increment-13.md`; E08
  decisions 343, 344): one setup flag `BattleSetup::creature_bank` (`shared/`, E08 T122, with every
  exhaustive literal's `false` in your crates); with it **no tactics phase** (BAT-TAC-001; the flag
  in the state, since `setup` and `begin` may be apart), **no obstacles** (`generate_setup_obstacles`
  none; a setup with obstacles or a siege beside the flag refused, a new `SetupError`), **no war
  machine set up** (listed ones checked, not placed, not in the outcome) **with the Ammo Cart's
  supply kept** for a side listing one (BAT-WAR-004's bank case only; your r3 point on a destroyed
  cart stays yours), and **the layout**: a Tier 2 `[battle]` parameter `bank_positions` (attacker
  (6, 3), (10, 3), (5, 5), (8, 5), (11, 5), (6, 7), (10, 7); defender (15, 0), (15, 10), (2, 10),
  (2, 0), (15, 5), (2, 5), (8, 0); 0x63D0E0), each side's stacks by their ordinal among the filled
  slots, the formation ignored, compiler-checked so no placement fails (BAT-SET-001). Not in the
  observation (decision 344). Versions in T124a: battle state hash v56, battle part v8, pack
  format v53, your golden replays rehashed. **Yours to add when a scenario first needs one:** a
  `creature_bank` key in the scenario files and battle replays (with the replay format's bump).
  Also: D-67 corrected (E08 decision 342): `adventure_battle` gives a human's levels at 0x4AE324
  before a bank's reward; of its 14 callers only `bank_battle_reward` passes the flag.

- 2026-11-15 (the adventure lane, E08 T118a, `adventure/e08-t118a`; **the creature bank battle,
  for your review**; survey `docs/rules/surveys/E08-banks.md` §5, the epic's decision 338): the
  user gave the bank battle to the adventure lane, `cross-lane`, in E08 increment 13, after a
  `shared/` change of `BattleSetup` (`h3-contracts`); tell its inbox if you see a better shape or
  want to build it yourselves. What the original's bank flag (battle +0x53C5, H3API's
  `creatureBank`; `adventure_battle`'s last argument, stored by `battle_side_setup` 0x463C48) does:
  **no tactics phase** (0x462C5D, BAT-TAC-001's note); **no obstacles** (0x4662B3, BAT-GEO-004);
  **no war machines on the field** (`battle_war_machine_stacks` 0x463758; the Ammo Cart's passive
  test reads the hero's artifact, BAT-WAR-004, so its unlimited shots stay); **the layout**
  (0x4636B7, the table `bank_battle_hexes` 0x63D0E0): each side's stacks, in slot order, on the
  hexes of their **ordinal among the filled slots**, the formation ignored: the attacker 57, 61,
  90, 93, 96, 125, 129, the defender 15, 185, 172, 2, 100, 87, 8 (BAT-SET-001 amended). The guards
  defend without a hero (owner −1). The sea banks (the Shipwreck 85, the Derelict Ship 24) are
  fought on the deck by the caller's battlefield key, as now. Open for you (D-69, batch E13's
  E13-4): a two-hex guard on 172, 2 or 87: which hexes it covers under BAT-SET-001's footprint
  rule (VCMI keeps a second table, 171, 1, 86, for two-hex guards). The battle's stored seed is
  170 101 x + 223 252 y + 250 037 z + 82 037 of the bank's tile (unread inside the battle).

- 2026-11-05 (the client lane, E17 M6 T5b1; informational unless you take it): **a walk of the
  original's computer the engine's chooses differently.** In `clients/situations/seq-auto-cast.toml`
  (`seq-cast.toml`'s battle: the attacker's Champions first, speed 9, Orrin with Bless only;
  `sequences.toml`'s `auto-cast`, recorded 2026-11-05) the original's computer, playing the
  human's side under Auto Combat (Combat Auto Creatures and Spells set; a human side takes the
  branches marked for Hard, ADR-0027 point 8), walked the Champions to hex 62 at the opening
  turn; `h3_battle_ai::choose` and `choose_fair` (seed 1) walk them to 45. Its casts agree (none
  at the Champions' turn, Bless at the Marksmen's). Not compared further: the recording's
  commands are the original's. A candidate for E23's divergence checks if the walker's scoring
  should match here.

- **For E11, when RoE and AB rules are played** (from the town lane's `TWN-STUB-8`, 2026-11-05;
  the branch read 2026-11-15): a siege-aware obstacle draw for the base pack's walled Stronghold
  (TWN-SIE-005, `h3_towns::siege_obstacles`), which the game then calls (`h3_game`'s
  `town::siege_setup`). The original (`installation.md` §7.27, "The Stronghold's branch in the
  placement"): every obstacle hex off a blocked hex (the 11 wall hexes have the bit), and the
  anchor plus the type's width below its row's wall hex less 2 (row 5's 96 taken as 93), checked
  on the anchor's row only. The engine's draw is VCMI's (BAT-GEO-004) and places none at a fort
  (`generate_setup_obstacles`), while `BattleState::start` refuses an obstacle on a wall hex
  (`SetupError::InvalidSiege`): the wall hexes and the gate row's exception as the siege field's
  data, a draw that keeps to them, and a test per row.

- **For E23 (the original's battle AI), to read with it:** the original's pathfinder 0x4B3400 is read (§7.40, §7.41) and built (`pathfinder.rs`, E23 T1c): the walks and the berserk turn follow it. `aim.rs`'s threat records (0x422B20, the turrets' and the Ballista's turns) still search on their own (flyers by hex distance, no marks, no first-in-first-out): move them onto `pathfinder::Search`'s marks (BAT-Q53) with the computer's walker (T3). T1b's readings at risk from T1c: S-11's three outcomes, a berserk flyer ending on a hex it passed over, the war machine aimed at by its front, a walk beyond the speed into its side's seen pit (`E23-log.md`). The automatic Catapult's aim with Ballistics (the computer's and hidden battles': the gate first, §7.29) is not modelled: the agent aims. The moat avoidance's further "automatic" cases (0x474BF0: a hidden battle, an options flag, a remote turn; BAT-SIE-022). BAT-Q33 (who commands a side) decides with it.

- **The controller against the owner** (the engine mostly reads the owner; a hypnotized stack's morale and luck are the controller's since batch 2g HM): a hypnotized shooter's bows, which the original looks for on the controller's hero for the walls' halving (0x46751A to 0x46753A, BAT-SIE-008) and the distance's (0x467600, BAT-RNG-002), where the engine sets traits from the owner's hero at the start (`hero::trait_effects`; F-r2b-2); Hypnotize ending on the active stack mid-turn keeps the acting side until the next activation (0x4653AE), the engine hands the turn to the owner at once (r3); the battle's end with only hypnotized stacks left (unread; the engine counts them for their owner).

- **War machines:** the original sets up none with the battle's bank flag (+0x53C5; 0x463758 to 0x46375D): built for a bank by E08 T124a (`BattleSetup::creature_bank`: none set up, a listed cart's supply kept; a side without a hero brings none since 2026-10-24, BAT-WAR-001); the Ammo Cart's test is "the shooter's side's hero wears artifact 5" (0x43F9E1) without a test that the cart stands (BAT-WAR-004; r3); the original's Defend gives a war machine no bonus, nor any stack in the tactics phase (0x479722; does the engine?); the Genie's and the Enchanters' usefulness tests count a Ballista without shots as a shooter only under Q-0004, the original always (0x4428F0).

- **Turns** (batch 2g aligned the wait phase, the freeze's tie-break and the wait phase's turn start, and recorded the good morale before a re-entry, BAT-MOR-003): a `rebirth` creature a mine or Fire Wall kills on its way walks on and attacks after its rebirth (the original probably too, 0x478900; r5b); an active Phoenix killed by its own hero's cast and reborn stays the active stack (unread; r1); an active stack hit by a redirected Blind in its own turn finishes it (r2b); every routine that turns a stack (0x446720: the strike, the melee attack's struck stack, a walk's steps, the creature's cast, the flight, the teleporting move, the raise; §7.25) may measure at its turned front hex: the breath's second hex and the attack patterns are leads, the placement and the berserk shooter's distance (#150's call 2) unread.

- **The surrender's price:** the halving before the discount (0x47506B; one Pixie at 10 %: 10, not 11) is not observed; the temporary creatures are left out since batch 2g SP (BAT-FLE-002).

- **Numbers and draws:** BAT-MAG-058 takes the larger of a cast Fire Shield and the creature's own, the original the cast one first (no difference with the base values; r2b); the physical damage's stage 1 runs its hero step also at a rate of 0, rounding through `f32` (0x44371C; a difference only above 2^24; r2a); the original rolls also at a chance of 0 or 100 and walks an area's stacks from the last to the first (only the random stream differs; r1); the automatic Catapult's tie among one weakest wall draws `range_inclusive(1, 1)`, one value, where the original's `random_between` 0x50B230 draws nothing for equal bounds (0x50B236; the adventure lane, 2026-11-06: the outcome the same); the obstacle generator is not draw-exact (BAT-GEO-004 is VCMI's: the original draws one `rand()` per tried type and anchor, 91 usual types, 34 absolute, 151 anchors, through the C runtime's generator; its Stronghold siege branch 0x466147 is not modelled; does it keep an obstacle off another's free anchor? ADV-QUIRK-5's "layouts match the original's" needs both).

- **Other readings:** the corpse limit of 14 per hex (not modelled; it also bounds the Pit Lord's search); the stack bit `0x4000000` set after a creature's cast; a `rebirth` creature with several casts (data only); BAT-QUIRK-17's later variant, a sacrificed Phoenix reborn right after the sacrifice (BAT-Q18); the creature flag bit `0x800` (Arch Mage, Beholder, Evil Eye; `h3-harvest`); the Faerie Dragon's key path under the Orb refuses a Chain Lightning at the caster's side's stacks, which the engine offers (BAT-Q32 (1)); the knights' Curse stays at 25 % and mastery 0 until a statistics batch (BAT-Q28 row 13); Q43 (the sprite offset). From E04c's security review: whether the turrets' counts (BAT-SIE-016) show the town's building count where the original does not (its tower window is unread), and how long one command runs with many rounds of turret turns inside it (no bound measured).

- **Code left over:** BAT-STUB-1's per-stack mark at the next observation-encoding bump, with BAT-Q35's answer (`cross-lane` with the AI lane). From E04b's review fixes: a bound on the `SpellDamage` rates in `hero::check` (the fix saturates instead; r1 F-5); a `fixed` duration of up to 65 535 rounds on a turn-stopping change still compiles (r2b F-7); `h3_battle::stack_spell` answers without a viewer and `BattleState::spell_obstacles()` returns the hidden ones too (none leaks today). Doubts of the fold-in's read, none a defect: at a round start the end check comes before the obstacles' countdown, so `ObstacleRemoved` can follow `BattleEnded`, a draw there untested (A6); without BAT-QUIRK-5 a created stack is still a Sacrifice target (B7); a walker that strikes and returns is put back without entering anything when its start is out of reach, its path computed once across rebirths (B9); `magic::opening` asks for an active effect where a hero's Expert Dispel does not; a stack that resists while a chain looks for its next stack is passed over without `SpellResisted`.

- **Docs:** code-reading sources that word older readings: BAT-MAG-006, 022, 032, 035, 048, 049, 057, 066, BAT-MOR-004; the battle tests' profile `t:faithful` has no quirks (a misleading name). (`packs/README.md`'s "Profiles" and `installation.md` §7.23's order: done 2026-11-02.)

- **Performance:** `random-rollouts-full` was re-baselined in E04b T11b at +3.3 % Ir per step without a per-function profile (candidates: a `may_affect` more per spell without a target in `castable_spells`, the automatic Ballista's reach, dead stacks' kept effects), and stands at +1.34 % on `master` (2026-10-19). Its Ir swung by 7 to 12 % with the inlining of `Field::neighbor` three times in E04a: a precomputed neighbour table in `Field` if it swings again.

- **Tooling, later:** `scripts/exe.py switch` misses a `dec` (and `inc`) before the range check, so it labels the dispatcher's cases (0x47902F) one lower than the action (E23 T1a1; `Code._switch_base` handles `lea`, `add`, `sub` only: a `shared/` fix); a `Trace` sink for rule-level explanations (02-architecture §10); profile-level Tier-2 parameters (ADR-0014; data quirks are done, ADR-0007); `cargo-fuzz` targets for `h3-formats`; the `.h3m` writer's tile pictures beyond dirt, grass and water (from a game map first); a batch for what batch 2g left (BAT-Q28 rows 1, 3, 5, 6, 11, 12 and 13's statistics, BAT-Q32's key path, the survey's S-26, S-27).

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").

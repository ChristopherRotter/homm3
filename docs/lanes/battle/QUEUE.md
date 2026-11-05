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
  armies); (3) the closure of increment 1 (move it to `E26-archive.md`). *Depends on:* the
  user. *Status:* waiting (the user).

## Inbox

Consolidated on 2026-10-19 (the user): the entries from E01 to E04c's closure are merged below by kind, done and duplicate parts dropped; the origin stays in brackets (an epic and task, a review report such as r3 or F-r2b-2). History: git. Items for other lanes went into their inboxes; the First Aid Tent became BAT-Q51. The addresses are readings of the executable, the reviews' not read a second time. Where the readings stand: `installation.md` §7.8–§7.33; the reviews' reports stay on `battle/e04b-t11b-review-reports` and `battle/e04c-t10b-review-reports`; their experiment candidates are in `docs/experiments/README.md` ("From the battle queue's inbox", "From the E04b review", "From E04c").

- 2026-11-05 (the town lane, E09 T8b1, `town/e09-t8b1`; **for a task of yours when RoE and AB
  rules are played**, `TWN-STUB-8`): the base pack's walled Stronghold (a Citadel or a Castle)
  draws the usual natural obstacles in a siege (TWN-SIE-005, `h3_towns::siege_obstacles`; the
  SoD pack clears it). The game cannot draw them yet: `generate_setup_obstacles` places none at a
  fort and `generate_battlefield_obstacles` knows no walls, so an obstacle on a wall hex makes
  `BattleState::start` refuse the siege (`SetupError::InvalidSiege`). The original's 0x466010
  keeps each obstacle hex off a blocked hex (the walls get the blocked bit first, 0x4662E3) and
  takes a branch for a siege of town type 6 on the game's version (0x466147 to 0x466194, not read;
  `installation.md` §7.26). Wanted: that branch read and a draw that keeps off the walls (a
  siege-aware `generate_*`, or the walls' hexes as an input); the game then calls it for a siege
  whose town draws natural obstacles (`h3_game`'s `town::siege_setup`, `TWN-STUB-8`).
- 2026-11-04 (the town lane, E09 T7b2a, `town/e09-t7b2a`; **a failing property case on `master`'s
  code, for a task of yours**): a local `cargo test --workspace` found a case of `h3-battle`'s
  `tests::properties::a_world_shows_its_viewer_the_battle_and_reads_nothing_hidden`
  (`properties.rs:3370`): the defender's observation at round 2 of a siege (a Castle town, fort 1)
  differs between the battle and its world. Independent of T7b2a (`h3-battle` depends on nothing
  it changed); reproduced by appending `cc 4cad6d100daee416d444b56e63fed21f1c92753203fcb88c33ec2cee9d69f1ec`
  to the property's regression file (not committed).
- 2026-11-03 (the town lane, E09 T5b1; **a doc to correct**, no rule): `docs/formats/installation.md` §7.27's
  defending hero's bonuses name the Fortress' building 21 the Blood Obelisk and 22 the Glyphs of Fear; by the
  text files' rows and the requirements (22 needs 21) 21 is the Glyphs of Fear and 22 the Blood Obelisk. The
  values stand as read (21 +2 attack, 22 +2 defense: the code and VCMI; the game's descriptions say the reverse,
  E09 TD-8, decision 30). The same section's "which buildings that mask holds is not read" (the keep, 0x5BF4F0)
  is now read: the faction's building list less the forbidden, never the Village Hall (survey
  `E09-towns.md` §16.4).
- 2026-11-03 (CLT-Q40's request, carried by BAT-Q64; **a rule to check**, a code reading: compare
  with the wiki and VCMI): the original's Angelic Alliance joins the alignments when *any* hero of
  the side's player wears it, on the map or in a town (+0x54B2 from `player_hero_wears_artifact`
  0x4BACB0, read at 0x463AE2; the stacks' update reads it at 0x4697F2), where BAT-HER-007 takes the
  battle's hero's.

- 2026-10-22 (E23 T4c's blind check; a rule to confirm): the original's end routine 0x444510 returns early only for Disrupting Ray (47), so a dispel or Cure that reaches Acid Breath (80) may end it; BAT-MAG-010 keeps every permanent effect. Read the dispels' callers or play a batch row (Cure on a stack under Acid Breath) and align BAT-MAG-010 or record why not.

- **For E23 (the original's battle AI), to read with it:** the original's pathfinder 0x4B3400 is read (§7.40, §7.41) and built (`pathfinder.rs`, E23 T1c): the walks and the berserk turn follow it. `aim.rs`'s threat records (0x422B20, the turrets' and the Ballista's turns) still search on their own (flyers by hex distance, no marks, no first-in-first-out): move them onto `pathfinder::Search`'s marks (BAT-Q53) with the computer's walker (T3). T1b's readings at risk from T1c: S-11's three outcomes, a berserk flyer ending on a hex it passed over, the war machine aimed at by its front, a walk beyond the speed into its side's seen pit (`E23-log.md`). The automatic Catapult's aim with Ballistics (the computer's and hidden battles': the gate first, §7.29) is not modelled: the agent aims. The moat avoidance's further "automatic" cases (0x474BF0: a hidden battle, an options flag, a remote turn; BAT-SIE-022). BAT-Q33 (who commands a side) decides with it.

- **The controller against the owner** (the engine mostly reads the owner; a hypnotized stack's morale and luck are the controller's since batch 2g HM): a hypnotized shooter's bows, which the original looks for on the controller's hero for the walls' halving (0x46751A to 0x46753A, BAT-SIE-008) and the distance's (0x467600, BAT-RNG-002), where the engine sets traits from the owner's hero at the start (`hero::trait_effects`; F-r2b-2); Hypnotize ending on the active stack mid-turn keeps the acting side until the next activation (0x4653AE), the engine hands the turn to the owner at once (r3); the battle's end with only hypnotized stacks left (unread; the engine counts them for their owner).

- **War machines:** the original sets up none with the battle's bank flag (+0x53C5; 0x463758 to 0x46375D): the setup has no creature bank yet (a side without a hero brings none since 2026-10-24, BAT-WAR-001); the Ammo Cart's test is "the shooter's side's hero wears artifact 5" (0x43F9E1) without a test that the cart stands (BAT-WAR-004; r3); the original's Defend gives a war machine no bonus, nor any stack in the tactics phase (0x479722; does the engine?); the Genie's and the Enchanters' usefulness tests count a Ballista without shots as a shooter only under Q-0004, the original always (0x4428F0).

- **Turns** (batch 2g aligned the wait phase, the freeze's tie-break and the wait phase's turn start, and recorded the good morale before a re-entry, BAT-MOR-003): a `rebirth` creature a mine or Fire Wall kills on its way walks on and attacks after its rebirth (the original probably too, 0x478900; r5b); an active Phoenix killed by its own hero's cast and reborn stays the active stack (unread; r1); an active stack hit by a redirected Blind in its own turn finishes it (r2b); every routine that turns a stack (0x446720: the strike, the melee attack's struck stack, a walk's steps, the creature's cast, the flight, the teleporting move, the raise; §7.25) may measure at its turned front hex: the breath's second hex and the attack patterns are leads, the placement and the berserk shooter's distance (#150's call 2) unread.

- **The surrender's price:** the halving before the discount (0x47506B; one Pixie at 10 %: 10, not 11) is not observed; the temporary creatures are left out since batch 2g SP (BAT-FLE-002).

- **Numbers and draws:** BAT-MAG-058 takes the larger of a cast Fire Shield and the creature's own, the original the cast one first (no difference with the base values; r2b); the physical damage's stage 1 runs its hero step also at a rate of 0, rounding through `f32` (0x44371C; a difference only above 2^24; r2a); the original rolls also at a chance of 0 or 100 and walks an area's stacks from the last to the first (only the random stream differs; r1); the obstacle generator is not draw-exact (BAT-GEO-004 is VCMI's: the original draws one `rand()` per tried type and anchor, 91 usual types, 34 absolute, 151 anchors, through the C runtime's generator; its Stronghold siege branch 0x466147 is not modelled; does it keep an obstacle off another's free anchor? ADV-QUIRK-5's "layouts match the original's" needs both).

- **Other readings:** the corpse limit of 14 per hex (not modelled; it also bounds the Pit Lord's search); the stack bit `0x4000000` set after a creature's cast; a `rebirth` creature with several casts (data only); BAT-QUIRK-17's later variant, a sacrificed Phoenix reborn right after the sacrifice (BAT-Q18); the creature flag bit `0x800` (Arch Mage, Beholder, Evil Eye; `h3-harvest`); the Faerie Dragon's key path under the Orb refuses a Chain Lightning at the caster's side's stacks, which the engine offers (BAT-Q32 (1)); the knights' Curse stays at 25 % and mastery 0 until a statistics batch (BAT-Q28 row 13); Q43 (the sprite offset). From E04c's security review: whether the turrets' counts (BAT-SIE-016) show the town's building count where the original does not (its tower window is unread), and how long one command runs with many rounds of turret turns inside it (no bound measured).

- **Code left over:** BAT-STUB-1's per-stack mark at the next observation-encoding bump, with BAT-Q35's answer (`cross-lane` with the AI lane). From E04b's review fixes: a bound on the `SpellDamage` rates in `hero::check` (the fix saturates instead; r1 F-5); a `fixed` duration of up to 65 535 rounds on a turn-stopping change still compiles (r2b F-7); `h3_battle::stack_spell` answers without a viewer and `BattleState::spell_obstacles()` returns the hidden ones too (none leaks today). Doubts of the fold-in's read, none a defect: at a round start the end check comes before the obstacles' countdown, so `ObstacleRemoved` can follow `BattleEnded`, a draw there untested (A6); without BAT-QUIRK-5 a created stack is still a Sacrifice target (B7); a walker that strikes and returns is put back without entering anything when its start is out of reach, its path computed once across rebirths (B9); `magic::opening` asks for an active effect where a hero's Expert Dispel does not; a stack that resists while a chain looks for its next stack is passed over without `SpellResisted`.

- **Docs:** code-reading sources that word older readings: BAT-MAG-006, 022, 032, 035, 048, 049, 057, 066, BAT-MOR-004; the battle tests' profile `t:faithful` has no quirks (a misleading name). (`packs/README.md`'s "Profiles" and `installation.md` §7.23's order: done 2026-11-02.)

- **Performance:** `random-rollouts-full` was re-baselined in E04b T11b at +3.3 % Ir per step without a per-function profile (candidates: a `may_affect` more per spell without a target in `castable_spells`, the automatic Ballista's reach, dead stacks' kept effects), and stands at +1.34 % on `master` (2026-10-19). Its Ir swung by 7 to 12 % with the inlining of `Field::neighbor` three times in E04a: a precomputed neighbour table in `Field` if it swings again.

- **Tooling, later:** `scripts/exe.py switch` misses a `dec` (and `inc`) before the range check, so it labels the dispatcher's cases (0x47902F) one lower than the action (E23 T1a1; `Code._switch_base` handles `lea`, `add`, `sub` only: a `shared/` fix); a `Trace` sink for rule-level explanations (02-architecture §10); profile-level Tier-2 parameters (ADR-0014; data quirks are done, ADR-0007); `cargo-fuzz` targets for `h3-formats`; the `.h3m` writer's tile pictures beyond dirt, grass and water (from a game map first); a batch for what batch 2g left (BAT-Q28 rows 1, 3, 5, 6, 11, 12 and 13's statistics, BAT-Q32's key path, the survey's S-26, S-27).

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").

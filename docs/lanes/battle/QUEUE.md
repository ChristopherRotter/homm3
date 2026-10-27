# Battle lane — QUEUE

Pre-refined tasks of the battle lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

E23, the original's battle AI, is **closed** (2026-10-24; `docs/epics/E23-original-battle-ai.md`, its log's "T7 — the closure"; the cuts of its tasks: OPEN-QUESTIONS BAT-Q52 to BAT-Q61 and the epic file). E04b closed on 2026-10-12, E04c on 2026-10-16 (their epic files and logs; their cuts: OPEN-QUESTIONS BAT-Q10 to BAT-Q27, BAT-Q37 to BAT-Q50).

**Next (the user, 2026-10-24):** the inbox session's leftovers are done (#458, #464; CLT-Q4/Q6's seam and carry, the 112th session); next **E11, the historical rules of RoE and AB** (refined 2026-10-24, Q58; `docs/epics/E11-historical-rules.md`): T1, the sourcing and catalogue with the user's own original media.

## Inbox

Consolidated on 2026-10-19 (the user): the entries from E01 to E04c's closure are merged below by kind, done and duplicate parts dropped; the origin stays in brackets (an epic and task, a review report such as r3 or F-r2b-2). History: git. Items for other lanes went into their inboxes; the First Aid Tent became BAT-Q51. The addresses are readings of the executable, the reviews' not read a second time. Where the readings stand: `installation.md` §7.8–§7.33; the reviews' reports stay on `battle/e04b-t11b-review-reports` and `battle/e04c-t10b-review-reports`; their experiment candidates are in `docs/experiments/README.md` ("From the battle queue's inbox", "From the E04b review", "From E04c").

- 2026-10-27 (the adventure lane, E08 T59, `shared/e08-t59`, `cross-lane`; for your information,
  nothing to do): **your harvester and packs** gained increment 7's shared part: the gap-fill's
  `[heroes.params]` `mana_ceiling = 999` (source `exe-mana-ceiling`) written into `[hero_params]`
  (pack format v48, read by `h3-heroes` from E08 T60), and both profiles name
  `quirk.adventure.roe_witch_hut_mask` and `quirk.adventure.full_backpack_refuses` (registry
  positions 37 and 38, `ADV-QUIRK-21` and `-22` in `docs/rules/quirks-adventure.md`): your next
  flag is position 39. `base` re-harvested: three lines. Core ruleset hash v49: your ten replays'
  headers moved only (synthetic faithful `0xf957d87efe3a32c6`).

- 2026-10-27 (the AI lane, E06 T6f and T8b, `claude/jolly-volta-qejdfa`, `cross-lane` with the
  user's consent; **for you**): two additions to `h3-battle` the AI lane made and reads.
  **`h3_battle::turn_order(&Observation) -> TurnOrder`** (`turn.rs`): the rest of the round's
  turns and the next round's, sharing `next_active`'s key (`OrderKey`), so a change of the order
  rules (BAT-TRN-001..003, BAT-SIE-011/016) changes both at once; the AI lane's observation v22
  encodes it per stack, and a battle client could show it as a queue. **A search agent's worlds,
  `BattleState::determinized` and `reseed`** (`world.rs`, ADR-0025 point 12a): a copy with
  everything a side may not know (`observe_as`'s list) replaced; `state::world_templates`,
  `obstacles::place_unseen_again` and `Created::set_templates` serve it, and `templates` takes
  the armies' creatures. **A change of what a side may not know** (a new hidden field, a new
  hidden obstacle kind) **changes `world.rs` in the same change** (the ADR's point 2), as it
  changes `observe_as`; the property `a_world_shows_its_viewer_the_battle_and_reads_nothing_hidden`
  catches most. A trap for your `TRAPS.md` ("Battle code"). Known gap: AI-STUB-1.

- 2026-10-24 (the E11 refinement, `shared/e11-refinement`, Q58): **E11 is yours to lead**
  (`docs/epics/E11-historical-rules.md`): the historical rules of RoE and AB from the user's own
  original media. **The order (the user, 2026-10-24):** E23 closed first (`battle/e23-closure`),
  then one session on this inbox's leftovers, and only then E11's T1 (sourcing and catalogue, with
  the user, when the media are at hand). The sketch's "AB and SoD completeness" is folded; E03's
  note that AB/SoD abilities "stay E11" is history (built in E04b).

- 2026-10-23 (the AI lane, E06 T6a; the preview part done 2026-10-24, BAT-DMG-011): **the
  gated batch benchmark with `System` seats** (ADR-0027 point 11, E23 exit criterion 4),
  `benches/ai/batch-system-neutral-v1.toml` (the neutral kinds, the system actor defending; 6.81
  G instructions at `POLICY_VERSION` 19, about 532 000 a caller step), is lane-only: a change of
  `h3_battle_ai` moves it, and the AI lane re-baselines it.

- 2026-10-23 (the AI lane, E06 T7c, `claude/bold-curie-a42rc4`; **for you, with each
  `h3_battle_ai::POLICY_VERSION` bump**): the system actor is now a bot of the AI lane's tables,
  `python/records/bots-report-v1.json` (it against Random, Aggressor, Greedy on the report set)
  and the new `bots-neutral-v1.json` (every bot attacking it on the neutral set v1); both name
  the policy version (`engine.system_policy`), and the AI lane's Python test fails when it is
  stale. A pull request that changes the computer's choices regenerates both (from the root, in
  release about 85 s and 15 s; `python/records/README.md`):
  `cargo run --release -p h3-cli -- ai eval --set report-v1 --json python/records/bots-report-v1.json`
  and the same with `--set neutral-v1 --json python/records/bots-neutral-v1.json`. Today's
  numbers (version 18): the system actor beats Greedy 0.653 and Aggressor 0.681 on the report
  set, and attacks best on the neutral set (value +0.285, Greedy's +0.223).

- 2026-10-22 (E23 T4c's blind check; a rule to confirm): the original's end routine 0x444510 returns early only for Disrupting Ray (47), so a dispel or Cure that reaches Acid Breath (80) may end it; BAT-MAG-010 keeps every permanent effect. Read the dispels' callers or play a batch row (Cure on a stack under Acid Breath) and align BAT-MAG-010 or record why not.

- 2026-10-20 (the adventure lane, E08 T24b1, `adventure/e08-t24b1`; hygiene, with your next change to the file): `crates/h3-harvest/data/gapfill.toml`'s source `exe-necromancy` has `url = "docs/epics/E08-design.md"`; that note was rewritten for E08 increment 4 and no longer holds the Necromancy section: `docs/epics/E08-log.md` (T19a2b) does. Increment 4 will also want gap-filled hero data (the experience table, the forced skills, the classes' offer intervals: `E08-design.md` §1, §10); the cut (T24b2) asks you.

- **For E23 (the original's battle AI), to read with it:** the original's pathfinder 0x4B3400 is read (§7.40, §7.41) and built (`pathfinder.rs`, E23 T1c): the walks and the berserk turn follow it. `aim.rs`'s threat records (0x422B20, the turrets' and the Ballista's turns) still search on their own (flyers by hex distance, no marks, no first-in-first-out): move them onto `pathfinder::Search`'s marks (BAT-Q53) with the computer's walker (T3). T1b's readings at risk from T1c: S-11's three outcomes, a berserk flyer ending on a hex it passed over, the war machine aimed at by its front, a walk beyond the speed into its side's seen pit (`E23-log.md`). The automatic Catapult's aim with Ballistics (the computer's and hidden battles': the gate first, §7.29) is not modelled: the agent aims. The moat avoidance's further "automatic" cases (0x474BF0: a hidden battle, an options flag, a remote turn; BAT-SIE-022). BAT-Q33 (who commands a side) decides with it.

- **The controller against the owner** (the engine mostly reads the owner; a hypnotized stack's morale and luck are the controller's since batch 2g HM): a hypnotized shooter's bows, which the original looks for on the controller's hero for the walls' halving (0x46751A to 0x46753A, BAT-SIE-008) and the distance's (0x467600, BAT-RNG-002), where the engine sets traits from the owner's hero at the start (`hero::trait_effects`; F-r2b-2); Hypnotize ending on the active stack mid-turn keeps the acting side until the next activation (0x4653AE), the engine hands the turn to the owner at once (r3); the battle's end with only hypnotized stacks left (unread; the engine counts them for their owner).

- **War machines:** the original sets up none with the battle's bank flag (+0x53C5; 0x463758 to 0x46375D): the setup has no creature bank yet (a side without a hero brings none since 2026-10-24, BAT-WAR-001); the Ammo Cart's test is "the shooter's side's hero wears artifact 5" (0x43F9E1) without a test that the cart stands (BAT-WAR-004; r3); the original's Defend gives a war machine no bonus, nor any stack in the tactics phase (0x479722; does the engine?); the Genie's and the Enchanters' usefulness tests count a Ballista without shots as a shooter only under Q-0004, the original always (0x4428F0).

- **Turns** (batch 2g aligned the wait phase, the freeze's tie-break and the wait phase's turn start, and recorded the good morale before a re-entry, BAT-MOR-003): a `rebirth` creature a mine or Fire Wall kills on its way walks on and attacks after its rebirth (the original probably too, 0x478900; r5b); an active Phoenix killed by its own hero's cast and reborn stays the active stack (unread; r1); an active stack hit by a redirected Blind in its own turn finishes it (r2b); every routine that turns a stack (0x446720: the strike, the melee attack's struck stack, a walk's steps, the creature's cast, the flight, the teleporting move, the raise; §7.25) may measure at its turned front hex: the breath's second hex and the attack patterns are leads, the placement and the berserk shooter's distance (#150's call 2) unread.

- **The surrender's price:** the halving before the discount (0x47506B; one Pixie at 10 %: 10, not 11) is not observed; the temporary creatures are left out since batch 2g SP (BAT-FLE-002).

- **Numbers and draws:** BAT-MAG-058 takes the larger of a cast Fire Shield and the creature's own, the original the cast one first (no difference with the base values; r2b); the physical damage's stage 1 runs its hero step also at a rate of 0, rounding through `f32` (0x44371C; a difference only above 2^24; r2a); the original rolls also at a chance of 0 or 100 and walks an area's stacks from the last to the first (only the random stream differs; r1); the obstacle generator is not draw-exact (BAT-GEO-004 is VCMI's: the original draws one `rand()` per tried type and anchor, 91 usual types, 34 absolute, 151 anchors, through the C runtime's generator; its Stronghold siege branch 0x466147 is not modelled; does it keep an obstacle off another's free anchor? ADV-QUIRK-5's "layouts match the original's" needs both).

- **Other readings:** the corpse limit of 14 per hex (not modelled; it also bounds the Pit Lord's search); the stack bit `0x4000000` set after a creature's cast; a `rebirth` creature with several casts (data only); BAT-QUIRK-17's later variant, a sacrificed Phoenix reborn right after the sacrifice (BAT-Q18); the creature flag bit `0x800` (Arch Mage, Beholder, Evil Eye; `h3-harvest`); the Faerie Dragon's key path under the Orb refuses a Chain Lightning at the caster's side's stacks, which the engine offers (BAT-Q32 (1)); the knights' Curse stays at 25 % and mastery 0 until a statistics batch (BAT-Q28 row 13); Q43 (the sprite offset). From E04c's security review: whether the turrets' counts (BAT-SIE-016) show the town's building count where the original does not (its tower window is unread), and how long one command runs with many rounds of turret turns inside it (no bound measured).

- **Code left over:** BAT-STUB-1's per-stack mark at the next observation-encoding bump, with BAT-Q35's answer (`cross-lane` with the AI lane). From E04b's review fixes: a bound on the `SpellDamage` rates in `hero::check` (the fix saturates instead; r1 F-5); a `fixed` duration of up to 65 535 rounds on a turn-stopping change still compiles (r2b F-7); `h3_battle::stack_spell` answers without a viewer and `BattleState::spell_obstacles()` returns the hidden ones too (none leaks today). Doubts of the fold-in's read, none a defect: at a round start the end check comes before the obstacles' countdown, so `ObstacleRemoved` can follow `BattleEnded`, a draw there untested (A6); without BAT-QUIRK-5 a created stack is still a Sacrifice target (B7); a walker that strikes and returns is put back without entering anything when its start is out of reach, its path computed once across rebirths (B9); `magic::opening` asks for an active effect where a hero's Expert Dispel does not; a stack that resists while a chain looks for its next stack is passed over without `SpellResisted`.

- **Docs:** code-reading sources that word older readings: BAT-MAG-006, 022, 032, 035, 048, 049, 057, 066, BAT-MOR-004; `packs/README.md`'s "Profiles" names only Q-0003 and Q-0005; the battle tests' profile `t:faithful` has no quirks (a misleading name); `installation.md` §7.23 lists Armageddon's Blade's test before the polarity tests, the code has them first (0x5A82F8).

- **Performance:** `random-rollouts-full` was re-baselined in E04b T11b at +3.3 % Ir per step without a per-function profile (candidates: a `may_affect` more per spell without a target in `castable_spells`, the automatic Ballista's reach, dead stacks' kept effects), and stands at +1.34 % on `master` (2026-10-19). Its Ir swung by 7 to 12 % with the inlining of `Field::neighbor` three times in E04a: a precomputed neighbour table in `Field` if it swings again.

- **Tooling, later:** `scripts/exe.py switch` misses a `dec` (and `inc`) before the range check, so it labels the dispatcher's cases (0x47902F) one lower than the action (E23 T1a1; `Code._switch_base` handles `lea`, `add`, `sub` only: a `shared/` fix); a `Trace` sink for rule-level explanations (02-architecture §10); profile-level Tier-2 parameters (ADR-0014; data quirks are done, ADR-0007); `cargo-fuzz` targets for `h3-formats`; the `.h3m` writer's tile pictures beyond dirt, grass and water (from a game map first); a batch for what batch 2g left (BAT-Q28 rows 1, 3, 5, 6, 11, 12 and 13's statistics, BAT-Q32's key path, the survey's S-26, S-27).

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").

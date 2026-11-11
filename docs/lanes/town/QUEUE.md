# Town lane — QUEUE

Pre-refined tasks of the town lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E09-towns.md` (increment 3: growth, recruitment and the army rows, refined at
T20, `TWN-Q11`, decisions 64 to 67); done tasks in `E09-log.md`; increments 1 and 2 (closed by T10
and T20) in `E09-archive.md`. Increment 3's first round (decision 66): the survey T21 and its
checkpoint, then the drafts with the design note's Part E and the cut T22, accepted before
anything is built. Each task's cut confirmed at its start; every entry's acceptance includes the
Definition of Done (`docs/05-testing.md`).

**E09 T21 — increment 3's survey** (decisions 65, 66; queued 2026-11-11 by T20): a survey
`docs/rules/surveys/E09-growth.md` (an agent's name) as T12's: the code reading of `Heroes3.exe`
(the week's step for the towns: each dwelling's growth, the castle's bonus by fort level, the
hordes, the Grail's where it touches growth, the owned dwellings on the map if read, the week's
creature and the month's kinds with the Plague, ADR-0017 point 4; a dwelling's first creatures at
its build; the town events' creatures, `TWN-STUB-10`, TWN-EVT-004; recruitment at a town's
dwellings into the garrison or the visitor, E08's hire dialog `HireDialog` reused; the town
screen's two rows: the garrison's army, a garrison hero, the heroes' swap, and what a garrison hero
changes of the siege and the views), VCMI 1.7.5 and the wiki compared, a census of the shipped
maps (the towns' dwellings and garrisons at the start, the events' creatures, heroes in towns),
the disputes (`TD-13` on) and quirk candidates (`T-10` on), the experiment rows at risk (batch
TWN-3), the checkpoint's questions. With the adventure lane: E08 T108a reads its garrisons'
exchange (E08 decision 295) at the same time; agree one army reference through the inboxes
before either drafts (decision 67). Needs the original data. **At its start:** confirm the cut
(by family: growth and the events' creatures; recruitment; the two rows, measuring the reading
first). **Status:** next.

**E09 T22 — increment 3's drafts, the design note's Part E and the cut** (decision 66): the ADR
amendment drafts (ADR-0028, and the others the survey names) in `adr-drafts/`, Part E of
`E09-design.md` from §37, the building tasks' cut and the exit criteria, accepted by the user at
the checkpoint (`TWN-Q12` on). **At its start:** confirm the cut. **Status:** after T21.

## Inbox

Notes from earlier work for E09, gathered at the lane's opening (2026-10-31, T0); each points to
where the detail is (grep the date or id in the adventure lane's `QUEUE-ARCHIVE.md` or `QUEUE.md`).

- 2026-11-11 (the adventure lane, E08 T108a1 and T108a2, `adventure/e08-t108a` and
  `adventure/e08-t108a2`; **for T21's two rows and the army reference, E09 decision 67**): what
  T108a read of a garrison's exchange (survey `docs/rules/surveys/E08-garrisons-bonuses.md` §6):
  `garrison_exchange_dialog` 0x5D1130 builds the army dialog `garrison_dialog_create` 0x5CE830
  with the garrison's army and owner in the upper bar (no hero) and the visitor's in the lower (no
  town); its moves go through the town screen's own hover code (`town_garrison_hover` 0x5C7400,
  `town_garrison_hover_other` 0x5C8080: a move only from the current player's bar; into another
  owner's bar only into an empty slot), so the town's two rows and the map's garrisons likely
  share one dialog. Decided (E08 decision 299, D-61 the original): an ally's or a non-removable
  garrison takes the visitor's stacks into its empty slots only, no take, merge or swap. Sketched
  (survey §11, not decided): the exchange as a pending decision of the visitor's owner like the
  Leave Guards dialog (GAM-DEC-004 amended), the garrison's army in the adventure state's records.
  The adventure lane's T108b drafts the army reference after your T21's note in its inbox (the
  town lane's note of 2026-11-11 there): read the town's case of the same functions (a garrison
  hero, the swap) before assuming a new dialog.

- 2026-11-10 (the adventure lane, E08 T105b2, `adventure/e08-t105b2`, `cross-lane`; **a town's
  revisit, for your visit**; increment 11's design note §13): `AdventureCommand::Revisit { hero }`
  (ADV-TEL-008) visits the entered object under a hero again as a step's last step onto it would,
  no points, no step, no guard test: for a hero standing on its own town's entrance that is
  `visit_town` with `Visit::Enter` (`town_visit` as it stands), at 0 points too and as often as
  the player likes. `legal_commands` lists it for every such hero after the `BuildBoat`s (and
  before a town's builds once T18b2 lists them: `revisitable`'s place is after `buildable_shipyards`
  in `h3-game`'s `legal.rs`). Whatever a town's visit opens later (its screen, a garrison
  exchange) opens at a revisit too; nothing moved in your hashes. The replay format is v12
  (`revisit <hero>`): a town bump of the format is v13.

- 2026-11-10 (the server lane, E16 M5 T3d; **for when `TWN-STUB-13` is lifted**): your
  `TownChanged`, `TownStackSeen` and `TownIncomeSeen` (E09 T18b2a) are mapped and folded in
  `h3-protocol` (the fold replaces the viewer's town view); `SRV-STUB-12` and `-13` are lifted.
  The fold gate (`h3-protocol`'s `tests/game_fold.rs`, the 55 golden replays) now exempts the
  garrisons and the town owner facts only for a reveal and for `TWN-STUB-13` (3 viewer-commands:
  another team's town falling from level 3 to 2 as a Visions holder's reach is lost). Once
  Visions' reach is told, the gate fails with "lift [...] from it": drop `VISIONS` from those two
  parts in the same pull request (`cross-lane`), as a new event field or variant is mapped there
  (the global trap). This note replaces the server lane's of 2026-11-09 (T3c2).
- 2026-11-09 (the adventure lane, E08 T105a1, `adventure/e08-t105a1`; **a signature, for your
  callers**; increment 11's design note §13, §18): `AdventureState::apply` takes `(player, draws)`
  in place of `player` (`draws: &mut Rng`, the acting player's stream: a visit's draw, a
  monolith's exit, comes from it) and `visit_held` takes `draws` before the sink; the game passes
  them from `GameState::adventure_for(player)` (the adventure state, the store of heroes and that
  player's stream). Every caller in the adventure lane's paths is updated, none of yours called
  them; a new caller in `h3-game`'s `src/town*` (a town command that visits) uses
  `adventure_for`, and the adventure crate's tests `testing::stream()`. Nothing moved in a hash.
- 2026-11-05 (the adventure lane, E08 T97a, `adventure/e08-t97a`; **for the week's creature,
  increment 3**): the map's dwellings and Refugee Camps ignore the week's and the month's kind
  (ADV-DWL-003: the original's `dwelling_week_refill` 0x4B8A60 reads neither; survey
  `E08-dwellings.md` §6.1). The month's side is tested (`h3-game`'s
  `a_month_s_kind_never_reaches_a_dwelling_or_a_camp`: a Plague and a creature month); the week's
  creature is not drawn yet. When it lands, extend that test with a week of the hut's creature:
  the hut's and the camp's counts stay at their growth.
- 2026-11-05 (the adventure lane, E08 T95b, `adventure/e08-t95b`; **for a town's recruitment,
  E09 decision 4**): the hire dialog is built to be reused (increment 10's design note §13,
  "E09's reuse"). `h3_adventure::HireDialog { target: HireTarget, source: HireSource }`: extend
  `HireSource` with a town's dwelling and `HireTarget` with the garrison; the map's purchase rule
  stays one (`AdventureState::hire_stock`, `hire_room`, `check_hire` → `HirePurchase`, `hire`;
  ADV-HIR-002), and so does the game's decision (`h3-game`'s `state/hire.rs`: `Decision::Hire`,
  `Answer::Hire(HireAnswer::{Buy, Close})`, the legal set of each creature's most, `hire_range`,
  `DecisionView::Hire`, `GameState::hire_dialog`; GAM-DEC-008). The env's four purchases (action
  encoding v7) take the source's order, so a town's dwelling of up to four creatures needs no
  new entry. Game state hash v25 is T95b's.
- 2026-11-04 (the adventure lane, E08 T94b, `adventure/e08-t94b`; **for the towns' external
  dwellings' growth**; its versions half done by E09 T8a1, v21): A dwelling's counts and guards as
  they stand:
  `AdventureState::dwelling_counts` and `dwelling_guards` (by object index; none of a block while
  every dwelling is as its week began); the week's refill of every dwelling is the first step of
  `AdventureState::start_week`, before the weekly objects and the camps, reading no week's or
  month's kind (ADV-DWL-003; your towns' growth is separate, survey `E08-dwellings.md` §6.1).
- 2026-11-04 (the adventure lane, E08 T94a, `adventure/e08-t94a`; **for the towns' external
  dwellings' growth**, E09 survey §8.3; adventure design note §11): the static map lists the
  dwellings in object order (`StaticMap::dwellings`, `dwelling_index`; a random dwelling not tied
  to a town is resolved into its drawn faction's and level's dwelling when the game is built, so
  it is listed as a placed one; since E09 T11 one tied to a town too), their owners are `AdventureState::owner` (ownable since T93a, the
  setup's owner at the start) and their creatures the kind's (`ObjectKind::Dwelling::creatures`,
  each creature's faction the core's). Their counts and guards come with T94b (adventure state hash
  v20). A random dwelling tied to a town resolves since E09 T11, after the towns' draws (ADR-0017
  point 4).
- 2026-11-04 (the client lane, E17 M6 T3c3, `client/e17-m6-t3c3`; **when you change
  `h3-towns`' or `h3-pack-towns`' public API**): the battle client's binary now compiles the packs'
  town part (`h3_pack_towns::compile`) and reads `TownRuleset::town`, `TownRecord::building` and
  `buildings`, `Building::key` and `slot` and `BuildingId::from_index` (a siege's buildings and a
  Grail building by key in the stats dialog, the original's map by slot; design E17 §12.11). CI's
  clients job watches both crates (#704): a change there runs `clients/scripts/check.sh`, locally
  `scripts/verify-once.sh clients --base origin/master`; a dependency change updates
  `clients/Cargo.lock`.
- 2026-11-03 (the battle lane, CLT-Q40; its siege half done by T6b, `QUEUE-ARCHIVE.md`): the
  owner's Grail building that adds a hero's morale or luck (the Colossus, the Spirit Guardian)
  names itself as `EffectSource::Building { faction, index }` on that hero's `BattleEffect`, when
  the game gives it.
- 2026-11-03 (the adventure lane, E08 T87c, ADV-Q41; **for your increment 3's recruitment, nothing
  to build now**): E08's increment 10 is dwellings and hiring (decision 231): the map's creature
  dwellings (17, 20), the random dwellings with a faction set and the Refugee Camp, their weekly
  creatures and **the hire dialog**, built in the adventure lane first (E09 decision 4); the random
  dwellings tied to a random town resolve since E09 T11. Its survey (T88a,
  `docs/rules/surveys/E08-dwellings.md`) and design note (T88b) come first; this lane's inbox is
  told when the hire dialog lands.
- (adventure archive, 2026-10-03, E04a T3a) Not harvested: the spells' town chances and the hero
  classes' tavern weights (RoE lacks the Conflux column: an override kind or a per-faction table).
  Increment 4, the Tavern (weights: only the tavern's draw reads them, survey `E09-towns.md`
  §4.4), increment 5 (chances).
- (adventure queue, 2026-10-20, T24a2b1) `Marshland Menace` gives 15 hero types a map-wide
  experience of 2 000 that only its taverns reach. Increment 4, the Tavern.
- (ADR-0017 point 4; E08 decision 197) The week's creature, drawn from a stream of its own, and the
  Plague's effect on towns. Increment 3.
- (`ADV-STUB-9`, E10's) A Tavern town to hire a retreated hero again: a fact of the battle's
  retreat setup. Increment 4, the Tavern, with E10.
- (ADR-0013 point 13.6; E08 decision 199, A-30) The Castle's Lighthouse building (sea movement).
  Increment 5.
- (adventure archive, 2026-10-14, T19a2b) The Necropolis buildings' +10/+20 % Necromancy
  (0x4E3CD0 with the building bits 0x66CE40, 0x66CE68). Increment 5.
- (adventure archive, 2026-10-03, E04a T3b) Combination artifacts' costs (blank in the original):
  the Artifact Merchants. Increment 5.
- (adventure archive, 2026-10-11, T8a; T34b) The new day's reveal around owned objects after a
  Cover of Darkness hid tiles (0x4C7BA0). Increment 5.

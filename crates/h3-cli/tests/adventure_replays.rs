//! Golden adventure replays (E08 T9b, increment 1's exit criteria): every replay under
//! `fixtures/replays/adventure/` must reproduce its recorded state hashes on this platform and
//! build configuration. `cargo xtask determinism` runs this test in debug, release and release
//! with `-C target-cpu=native`; running it on another OS is the cross-platform check. The random
//! replays play the generated duel map `fixtures/maps/e08-duel-36.h3m` (re-record them with
//! `h3 adventure run … --record` when a rule or an encoding changes on purpose; the commit says
//! why), recorded with `--map fixtures/maps/e08-duel-36.h3m` and the options below. Since E08
//! T20b3b2 the duel holds increment 3's monsters, guarded piles and special grounds, a monster and
//! a guarded pile beside each hero, so random play meets them in every game, and since E08 T31 its
//! heroes' custom experience brings their first battles to a level-up, and since E08 T42 red's
//! surface hero flies (Angel Wings) and its underground hero walks on water (Boots of Levitation),
//! with the three skills and the other artifacts among the four (`fixtures/maps/README.md`); what
//! each replay meets (from the E08 T42 recording; a flight's or a water walk's pass is a step over
//! a tile its hero may only pass, before the path's last; since E08 T85b the seven duel replays of
//! 28 days are recorded again from their seeds, the day advance having changed their games: E08
//! T86a checked their lists again and replaced the four that had lost what they were chosen for,
//! increment 9's design note §19):
//!
//! - `duel-s1-d28` (`--seed 1 --agent-seed 1 --days 28`): a key, a monster's battle won and its
//!   level-up answered with the second offer; flights over tiles objects block and one over water,
//!   a visit at a flight's end (a Keymaster's Tent), water walks.
//! - `duel-s2-d7` (`--seed 2 --agent-seed 3 --days 7`): flights over tiles objects block, a water
//!   walk over water; no battle.
//! - `duel-s5-strict-d7` (`--seed 5 --agent-seed 8 --days 7 --profile base:strict-fixed`): in
//!   strict-fixed (the split, the obstacles and the levels from the streams) a flight pursued and
//!   its battle won, the level-up answered with the first offer; a flight over water and over
//!   blocked tiles.
//! - `duel-s3-easy-d7` (`--seed 3 --agent-seed 4 --days 7 --difficulty 0`): at the easy difficulty
//!   (E08 T14b: the replay's difficulty, which `verify` plays at) a monster's battle won and its
//!   level-up answered with the first offer.
//! - `duel-s3560-d28` (`--seed 3560 --agent-seed 3560 --days 28`; E08 T86a, for T42's
//!   `duel-s692-d28`): a hero battle won by its attacker, blue's surface hero, which captures red's
//!   flyer's two artifacts and flies with its Angel Wings from then on; two Cartographers
//!   declined, a declined join and a group joining, a pursuit, four monsters' battles won, a
//!   guarded pile fought at once and picked up, a key, level-ups answered with both offers;
//!   flights over blocked tiles, a visit at a flight's end (a Cartographer), water walks.
//! - `duel-s4893-d28` (`--seed 4893 --agent-seed 4893 --days 28`): a border guard's passage
//!   accepted with its key (two keys taken), a declined join and a group joining, a declined
//!   flight and a pursuit won, a pile's question accepted and its guards beaten, level-ups answered
//!   with both offers; flights over blocked tiles and six over water, a visit at a flight's end,
//!   water walks, the border guard visited at a water walk's end.
//! - `duel-s4749-d28` (`--seed 4749 --agent-seed 4749 --days 28`; E08 T86a, for T42's
//!   `duel-s2517-d28`): a key, a Cartographer accepted, a declined flight, two monsters' battles
//!   won, a pile's question declined and another's accepted and its guards beaten, a pile fought at
//!   once whose guards beat the visitor (their survivors stay, the hero leaves the map), level-ups
//!   answered with the second offer; flights over blocked tiles and one over water, a visit at a
//!   flight's end, water walks.
//! - `duel-s645-d28` (`--seed 645 --agent-seed 645 --days 28`; E08 T86a, for T42's
//!   `duel-s3127-d28`): a Cartographer declined, a pursuit, two monsters' battles won and one lost,
//!   a pile's question accepted and its guards beaten, a pile fought at once whose guards beat the
//!   visitor (both of blue's heroes leave the map), level-ups answered with both offers; flights
//!   over blocked tiles and one over water, a visit at a flight's end.
//! - `duel-s1308-strict-d28` (`--seed 1308 --agent-seed 1308 --days 28 --profile
//!   base:strict-fixed`): in strict-fixed a key, a Cartographer accepted, a declined join and a
//!   declined flight, a pursuit, monsters' battles won, a pile's question accepted and its guards
//!   beaten, a hero battle lost, the winner capturing two artifacts, level-ups answered with both
//!   offers; flights over blocked tiles and over water, a water walk.
//! - `duel-s5249-d28` (`--seed 5249 --agent-seed 5249 --days 28`; E08 T86a, for T42's
//!   `duel-s2931-d28`): an offer to join for gold accepted and a join accepted (two groups
//!   joining), a declined flight and a pursuit, three monsters' battles won, a pile's question
//!   declined, level-ups answered with the second offer; flights over blocked tiles, a flight
//!   ending in a monster's battle, water walks.
//!
//! In every one red's underground hero gains levels 2 to 4 at load (E08 T31: the duel's custom
//! experience, `fixtures/maps/README.md`; from its level seed in faithful, from the setup stream in
//! strict-fixed). The last six came from a search over seeds (E08 T42, as T31's: `--seed N
//! --agent-seed N --days 28` in each profile, with a scratch probe replaying each game's commands
//! and noting its offers, answers, battles and their ends, its flights and water walks, not
//! committed); E08 T86a searched again after T85b's day advance, N up to 6 000: of the faithful
//! games 85 have a hero battle (46 won by its attacker, 79 with a capture), 24 a guard's passage
//! accepted, 181 a pile's guards beating the visitor, 680 a pile's question declined, 750 a
//! monster's battle lost, two an offer to join for gold accepted (three of the strict ones), 3 287
//! a flight over water, 2 084 a water walk over water and every game but two a flight over blocked
//! tiles. T42's `duel-s692-d28` had lost its hero battle, `duel-s2517-d28` its pile's question
//! declined, `duel-s3127-d28` its monster's battle lost and `duel-s2931-d28` its offer to join for
//! gold: each was replaced by the game closest to its row (`duel-s3560-d28`, `duel-s4749-d28`,
//! `duel-s645-d28`, `duel-s5249-d28`); `duel-s1-d28`, `duel-s4893-d28` and
//! `duel-s1308-strict-d28` kept theirs. Before, T42's six had replaced T31's five
//! (`duel-s767-d28`, `duel-s2626-d28`, `duel-s236-d28`, `duel-s12-d28`, `duel-s929-strict-d28`)
//! and `duel-s3-easy-d7`'s offer to join for gold.
//! History: E08 T16a re-recorded the first four when the duel got increment 2's objects and added
//! two searched ones, E08 T19b1 two with hero battles (format v3); every version bump since (T20a1,
//! T20a2a, T20b1, `shared/e08-t20b2`, E04c T9, T21a, T29, T30) re-recorded them all with their
//! options, their commands unchanged; a change of the duel changes every command: E08 T20b3b2
//! (monsters, guarded piles, grounds), E04c T10b (the original's obstacles, `cross-lane`) and E08
//! T31 (the heroes' experience) re-recorded the first four with their options and replaced the
//! searched ones, which no longer met what they were chosen for. E08 T38b (ADV-MAP-002's count:
//! every tile no object blocks, not the coast) moved red's levels at load in `duel-s1-d28`,
//! `duel-s3-easy-d7` and T31's `duel-s236-d28`: re-recorded with their options, their commands
//! unchanged, only their checkpoints and final hashes moved. E08 T40 (the heroes' day's points,
//! Pathfinding and radius from their details; adventure ruleset part v10) changed every game:
//! all nine re-recorded with their options, their commands new. E08 T42 (the duel's kits) changed
//! every game again: the first four re-recorded with their options, the searched ones replaced and
//! the tenth added. E23 T5b2b (battle state hash v52, `cross-lane`) re-recorded all ten with their
//! options, their commands unchanged: nine moved checkpoint hashes taken inside a battle,
//! `duel-s2-d7` none; so did E23 T6 twice (battle state hash v53: the tactics phase's rounds,
//! then the retreat facts and the attacking hero's own ground). E08 T49a (the worn artifacts by
//! position: game state hash v12, map import v9, the duel regenerated with its heroes' Spell Books
//! on the map's book entry) re-recorded all ten with their options, their commands unchanged:
//! every checkpoint moved. E08 T49b (map import v10: an equipped Titan's Thunder adds a Spell
//! Book, which the duel's heroes do not wear) moved only the header's import version; every hash
//! stayed. E08 T50 (adventure state hash v11: the tactics flag; adventure ruleset part v11: a
//! stack's most creatures) re-recorded all ten with their options, their commands unchanged:
//! every hash moved. CLT-Q4/Q6 (battle state hash v54: the viewers' facts, `cross-lane`)
//! re-recorded all ten with their options, their commands unchanged: nine moved checkpoint hashes
//! taken inside a battle, `duel-s2-d7` none. E08 T51b (game state hash v13: the meetings and the
//! army dialogs) re-recorded all ten with their options, their commands unchanged (no game meets
//! an own hero or a join without room): every hash moved. E08 T54 (the battle seam's facts: game
//! state hash v14, adventure state hash v12, map import v11; the duel's players a human may play
//! count as human, so the bad-morale let-off no longer reaches their sides in faithful, which no
//! game's battle met) re-recorded all twelve with their options, their commands unchanged: every
//! hash moved. E08 T60a (game state hash v15: a hero's battle bonuses, none of which a game here
//! gives) re-recorded all twelve with their options, their commands unchanged: every hash moved.
//! E08 T61a (adventure state hash v13, ruleset part v12, map import v12: increment 7's kinds and
//! what objects hold, none of which either map has) re-recorded all twelve with their options,
//! their commands unchanged: every hash moved. E08 T61b (adventure state hash v14, map import v13:
//! the draws when the game is built, the visits' block, the scenario's bans; neither map has an
//! object that draws, nor bans) re-recorded all twelve with their options, their commands
//! unchanged: every hash moved. E08 T62a (game state hash v16: a gift's price and a find's
//! question among the offer kinds, neither map has one) re-recorded all twelve with their options,
//! their commands unchanged: every hash moved. E08 T62b1 (game state hash v17: Pandora's Box's and
//! the Pyramid's questions among the offer kinds, neither map has one) re-recorded all twelve with
//! their options, their commands unchanged: every hash moved. E08 T62b2 (adventure state hash
//! v15: each player's colour and human flag) re-recorded all twelve with their options, their
//! commands and days unchanged: every hash moved. E08 T63a (adventure replay format
//! v7: the answers typed by the decision's kind, `answer offer accept`, `answer level-up first`,
//! `answer close` for a meeting or an army dialog; game state hash v18: the choices and the
//! Universities among the decisions, neither map has one) re-recorded all twelve with their
//! options (the scripted two from their commands in v7's words), their commands unchanged but for
//! their words: every hash moved. E08 T63b (game state hash v19: the rests, GAM-DEC-007, the
//! guard's wait no longer in a decision; a won guards' battle's pickup after the winner's levels)
//! re-recorded all twelve with their options, their commands unchanged: every hash moved. E08
//! T83a (adventure ruleset part v14, adventure state hash v17, map import v15: increment 9's
//! kinds and parameters, an abandoned mine's set; none of the four maps has an increment 9
//! object) re-recorded all thirty-two with their options (the scripted ones from their commands),
//! their commands and days unchanged: every hash moved. E08 T83b1 (adventure ruleset part v15:
//! the flag radius; adventure state hash v18: owners, weekly contents, an abandoned mine's draws,
//! a Shipyard's boat tile; none of the four maps has an increment 9 object) re-recorded all
//! thirty-two from their commands, the random ones also played again with their options to the
//! same commands: every hash moved. E08 T83b2a, T84a and T85b1 re-recorded them as their records in
//! `E08-log.md` say; T85b1's day advance changed the seven duel games of 28 days, recorded again
//! from their seeds (above). E08 T93a (the part's hash) and T93b (map import v16) re-recorded
//! them all from their commands: T93a moved the hashes, T93b the import version alone. E09 T7b1
//! (game state hash v23: the hero pool) re-recorded them all from their commands: only the
//! hashes moved (their maps have no random hero, pick or start that reads a random faction). E08
//! T94b (adventure state hash v20) re-recorded them all from their commands: every hash moved.
//! E09 T8a1 (adventure state hash v21, adventure ruleset part v17, map import v18: the town as a
//! map object) re-recorded all thirty-nine from their commands: only the hashes and the versions
//! moved. E08 T95b (game state hash v25: the hire dialog; replay format v10: its answers; none of
//! the maps holds a dwelling, a random dwelling or a camp) re-recorded all thirty-nine from their
//! commands: only the hashes and the versions moved. E08 T97b1 (map import v19: a dwelling entered
//! from water imported, a tied random dwelling's checks; none of the maps holds either)
//! re-recorded all forty-seven from their commands: only `import_version` moved. E09 T11 (map
//! import v20: a random dwelling tied to a town typed; none of the maps holds one) moved the
//! fifty-five replays' `import_version` alone, which this test confirms.
//!
//! **Scripted** (E08 T52, increment 6: the random agent manages nothing), on the generated
//! `fixtures/maps/e08-army-12.h3m` (red's Valeska, hero 0, with a full army and twelve artifacts
//! in her backpack; her mate Caitlin, hero 2, east of her with expert Scholar, advanced Wisdom,
//! six spells and Elixir of Life's parts worn; six compliant goblins west of her), recorded with
//! `h3 adventure run --map fixtures/maps/e08-army-12.h3m --seed 1 --checkpoint-every 5 --script
//! <script> --record <replay>`; a replay file is its own script (`--script <replay>` re-records
//! its commands when a hash moves):
//!
//! - `army-meeting-s1`: Valeska steps onto Caitlin: Scholar teaches her the five spells of level 2
//!   at most she lacks (HER-SPL-007), the meeting opens (GAM-MET-001, GAM-DEC-003); her Wood
//!   Elves go into Caitlin's empty slot, three Archers onto Caitlin's by a split, Caitlin's
//!   Pikemen and her Swordsmen swap, she dismisses her Monks (ADV-ARM-001 to -004); Centaur's
//!   Axe goes from her backpack to Caitlin's right hand, the Ring of Vitality to her ring and
//!   back (HER-ART-005); Caitlin assembles Elixir of Life, takes it apart and assembles it again
//!   (HER-ART-006); a formation and a flag are set (ADV-ARM-006); the meeting closes (`decline`)
//!   and Valeska walks on; both players end their turns.
//! - `army-dialog-s1`: Valeska steps onto the goblins, whose offer to join she accepts with no
//!   room: the army dialog opens with the six (ADV-MON-008, GAM-DEC-004); she dismisses her Wood
//!   Elves, takes four goblins by a split from the group into the freed slot and moves her Monks
//!   into the group; closed, the group's two goblins and the Monks are lost; both players end
//!   their turns.
//!
//! **Rewards** (E08 T64a, increment 7), on `fixtures/maps/e08-rewards-36.h3m` (every object class
//! of increment 7, written by `h3_map::writer`; `fixtures/maps/README.md`): red's Orrin (hero 0,
//! a full army), Adela (hero 1, basic Wisdom alone, a Spell Book, 950 experience) and Mephala
//! (hero 2), blue's two heroes. Random play, recorded with `--map
//! fixtures/maps/e08-rewards-36.h3m --seed N --agent-seed N --days 28`: the three of a search
//! over the seeds 1 to 400 (a scratch probe noting the objects each game's moves end on and the
//! decisions it opens, not committed; one game visits 3 to 9 of the 31 classes, every class in
//! some game) whose union meets the most:
//!
//! - `rewards-s162-d28`: a Learning Stone's level-up, the box east of Orrin accepted without room
//!   (its army dialog closed, the wood elves lost, then its experience and level-up), a Witch Hut,
//!   a chest's gold, a meeting, Marletto Tower, a School of Magic paid, a random major and minor
//!   artifact picked up, a guard's battle won, blue's event with guards (its battle won, then its
//!   artifact).
//! - `rewards-s243-d28`: the event east of Mephala, a Star Axis, a Spell Scroll, a Shrine, the
//!   pursuit of the fleeing gnolls with an artifact declined (they leave), the event for everyone
//!   (morale and luck), a Warrior's Tomb searched (its artifact and −3 morale), a School of War
//!   left, the Pyramid declined.
//! - `rewards-s88-d28`: a Scholar, a guard's battle and days later the event it guarded, Adela on
//!   the guarded Learning Stone (its level-up, then the guard's battle), a School of Magic paid,
//!   the University's four purchases, a chest's experience and its level-up, a Witch Hut.
//!
//! **Scripted** (E08 T64a: the order-dependent paths of increment 7's design note §16), on the same
//! map, recorded with `h3 adventure run --map fixtures/maps/e08-rewards-36.h3m --seed 1
//! --checkpoint-every 5 --script <script> --record <replay>` (the battles' commands from a scratch
//! driver that attacks or shoots when it can, else defends; not committed); each ends with both
//! players' turns:
//!
//! - `rewards-box-dialog-s1`: Orrin opens the box east of him (ADV-PIK-013) with no room: the army
//!   dialog opens with the ten wood elves (GAM-DEC-004); he dismisses his cavaliers and takes the
//!   elves; only once it is closed does the box's experience come, with its level-up (ADV-RWD-001,
//!   GAM-DEC-007, D-39).
//! - `rewards-event-stop-s1`: Mephala's move east stops on red's event (ADV-MOV-011, ADV-OBJ-007),
//!   which fires (500 gold) and is removed; her next move crosses its tile.
//! - `rewards-university-s1`: Mephala reaches the University in two moves (the second past her
//!   first reveal), buys two of its four skills and leaves; the next day she enters again and only
//!   the other two are offered; she buys one (ADV-TCH-007, GAM-DEC-006).
//! - `rewards-guarded-box-s1`: Adela opens the box on (31, 4); its ten pikemen fight at once, she
//!   wins, and her level-up (advanced Wisdom first) comes before the box's spells: Bless and
//!   Prayer, a level-4 spell her Wisdom now allows (GAM-DEC-007, HER-SPL-005).
//! - `rewards-event-guard-s1`: Orrin steps onto the event on (6, 8): the gnolls guarding it fight
//!   first, then the event fires for him (five crystals; `ADV-QUIRK-29`).
//! - `rewards-stone-guard-s1`: Orrin enters the Learning Stone on (3, 8): its 1 000 experience and
//!   level-up first; only once he answers do the gnolls guarding it fight (ADV-MON-006,
//!   GAM-DEC-007).
//!
//! **Sea** (E08 T75a, increment 8), on `fixtures/maps/e08-sea-36.h3m` (a strait between two
//! shores, written by `h3_map::writer`; `fixtures/maps/README.md`): red's Sylvia (hero 0, the
//! Admiral's Hat, twenty archangels), Orrin (hero 1), Mephala (hero 2, Angel Wings) and Tyris
//! (hero 3, Boots of Levitation, five angels) on the west; blue's Voy (hero 4, the Necklace, a full
//! backpack) and Fiona (hero 5, the Sea Captain's Hat) on the east and Astral (hero 6, one pikeman)
//! on red's shore; each beside an empty boat but Mephala and Tyris. Random play, recorded with
//! `--map fixtures/maps/e08-sea-36.h3m --seed N --agent-seed N --days 28`: the three of a search
//! over the seeds 1 to 3 000 (a scratch probe noting each game's boardings, landings, pickups,
//! events on water, the battles by their battlefield and a boat leaving; not committed) whose
//! union meets the most:
//!
//! - `sea-s103-d28`: two boardings, a Flotsam and a bottle taken, both events on water (the
//!   guarded one's battle on the deck), a hero battle on land.
//! - `sea-s1503-d28`: a boarding, the Shipwreck Survivor taken, a hero battle on the shore, a
//!   monster's battle on land.
//! - `sea-s2611-d28`: five boardings and three landings, Favorable Winds sailed, a Sea Chest taken,
//!   a battle on the two boats whose beaten hero leaves with its boat, a monster's battle on land.
//!
//! **Scripted** (E08 T75a: increment 8's design note §15's paths), on the same map, recorded with
//! `h3 adventure run --map fixtures/maps/e08-sea-36.h3m --seed 1 --checkpoint-every 5 --script
//! <script> --record <replay>` (the moves from a scratch probe walking each hero to its tile
//! through the explored reachable ones, the battles' commands from the original's battle AI for
//! both sides, E23's `h3_battle_ai::choose`; not committed); each ends with both players' turns:
//!
//! - `sea-board-hat-s1`: Sylvia boards her boat with the Admiral's Hat: her points after the step
//!   become the share of the sea's maximum (1 700 of 2 000 to 3 251 of 3 825), she sails east and
//!   lands on (24, 9), the share back to the land's (ADV-SEA-002, -003; D-41).
//! - `sea-board-s1`: Orrin boards his boat without the Hat: his points become 0; the next day he
//!   sails the strait and lands on (24, 10), his points 0 again (ADV-SEA-002 to -004).
//! - `sea-deck-shore-s1`: Astral boards his boat on red's shore; the next day Tyris attacks him
//!   from the coast: the battle on the deck (GAM-BAT-001, ADV-SEA-006); Astral beaten, his boat
//!   leaves the map with him (`BoatRemoved` after `HeroRemoved`; GAM-BAT-003).
//! - `sea-deck-monster-s1`: Sylvia sails onto the water the pikemen on (16, 4) guard: they attack
//!   her at once, the battle on the deck at their tile (ADV-SEA-005, ADV-MON-005).
//! - `sea-two-boats-s1`: Sylvia sails toward Voy; he boards his boat; the next day she attacks
//!   him from the next water tile: the battle on the two boats; Voy leaves with his boat, she
//!   stays in hers (GAM-BAT-001, -003; ADV-SEA-006).
//! - `sea-landing-guards-s1`: Sylvia sails over Favorable Winds and lands on (24, 16), which the
//!   gnolls on (25, 16) guard: after the landing they fight her on land, at their tile (ADV-SEA-003;
//!   E8-15).
//! - `sea-event-s1`: Sylvia sails onto the event on (14, 7), which fires and ends the move, then
//!   onto the one on (17, 7), whose pikemen fight her on the deck before it fires (ADV-OBJ-007,
//!   ADV-MOV-011).
//! - `sea-flyer-s1`: Mephala flies over the plain water beside the Flotsam on (12, 16) and lands
//!   beside it; no flight ends on or passes its tile (D-46, ADV-PIK-018): a refusal a replay
//!   cannot hold, so `h3-map`'s `sea_map.rs` checks it on the same map.
//!
//! **Economy** (E08 T86a, increment 9), on `fixtures/maps/e08-economy-36.h3m` (written by
//! `h3_map::writer`; `fixtures/maps/README.md`): red's Lord Haart (hero 0, Estates, the Endless
//! Sack of Gold, two Crystal Dragons, ten archangels and twenty pikemen), Rissa (hero 1,
//! Mysticism, a mercury specialist, the Charm of Mana) and Orrin (hero 2) below the Shipyard;
//! blue's Sephinroth (hero 3, a crystal specialist, the Mystic Orb of Mana) and Caitlin (hero 4, a
//! gold specialist, the Endless Purse of Gold); the seven mines (blue's Crystal Cavern owned at
//! load, two guarded by a monster beside their entrance), an abandoned mine, two Lighthouses and a
//! Shipyard on the sea's coast, two of each weekly object, growing and fixed monsters, three timed
//! events. Random play, recorded with `--map fixtures/maps/e08-economy-36.h3m --seed N
//! --agent-seed N --days 60` (two month starts): the three of a search over the seeds 1 to 3 000
//! (a scratch probe noting each game's months, flags, battles by what they were fought for, Leave
//! Guards, boats, weekly visits and the taxes; not committed: 1 916 games have a creature month,
//! 1 252 fight a monster it added, 313 the abandoned mine and 65 win it, 145 build a boat by
//! `BuildBoat`, 59 visit a weekly object emptied that week; none leaves guards in a mine or
//! fights an owned mine's guards, which the scripted ones hold) whose union meets all 28 features
//! counted:
//!
//! - `economy-s792-d60`: a creature month whose added monsters are fought, the Shipyard flagged and
//!   a boat bought at its visit, a later visit refused its offer (`ShipyardBlocked`), a hero
//!   battle, three weekly objects marked, a level-up, a tax floored at 0.
//! - `economy-s935-d60`: a creature month, the abandoned mine fought and won, Leave Guards opened
//!   (closed with nothing left), a weekly object visited again in its week, an added monster
//!   fought.
//! - `economy-s1451-d60`: a Plague month, three mines flagged and one taken from its owner, a
//!   Lighthouse flagged, the Shipyard flagged and boats built by `BuildBoat`, a boarding and a
//!   landing.
//!
//! In each the timed events fire and a mine's guarding monster is fought; the weekly growth moves
//! the monsters.
//!
//! **Scripted** (E08 T86a: increment 9's design note §15's paths), on the same map, recorded with
//! `h3 adventure run --map fixtures/maps/e08-economy-36.h3m --seed 1 --checkpoint-every 5 --script
//! <script> --record <replay>` (the moves and the battles' commands from the sea's scratch probe;
//! not committed); each ends with both players' turns:
//!
//! - `economy-guards-s1`: Haart fights the six gnolls on his way and the next day flags the
//!   neutral sawmill (ADV-OWN-001), steps off and onto it again: Leave Guards opens (ADV-MIN-002,
//!   GAM-DEC-004); he moves his twenty pikemen into its group and closes it, the pikemen the mine's
//!   guards (`GuardsChanged`), and walks away. Blue's Sephinroth visits the mine: its guards are
//!   asked, then fought, red playing them as their owner's side without a hero (GAM-BAT-001);
//!   Sephinroth wins and flags the mine for blue, whose next day's income has its wood; day 3's tax.
//! - `economy-abandoned-s1`: Haart reaches the abandoned mine on day 2 (crystal drawn, 197
//!   troglodytes); asked, he fights them, wins and answers his level-up, and only then is the mine
//!   flagged for red (ADV-MIN-002); day 3's income has its crystal.
//! - `economy-shipyard-s1`: Orrin visits the Shipyard: it is flagged for red and offers a boat
//!   (ADV-SHY-002); accepted, 1 000 gold and 10 wood paid, the boat on its tile (30, 20); Orrin
//!   boards it, the next day sails off, and red builds a second boat by `BuildBoat` without a visit
//!   (ADV-SHY-003); day 3's tax finds no wood left, floored at 0 (GAM-CAL-004).
//! - `economy-windmill-s1`: Rissa visits the Windmill on (9, 6) (three crystals, marked), steps off
//!   and onto it again that day and the next (`ObjectEmpty`), and again on day 8 after the week's
//!   reset (three gems; ADV-WEK-001, ADV-VIS-006); the week's start also pays the Crystal Dragons'
//!   three crystals (GAM-RES-005).
//!
//! **Dwellings** (E08 T96a, increment 10), on `fixtures/maps/e08-dwellings-36.h3m` (written by
//! `h3_map::writer`; `fixtures/maps/README.md`): red's Orrin (hero 0, ten archangels), Valeska
//! (hero 1, five halberdiers) and Thorgrim (hero 2, seven stacks: D-52's full army), blue's Adela
//! (hero 3) and Rion (hero 4); a neutral Guardhouse and Barracks, the Dragon Cliffs (three Green
//! Dragons guard it), red's Golem Factory and blue's Elemental Conflux (class 20, guarded, owned
//! at load); the random dwellings 216 (neutral, the Angel Portal among its outcomes), 217 (red's)
//! and 218 (blue's); two Refugee Camps. Random play, recorded with `--map
//! fixtures/maps/e08-dwellings-36.h3m --seed N --agent-seed N --days 22` (three week starts): the
//! three of a search over the seeds 1 to 3 000 (a scratch probe noting each game's guards'
//! questions and battles, flags, hire questions, free creatures, hire dialogs and purchases by
//! source, camp visits by week, the random dwellings' draws and visits, hero battles and
//! level-ups; not committed: 396 games ask a dwelling's guards, 202 fight them, 52 win and flag
//! the dwelling after, 153 lose with survivors kept; 709 flag an unguarded dwelling; 154 have free
//! creatures join, 57 refused for room; 185 buy at a one-creature dwelling, 180 at a class 20, 112
//! at a camp; 147 draw the Angel Portal and 16 visit it; 119 have a hero battle, one a level-up
//! (1 087); 8 visit a camp again in its week, 15 visit a dwelling refilled after a purchase; none
//! finds a camp deserted) whose union meets 19 of the 22 features counted (free creatures joining,
//! a camp visited again in its week and a refill visited are not in it: the scripted ones hold
//! them):
//!
//! - `dwellings-s765-d22`: Valeska accepts the hire question at red's Golem Factory and buys the
//!   most of its Gold Golems, then of its Iron Golems, the dialog of four creatures open after each
//!   purchase (GAM-DEC-008), then closes it; Thorgrim accepts the Dragon Cliffs' guards' question
//!   and loses to the three Green Dragons, the survivors its guards; the next week Orrin fights
//!   them and wins: the Cliffs flagged for red after the win, their hire question declined.
//! - `dwellings-s2192-d22`: the 216 drawn as the Angel Portal (three Angels guard it), at whose
//!   moved entrance (18, 27) Rion is asked its guards' question and declines; Thorgrim flags the
//!   Guardhouse and accepts its question: its free pikemen find no room in his full army
//!   (`JoinRefused`, D-52); he accepts the Refugee Camp's on (9, 12) and closes its dialog; Orrin
//!   flags the Barracks and buys the most of its swordsmen, the one-creature dialog closing on the
//!   purchase.
//! - `dwellings-s1087-d22`: Valeska flags the Barracks and closes its dialog; Adela buys the most
//!   of the camp's on (14, 31) two Efreet Sultans, the dialog closing on the purchase; Thorgrim
//!   attacks Adela, wins and answers his level-up with the first offer (a hero battle, the only
//!   level-up of the search); the 217 drawn as a Golem Factory.
//!
//! **Scripted** (E08 T96a: increment 10's design note §14's paths), on the same map, recorded with
//! `h3 adventure run --map fixtures/maps/e08-dwellings-36.h3m --seed 1 --checkpoint-every 5
//! --script <script> --record <replay>` (the moves and the battles' commands from a scratch probe,
//! E23's `h3_battle_ai::choose` for both sides; not committed); each ends with both players'
//! turns. A refusal is a command the game rejects, the state unchanged, so no replay holds one: it
//! shows as the legal set without that `Buy` (the most bounded by the room and the treasury,
//! GAM-DEC-008), the dialog then closed.
//!
//! - `dwellings-guards-s1`: Valeska steps onto the neutral Dragon Cliffs (8, 4): the guards'
//!   question (ADV-DWL-004); accepted, the three Green Dragons fight unowned, played by the system
//!   actor (GAM-BAT-001), and win: Valeska leaves the map (GAM-BAT-003), the three stay its guards
//!   and the Cliffs neutral. Orrin asks the same day and wins with his ten archangels: the Cliffs
//!   flagged for red (ADV-OWN-001; its radius-3 reveal finds nothing new), then the hire question;
//!   accepted, the dialog of one Green Dragon lists no `Buy` (its crystal: red's went to day 1's
//!   event, `CannotPay`) and is closed.
//! - `dwellings-free-s1`: Thorgrim flags the neutral Guardhouse (13, 4) for red (ADV-OWN-001);
//!   accepted, its fourteen pikemen (level 1, the free level) find no room among his seven stacks:
//!   `JoinRefused`, the count kept (ADV-HIR-001, D-52: all of it or none; ADV-ARM-007), and no
//!   dialog (no creature above the free level). He steps off; Valeska visits the red Guardhouse:
//!   no flag, the question, the fourteen join free in her free slot (`CreaturesJoined`, the count
//!   0); Orrin's visit after her finds none (`NoneToRecruit`).
//! - `dwellings-dialog-s1`: Thorgrim at red's Golem Factory (18, 6) (class 20, four creatures, none
//!   free): the dialog lists no `Buy` (no room, `NoRoom`; ADV-HIR-002) and is closed; he steps off.
//!   Orrin's dialog there: the most of the Diamond Golems (two, 1 500 gold; ADV-HIR-002,
//!   GAM-DEC-008), then four of the six Iron Golems, less than the most, the dialog of several open
//!   until he closes it. Valeska flags the neutral Barracks (13, 9) (ADV-OWN-001, ADV-DWL-004): its
//!   dialog of one creature closes after her purchase of two of its four Swordsmen
//!   (`DecisionClosed`).
//! - `dwellings-camp-s1`: Orrin visits the Refugee Camp on (9, 12) (ADV-RFG-001): its question
//!   names its three Gold Golems; he buys two, the dialog of one creature closing; off and on again
//!   the same day he buys the one left; the next day the camp is deserted (`ObjectEmpty`, no
//!   question). Day 8's week step draws its new creature (ADV-RFG-001, ADV-DWL-003: six Ice
//!   Elementals), bought at the most.
//! - `dwellings-random-s1`: blue's Rion reaches the neutral random dwelling 216, built as a
//!   Homestead (ADV-DWL-005), at its drawn entrance (17, 27) left of its anchor (18, 27): flagged
//!   for blue (ADV-OWN-001), the question, its seven Wood Elves bought at the most (ADV-HIR-002).
//!   The next day Orrin visits red's random 217 on (20, 11), built as a second Golem Factory owned
//!   at load: no flag, the dialog, its six Stone Golems bought, the dialog closed.
//!
//! **Towns** (E09 T9a, the town lane's increment 1), on `fixtures/maps/e09-towns-36.h3m` (written
//! by `h3_map::writer`; `fixtures/maps/README.md`): red (Castle) and blue (Stronghold); red's Orrin
//! (hero 0, ten archangels), Valeska (hero 1, five halberdiers), Sorsha (hero 2, eight Rogues), a
//! random hero (hero 3) and the hero its main town generates (hero 6); blue's Yog (hero 4, fifteen
//! Gorgons, moved onto the Tower's entrance: its visitor) and Gurnisson (hero 5); eight towns by
//! `TownId`: red's main Castle with a Capitol, red's random town (random stacks), a neutral Inferno
//! (garrison drawn) and Necropolis (empty), neutral random towns aligned to blue and drawn, blue's
//! Tower (a garrison and Yog) and blue's main Stronghold (a Capitol, empty). Random play, recorded
//! with `--map fixtures/maps/e09-towns-36.h3m --seed N --agent-seed N --days 21`: the three of a
//! search over the seeds 1 to 5 000 (a scratch probe noting each game's captures with and without a
//! battle by each player, sieges against a garrison alone and against a visitor with the merge, won
//! and lost, the Capitol's limit, a hero entering its own town, hero battles, and heroes 3 and 6
//! taking part; not committed: 858 games have red capture a town undefended, 158 blue; 845 win a
//! siege against a garrison alone, 479 lose one; 5 win a siege against a visitor with the merge, 46
//! lose one; 94 limit a Capitol; 1 687 enter an own town; 214 capture the other player's town; 958
//! have a hero battle) whose union meets all twelve features counted:
//!
//! - `towns-s173-d21`: on day 2 Sorsha attacks blue's Tower, Yog defending with the garrison merged
//!   into his army, and loses; on day 12 Valeska enters red's random town; on day 16 Yog besieges
//!   the neutral town aligned to blue, its garrison alone, wins and captures it for blue.
//! - `towns-s1702-d21`: on day 7 hero 6 besieges the Tower, its garrison alone after Yog left, and
//!   loses; on day 12 the random hero takes blue's main Stronghold undefended, its Capitol limited
//!   to a City Hall (`TownChanged`: red keeps Haven's); hero battles on days 16 (Gurnisson against
//!   Orrin) and 20 (Yog against Orrin), each won by the defender.
//! - `towns-s3435-d21`: on day 2 Orrin besieges the Tower held by Yog with the garrison merged and
//!   wins: the Tower is red's; on day 11 Gurnisson retakes it undefended; on day 21 he enters
//!   blue's main Stronghold.
//!
//! **Scripted** (E09 T9a: design note `docs/epics/E09-design.md` §18's), on the same map, recorded
//! with `h3 adventure run --map fixtures/maps/e09-towns-36.h3m --seed 1 --checkpoint-every 5
//! --script <script> --record <replay>` (the moves and the battles' commands from a scratch probe,
//! E23's `h3_battle_ai::choose` for both sides; not committed); each ends with both players'
//! turns. A move stops where the explored area ends, so a day's walk may take several.
//!
//! - `towns-capture-s1`: Valeska walks onto the neutral Necropolis' entrance (12, 18) and captures
//!   it undefended (TWN-CAP-001: `TownCaptured`, the reveal; no battle). Red's next day's gold is
//!   10 000 (`towns-income-s1`'s 9 500 and the Necropolis' Village Hall: a town captured in a day
//!   pays its new owner from the next, TWN-INC-001); she steps off and enters it again, her own.
//! - `towns-siege-s1`: Orrin besieges the neutral Inferno (18, 5), its drawn garrison alone (five
//!   Hell Hounds; TWN-SIE-001): his archangels win before the garrison acts, the town is captured
//!   for red after the battle (TWN-CAP-003), Orrin stays on (17, 6) and enters it the same day.
//! - `towns-visitor-s1`: Sorsha attacks blue's Tower from (28, 11): the garrison merges into Yog's
//!   army before the battle (TWN-SIE-002: `StackChanged` on the garrison's slots and his, the ten
//!   Stone Gargoyles into his slot 1, the thirty Gremlins into his first free slot 2), blue plays
//!   its side and wins; Sorsha leaves the map, the Tower stays blue's.
//! - `towns-capitol-s1`: Sorsha walks south, away from Yog, and captures blue's main Stronghold
//!   (29, 22), its garrison empty, while red holds Haven's Capitol: `TownChanged`, its Capitol
//!   limited to the City Hall (TWN-CAP-002); red's next day's gold 11 500 (the City Hall's 2 000,
//!   not a Capitol's 4 000), blue's 500.
//! - `towns-income-s1`: both players end their turns to day 8: each of the seven day starts pays
//!   red 9 500 gold (Haven's Capitol 4 000 and Grail 5 000, the random town's Village Hall 500),
//!   1 wood and 1 ore (Haven's Resource Silo, the Castle's; TWN-INC-001, -002), blue 4 500 (its
//!   Stronghold's Capitol, its Tower's Village Hall).

use std::path::PathBuf;

#[test]
fn golden_adventure_replays_reproduce_their_hashes() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = root.join("fixtures/replays/adventure");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "toml"))
        .collect();
    files.sort();
    assert!(
        files.len() >= 55,
        "expected the E08 and E09 replay fixtures in {}",
        dir.display()
    );
    for file in &files {
        let verified = h3_cli::adventure::verify_file(file, &root.join("packs"), &root);
        assert!(
            verified.is_ok(),
            "{}: {}",
            file.display(),
            verified.unwrap_err()
        );
    }
}

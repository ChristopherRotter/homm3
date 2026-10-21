# Generated maps

Adventure-lane fixtures (`lanes.toml`): `.h3m` files our own code generates, committed once, so
that tests, golden replays and benchmarks play a real map file without the original game's maps
(those stay local, `docs/formats/h3m.md`; the user dropped hand-made editor maps, E08 2026-10-11).

Each file is laid out by `crates/h3-map/src/tests/fixtures.rs` from a plan built with a fixed
generator, written gzip-compressed as the game stores maps. The typed objects (E08 T16a) are
drawn after the terrain and the blockers, so those stayed as they were; each object's tiles are
free land off the roads (a guard's or a gate's on them), its visitable tile keeps two free
neighbours it is reached from not from above, and none covers a hero or another object. Increment
3's (E08 T20b3b2) are drawn after those, so those stayed as they were too: the property plans'
monsters, guarded piles and special grounds (`play.rs`), no monster within two tiles of a hero. That
file's test reads each committed map and checks that it is still its plan's map and imports. A
change of a committed map changes its content hash and so invalidates every replay that names it
(ADV-Q2): regenerate only on purpose, with `H3_WRITE_FIXTURES=1 cargo test -p h3-map --lib
tests::fixtures`, and say why in the commit message.

| File | Map | Played by |
|---|---|---|
| `e08-duel-36.h3m` | SoD, 36 × 36 with an underground; red and blue with two heroes each, one per level, their types' starting armies; since E08 T31 a custom experience each, 970, 950 and 930 a little below level 2, so that a hero's first battle gives it a level-up, and red's underground hero's 4 570, which gives it levels 2 to 4 at load; terrain in 6 × 6 patches, a road per level, 72 blockers; since E08 T16a 66 typed objects of increment 2 (2.9 % of the land tiles, as the shipped maps: 3.7 % on their 36 × 36 maps, 2.5–3.0 % on larger ones): 26 piles of the seven resources (drawn amounts but one gold pile of a fixed 300), 24 random resources, 4 campfires, a land Cartographer, a green Keymaster's tent and a pile and a random resource within each surface hero's first reveal, a red tent, a campfire, a pile and a random resource within each underground hero's, a green border guard and gate on the surface road, a red guard and gate on the underground road, a Redwood Observatory, a Pillar of Fire; since E08 T20b3b2 14 wandering monsters (beside each hero one, the plans' five kinds in turn: compliant goblins, hostile gogs with 100 gold, weak pikemen, savage wolf riders that never flee, friendly centaurs with 50 gold; two of each kind anywhere), 8 guarded piles (beside each hero one, in turn 500 gold whose eight gremlins and two gnolls ask first and four crystals whose twenty skeletons fight at once; two of each anywhere) and 6 special grounds (Cursed Ground under the first hero's monster, Magic Plains under the third's, two of each anywhere) | `h3-sim`'s adventure env tests (E08 T9a); the golden replays in `fixtures/replays/adventure/`, the random-play benchmark (`benches/adventure/`; E08 T9b, decision 11) |
| `e08-xl-144.h3m` | SoD, 144 × 144 with an underground (41 472 tiles); red and blue with four heroes each, two per level; since E08 T31 the duel's experience for the first four, a little below level 2 for the others; the same patchwork, 1 152 blockers; since E08 T16a 16 times the duel's typed objects (1 056, 3.0 % of the land tiles): the same four beside each hero, four green guards and gates on the surface road and four red ones underground, the rest anywhere on their level (tents, guards and gates green on the surface, red underground; Cartographers and observatories on the surface, Pillars of Fire underground) or the map; since E08 T20b3b2 increment 3's as the duel's, 16 times as many anywhere (168 monsters, 72 guarded piles, 68 grounds) | the path-search and fork benchmarks (`benches/adventure/`; E08 T9b, decision 11) |

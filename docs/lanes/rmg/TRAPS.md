# RMG lane — standing traps

Grouped by area. Read the group for the area your task touches; delete a trap when it stops applying, merge duplicates, add a new one under its area (a trap of every lane goes into `docs/HANDOFF.md` or the topic doc).

## Reading the executable

- The generator's own range is 0x530EE0–0x54D0F0, but its closure calls about 250 game functions outside it (text tables, `template_file_read`, `msk_load`, object tables): many are already in `docs/exe/map.toml`, so `lookup` before reading.
- H3API's names for the generator (`H3RmgRandomMapGenerator`, `H3RmgMap`, `H3RmgObject`) come from the 3.2 build shifted by -0x4F0 in this region: a pointer where to read, never a rule's source (`docs/exe/README.md`, "The H3API port").
- The seed is `time()` in seconds (0x536148); the Random choices are drawn earlier with the same `rand` (0x586168): an oracle case with a Random choice depends on the state of `rand` before the generation.

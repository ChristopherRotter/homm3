# RMG lane — standing traps

Grouped by area. Read the group for the area your task touches; delete a trap when it stops applying, merge duplicates, add a new one under its area (a trap of every lane goes into `docs/HANDOFF.md` or the topic doc).

## Reading the executable

- The generator's own range is 0x530EE0–0x54D0F0, but its closure calls about 250 game functions outside it (text tables, `template_file_read`, `msk_load`, object tables): many are already in `docs/exe/map.toml`, so `lookup` before reading.
- H3API's names for the generator (`H3RmgRandomMapGenerator`, `H3RmgMap`, `H3RmgObject`) come from the 3.2 build shifted by -0x4F0 in this region: a pointer where to read, never a rule's source (`docs/exe/README.md`, "The H3API port").
- The seed is `time()` in seconds (0x536148); the Random choices are drawn earlier with the same `rand` (0x586168): an oracle case with a Random choice depends on the state of `rand` before the generation.
- The call closure misses the methods reached only through vtables: of the range's 255 functions, 154 are in the closure of 0x54BF00 and 0x54C090 by direct calls, 101 (about 16 kB) only in vtables (the objects' and object generators' methods). Find them from the vtables (`refs` on a vtable's address).
- The C runtime's `rand` keeps its state per thread (`crt_rand`): the generation and the writing run on Begin's thread, one stream; the writer draws too (`rmg_teams_draw`, after every draw of the generation).
- `docs/exe/` is not scanned for citations: an address in a map summary needs no entry of its own, one in the survey or the lane's docs does (`cargo xtask check-exe-map`).

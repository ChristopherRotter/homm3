# Server lane workloads

Instruction-count workloads of the server lane (`benches/README.md`).

- `session-random-play-full` (E16 M1 T4): `h3-server-bench` plays 20 hosted battles of the
  synthetic `full` scenario with two scripted clients that answer every turn with a strike if
  there is one; callgrind counts inside `h3_server::bench::serve_frame`, the server's whole work
  for one client frame (the hello gate and the session core: decode, validate, apply, project,
  encode). About 2 100 client frames, 8 300 server frames, 4.6 MB sent; 273 million instructions,
  about 55 % of them in the battle engine (legal commands 22 %, apply 19 %, castable spells
  12 %) and 34 % in encoding.
- **Lane-only.** The count follows the battle lane's rules, synthetic packs and engine speed, so
  the full verify does not measure it (a battle change must not fail CI on a server file). Check
  it at the start of each session (`verify --scope server`): a drift there came from `master`;
  re-baseline it (`cargo xtask bench-check --update --workload session-random-play-full`) and
  name the master range in the commit. A drift after a change of yours is your regression.
- The count depends on the command line too (the arguments' allocations shift the heap that
  `malloc` walks): measure with the file's `args`, not by hand.

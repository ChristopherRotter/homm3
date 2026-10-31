# E11 refinement and the SoD 3.2 retail findings (copied from the Project doc `claude/e11-refinement-status.md`, 2026-10-24 / 2026-10-26)

Copied into the repo on 2026-10-31 so the Project doc could be cut to a pointer. Authoritative E11 state: `docs/epics/E11-historical-rules.md`.

Repo `D:\Users\Christopher\Source\Repos\homm3` (GitHub `ChristopherRotter/homm3`). Three PRs were waiting for the user's merge on 2026-10-24, in this order: #448 `shared/e23-closure`, #449 `battle/e23-closure`, #447 `shared/e11-refinement` (`cross-lane`; E11's epic file, `lanes.toml` (`docs/epics/E11*` → battle), BACKLOG, OPEN-QUESTIONS Q58, inbox notes for the battle and adventure lanes).

## The order (the user, 2026-10-24)

The battle lane closes E23 first. Its next session works off the inbox's leftovers: the AI lane's turret-shot preview, BAT-Q62's row, BAT-Q28/32/35/58. Only then does E11 open with T1, using the user's RoE and AB media.

## Decisions (the user, 2026-10-24, Q58)

1. "AB and SoD completeness" is **folded** like E07. Every epic covers RoE/AB/SoD for what it builds. Leftovers: E08 increment 6 (assembling combinations), E09 (Conflux as a town, the moat by expansion), E12 (AB/SoD objects), E20 (campaigns). A pack-set test matrix is in BACKLOG's inbox.
2. E11 is re-scoped to the **historical rules of RoE and AB**: `base` alone plays RoE's executable, `base`+`ab` AB's, all three SoD's.
3. The **latest patch** of each edition only.
4. The edition **follows the enabled packs** (decision 104 extended), with no separate setting. Quirk profiles stay orthogonal.
5. Sources: the **user's own original media**, with official patches only. They are installed apart from the GOG installation and copied into the gitignored `original-data/roe/`, `original-data/ab/`.
6. **The battle lane leads**; each difference is built by the lane that owns its rule.
7. The original's random map generator (an AB feature) is for **E14's refinement** to decide.

## E11 tasks

T1 sourcing and catalogue (with the user; stop), T2 the function-level diff (`scripts/exe.py` learns the other builds; it refuses all but SoD's hash today), T3 the survey `E11-editions.md` (stop: what to build, slip classes), T4 the ADR draft (the edition as pack data; amends ADR-0006 and ADR-0017 point 1; stop), T5 re-harvest `base`/`ab` from their own data, T6+ the differences built.

## SoD 3.2 retail, a pre-T1 finding (2026-10-26)

The user installed SoD 3.0 and applied the official `SD30to32` update on a **Windows 7 machine** (the installer does not start on Windows 10), then copied the folder to `D:\Heroes3\HoMM 3 SoD`. The folder also holds an old GOG install's files (2018 `unins000.ini` = `GOGHOMM3COMPLETE`, `gogwrap.exe`, the Complete README); the executable and `Data\` are SoD's (version resource, data diff below).

- **`HEROES3.ICD`** is the real program: version resource 3.2, linked 2000-09-08 00:50:34 UTC, 2 732 077 bytes, SHA-256 `5693263f72cdb7a559cf1901796549fc12870851edef94eb24e6723e612fb0a1`. `Heroes3.exe` is SafeDisc's loader (344 951 bytes, SHA-256 `e94fd33f041d9bd3cb0f1e8ad3b7150568f3d94b351333d0dba45be70ce8927e`), beside `secdrv.sys`, `clokspl.exe`, `dplayerx.dll`, `drvmgt.dll`.
- **Complete 4.0 is a sibling build of SoD 3.2.** GOG's `Heroes3.exe` (`aac39e8c…`) was linked 91 s later (00:52:05 UTC), with the same section layout; 3.2's `.text` is 0x5E0 bytes larger.
- **Data:** all four shared LODs have the same entries. Every rule text table is byte-identical to Complete's. Only `CREDITS.TXT`, the main-menu and campaign-select art (9 PCX, 15 DEF) and `H3ab_spr.lod`'s `ARTIFACT.DEF` differ. So "SoD at its latest patch = Complete 4.0" (the epic's goal line) holds on the data side.
- **SafeDisc blocks static reading.** The ICD's `.text` and `.data` are encrypted on disk (entropy 8.0; `.rdata`/`.rsrc` are plain), so `exe.py` can read neither the code nor its static tables, whatever machine it was installed on. **T1 risk the epic does not name:** if the RoE/AB media are protected the same way (check for a `.ICD` next to `Heroes3.exe`), T2's static diff is blocked under decision 5 ("never a cracked or modified executable"). The user decides at T1. Routes that keep decision 5: an unprotected official build, or observation only (the original run with its disc on an OS it supports).
- **Windows 7 is the candidate observation platform.** Microsoft's MS15-097 (KB3086255, 2015) turned the `secdrv` service off on Vista through 8.1 and documents turning it back on temporarily (elevated: `sc config secdrv start= demand`, `sc start secdrv`; off again with `sc config secdrv start= disabled`). Windows 10/11 do not support it. **Open (asked 2026-10-26):** does SoD 3.2 itself start on the Win 7 machine with its disc? If yes, T1's "does it run on the user's machine" and Q28's RoE/AB batches have a platform.
- The 3.2 readme (`H3SDUpdate.txt`) names an AB **v2.1** update: a lead for T1's version list. Its 3.2 rule changes (Conflux Grail needs Wisdom, no Titan's Lightning Bolt; the double-cast fix) are already in Complete.
- Not useful for E11's core: SoD gives neither RoE's nor AB's executable or tables.

## E23's closure record

Criterion 4: the AI lane's `batch-system-neutral-v1` (#442). Criterion 6: #441's full local Linux verify with the original data (16/16) and the user's Windows run (2026-10-23). Open as recorded: BAT-STUB-8, -10, -11, -15, -16.

## Traps

- The device link dropped mid-session, so the PRs were made from a sandbox clone. The user's local `master` was 8 commits behind `origin/master`.
- GitHub from the sandbox: `gh pr create` fails (no GraphQL); use `gh api repos/.../pulls` and `.../issues/<n>/labels`. A `--depth 1` clone tracks only `master`: add a fetch refspec per pushed branch (or run `scripts/sandbox-setup.sh`'s step 0) so the stop hook sees the push.

# E27 — design note (the game shell)

Working detail of `docs/epics/E27-game-shell.md`: the readings, the clips, layouts. Grep a section
number or a name; never read it whole once it grows.

## 1. The video archives (M0, 2026-11-12)

Listed on the user's computer from the installation's `Data/VIDEO.VID` (173 entries, 629 MB)
and `Data/H3ab_ahd.vid` (7 entries, 1.6 MB), read only: a `u32` count, then per entry a 40-byte
name and a `u32` offset; an entry's size is up to the next offset. Many clips ship twice, as
Bink (`.bik`) and Smacker (`.smk`); the game opens the Bink version first (`video_open`
0x44D5A0), so a clip's Smacker copy plays only where no Bink one exists. Sizes rounded, in MB.

| Use (as known at M0; T1a and T1b read where each plays) | Bink | Smacker only |
|---|---|---|
| The start | `NWCLOGO` 4.2, `H3INTRO` 54.5, `H3X1intr` 31.8 (where it plays: T1a) | `3DOLOGO` 0.9 |
| Loading | | `ProgressBar` 0.3 |
| High scores | | `HSANIM` 4.5, `HSLOOP` 1.8 |
| Credits | (`CREDITS.BIK` and `.SMK` in `VIDEO.VID` are empty) | `Credits.smk` 0.03 (`H3ab_ahd.vid`) |
| A scenario's end (T1b) | `LOSEGAME` 4.0 | |
| The battle's result window (E17: `battle-videos.vid`) | `WIN3`, `LBSTART`, `LBLOOP`, `RTSTART`, `RTLOOP`, `surrender`, `defendall`, `defendloop`, `LOSECSTL`, `LOSECSLP` | |
| The spell book's page turns (E17: "Animate SpellBook" off in the harness) | | `PGTRNLFT`, `PGTRNRGH` |
| Campaigns (E20, decision 7): RoE's selection | `CGOOD1`–`3`, `CEVIL1`–`2`, `CNEUTRAL`, `CSECRET` | |
| Campaigns (E20): SoD's selection, AB's (`H3ab_ahd.vid`) | `Birth`, `elixir`, `hack`, `new`, `rise`, `spectre`, `unholy`; `C1ab7`, `C1db2`, `C1ds1`, `C1fl3`, `C1fw1`, `C1pf2` | |
| Campaigns (E20): the scenarios' clips | RoE's `GOOD*`, `EVIL*`, `NEUTRAL*`, `SECRET*`, `ENDGAME` (Smacker copies too); SoD's `H3x2_UAm` | AB's `H3AB*` (34), SoD's `H3x2_*` (44) |
| Elsewhere (a town's, unknown) | `TAVERN`, `TESTING` | |

So the shell (M2 to M4) needs Smacker for the start's first clip, the loading bar, the high
scores and the credits (decision 5); `h3-bink` decodes the rest. The shell's clips together are
about 102 MB (70 MB without `H3X1intr`), one archive of their own (M1 T2).

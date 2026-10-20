# E08 progression survey (E08 T24a): experience, levels, level-ups

Date: 2026-10-20. Scope: E08 increment 4 (decision 48: hero progression only), input for T24b's
ADR drafts and the cut. Written in parts (decisions 51 to 53): **T24a1** §1 experience to levels
and §2 the level-up, from the code and VCMI; **T24a2a** §3 levels at load and §4 what follows the
level, the same way; **T24a2b1** §5 the wiki on §1–§4 and §6 the census of the shipped maps;
**T24a2b2** the rest (§7 the batch's plan, the proposed rule ids, a fresh agent's second reading,
the questions for the user's checkpoint). Input, not rules (`README.md`).

**Sources** (retrieved 2026-10-20):
- `original-data`: a code reading of the user's `Heroes3.exe` (SoD 4.0 Complete); every address
  cited has an entry in `docs/exe/map.toml` or `structs.toml`. The reading follows the gain of
  experience (`hero_gain_experience` 0x4E33B0) into the level-up (`hero_level_up` 0x4DA720), its
  offers (`hero_secondary_offer` 0x4DAD00) and the learning (`hero_learn_secondary_skill`
  0x4E22D0), the human owner's path to its answer and the computer's to the function that
  chooses (`ai_level_up_choice` 0x52BBD0). Not followed: the level-up dialog's drawing
  (0x4F8880), network play's timer, campaigns (one branch noted, §2.3), the level-up's 26 other
  callers (objects, battles; §3 and increment 5), the routines that copy a hero's record.
- `vcmi`: VCMI 1.7.5 (commit `6901839`): `lib/entities/hero/CHeroHandler.cpp`
  (`loadExperience`, `level`, `reqExp`), `lib/entities/hero/CHeroClass.cpp`,
  `lib/entities/hero/CHeroClassHandler.cpp`, `lib/mapObjects/CGHeroInstance.cpp`
  (`canLearnSkill`, `maxlevelsToWisdom`, `maxlevelsToMagicSchool`, `levelUpAutomatically`),
  `lib/callback/GameRandomizer.{h,cpp}` (`rollPrimarySkillForLevelup`,
  `rollSecondarySkillForLevelup`, `rollSecondarySkills`), `server/CGameHandler.cpp`
  (`levelUpHero`, `giveExperience`), `lib/mapping/MapFormatH3M.cpp` (the level limit); links
  `https://github.com/vcmi/vcmi/blob/1.7.5/<file>`.
  T24a2a follows the new game's setup of the map's placed heroes (`placed_heroes_setup`
  0x4CAE10, `placed_hero_setup` 0x4D8B30) and the battle's end into the levels (`battle_end`
  0x477470, `adventure_battle` 0x4AD470); H3API's names for the game's `heroSetup` and
  `campaignInfo` decide which setup is which (§3.1).
- `wiki`: read in T24a2b1 with the user's approval (2026-10-20): the pages §5 names,
  `https://heroes.thelazy.net/index.php/<Page>`.
- `original-data` (the census, T24a2b1): the installation's 160 shipped maps read with `h3-map`'s
  reader (§6).
- `computed`: the tables, the worked level-up and §6's counts come from throwaway scripts that
  follow the reading (not committed); test vectors to confirm, not facts.

## 0. Findings in short

- **Experience to levels** (§1): levels 1 to 12 from the table (0 to 20 600), beyond it each step
  is the previous one × 1.2, truncated (24 320, 28 784, 34 140, …); VCMI's table is the same
  through level 75. The original sums in 32 bits and its thresholds wrap from level 76 on:
  **level 74 is the last right one**; from 1 810 034 207 experience the hero is level 88 (75 to
  87 are skipped), more gives other wrapped levels, 2³¹ − 1 hangs the game and a gain past it
  makes the experience negative and the level 0 (A-7); VCMI goes to level 197.
- **The scenario's level limit** (game +0x1F877, 0 none) caps the experience at the limit's
  threshold and *lowers* experience above it at the next gain (VCMI keeps it, D-19); a limit
  above 75 caps at a wrapped threshold, negative for 92 of the 180 values (A-8). What copies the
  map's limit into the game was not found (§1.3).
- **The level-up** (§2) runs one level at a time. Each level reseeds the C runtime's generator
  with 214 013 L + 156 823 S + 154 079 (L the new level, S the hero's level seed +0x8F, 1–255
  from the new game, never advanced): a level's primary skill and offers follow from the seed
  byte, the level, the class, the hero's skills and two counters (D-14).
- **The primary skill**: the class's chances (`primary_gain`) for levels 2–9, from level 10 on
  the second row, chosen by the new level (VCMI by the old one, D-15); +1 with no bound (127
  wraps, A-9).
- **Two offers**: an advance of a known skill, then a new skill (each with a fallback to any
  skill below Expert); Wisdom first when it is due, then a magic school when one is due, else a
  draw by the class's weights. Due: 3 levels for the magic classes (Wisdom and schools), 6 and 4
  for the might classes, **the Elementalist counted as might** (VCMI: magic; D-16, A-10). Small
  weight differences against VCMI (D-17, D-18).
- **The answer**: a human owner at its own computer answers each level's dialog before the next
  level is drawn (left: the first offer, right: the second; one offer is learned without a
  choice). A computer player's hero, and **any hero while the new game's setup runs**, never
  asks: 0x52BBD0 chooses (the lead for §3's levels at load).

- **Levels at load** (§3, T24a2a): a placed hero with a custom experience is set up from its
  record in the game's `heroSetup` (0x4D8B30): its custom skills, army, spells and artifacts
  first, then the experience is gained from 0 with the levels given, during the new game's
  setup, so **the computer's evaluation chooses every offer** (VCMI takes the first, D-20) and
  the scenario's level limit caps it (VCMI ignores the limit at load, D-21). Batch E3c's E3-6 is
  consistent with it: one seed byte of 255 (75) gives its primary gains and offers that allow
  its advances. **The function earlier E08 work read as the placed hero's setup,
  `map_hero_setup` 0x486590, sets up a campaign's carried-over hero** (H3API's
  `keepExperience` to `keepArtifacts`): the rules that cite it hold on the real path, their
  citations are stale (QUEUE inbox).
- **What follows the level** (§4): the battle's experience reaches the levels at once for a
  winner not played at this computer, and after the battle's screen for one played here
  (0x4AE324, 0x4AE42C); the engine's rules that read the level (`h3-heroes`' specialties, the
  spell points) need no change but the level's source; VCMI shows another player's hero
  without its level.

- **The wiki** (§5, T24a2b1) agrees with the reading on the thresholds, the levels past 74 (88,
  100, 108, 868, 3 732, 5 920, 6 424, then a hang: the reading's arithmetic gives the same
  sequence), the hero's fixed seed (D-14), the primary rows from the level-up to level 10
  (D-15), the Wisdom and school intervals and the Elementalist counted as might (D-16, A-10), a
  class-banned known skill weighing 1 (D-18); it is silent on the level limit and a known
  school's weight (D-17); it differs on the primary bound (A-9: a reset at 129) and on **D-20**:
  a hero levelled at load always takes the left offer, as VCMI's first. E3-6 fits the reading's
  choice, not always-left (§5.2): a conflict for the second reading and the batch (T24a2b2).
- **The census** (§6): 103 placed heroes and prisoners have a custom experience (42 heroes,
  61 prisoners; levels 3 to 15, two prisoners 42); no shipped map has a level limit, a custom
  primary skill above 9, a hero that knows a skill its map bans, an Elementalist with
  experience, or experience anywhere near A-7. Without the import's refusal of an experience 5
  maps import whole (3 more: `Adventures of Jared Haret`, `Faeries`, `Knee Deep in the Dead`),
  with 9 heroes and 41 level-ups at load.

## 1. Experience to levels

### 1.1 The table and the steps beyond level 12

`experience_by_level` (0x679C88, i16 × 12): 0, 1 000, 2 000, 3 200, 4 600, 6 200, 8 000, 10 000,
12 200, 14 700, 17 500, 20 600, the thresholds of levels 1 to 12. A hero's level is the number of
thresholds at or below its experience (0x4DA743–0x4DA7C6 in the level-up, the same loop at
0x4E354D in the gain). Beyond level 12 the step from 12 to 13 is trunc(1.2 × (20 600 − 17 500)) =
3 720 and each further step trunc(1.2 × the previous step): the product of an f64 1.2 and the
step converted to f64, truncated by `crt_ftol`; thresholds and steps are 32-bit integers.

| Level | 13 | 14 | 15 | 16 | 20 | 30 | 40 | 50 | 60 | 70 | 74 | 75 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Threshold (computed) | 24 320 | 28 784 | 34 140 | 40 567 | 81 961 | 496 963 | 3 066 455 | 18 975 933 | 117 483 099 | 727 413 425 | 1 508 362 195 | 1 810 034 207 |

A step's product is exact whenever 1.2 × step is an integer (the f64 1.2 errs by less than half
an ulp of the product), so the steps equal VCMI's integer form step + step / 5 (§1.4) at every
level up to the overflow.

### 1.2 The highest level and the hang

Level 76's threshold, 2 172 040 621, exceeds 2³¹ − 1 and wraps to −2 122 926 675; from level 86
on the steps wrap as well (`crt_ftol` stores a 64-bit integer and returns its low word,
0x6179ED: level 86's step is −2 053 519 008). Both level loops (0x4DA79F and 0x4E35A3) stop at the
first level whose running, wrapped threshold exceeds the experience. So (computed from the
reading):
- up to 1 810 034 206 experience the levels are right, **74 the highest** (from 1 508 362 195);
- from 1 810 034 207 (level 75's threshold) the first threshold above is level 89's wrapped
  2 073 739 175: **the hero is level 88**, and the level-up gives levels 75 to 88 at once; levels
  75 to 87 are never held;
- from 2 073 739 175 level 100, at 2 100 000 000 level 108, and so on by the wrapped sums; at
  2³¹ − 1 no threshold can exceed the experience and **the loop never ends** (the game hangs);
- the experience is a 32-bit signed word (+0x51) and the gain adds without a bound (0x4E353D):
  past 2³¹ − 1 it turns negative, whose level is 0, and the level-up then lowers the hero to
  level 0 (§2.1).

Whether a scenario can bring a hero there (custom experience, events, the Tree of Knowledge) is
§6's census and the wiki's (T24a2). Quirk candidate **A-7** (§2.7).

### 1.3 Gaining experience (`hero_gain_experience` 0x4E33B0)

Arguments: the points, whether to give the levels, whether to tell the owner. The scenario's
level limit is the byte at game +0x1F877 (0 none).

- **No limit** (0x4E353D): the experience += the points (32-bit), the new level found as in §1.1;
  if asked, the level-up (0x4DA720) runs; returns the levels gained (the new level found minus
  the level before).
- **A limit L**: the cap is L's threshold, computed as in §1.1 (0x4E33F2–0x4E3451). While the
  experience plus the points stays at or below the cap, as without a limit. Above it: if the
  experience is below the cap it becomes the cap and, if asked, the level-up runs; then
  experience above the cap is **lowered to the cap** (0x4E347E: a hero placed above the limit
  loses the excess at its next gain); a human owner at its own computer (0x4CE970), if asked,
  gets the general text's message that the hero can gain no more (the row VCMI shows in the same
  place); returns 0 even when levels were given.
- A limit above 75 computes a wrapped cap (§1.2): negative for 92 of the limits 76 to 255 (76
  to 79, 82, 83, 85, 87, 90, 91, …), else a threshold unrelated to the limit (80: 208 973 836,
  level 63's experience; 89: 2 073 739 175, level 100's). With a negative cap any gain sets the
  experience to that negative number and gives no level (the level-up is called only from below
  the cap): quirk candidate **A-8**.

Who sets +0x1F877 was not found: it lies inside H3API's `mapInfo` block (game +0x1F86C, +0xB; the
port's field names at +0xB to +0xD do not fit the team table read at +0x1F879) and no instruction
writes the offset by name, so the map's header byte (`docs/formats/h3m.md`: the level limit after
the difficulty, AB and SoD) probably reaches it by a block copy. The only instruction that names the
offset is `hero_gain_experience`'s read (0x4E33DD). T24a2's census reads the shipped maps' limits; VCMI reads the same header
byte (below).

### 1.4 VCMI

- `CHeroHandler::loadExperience`: levels 1 to 15 listed (the reading's through 34 140), then
  steps `prevDiff + prevDiff / 5` in 64 bits until the next would overflow: 197 levels. `level`
  is the count of thresholds at or below the experience (`upper_bound`), as the reading's.
- `CGameHandler::giveExperience`: the cap is the threshold of the map's level limit (else of the
  highest level); experience above it is **kept** ("don't decrease if hero already over top"),
  a message when the gain is cut. `MapFormatH3M` reads the header's limit, clamped to at most 100.
- The levels are then given one at a time (`levelUpHero`, recursively, while `gainsLevel`).

## 2. The level-up (`hero_level_up` 0x4DA720)

### 2.1 One level at a time

The function finds the level of the hero's experience (§1.1). If the hero's level (+0x55) is
higher, it is lowered to it and nothing else changes (0x4DACA8); if equal, nothing happens.
Otherwise, for each level up to it, in order (the loop back at 0x4DACA2): the level +1, the
message's first line (the hero gained a level), the seed (§2.2), the primary skill (§2.3), the two
offers (§2.4), the counters, the answer (§2.5). A human answers each level's dialog before the
next level's draws: a later level's offers depend on the earlier answers (the hero's skills enter
the draw).

### 2.2 The seed

0x4DA801–0x4DA830: `set_random_seed(214 013 L + 156 823 S + 154 079)` (32-bit), L the hero's new
level, S its level seed (byte +0x8F). Every draw of the level is the C runtime's `rand` through
`random_between(a, b)` = a + rand() mod (b − a + 1). S is drawn once per hero at the new game
(`new_game_heroes`, 0x4BFEC2: `random_between(1, 255)`), and no code of the level-up writes it;
six routines copy +0x8F together with the two counters of §2.4 (+0x3F, +0x90), by their look
copies of a hero's record (not read). So a level's primary skill and offers are a function of S,
L, the hero's class, its secondary skills and the counters: reloading a game, or reaching the
level by another gain, gives the same draws. The new game also sets both counters to 0 (0x4BFECD,
0x4BFED4).

### 2.3 The primary skill

The level's first draw: r = `random_between(1, 100)` (0x4DA83F). The chances are the hero
class's (the class record at the pointer 0x67DCEC + 0x40 × class, the class at hero +0x30):
+0x10 for a new level up to 9, +0x14 from 10 (0x4DA84D: the level already raised), the packs'
`primary_gain` rows. r walks the four chances in order (Attack, Defense, Spell Power, Knowledge)
and the skill it stops on gets +1 (0x4DA8F7). There is no upper bound: the byte +0x476 + skill is
signed and holds the worn artifacts' bonuses too (`structs.toml`), so 127 + 1 becomes −128; the
readers clamp to 0–99 (0x4E5400 for Attack and Defense). Quirk candidate **A-9**.

A campaign branch (the global 0x69774C set, the game's +0x1F45C = 14, hero type 45, Solmyr's
slot: VCMI's Yog) draws again, `random_between(1, a + d)` with the Barbarian's (class 12) Attack
and Defense chances, so only Attack or Defense; its offers use a fixed list of banned skills
(0x679CA0: the eleven magic skills). VCMI gives Yog Attack or Defense from his own class's two
chances. Campaigns are outside E08.

Worked (computed): a Knight (`primary_gain` 35/45/10/10) with S = 1 reaching level 2: seed
738 928, r = 3: Attack. With S = 200: seed 31 946 705, r = 91: Knowledge.

### 2.4 The offers (`hero_secondary_offer` 0x4DAD00)

Arguments: the hero, the lowest skill level a candidate may have, the level it must stay below,
and a skill to exclude. A skill is a candidate when its level (+0xC9 + skill) is in that window
and it is not the excluded one. With eight skills (+0x101) the lowest level becomes 1: only known
skills; if then no window is left, no offer (−1). The map's banned skills are the game's
+0x4E658 (H3API's `bannedSkills`, a byte per skill). In order:

1. **Wisdom** (skill 7) is returned outright when the hero's level has reached the level at which
   Wisdom was last offered (+0x90, a byte) plus the class's interval, Wisdom is a candidate, it is
   not excluded and not banned.
2. **A magic school** (14 Fire, 15 Air, 16 Water, 17 Earth: 0x679CBC) when the hero's level has
   reached the level at which a school was last offered (+0x3F) plus the class's interval and the
   excluded skill is no school: a draw among the candidate schools the map does not ban, **a known
   school weighing 1**, an unknown one its class weight; with no weight at all, step 3.
3. **Any candidate**, drawn by weight: the class's (record +0x18 + skill: the packs'
   `skill_weights`, an absent skill 0); a known skill weighs at least 1; a banned skill 0, or 1
   if the hero knows it. No weight at all: no offer.

A draw is `random_between(1, total)`, then the candidates in skill order, each subtracting its
weight until the rest is at or below 0. **The intervals** (0x4DAD74): Wisdom 3 and schools 3 for
the classes 1, 3, 5, 7, 9, 11, 13 and 15 (Cleric, Druid, Wizard, Heretic, Necromancer, Warlock,
Battle Mage, Witch), else Wisdom 6 and schools 4: **the Elementalist (17) is treated as a might
class** (the list stops at the RoE classes; the Planeswalker, 16, is might either way).

The level-up asks for two offers (0x4DA94D–0x4DA9AE):
- **The first**: an advance of a known skill (window 1–2); if none, any skill below Expert
  (window 0–2: then a new one).
- **The second**: a new skill (window 0) other than the first; if none, any skill below Expert
  other than the first (with eight skills: another advance).
- **The counters**: a first offer that is a school sets +0x3F to the level before the second is
  drawn; after both, an offered Wisdom sets +0x90 (the level's low byte) and an offered school
  +0x3F. Neither offer: the level brings the primary skill only.

Each level draws 1 + 0 to 4 numbers (the primary; per offer none for Wisdom, one per draw).
Worked (computed, continuing §2.3): the Knight knowing basic Leadership only, S = 1: first offer
Leadership (the only candidate, a draw of 1 to 10 still made), second Tactics; with S = 200:
Leadership, Sorcery.

### 2.5 The answer

Learning (`hero_learn_secondary_skill` 0x4E22D0, H3API's `LearnSecondarySkill(id, increase)`): a
known skill rises by the amount, at most to Expert (3); a new one is learned only below eight
skills, at the amount, in the next slot (+0xE5 + skill = the count + 1; the count +0x101 + 1);
the ninth is refused silently, which the offers' windows already prevent.

- **A human owner at its own computer** (`player_id_is_human_here`) while no new game's setup
  runs (the global 0x698400 clear): a sound and the level-up dialog (0x4F8880; modal). No offer:
  the primary skill is shown. One offer: shown and learned without a choice (0x4DAB12). Two: the
  player takes the left one (the first offer; the dialog's result 2010) or the right (the
  second, 2011); a result of 9999 (in a network game a timer is set first; the time running out,
  inferred) hands the choice to 0x52BBD0.
- **Otherwise**: a computer player's hero, a human's hero played on another computer, a hero
  without an owner, and **any hero while `new_game_heroes` runs** (it sets 0x698400 at 0x4BFE7B
  and clears it at 0x4C04E2): no dialog. One offer is learned; with two, `ai_level_up_choice`
  (0x52BBD0) chooses: both known or both new, the higher of two values (0x524690, not read; the
  first offer on a tie); otherwise 0x524DD0 (not read) decides whether the new skill is taken.
  Its last argument is set unless the setup runs or the hero has no owner. For E10's computer
  players; for §3, the levels a map hero gets at load.

What the hero's owner and other players see of a level-up, and how the battle's experience
(increment 3) reaches 0x4E33B0: §4 (T24a2).

### 2.6 VCMI 1.7.5 compared

- **Draws**: per hero type a `HeroSkillRandomizer` seeded from the game's generator at its first
  use, its stream advancing with every draw; the two counters count level-ups since the last
  Wisdom or school offer, from 1, and reset when one is offered. For a hero from level 1 the
  counters equal the reading's level differences.
- **Primary**: `rollPrimarySkillForLevelup` runs before the level rises (`levelUpHero`), so
  `level < 10` takes the first row up to the level-up *to* 10.
- **Offers**: `rollSecondarySkills` draws advances from the known skills below Expert, then new
  skills (`canLearnSkill`: fewer than the skills per hero, allowed on the map, not known, a class
  weight above 0), the totals by the settings (one advance and two in all, more advances when
  fewer new skills remain): the reading's two offers and fallbacks. Wisdom (`MAJOR`) and the
  schools (`MINOR`) are forced as the reading's, by `isMagicHero` (the class's affinity; the
  Elementalist's is magic): 3/3 or 6/4. Every candidate weighs its class weight, at least 1.
- **Answer**: a hero with one offer, or without an owner, takes the first; two offers ask the
  owner (`CHeroLevelUpDialogQuery`), level by level. `levelUpAutomatically` (levels at load) takes
  the first offer: §3.
- **Learning**: the skills per hero (8) and Expert bound it.

### 2.7 Disputes and quirk candidates (T24a1's)

Disputes (the wiki, a code confirmation and the batch's rows: T24a2):

| Id | Question | Sources | Needed by | Settle by |
|---|---|---|---|---|
| D-14 | A level's draws: reseeded from (S, L) with the C runtime's `rand` (the reading), or a hero's own stream (VCMI) | the reading against VCMI | the level-up's draws, ADR-0017's seeds (decision 9) | the user at T24a2's checkpoint; recommended: as decision 26, the reading's seed and generator in `faithful` (the offers follow S and L, as the original's), the owner's stream in `strict-fixed` (a fix-optional quirk flag); S drawn at the build from the game's stream, server-only |
| D-15 | The primary's second row from the new level 10 (the reading) or from the old level 10, i.e. new level 11 (VCMI) | the reading against VCMI | the primary draw | the user; recommended: the reading (one level's row; a batch row reading S could confirm it, §7) |
| D-16 | The Elementalist's intervals: 6 and 4 (the reading) or 3 and 3 (VCMI) | the reading against VCMI | the offers | the user, with A-10 |
| D-17 | A due school among known schools: each weighs 1 (the reading) or its class weight (VCMI) | the reading against VCMI | the offers | the user; recommended: the reading |
| D-18 | A banned skill the hero knows weighs 1 (the reading) or its class weight (VCMI) | the reading against VCMI | the offers (maps that ban skills: §6) | the user; recommended: the reading |
| D-19 | Experience above the level limit's cap: lowered at the next gain (the reading) or kept (VCMI); beyond level 74 the wrapped levels of A-7 (the reading) or levels to 197 (VCMI, a limit at most 100) | the reading against VCMI | the experience's bound | the user, with A-7 and A-8 |

Quirk candidates (for the user to classify; §6's census sizes A-7, A-8):

| Id | Title | Impact | Suggested class | Confidence it exists in SoD |
|---|---|---|---|---|
| A-7 | Past level 74 the 32-bit thresholds wrap: 1 810 034 207 experience makes level 88 (75 to 87 skipped), more gives wrapped levels, 2³¹ − 1 hangs the game, a gain past it makes the experience negative and the level 0 | state-corrupting (a hang, a lost level) | always fixed: the engine bounds the experience (options for T24b: at 1 810 034 206, so level 74 stays the highest, or VCMI's longer table) | high: the arithmetic (computed) and the wiki's levels agree (§5) |
| A-8 | A level limit above 75 caps at a wrapped threshold: negative for 92 of 180 limits (a negative experience at the next gain), else a level unrelated to the limit | state-corrupting | always fixed (e.g. the import refuses or clamps such a limit) | high if a map can hold such a limit; no shipped map has a limit (§6.2) |
| A-9 | A primary skill past 127 wraps to −128 on a level-up | gameplay-altering, rare (a 99 plus worn artifacts plus level-ups) | always fixed or fix-optional (the user) | high: the reading; the wiki differs (a reset to 0 at 129, §5); no shipped hero starts above 9 (§6.1) |
| A-10 | The Elementalist gets Wisdom and schools on the might classes' intervals (D-16) | gameplay-altering (small: the Elementalist's offers) | the user: a faithful rule, or a fix-optional flag | high: the reading and the wiki agree (§5); no Elementalist levels at load on the shipped maps (§6.1) |

### 2.8 Leads for T24a2

- §3: during the new game's setup every level-up takes the computer's path (§2.5), with 0x52BBD0's
  argument clear; the counters are 0 there. Find where the setup gives a map hero its levels and
  compare batch E3c's E3-6 with the two offers each level would make. *(Done in §3: the
  function named here, 0x486590, turned out to be a campaign's carried-over hero's setup.)*
- §7: the harness can read a hero's seed byte (+0x8F) and counters from memory: the primary skill
  and both offers of every level then follow from §2.2–§2.4, a strong test of the whole draw
  (the worked example above is a first vector).
- Not read, for later: the level-up's other 26 callers (the objects that give levels, increment
  5), the computer's two value functions (E10), a campaign case that gives a hero experience
  during the new game's setup (the call at 0x4C004D in `new_game_heroes`).

## 3. Levels at load (T24a2a)

### 3.1 Which setup is the placed hero's

`new_game_heroes` (0x4BFE70) first gives all 156 hero records their defaults (the experience
40–90, the starting army, the movement points, the seed byte S, the counters at 0: §2.2), then
calls two setups (0x4C0037, 0x4C0042):

- **`placed_heroes_setup`** (0x4CAE10): for each hero type, its record of 0x334 bytes in the
  game's array at +0xA4 (H3API's `heroSetup`, `H3SetupHero[156]`, "to read data from h3m file";
  `structs.toml` `hero_setup`), `placed_hero_setup` (0x4D8B30) sets the hero up from it; a
  record with a position (+0x301 not negative) is the map's placed hero: it joins its owner's
  heroes and the map is revealed around it (0x4CAF74). The record holds the placed hero's body of
  the map file in its order: owner, type, name, custom experience (+0x1A, +0x1C), portrait,
  custom secondary skills (+0x22), army (+0x38), artifacts (+0x67), patrol, biography, spells
  (+0x31C), custom primary skills (+0x32C).
- **`carried_heroes_setup`** (0x487290, earlier `map_heroes_setup`), only when
  `new_game_heroes` has its third argument (one of its two callers passes one; not read
  further): it walks the game's `campaignInfo` (+0x1F458) `crossoverHeroes`
  (+0x3C) and calls **`carried_hero_setup`** (0x486590, earlier `map_hero_setup`) per carried
  hero, which tests the scenario header's five flags at +0x44 to +0x48 (H3API's
  `keepExperience`, `keepPrimarySkills`, `keepSecondarySkills`, `keepSpells`, `keepArtifacts`;
  VCMI's `whatHeroKeeps` in the same order): with the first set it copies the experience and the
  level from the earlier scenario's hero (0x486624).

So the reading E08 T6b to T22b made of 0x486590 as "the placed hero's setup" was the campaign's.
What those tasks concluded holds on the real path, which does the same where they looked: the
starting army is drawn for every hero first (`new_game_heroes` 0x4BFEA6) and a custom army
replaces it (0x4D8C6B, GAM-SET-001); 0x4CAE10 reveals around a placed hero (ADV-FOG-003); the
spell points come from Knowledge clamped to 1–99 and the Intelligence getter's arithmetic,
after the artifacts are equipped and the levels given (0x4D8EAE to 0x4D8F38, HER-MAN-001); no
write of the formation byte +0x48 and no read of the body's formation (ADV-QUIRK-8). One
difference: the placed hero's custom primary skills are copied as they are (0x4D8BFD), the
clamp to 0–99 T17a read (Spell Power and Knowledge at least 1) is the carried-over hero's. The
stale citations (rule refs, ADV-QUIRK-8, a code comment): the lane's QUEUE inbox.

### 3.2 The placed hero's levels (`placed_hero_setup` 0x4D8B30)

In order: owner, type, class, position and patrol; the custom name; the portrait; **custom
primary skills** (+0x32C: the four bytes copied, 0x4D8BF1); **custom secondary skills** (+0x22:
all cleared, then each learned at its level through 0x4E22D0, 0x4D8C59); the custom army; the
custom spells; the artifacts (the worn ones removed, the record's equipped with their bonuses,
the backpack); the biography. Then, **with a custom experience** (+0x1A): the experience is set
to 0 (0x4D8E02) and `hero_gain_experience(the record's experience, give the levels, quiet)`
runs (0x4D8EA2), then the level-up once more (0x4D8EA9, nothing left to give); last the spell
points and the movement points. Without a custom experience the hero keeps the default 40–90
(below level 2's 1 000).

So the levels' gains land on the custom skills and the worn artifacts' bonuses; the draws
follow §2 with the S the setup drew a moment before and the counters at 0; **every two-offer
level is chosen by `ai_level_up_choice` (0x52BBD0) with its flag clear**, since
`new_game_setup_running` (0x698400) is set throughout (§2.5); a one-offer level learns it. The
scenario's level limit caps the experience as in §1.3 (the levels end at the limit, the
experience is lowered to its threshold). A campaign case replaces the experience (campaign 8,
scenario index 3: the experience of a level from the campaign's state, 0x4D8E19 to 0x4D8E99).

### 3.3 Batch E3c's E3-6 against the reading

E3-6's hero (a Death Knight, class 8: might, intervals 6 and 4) had 5 000 experience, 2/3/1/1
and eight basic skills (Pathfinding, Archery, Logistics, Scouting, Navigation, Mysticism,
Offense, Armorer): no Wisdom, no school, eight skills, so every offer at levels 2 to 5 is a
plain draw among the known skills by the Death Knight's weights, both offers advances. It
started at level 5 with 3/6/1/1 (Attack +1, Defense +3), Pathfinding and Scouting advanced,
Logistics expert. Computed from §2 for every S of 1 to 255: 3 seeds give those primary gains (75,
86, 247; about 4.8 expected by chance); of them only S = 75 offers pairs from which those
advances can be chosen (Pathfinding or Navigation, Archery or Logistics, Navigation or
Scouting, Mysticism or Logistics: chosen left, right, right, right). **Consistent, not a
confirmation**: the run did not read S; a run that does (§7) predicts the gains and offers
before the choice, and then tests 0x52BBD0's choice as well.

### 3.4 VCMI

`CGHeroInstance::initHero`: a set experience calls `levelUpAutomatically`, which while
`gainsLevel` (the next level's threshold reached and below VCMI's highest level; **the map's
level limit is not consulted**) rolls the primary skill and the offers as at a level-up (§2.6)
and takes **the first offer**. The formation is reset to loose (as the reading's, which never
copies it).

### 3.5 Disputes (T24a2a's)

| Id | Question | Sources | Needed by | Settle by |
|---|---|---|---|---|
| D-20 | At load, which offer is learned: the computer's evaluation (0x52BBD0; the reading) or the first (VCMI) | the reading against VCMI and the wiki (always the left offer; §5.2: E3-6 fits the reading) | the import's levels (decision 48) | the user at T24a2b2's checkpoint; the evaluation (0x524690, 0x524DD0) is unread: E10's computer players need it as well. Options for T24b: read and port it, or the first offer (VCMI) as a documented stand-in |
| D-21 | At load, a custom experience above the scenario's level limit: capped and lowered to the limit's threshold (the reading) or levelled to its own level (VCMI) | the reading against VCMI | the import | the user; recommended: the reading (§6's census: does a shipped map have such a hero?) |

## 4. What follows the level (T24a2a)

### 4.1 The battle's experience

`battle_end` (0x477470) computes the winner's experience (`battle_experience` 0x46A350) and
gains it in one of two exclusive places: **a winner whose player is not a human at this
computer** (the side's byte +0x54A6 clear: a computer player, another computer's player),
`hero_gain_experience` with the levels given and the owner told (0x477766), before Necromancy and
Eagle Eye, so the levels follow the computer's path at once (§2.5); **a winner whose player is a
human here** (tested at 0x47789A to 0x4778B8), after the victory's music, `hero_gain_experience`
with the levels given only if that byte is clear, so not (0x477948), then the Eagle Eye
message. `adventure_battle` (0x4AD470) then gives both heroes'
levels after the battle's screen (0x4AE324, 0x4AE42C; in network play the remote hero's through
the network, 0x4AE37C, 0x4AE405): the human's level-up dialogs come on the map, one level after
another (§2.1). The loser leaves the map (increment 3), so only the winner levels. VCMI
(`BattleResultProcessor` → `CGameHandler::giveExperience` → `expGiven` → `levelUpHero`): the
winner's experience, then the levels one at a time, a query to the owner for each two-offer
level.

For the engine: increment 3's outcome already gives the winner experience (`heroes_after_battle`,
HER-EXP-001); increment 4 turns the gained experience into levels after the battle: the level-ups
as the winner's owner's pending decisions (T24b), one level at a time, each level's offers drawn
after the previous answer (§2.1).

### 4.2 The rules that read the level

`h3-heroes` reads `HeroDetails.level` (a `u8`, at least 1; its experience "not checked against
the level" until this increment) in: the skill specialties' growth (`bp_per_level` basis points
per level, `grown_effects`: HER-BAT-004, HER-BAT-005, and through it HER-EXP-001's Learning and
HER-MAN-001's Intelligence), the creature specialties (`bp_per_step` × hero level ÷ creature
level), the spell specialties (the level passed to the battle). The original's
getters read hero +0x55 the same way (`hero_learning` 0x4E4840, `hero_intelligence` 0x4E48B0,
`hero_necromancy_power` 0x4E3CD0, `hero_scouting_radius` 0x4E4070, `hero_surrender_factor`
0x4E4580). Nothing in them changes with increment 4 but the level's source: the level comes from
the experience (§1) instead of the details' field alone, and the spell points of a hero
levelled at load are computed after its levels (§3.2). Not read: the Logistics specialty's
growth in `hero_max_land_movement` (the battle lane's lead, QUEUE inbox; increment 5's movement
profile).

The engine keeps a hero's base primary skills without the artifacts (`HeroDetails.primary`,
at most the ruleset's `primary_max`), where the original adds the worn artifacts' bonuses into the
same byte (`structs.toml` hero +0x476): a level-up's +1 is to the base, and A-9's wrap is the
original's byte, not the engine's.

### 4.3 What another player sees

VCMI 1.7.5's view of a hero (`InfoAboutHero`): name, portrait, class and army, with details
(primary skills, mana, luck, morale) only at its detailed and in-battle levels: **no level** at
any. The original's right-click view was not read (survey `E08-battles.md`
§9). The engine shows another player's hero by tile, owner and identity (ADV-FOG-006). A
level-up's draws and offers are the owner's: the offers and the seed byte are hidden information
for every other player (decision 9); the gains show to others only as far as primary skills do
(in a battle against the hero, §9 of `E08-battles.md`).

## 5. The wiki against §1–§4 (T24a2b1)

Read through WebFetch (summarised; short verbatim quotes asked for, the pages' raw markup),
2026-10-20, with the user's approval: `https://heroes.thelazy.net/index.php/Experience`,
`…/Secondary_skill` (section "Secondary skill advancement rules"), `…/Hero_class`,
`…/Primary_skill`, `…/Prison`, `…/Campaign` and the search for "level limit". None of the
passages below carries a version mark; the pages mark HotA's changes apart (`{{hota}}`,
`{{-wh}}`), so they are read as the SoD game's.

### 5.1 Point by point

| Survey | The wiki | Against the reading |
|---|---|---|
| §1.1 the thresholds | Experience: levels 1 to 20 (13 = 24 320 … 17 = 48 279, 20 = 81 961) and 70 to 75 (74 = 1 508 362 195, "75/88" = 1 810 034 207) | the same numbers |
| §1.2 the highest level (A-7) | "after reaching level 75, a hero gets level 88 at once (13 more level-ups consecutively)"; the next levels 100, 108, 868, 3 732, 5 920; "The final usable level is level 6424", beyond it an endless loop of level-ups; HotA: at most 74, the overflow fixed | **agrees**, and more exactly than §1.2 said: the levels a hero can hold are the records of the wrapped thresholds, and the reading's arithmetic (32-bit sums, `crt_ftol`'s low word; computed) gives exactly 75 → 88, 100, 108, 868, 3 732, 5 920, 6 424. Past 6 424 (from 2 147 400 657, level 6 425's wrapped threshold) the next record would make the hero level 72 924, which the 16-bit level (+0x55) cannot hold: the wiki's endless loop there fits that bound (where the loop then runs on was not read). A-7's confidence stays high |
| §1.3 the level limit | no page; the search finds only the Campaign page's tips (a hero that "has already hit the level limit") | silent: the reading stands alone against VCMI (D-19, D-21; an experiment candidate) |
| §2.2 the seed (D-14) | "The random seed for offering skills is separate for every hero and is fixed at the start of the scenario", unhired heroes included; reloading brings the same offers; skills learned elsewhere (Witch Hut, Scholar) change the sequence | **agrees** with the reading (S per hero from the new game; a level's offers follow S, L and the hero's skills). VCMI's own stream per hero would repeat on a reload too, but not when the order of other draws changes |
| §2.3 the primary rows (D-15) | Hero class: "chances to raise them at 2-9 and 10+ level-ups", numbered by the level reached; Primary skill: "levels 1 to 9" and "after that" | **agrees** with the reading (the new level 10 takes the second row), against VCMI; the five rows checked (Knight, Cleric, Death Knight, Planeswalker, Elementalist) equal the packs' `primary_gain` |
| §2.3 the bound (A-9) | Primary skill: "The upper limit is 99. At 129 the skill value is reset to 0." | **differs**: the reading's byte wraps past 127 to −128; a value that acts or shows as 0 from there would fit a clamp of negative values (not read), but the wiki's 129 does not fit the wrap at 128. Reachable only past 99 with artifacts (§6: no shipped map's hero starts near it) |
| §2.4 the offers | two different skills: first a known one at Basic or Advanced, second by the class's weights; two new ones when nothing below Expert is known; "if there is only one skill to offer, it will be forcefully added"; none: the primary skill only; eight skills: two advances; a class's weights in 112ths | **agrees** (every class's `skill_weights` in the packs sums to 112) |
| §2.4 Wisdom and schools | Wisdom due every 3 levels for magic classes, 6 for might (at levels 3 and 6 when never offered, then counted from the last offer); a school every 3 and 4; Wisdom first; a counter reset whenever its skill is offered, by chance or forced | **agrees** (+0x90, +0x3F; an offer by chance resets as well) |
| D-16, A-10 the Elementalist | "The only exception is the Elementalist class, to which are considered Might heroes by the levelling system"; Hero class: "a might hero (or an Elementalist)"; HotA made it level as a magic class | **agrees** with the reading against VCMI; A-10's confidence rises to high (the reading and the wiki) |
| D-17 a due school | "The exact School of Magic offered is randomly chosen based on the relative probabilities" | silent on a known school's weight |
| D-18 banned skills | "If the hero's class is banned from obtaining a specific skill but the hero has it, consider the weight to be equal to 1"; step 1 ignores "globally banned skills" (Navigation on land-locked maps) with no such exception | the class ban **agrees** with the reading (a known skill weighs at least 1); for a skill **the map** bans and the hero knows, the wiki's letter gives 0, the reading 1, VCMI the class weight: three answers, reached only when a hero knows a skill its map bans (§6) |
| §3 levels at load | heroes starting with more than 1 000 experience, "such as" a Prison's, "are assumed to have always selected the left-hand skill option during their fictitious level-ups", so they advance known skills first | the gains at load **agree**; the choice **conflicts** (D-20, §5.2) |
| §4 | not asked (the battle's experience; the view of another player's hero) | — |

Seen on the way: a new game's hero starts with 40 to 90 experience (Experience; the reading's
default, §3.1); a prisoner is not freed while its player has eight heroes outside towns
(Prison; for the increment that builds prisons).

### 5.2 D-20 is a conflict of sources

The wiki (always the left offer at load) agrees with VCMI (the first offer) against the reading
(`ai_level_up_choice` 0x52BBD0 chooses). Batch E3c's E3-6, a placed Death Knight levelled at
load in SoD, does not fit always-left: under the reading's draws only S = 75 gives both its
primary gains and offers from which its advances can be taken, and those need left, right,
right, right (§3.3; the other two seeds that give the gains, 86 and 247, offer pairs without
them). So E3-6 sides with the reading, if the reading's draws are right, which the run did not
test (S unread). By `docs/04-rules-and-sources.md` §2 the reading is not built on alone: the
second reading (T24a2b2) re-reads 0x4D8B30's path to the levels and 0x52BBD0 for a left-only
branch at load, and the batch (§7) reads S and the offers of every level at load, which settles
it. Until then D-20 is the user's at the checkpoint (an ADV-Q, T24a2b2).

## 6. The census (T24a2b1)

Counted on 2026-10-20 over the installation's 160 shipped maps (`original-data/maps.tar`: 49 RoE,
55 AB, 56 SoD) with `h3-map`'s reader, a throwaway example and script (not committed); the levels
from §1.1's thresholds. Decision 22's "42 of 116 placed heroes" counted the 164 maps of T18b (the
user's four maps among them); over the 160 the heroes with an experience are the same 42.

### 6.1 Placed heroes with a custom experience, and their levels

The hero objects: 108 heroes (class 34), 89 prisons (62) and 176 random heroes (70, refused by
the import as `RandomHero`; 18 of them carry an experience too, drawn heroes for whoever builds
random heroes). Of the 197 heroes and prisoners, **103 have a custom experience, every one at
least 1 000** (none between 1 and 999): 42 heroes in 18 maps (RoE 13, AB 19, SoD 10) and 61
prisoners in 28 maps. Each takes it from its own body; no placed hero takes a map-wide entry's
(SoD's hero settings: only `Marshland Menace` has entries with an experience, 15 hero types at
2 000, level 3, reached through its taverns, E09).

| Level at load | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 13 | 15 | 42 | Level-ups |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Heroes (42) | 6 | 6 | 4 | 1 | 4 | 2 | 8 | 2 | 6 | | 3 | | 273 |
| Prisoners (61) | 4 | 6 | 5 | 1 | 12 | 1 | 17 | 5 | 1 | 3 | 4 | 2 | 495 |

The highest: two prisoners of `Pandora's Box ` with 5 000 000 (level 42); among the heroes
40 000 (level 15: `Adventures of Jared Haret`'s Theodorus, `Good Witch, Bad Witch`'s Rosic). 20
of the 42 heroes and 21 of the 61 prisoners are of a magic class; **no Elementalist** and one
Conflux hero (Pasis, a Planeswalker prisoner, level 13): D-16 and A-10 do not reach the levels at
load on the shipped maps. Prisons are untyped objects in the import (their levels at load come
with the increment that builds them, as the wiki's "fictitious level-ups" of a prisoner, §5).

**Custom primary skills:** 6 heroes (4 in `Adventures of Jared Haret`, 2 in `Good Witch, Bad
Witch`), the largest value 9: **none above 99**, so the import's clamp, which the placed hero's
setup does not have (§3.1, the QUEUE inbox), changes no shipped map; A-9 is not reached at load.

### 6.2 Level limits (A-8, D-19, D-21)

**No shipped map has a level limit**: the header byte is 0 on all 111 AB and SoD maps (RoE has
none). A-8, the limit's part of D-19 and D-21 come only with other maps (the editor's setting).

### 6.3 Banned skills (D-18)

19 maps ban skills, all SoD: Navigation all 19, Eagle Eye 6, Scouting 1, and `Heroes of Might not
Magic` (and its allied copy) 13 (Wisdom, Mysticism, the four schools, Necromancy, Scholar,
Intelligence, Sorcery, Resistance besides Navigation and Eagle Eye). **No placed hero or
prisoner knows a skill its map bans**, by its custom or its starting skills: D-18's map-ban
case does not arise at load. It can through the taverns (2 to 91 hero types per banning map
start with a banned skill; the maps' allowed heroes not counted; whether a hired hero keeps such
a skill was not read) and through skills learned on the map (Witch Huts, Scholars): E09 and
increment 5.

### 6.4 Experience near A-7

The largest custom experience is 5 000 000; the objects' experience rewards (82 in 35 maps:
events, Pandora's boxes, seer's huts) give at most 35 000 each and 85 000 per map (`Back For
Revenge`): far below level 74's 1 508 362 195. A-7 comes only from long play, never from a
map's data.

### 6.5 The maps that import whole without the refusal

With `Unsupported::HeroExperience` lifted (a throwaway patch, not committed), **5 maps import
whole and validate**: today's two and `Adventures of Jared Haret`, `Faeries` and `Knee Deep in
the Dead` (refused today for an experience first). `Unholy Quest`, T18b's fourth, has been refused
since T20a2a for a monster's artifact. The rest: a hero generated at a main town 128, a random
hero 18, a monster's artifact 8, a hero standing in a town 1. The three maps' 9 heroes get 41
level-ups at load: Theodorus (Wizard) 15, Josephine 7 and Thane 6 (Alchemists), Gunnar 5 and
Lorelei 4 (Overlords), Christian 4 and Edric 3 (Knights), Nymus 3 (Demoniac), Sandro 3
(Necromancer).

### 6.6 For T24b

On the maps increment 4 can import, levels at load mean 9 heroes at levels 3 to 15: D-20's
choice decides their skills; no level limit, no Elementalist, no banned skill, no primary skill
above 99 and nothing near A-7 among them. The other cases (the limit, A-7 to A-9, D-18's map
ban) need other maps or long play: T24b decides whether increment 4 builds them or bounds them
(A-7, A-8 are state-corrupting, always fixed).

## 7. The batch's plan *(T24a2b2)*

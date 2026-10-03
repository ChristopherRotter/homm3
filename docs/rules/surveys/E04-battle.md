# E04a T1: source survey for heroes in battle

Date: 2026-10-03. Scope: E04a battle mechanics (tactics, retreat, surrender, outcome, hero-run war machines, the hero's presence), plus an outline of what E04b (magic) and E04c (sieges) need from the seam. Research only.

## How to read this file

**Source labels**
- `original-data` means the GOG Complete 4.0 text tables in `original-data/extracted/H3bitmap/` (SoD). Line numbers are physical 1-based lines of the file. Where a VCMI index exists, it is that number minus 1 (for example `GENRLTXT.TXT` line 33 is `core.genrltxt.32`). In this installation, `H3ab_bmp/SSTraits.txt` is byte-identical to the SoD copy apart from line endings (checked with diff).
- `vcmi` means VCMI 1.7.5, tag `1.7.5`, commit `6901839a`. Links have the form `https://github.com/vcmi/vcmi/blob/1.7.5/<path>#L<a>-L<b>`.
- `wiki` means heroes.thelazy.net, retrieved 2026-10-03.
  - Pages were read as raw wikitext through WebFetch (`index.php?title=<Page>&action=raw`). Direct curl to the wiki is blocked by the egress proxy.
  - Raw wikitext matters for one template: `{{swh|A|B}}` shows **A in HotA and B in SoD/classic**. This was checked on three pages: First Aid Tent `{{swh|20-25|1-25}}`, Necromancy `{{swh|5%|10%}}` and Artillery. Every wiki value below is the SoD branch unless it says "HotA".
  - The WebFetch summariser sometimes invented details: on the Combat page it claimed "including war machines" for tactics. Only statements that were re-checked as verbatim lines are used here.
- Precedence: original data > observation > reverse engineering > VCMI > wiki. A disagreement with gameplay impact is listed under **Disputes** (section 10).

**Coordinates.** VCMI's `BattleHex` uses `x = hex % 17` and `y = hex / 17` on a 17 × 11 field. Columns 0 and 16 cannot be stood on ([lib/battle/BattleHex.h L20-22](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/BattleHex.h#L20-L22), [L132-140](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/BattleHex.h#L132-L140)). This is identical to our BAT-GEO-001.

---

## 1. Tactics phase

### 1.1 Who gets it

- **R1.1a The skill levels subtract.**
  - `vcmi`: Tactics gives two bonuses, `BEFORE_BATTLE_REPOSITION` and `BEFORE_BATTLE_REPOSITION_BLOCK`, each worth **2 / 4 / 6** at Basic / Advanced / Expert ([config/skills.json L577-610](https://github.com/vcmi/vcmi/blob/1.7.5/config/skills.json#L577-L610)).
  - `vcmi`: `diffAttacker = reposition[att] − block[def]` and `diffDefender = reposition[def] − block[att]`. If `diffAttacker > 0` the attacker gets the phase, else if `diffDefender > 0` the defender gets it, else nobody. If both were positive, the attacker would be chosen; this cannot happen with H3 data ([lib/battle/BattleInfo.cpp L397-437](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/BattleInfo.cpp#L397-L437)).
  - `wiki` (Tactics): "Same level of Tactics: Cancelling each other out". The examples are Advanced vs Basic → Basic, Expert vs Basic → Advanced, Expert vs Advanced → Basic. Each level "negates one/two/three levels of an enemy hero's Tactics skill". [Tactics](https://heroes.thelazy.net/index.php/Tactics)
  - `original-data`: `GENRLTXT.TXT` l.432-434: "In the Tactics Phase, a hero with superior Tactics skill may arrange his armies before combat begins."
- **R1.1b A side without a hero blocks nothing.** Neutral monsters and a garrison without a hero give no block, so a hero with Tactics always gets the phase against them (`vcmi`: the bonuses are summed only for present heroes, BattleInfo.cpp L400-408).
- **R1.1c The "Tactics formation" toggle on the hero screen.**
  - `original-data` `HeroScrn.txt` l.26-27 ("Disable/Enable Tactics formation") and l.32: "Enabling Tactics formation will allow you to arrange your troops prior to combat (must have the Tactics skill)."
  - `vcmi`: the flag `tacticFormationEnabled` defaults to true ([lib/mapObjects/CGHeroInstance.h L85](https://github.com/vcmi/vcmi/blob/1.7.5/lib/mapObjects/CGHeroInstance.h#L85), [client/windows/CHeroWindow.cpp L219-222, L325-332](https://github.com/vcmi/vcmi/blob/1.7.5/client/windows/CHeroWindow.cpp#L219-L222)).
  - `vcmi`: when the toggle is off, only the hero's **own reposition value** is dropped. **The block still counts** (BattleInfo.cpp L404-406). The original behaviour is unknown, see OQ-1.
- **R1.1d No tactics in creature banks.** `vcmi`: the layouts `creatureBankNarrow` and `creatureBankWide` have `"tacticsAllowed": false` and `"obstaclesAllowed": false`; the default layout has true ([config/gameConfig.json L624-686](https://github.com/vcmi/vcmi/blob/1.7.5/config/gameConfig.json#L624-L686)). `wiki` (Creature Bank): "Tactics also does not work in creature banks." [Creature Bank](https://heroes.thelazy.net/index.php/Creature_Bank)

### 1.2 Placement area

- **R1.2a The zone is a band of columns.** `tacticDistance d = 1 + diff`, which gives **d = 3 / 5 / 7** for an effective Basic / Advanced / Expert ([BattleInfo.cpp L426-433](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/BattleInfo.cpp#L426-L433); [lib/battle/BattleInfo.h L51-52](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/BattleInfo.h#L51-L52) comments "how many hexes we can go forward (1 = only hexes adjacent to margin line)").
  - The allowed destination hexes are ([lib/battle/CBattleInfoCallback.cpp L1666-1679](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/CBattleInfoCallback.cpp#L1666-L1679)):
    - attacker: `1 ≤ x ≤ d`
    - defender: `16 − d ≤ x ≤ 15`
  - `x` is the raw offset column, so the band has a **zig-zag edge** on the hex grid, the same in every row index.
  - Only the destination (head) hex is tested. A two-hex unit's tail lies at x−1 (attacker) or x+1 (defender), so it is inside the band or on the edge column. Column 0 cannot be stood on, which rejects an attacker head at x = 1.
  - `original-data` `SSTRAITS.TXT` (Tactics): "rearrange your troops ... within three / five / seven hex rows of the commanding hero".
  - `wiki` (Tactics, user commentary): "With Basic/Advanced/Expert Tactics, the distance is reduced to 11/9/7 hexes". This matches columns 1..3/5/7 against column 15 (a stack must reach column 14).
- **R1.2b Path rules: a free placement within reachability, ignoring speed.**
  - `vcmi`: during the phase the available hexes are every hex that is **reachable** (`cache.isReachable`) and inside the band; the speed limit is not applied ([CBattleInfoCallback.cpp L624-659](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/CBattleInfoCallback.cpp#L624-L659)).
  - `vcmi`: on the server, a unit with movement range > 0 gets range `BFIELD_SIZE` during tactics ([server/battles/BattleActionProcessor.cpp L792-801](https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleActionProcessor.cpp#L792-L801)).
  - So a walker needs **some walking path of any length**. The path may pass through hexes outside the band; only the destination is checked (L748-751).
  - A flyer can go to **any free hex** in the band, because flying reachability means accessible ([CBattleInfoCallback.cpp L1712-1720](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/CBattleInfoCallback.cpp#L1712-L1720)).
  - The destination must be free of stacks and obstacles (L774-778).
  - `wiki`: nothing on paths, see OQ-2.
- **R1.2c Units that cannot move cannot be placed.**
  - `vcmi`: the tactics range is widened only `if (unitMovementRange > 0)`. War machines have movement range 0 (SIEGE_WEAPON, [lib/BasicTypes.cpp L147-150](https://github.com/vcmi/vcmi/blob/1.7.5/lib/BasicTypes.cpp#L147-L150)), so **war machines cannot be repositioned**.
  - Two-hex creatures can be repositioned (accessibility is checked for both hexes; a "shift" fallback applies at L765-772).
  - `wiki`: does not say. The Combat page states the hero arranges "their troops".
- **R1.2d A stack may be moved any number of times, in any order.**
  - `vcmi`: moves during tactics do not change stack state ("moves in tactics phase do not affect creature status") ([lib/gameState/GameStatePackVisitor.cpp L1320-1335](https://github.com/vcmi/vcmi/blob/1.7.5/lib/gameState/GameStatePackVisitor.cpp#L1320-L1335)).
  - `vcmi`: any living stack of the tactics side may act ([BattleActionProcessor.cpp L655-662](https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleActionProcessor.cpp#L655-L662)).
  - `vcmi`: the client offers "next stack" and "end tactics" shortcuts.
- **R1.2e Allowed actions during the phase.** `vcmi` `isTacticsAction` allows WALK, END_TACTIC_PHASE, RETREAT and SURRENDER ([lib/battle/BattleAction.cpp L255-264](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/BattleAction.cpp#L255-L264), enforced in [BattleActionProcessor.cpp L1710-1723](https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleActionProcessor.cpp#L1710-L1723)).
  - No attacks.
  - No spells: `ONGOING_TACTIC_PHASE` ([CBattleInfoCallback.cpp L135-136](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/CBattleInfoCallback.cpp#L135-L136)).
  - `wiki` (Combat, verbatim): "During this arranging, the hero cannot cast spells, but may retreat or surrender."
  - Side note: VCMI's check would also accept RETREAT from the *non*-tactician during the phase (only `player == sideToPlayer(ba.side)` is tested). That is not a rule worth copying.

### 1.3 End of phase and what follows

- **R1.3a The phase ends** when the tactician sends `END_TACTIC_PHASE`, or when the turn timer sends it automatically ([server/TurnTimerHandler.cpp L332](https://github.com/vcmi/vcmi/blob/1.7.5/server/TurnTimerHandler.cpp#L332)). The tactic distance is then set to 0 ([GameStatePackVisitor.cpp L1324-1328](https://github.com/vcmi/vcmi/blob/1.7.5/lib/gameState/GameStatePackVisitor.cpp#L1324-L1328)).
- **R1.3b Round numbering.**
  - The battle starts with `round = 0` ([BattleInfo.cpp L171](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/BattleInfo.cpp#L171)).
  - With no tactics, `onTacticsEnded` runs at once ([server/battles/BattleFlowProcessor.cpp L135-143](https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleFlowProcessor.cpp#L135-L143)).
  - `onTacticsEnded` does three things, then calls `startNextRound` (round becomes 1) and activates the first stack by normal turn order ([L230-249](https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleFlowProcessor.cpp#L230-L249), [BattleInfo.cpp L651-658](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/BattleInfo.cpp#L651-L658)):
    - summons guardians;
    - runs the "stack enchanted" triggers;
    - casts the heroes' opening battle spells.
  - **The tactics side gets no extra initiative.** The phase is "round 0" and not a combat round.
  - `wiki` (Combat): "If it is the beginning of the battle, the attacker will have the initiative" refers to speed ties in round 1, not to tactics.
- **R1.3c Visibility.**
  - `vcmi`: every tactics move is an ordinary `BattleStackMoved` pack sent to all clients ([BattleActionProcessor.cpp L849-857](https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleActionProcessor.cpp#L849-L857)), so the enemy sees the moves.
  - `vcmi`: the *distance* is hidden from players who do not know the tactics side ([CBattleInfoCallback.cpp L1655-1664](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/CBattleInfoCallback.cpp#L1655-L1664)).
  - `wiki`: silent.
  - Gameplay impact is nil: the opponent cannot act until round 1, and positions are public from round 1.
- **R1.3d Interplay with formation.** The formation (loose/tight, our BAT-SET-001) decides the *initial* positions. The tactics phase moves stacks from there. VCMI has no other coupling (formation in [lib/battle/BattleLayout.cpp L61-70](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/BattleLayout.cpp#L61-L70); tactics after setup). `original-data` HeroScrn.txt l.24-25 and l.30-31 cover the formation buttons, and l.26-27 and l.32 the separate tactics toggle.
- **R1.3e Obstacles.**
  - Random obstacles are placed before the phase. The tactics zone never contains rock obstacles in columns ≤ 2 or ≥ 14, because the obstacle generator rejects blocked hexes with `x <= 2 || x >= 14` ([BattleInfo.cpp L269-271](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/BattleInfo.cpp#L269-L271)). Columns 3..7 can hold obstacles.
  - Destinations on obstacles are not accessible.
  - In sieges (E04c), "stoppers" such as the moat end a walk ([CBattleInfoCallback.cpp L1406](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/CBattleInfoCallback.cpp#L1406)).

---

## 2. Retreat

- **R2a Conditions.** `vcmi` `battleCanFlee` ([lib/battle/CBattleInfoEssentials.cpp L297-328](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/CBattleInfoEssentials.cpp#L297-L328)):
  - (1) the side has a hero;
  - (2) not `BATTLE_NO_FLEEING`, but only checked **when the enemy also has a hero**:
    - Shackles of War carries the bonus with propagator `BATTLE_WIDE` ([config/artifacts.json L1828-1838](https://github.com/vcmi/vcmi/blob/1.7.5/config/artifacts.json#L1828-L1838));
    - both armies attach to the battle node ([BattleInfo.cpp L62-69](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/BattleInfo.cpp#L62-L69)), so it applies to both heroes;
  - (3) the rule "no flee after casting in the first N rounds as attacker" has `noSpellHitAndRunRounds: 0` by default (HotA/PvP rule, [gameConfig.json L617-618](https://github.com/vcmi/vcmi/blob/1.7.5/config/gameConfig.json#L617-L618)), so it does not apply;
  - (4) a besieged defender only with an **Escape Tunnel** (Stronghold).
  - `wiki` (Combat#Retreat, verbatim lines):
    - "A player may retreat from (almost) any combat involving their hero, including before combat during their Tactics Phase."
    - "A player cannot retreat if either hero wields the Shackles of War or if the hero is defending in a siege (unless they are defending a Stronghold town with an Escape Tunnel …)". The Diplomat's Cloak exception is HotA only. [Combat](https://heroes.thelazy.net/index.php/Combat)
  - `wiki` (Shackles of War): "Neither hero may retreat or surrender in combat." and "**In HotA, the effect is active in hero-to-hero battles only.**" [Shackles of War](https://heroes.thelazy.net/index.php/Shackles_of_War). So in SoD the Shackles also stop retreat from neutrals, while VCMI follows HotA. → **D-6**.
  - `original-data` `ARTRAITS.TXT` l.380: "When these shackles are wielded in combat, neither you nor your opponent may retreat or surrender." `GENRLTXT.TXT` l.397: "The Shackles of War are present.  %s can not retreat!"; l.33: "Are you sure you want to retreat?"
- **R2b When it can be issued.**
  - `vcmi`: only when one of the player's own units is the active unit, or during the player's tactics phase ([BattleActionProcessor.cpp L1705-1749](https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleActionProcessor.cpp#L1705-L1749)).
  - Whether the active unit can be a hero-controlled war machine is not restricted.
  - **Neutral monsters and garrisons without a hero can never retreat.**
- **R2c Result and hero fate.**
  - `vcmi`: `EBattleResult::ESCAPE`, winner = the other side ([BattleActionProcessor.cpp L63-73](https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleActionProcessor.cpp#L63-L73)).
  - `vcmi`: the hero goes to a tavern slot with role RETREATED, with **army replaced by 1 creature of the hero type's default stack** and movement points not refilled ([server/processors/HeroPoolProcessor.cpp L80-93](https://github.com/vcmi/vcmi/blob/1.7.5/server/processors/HeroPoolProcessor.cpp#L80-L93); called from [server/battles/BattleResultProcessor.cpp L640-644](https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleResultProcessor.cpp#L640-L644)).
  - The hero keeps all artifacts, because the transfer runs only for `NORMAL` (L524). Destroyed war machines are still removed (§4.4).
  - `wiki` (Combat):
    - "The retreated hero retains their all of their artifacts, but loses their army. They will be immediately available to hire again in the player's Tavern (with one unit of their default stack)."
    - "If a player retreats from combat, the retreated hero receives no experience from the battle."
    - The 7th-day tavern rule is an orchestrator matter.
  - `original-data` `GENRLTXT.TXT` l.358 "The enemy has fled!", l.364 "The cowardly %s flees from battle.", l.367 "Your cowardly forces flee from battle."
- **R2d Experience after a retreat.**
  - `vcmi`: the winner gets XP for the creatures it **killed** (casualties only; the retreating side's survivors are not casualties) and **no +500** (the +500 needs `NORMAL`, [BattleResultProcessor.cpp L236-242](https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleResultProcessor.cpp#L236-L242)). The retreating hero gets nothing (only the winner's XP is applied, L329-336).
  - `wiki` (Experience, verbatim): "A hero will not receive this bonus if the enemy hero escapes or surrenders." "If a hero escapes or surrenders, then that hero does not gain any experience …" "A hero from whom the enemy hero escaped receives experience for the killed enemy creatures, but does not gain experience for the defeated hero." [Experience](https://heroes.thelazy.net/index.php/Experience)
  - The sources agree.

---

## 3. Surrender

- **R3a Conditions.** `vcmi` `battleCanSurrender` = `battleCanFlee` ∧ ¬(the side is a besieged defender) ∧ the enemy has a hero ([CBattleInfoEssentials.cpp L389-398](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/CBattleInfoEssentials.cpp#L389-L398)).
  - The Escape Tunnel does **not** allow surrender.
  - Neutral monsters and garrisons without a hero can never be surrendered to.
  - There must be enough gold ([BattleActionProcessor.cpp L75-94](https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleActionProcessor.cpp#L75-L94)).
  - `wiki` (Combat#Surrender): "In hero-on-hero combat exclusively …", "Surrendering is impossible when defending a siege or if either hero wields the Shackles of War", "possible at any point during battle, even before combat during the surrendering player's Tactics Phase", "If the player does not have enough gold to surrender, then surrendering is impossible."
  - `original-data` `GENRLTXT.TXT` l.398: "The Shackles of War are present.  %s can not surrender!"
- **R3b The enemy's consent.**
  - `wiki`: "**The enemy may refuse a surrender, which continues the combat.** In HotA the enemy cannot refuse a surrender."
  - `original-data` `GENRLTXT.TXT` l.37-39 is the offer dialogue: `"%s states: ""I will accept your surrender and grant you and your troops safe passage for the price of %d gold."""`.
  - `vcmi`: surrender is immediate, with no consent step.
  - → **D-5**. This affects the seam: a two-step offer/answer protocol.
- **R3c Cost.**
  - `vcmi`: `cost = floor( Σ_{alive units from army slots} goldCost(creature) × count × (100 − discount) / 100 )`, floored at 0 ([CBattleInfoCallback.cpp L2347-2369](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/CBattleInfoCallback.cpp#L2347-L2369); [lib/battle/Unit.cpp L238-245](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/Unit.cpp#L238-L245); gold only, [lib/battle/CUnitState.cpp L391-394](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/CUnitState.cpp#L391-L394)).
    - It uses the current counts. War machines and summoned units are excluded (not a valid army slot).
    - `discount` is the sum of `SURRENDER_DISCOUNT`, so it combines **additively**:
      - Diplomacy **20 / 40 / 60** ([config/skills.json L118-153](https://github.com/vcmi/vcmi/blob/1.7.5/config/skills.json#L118-L153));
      - Statesman's Medal, Diplomat's Ring, Ambassador's Sash **10 each** ([config/artifacts.json L1007-1042](https://github.com/vcmi/vcmi/blob/1.7.5/config/artifacts.json#L1007-L1042)).
    - **VCMI uses factor 1.0, the full recruitment cost.**
  - `wiki` (Combat#Surrender): "The amount of gold required to surrender is equal to **half the recruitment cost of the remaining creatures** in the hero's army, with any resources other than gold being excluded in the calculation."
    - Reductions are Diplomacy and the Ambassador's Sash, Diplomat's Ring and Statesman's Medal (SoD list).
    - The Diplomacy page gives 20% / 40% / 60%; the artifact pages give "-10% surrender cost". How they combine is not stated.
  - `original-data`: SSTRAITS Diplomacy "reduces the cost of surrendering by 20% / 40% / 60%"; ARTRAITS l.203/206/209 "reduces the cost of surrendering" (no number).
  - → **D-1** (factor 0.5 vs 1.0) and **D-2** (additive vs multiplicative reductions).
- **R3d What the surrendering hero keeps, and the result.**
  - `vcmi`: `EBattleResult::SURRENDER`, winner = the other side. The hero goes to a tavern slot with role SURRENDERED and **keeps its surviving army**: casualties are applied, the army is not cleared ([HeroPoolProcessor.cpp L68-78](https://github.com/vcmi/vcmi/blob/1.7.5/server/processors/HeroPoolProcessor.cpp#L68-L78)). It keeps its artifacts. There is no +500 for the winner and no XP for the surrendering hero.
  - `wiki`: "Unlike retreating, the surrendered hero retains their surviving army in addition to all of their artifacts upon recruiting them again."
  - `original-data` `GENRLTXT.TXT` l.120 "%s surrenders, and gains safe passage for %d gold.", l.357 "The enemy has surrendered!", l.363/366.

---

## 4. Outcome after the battle

### 4.1 End of battle and casualties
- **R4.1a Who is still alive.**
  - `vcmi` `battleIsFinished`: a side is "alive" while it has a living unit that is neither a turret nor a war machine (`SIEGE_WEAPON`). When one side is empty, the winner is decided over units that are **not clones, not summoned and not commanders**; if neither side has such units, the result is a draw ([CBattleInfoCallback.cpp L2408-2440](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/CBattleInfoCallback.cpp#L2408-L2440)). E03 already has BAT-END-001/002.
  - `wiki` (Combat): "If the enemy has no units which survive after the battle (e.g. only summoned elementals), their hero is also considered as retreated." This is relevant only in E04b, see D-11.
- **R4.1b Counting casualties.**
  - `vcmi`: at `setBattleResult`, for every stack that is **not summoned and not a turret**, `killed = initialCount − currentCount + resurrected` (floored at 0) is added to `casualties[side][creatureType]` ([BattleResultProcessor.cpp L660-689](https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleResultProcessor.cpp#L660-L689); [CUnitState.cpp L549-554](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/CUnitState.cpp#L549-L554)).
  - **War machines are included** in these casualties.
- **R4.1c Applying casualties to armies.**
  - `vcmi` `CasualtiesAfterBattle` ([BattleResultProcessor.cpp L41-184](https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleResultProcessor.cpp#L41-L184)):
    - temporarily resurrected units are first taken away again (`takeResurrected`, E04b);
    - each army stack's count becomes its alive count, and a dead stack is erased;
    - **a damaged top creature is forgotten**: only counts persist, so the stack is at full health afterwards;
    - permanently summoned units are added to the army (E04b);
    - destroyed war machines are removed from the hero (§4.4).
  - The winner's own dead stay dead. The First Aid Tent never resurrects (§5.2).

### 4.2 Experience for the winner
- **R4.2a Formula (vcmi).**
  - Step 1: `XP = Σ_{loser casualties} baseHP(creature type) × killed`. The base HP comes from the creature type, without artifact HP bonuses ([BattleResultProcessor.cpp L212-225](https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleResultProcessor.cpp#L212-L225)).
  - Step 2: `+500` if the result is `NORMAL` and the **loser had a hero**. VCMI adds the 500 to both sides' tallies but applies only the winner's (L236-242).
  - Step 3: `+500` to the attacker if it wins a battle at a defended town (E04c, L244-247).
  - Step 4: multiply by `HERO_EXPERIENCE_GAIN_PERCENT/100` and truncate (`calculateXp`, [lib/mapObjects/CGHeroInstance.cpp L782-785](https://github.com/vcmi/vcmi/blob/1.7.5/lib/mapObjects/CGHeroInstance.cpp#L782-L785)). The base is 100 ([gameConfig.json L837-842](https://github.com/vcmi/vcmi/blob/1.7.5/config/gameConfig.json#L837-L842)); **Learning adds +5 / 10 / 15 % of base** ([skills.json L665-693](https://github.com/vcmi/vcmi/blob/1.7.5/config/skills.json#L665-L693); `original-data` SSTRAITS Learning 5% / 10% / 15%). The +500 is multiplied too.
  - Step 5: a draw gives no XP to anybody (L214-218).
  - Step 6: only the winner's hero, if any, receives XP, capped by the level limit with message `GENRLTXT.TXT` l.3 "%s can gain no more experience in this scenario." ([server/CGameHandler.cpp L349-381](https://github.com/vcmi/vcmi/blob/1.7.5/server/CGameHandler.cpp#L349-L381)).
  - `original-data` `GENRLTXT.TXT` l.361-362: "A glorious victory! … For valor in combat, %s receives %d experience".
- **R4.2b Formula (wiki, Experience, verbatim).** "From winning a combat, the hero receives one experience point for each HP of the enemy creatures he managed to vanquish in the battle (**clones, war machines and bonus health from artifacts do not count**)." "Resurrected creatures are only counted once, except Demons raised by Pit Lords." "If an enemy hero is defeated or an enemy town is captured during a siege, the hero receives 500 extra experience points." "Heroes with Learning gain extra experience." HotA fixed the experience overflow.
  - → **D-3**: VCMI counts killed war machines (Ballista 250 HP, Tent 75, Cart 100), the wiki does not.
- **R4.2c Neutral or garrison defenders.** No hero means no +500; XP comes only from killed HP. A hero beaten by neutrals or by a garrison without a hero gives XP to no one, because the winning side has no hero.

### 4.3 Artifacts of the loser
- **R4.3a Capture rule (vcmi).** Only if `result == NORMAL` ∧ the winner has a hero ∧ the winner has non-summoned units left ([BattleResultProcessor.cpp L523-586](https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleResultProcessor.cpp#L523-L586)).
  - What moves to the winner: every artifact in the 14 common worn slots (head, shoulders, neck, both hands, torso, both rings, feet, misc 1-5; [lib/entities/artifact/ArtifactUtils.cpp L91-112](https://github.com/vcmi/vcmi/blob/1.7.5/lib/entities/artifact/ArtifactUtils.cpp#L91-L112)), unless locked ([L160-165](https://github.com/vcmi/vcmi/blob/1.7.5/lib/entities/artifact/ArtifactUtils.cpp#L160-L165)), plus every backpack artifact **except the Grail**.
  - Each one goes to the first fitting slot, else the backpack (unlimited in VCMI's default, `backpackSize: -1`).
  - **Not transferred: war-machine slots and the spell book.**
  - If no winning hero exists (neutral or garrison wins), nothing is transferred and the items vanish with the removed hero.
  - `original-data` `GENRLTXT.TXT` l.35 "You have captured an enemy artifact!"
  - `wiki` (Spell Book): "A spell book cannot be traded to another hero, nor can it be lost after a defeat." `wiki` (Combat): retreating is useful "To preserve artifacts which will otherwise fall into an enemy hero's hands." No wiki line on war machines; see OQ-5.
- **R4.3b The loser hero (NORMAL).** It is removed from the map (L594-599). Re-hire rules are the orchestrator's.

### 4.4 War machines after the battle
- **R4.4 Destroyed machines are lost.**
  - `vcmi`: for **both sides and every result**, a war-machine stack with count 0 removes its artifact from the hero, **except the Catapult**, which always survives ([BattleResultProcessor.cpp L72-90](https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleResultProcessor.cpp#L72-L90)).
  - The loser's surviving war machines vanish with the loser after a NORMAL defeat (not captured, §4.3). A retreating or surrendering hero keeps its surviving war machines.
  - `wiki` (War Machine): "War Machines (except Catapults) can be traded between heroes, but cannot be dismissed or placed in the hero's backpack." `original-data` `GENRLTXT.TXT` l.181 "Alas, your backpack cannot hold something as large as a %s."

### 4.5 Necromancy (outline; applied by the orchestrator)
- **R4.5a VCMI algorithm** ([CGHeroInstance.cpp L978-1072](https://github.com/vcmi/vcmi/blob/1.7.5/lib/mapObjects/CGHeroInstance.cpp#L978-L1072); applied in [BattleResultProcessor.cpp L507-512](https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleResultProcessor.cpp#L507-L512)).
  - **When**: the winner has a hero with units left and the battle is not a draw, under any result type (it also works after the enemy flees or surrenders).
  - **Requirements**: an `IMPROVED_NECROMANCY` source (the skill, or the Cloak of the Undead King) and a raise percentage.
  - **Percent**: `pct = clamp(Σ UNDEAD_RAISE_PERCENTAGE, 0, 100)` (VCMI sums the bonus values):
    - skill 10 / 20 / 30 ([skills.json L367-403](https://github.com/vcmi/vcmi/blob/1.7.5/config/skills.json#L367-L403));
    - Amulet of the Undertaker +5, Vampire's Cowl +10, Dead Man's Boots +15 ([artifacts.json L833-866](https://github.com/vcmi/vcmi/blob/1.7.5/config/artifacts.json#L833-L866));
    - town Necromancy Amplifiers (player-wide).
  - **Raised type**: the best of the creatures allowed by the necromancy power. The skill gives power 1 / 2 / 3. The Cloak unlocks Walking Dead (power ≥ 1), Wight (≥ 2) and Lich (≥ 3) ([artifacts.json L1944-1965](https://github.com/vcmi/vcmi/blob/1.7.5/config/artifacts.json#L1944-L1965)).
  - **No free slot**: if there is no free slot or stack of that type, an upgraded stack already in the army is used, at **2/3 rate**.
  - **Count**: `raised = max(1, floor( Σ_{loser casualty types} min(HP_killed / HP_raised, 1) × killed × pct/100 × (2/3 if substituted) ))`.
  - **Inputs**: the loser's casualties **including war machines**, excluding summoned and clones.
- **R4.5b Wiki (Necromancy, SoD branch).**
  - Percent: 10% / 20% / 30% (HotA 5 / 10 / 15). Amplifier +10% in SoD. Artifacts +5 / 10 / 15, additive. Cap 100%.
  - "All killed enemy creatures can be raised as skeletons. This includes sacrificed creatures, cloned creatures, summoned elementals and even war machines."
  - HP efficiency is 1 hp = 16.6% … ≥6 hp = 100%, the same as `min(HP/6, 1)` for Skeletons.
  - "These rules in SoD are applied to **every killed stack individually** (in HotA … by the overall number of killed creatures), and always rounded towards 1."
  - Skeleton Warriors are used "only 66.6% compared to Skeletons"; with no slot and no skeletons, nothing is raised.
  - **Bug (SoD):** "AI heroes always raise exactly 1 Skeleton (or Lich) even if they should have raised more creatures (fixed in HotA)". This is a quirk candidate.
  - [Necromancy](https://heroes.thelazy.net/index.php/Necromancy)
  - `original-data`: SSTRAITS Necromancy "allows 10% / 20% / 30% of the creatures killed in combat to be brought back from the dead as Skeletons". ARTRAITS l.393-398 (Cloak): "30% … If hero already has the Necromancy skill then the percentages are added … Basic: Zombies, Advanced: Wights, Expert: Liches". GENRLTXT l.172-173 give the messages.
  - → **D-7** (per-stack vs aggregated rounding; minimum 1); **D-9** (summoned and clones, E04b).

### 4.6 Eagle Eye (outline; E04b data, applied by the orchestrator)
- `vcmi`: after the battle, if the winner hero has `LEARN_BATTLE_SPELL_LEVEL_LIMIT`, then for each entry of the **loser's hero-cast history** (spells the loser's hero cast, [GameStatePackVisitor.cpp L1378-1382](https://github.com/vcmi/vcmi/blob/1.7.5/lib/gameState/GameStatePackVisitor.cpp#L1378-L1382)) the spell is learned when all of these hold ([BattleResultProcessor.cpp L466-484](https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleResultProcessor.cpp#L466-L484); [skills.json L331-366](https://github.com/vcmi/vcmi/blob/1.7.5/config/skills.json#L331-L366)):
  - its level is ≤ 2 / 3 / 4;
  - it is not already known;
  - a roll `< 40 / 50 / 60`% succeeds.
  - The winner must have units left. There is **no Wisdom check**.
- `original-data` SSTRAITS: "40% chance to learn any 1st or 2nd level spell used in combat" / 50% up to 3rd / 60% up to 4th. GENRLTXT l.268 gives the message.
- `wiki` (Eagle Eye, SoD branch): the same numbers; "For a spell to be learned, the hero must have the appropriate level of Wisdom in relation to the spell." The HotA rework is different. Specialty: +5% per level (`original-data` HeroSpec.txt).
  - → **D-10** (E04b): Wisdom check; whether creature casts count.

### 4.7 Defender is a neutral stack or a garrison
- There is no hero, so: no tactics block, no retreat or surrender for that side, no surrender *to* it (R3a), no +500 XP, no artifact capture by it, and no luck under Q-0003 (§6).
- Creature banks have a special layout: no tactics, no obstacles, no war machines on the field (R1.1d).
  - `wiki` (Creature Bank): "war machines are not present on the battlefield (though they still provide passive bonuses)".
  - `wiki` (War Machine): the Ammo Cart "still provides unlimited ammo" there. `vcmi`: the Ammo Cart artifact counts even when not on the field ([lib/CStack.cpp L308-327](https://github.com/vcmi/vcmi/blob/1.7.5/lib/CStack.cpp#L308-L327)).
- A town garrison without a fort fights on a normal field; capturing the town adds +500 XP in VCMI (§4.2). With a fort, the battle is a siege (E04c).

---

## 5. Hero-run war machines

### 5.1 Where they come from
- **R5.1 War machines are artifacts.**
  - `original-data` `ARTRAITS.TXT` (slot columns) and `ArtSlots.txt`: **War Machine 1 = Ballista** (2500 g), **WM2 = Ammo Cart** (1000 g), **WM3 = First Aid Tent** (750 g), **WM4 = Catapult** (1000 g).
  - `vcmi` `setupBattle` creates one stack (count 1) per occupied slot at fixed hexes [52, 18, 154, 120] (attacker) and [66, 32, 168, 134] (defender) ([BattleInfo.cpp L296-331](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/BattleInfo.cpp#L296-L331); [gameConfig.json L630-631](https://github.com/vcmi/vcmi/blob/1.7.5/config/gameConfig.json#L630-L631)).
  - The **Catapult only appears for the attacker against a walled town**.
  - VCMI gives **no Ballista to a hero defending a town** ("bug #551"; E04c question).
  - Without a hero there are no war machines. Our `BattleSetup.war_machines` stays as the derived field (`h3-heroes` derives it from the WM slots).

### 5.2 Control rules
- **R5.2a With the skill, the player chooses.**
  - `vcmi`: Artillery, First Aid and Ballistics give `MANUAL_CONTROL` 100 for the Ballista, the First Aid Tent and the Catapult respectively. Artillery also controls the arrow towers (E04c) ([skills.json L611-664](https://github.com/vcmi/vcmi/blob/1.7.5/config/skills.json#L611-L664), [L842-877](https://github.com/vcmi/vcmi/blob/1.7.5/config/skills.json#L842-L877), [L297-330](https://github.com/vcmi/vcmi/blob/1.7.5/config/skills.json#L297-L330)).
  - `vcmi`: the automatic action runs only `if (!owner || !roll(MANUAL_CONTROL))` ([BattleFlowProcessor.cpp L457-463, L678-737](https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleFlowProcessor.cpp#L457-L463)). With the skill the machine becomes an ordinary player-controlled stack:
    - Ballista: SHOOT;
    - Tent: HEAL ([CBattleInfoCallback.cpp L278-330](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/CBattleInfoCallback.cpp#L278-L330));
    - the generic Wait and Defend buttons are not blocked for them ([client/battle/BattleWindow.cpp L772-781](https://github.com/vcmi/vcmi/blob/1.7.5/client/battle/BattleWindow.cpp#L772-L781)).
  - Without the skill, E03's automatic behaviour (BAT-WAR-002/003) stays.
  - `wiki` (Combat, siege paragraph): "If the player can control them, then they can be set to wait (starts at beginning of waiting phase), while defense provides no bonuses." This line is about towers and catapult; whether it also covers the Ballista and Tent is OQ-4.
- **R5.2b Turn order.** War machines have speed 0 and act last in the round, Ballista before Tent (`wiki` War Machine: "The Ballista acts at the end of every combat round, followed only by the First Aid Tent"). E03 has this.

### 5.3 Ballista with Artillery
- **R5.3a Shots and double damage.**
  - `vcmi`: `BONUS_DAMAGE_CHANCE` **50 / 75 / 100 %** and `HERO_GRANTS_ATTACKS` **0 / 1 / 1** extra shots, so **1 / 2 / 2 shots per turn** ([skills.json L611-664](https://github.com/vcmi/vcmi/blob/1.7.5/config/skills.json#L611-L664); extra shots in [BattleActionProcessor.cpp L440-457](https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleActionProcessor.cpp#L440-L457)).
  - `vcmi`: the chance is rolled **per shot** ([L1075-1081](https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleActionProcessor.cpp#L1075-L1081)). On success, `BONUS_DAMAGE_PERCENTAGE` 100 adds **+100% to the additive attack-factor sum**, together with luck, Archery and so on ([lib/battle/DamageCalculator.cpp L261-269, L480-495](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/DamageCalculator.cpp#L261-L269)).
  - `wiki` (Artillery, SoD branch): Basic "gives control of the ballista and defense towers to the hero. The ballista has a 50% chance to inflict double damage."; Advanced "… shoots twice and has a 75% chance …"; Expert "… shoots twice and has a 100% chance …". The control of arrow towers was "added in Armageddon's Blade". [Artillery](https://heroes.thelazy.net/index.php/Artillery)
  - `original-data` SSTRAITS Artillery: Basic "**allowing two shots** with a 50% chance to inflict double damage", Advanced 75%, Expert "two shots which inflict double damage".
  - → **D-4** (Basic: 1 shot in VCMI and wiki, 2 shots in the original text).
- **R5.3b Ballista damage and stats.**
  - Base attack 10, defense 10, damage 2–3, 250 HP, 24 shots (CRTRAITS, already harvested).
  - `vcmi`: the machine stack attaches directly to the hero's army node ([lib/CStack.cpp L63-82](https://github.com/vcmi/vcmi/blob/1.7.5/lib/CStack.cpp#L63-L82)), so the **hero's attack and defense (and every other army-wide bonus) apply to war machines** like to creatures.
  - `vcmi`: additionally, the damage range is **multiplied by (hero attack + 1)**, where hero attack counts only bonuses of source ARTIFACT or HERO_BASE_SKILL ([DamageCalculator.cpp L61-77](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/DamageCalculator.cpp#L61-L77)).
  - `wiki` (War Machine): "its damage range scales with the controlling hero's attack value according to the formula: (2–3)*(hero's attack + 1)" (HotA: +5). "Both the Artillery and Archery secondary skills increase the damage of the Ballista." "Ballistas … are affected by Luck." "War Machines are immobile and unaffected by Morale." It "can shoot even if enemies are adjacent to it" (`vcmi`: SIEGE_WEAPON implies free shooting, [lib/bonuses/BonusCache.cpp L201](https://github.com/vcmi/vcmi/blob/1.7.5/lib/bonuses/BonusCache.cpp#L201)).
  - `original-data` `HeroSpec.txt` (Ballista specialists): "Increases the Attack and Defense skill of any Ballista for each level attained after 4th level." That is T10.
- **R5.3c Ballista ammunition.** `wiki` (War Machine): "**Contrary to what its in-game statistics indicate, a Ballista has unlimited shots** (fixed in HotA)." `vcmi` 24 shots ([config/creatures/special.json L72-79](https://github.com/vcmi/vcmi/blob/1.7.5/config/creatures/special.json#L72-L79)); E03 BAT-WAR-002 uses 24.
  - → **D-8**. Its impact grows with Artillery's 2 shots per turn.
- **R5.3d Target choice.** With Artillery the player picks any enemy target the Ballista can shoot. VCMI has no extra restriction (`battleCanShoot`); turrets (E04c) are not valid targets.

### 5.4 First Aid Tent with First Aid
- **R5.4a Heal amount.**
  - `vcmi`: the tent casts the ability spell `firstAid` ([config/spells/vcmiAbility.json L52-95](https://github.com/vcmi/vcmi/blob/1.7.5/config/spells/vcmiAbility.json#L52-L95)): `heal` level "heal" (**no resurrection**), permanent.
  - `vcmi`: the effect value is `count × SPECIFIC_SPELL_POWER(firstAid)` ([CUnitState.cpp L452-455](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/CUnitState.cpp#L452-L455)). The First Aid skill grants **50 / 75 / 100**, so the tent heals a **fixed 50 / 75 / 100** HP of the top creature. Without the skill it heals power 10 (E03 Q26).
  - `vcmi`: target choice without the skill is random among allied, hurt, non-war-machine stacks ([BattleFlowProcessor.cpp L705-737](https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleFlowProcessor.cpp#L705-L737); [lib/CStack.cpp L277-280](https://github.com/vcmi/vcmi/blob/1.7.5/lib/CStack.cpp#L277-L280)). With the skill the player chooses ([BattleActionProcessor.cpp L548-583](https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleActionProcessor.cpp#L548-L583)).
  - `original-data` SSTRAITS First Aid: "gives control of the first aid tent to the hero, healing **50** / **75** / **100** points of damage to the first unit of the selected stack." `HeroSpec.txt`: First Aid specialists get "a 5% per level bonus to First Aid skill".
  - `wiki` (First Aid, SoD branch of `{{swh}}`): "healing **1-50** / **1-75** / **1-100** points of damage to the first unit of the selected stack".
    - HotA branch: "40 – 50 / 60 – 75 / 80 – 100" plus +5 / 10 / 15 % creature health (HotA 1.7.2 rework).
    - `wiki` (First Aid Tent): SoD base heal 1–25 (HotA 20–25); "Cannot resurrect"; heals "the top creature of an allied stack".
  - → **D-12** (fixed vs random 1–N), the companion of Q26.
- **R5.4b Ammo Cart.** No skill. `wiki`: "provides unlimited shots for creatures and War Machines with ranged attacks until the Ammo Cart is destroyed". 100 HP, defense 5. E03 has BAT-WAR-004.
- **R5.4c Catapult (outline, E04c).**
  - `original-data` `ballist.txt`, chance to hit Keep / Tower / Gate / Wall, shots, and damage 0 / 1 / 2:

    | Ballistics | Keep | Tower | Gate | Wall | Shots | Damage 0 / 1 / 2 |
    |---|---|---|---|---|---|---|
    | None | 5 % | 10 % | 25 % | 50 % | 1 | 10 / 60 / 30 % |
    | Basic | 7 % | 15 % | 30 % | 60 % | 1 | 0 / 50 / 50 % |
    | Advanced | 7 % | 15 % | 30 % | 60 % | 2 | 0 / 50 / 50 % |
    | Expert | 10 % | 20 % | 40 % | 75 % | 2 | 0 / 0 / 100 % |

  - `vcmi` mirrors this table ([vcmiAbility.json L107-](https://github.com/vcmi/vcmi/blob/1.7.5/config/spells/vcmiAbility.json#L107)). The Catapult is never lost (§4.4) and gets no shots without walls.
  - `original-data` `SSTRAITS.TXT` Ballistics: control, "aimed shots", two shots from Advanced, maximum damage at Expert. `GENRLTXT.TXT` l.368 "The Catapult must be equipped."

---

## 6. The hero's own presence in battle

- **R6a Attack and defense.**
  - `vcmi`: hero primary-skill bonuses reach every stack through the army node ([lib/CStack.cpp L63-82](https://github.com/vcmi/vcmi/blob/1.7.5/lib/CStack.cpp#L63-L82)). That covers army stacks (through their `CStackInstance`) and war machines (directly).
  - `wiki` (Primary skill): "A hero's attack skill value is added to each of the stack's attack rating …"; "Heroes' defense skill is added to the defense rating of each of the creatures in their army"; "The upper limit is 99. At 129 the skill value is reset to 0" (overflow is a quirk candidate).
  - Ballista extras: §5.3b.
- **R6b The hero is not a unit.** It cannot be attacked or targeted, has no hex and does not count for the end of the battle (`vcmi`: the hero is only `SideInBattle.hero`; units come from `stacks`, BattleInfo.cpp L160-440). No source says otherwise.
- **R6c Morale and luck.**
  - `vcmi`: a stack's morale or luck is the sum of every MORALE or LUCK bonus reaching it: creature, army composition, hero skills, artifacts, specialty, one-battle map bonuses (e.g. `BonusDuration::ONE_BATTLE` from map objects and events), battle-wide auras ([lib/BasicTypes.cpp L72-127](https://github.com/vcmi/vcmi/blob/1.7.5/lib/BasicTypes.cpp#L72-L127)).
  - Morale is clamped to **[−3, +3]** and luck to **[0, +3]**: `goodMoraleChance` [1, 2, 3], `badMoraleChance` [2, 4, 6], `goodLuckChance` [1, 2, 3], `badLuckChance` [] ([gameConfig.json L587-602](https://github.com/vcmi/vcmi/blob/1.7.5/config/gameConfig.json#L587-L602)).
  - Morale is ignored for NON_LIVING, MECHANICAL, UNDEAD, SIEGE_WEAPON and NO_MORALE units (L72-79).
  - `original-data` SSTRAITS: Leadership "increases your hero's troops' morale by 1 / 2 / 3"; Luck "increases your hero's luck by 1 / 2 / 3".
  - E03 already has the clamps (BAT-MOR-001, BAT-LCK-001: luck 0..3, no bad luck before HotA).
  - **Q-0003** "no luck without a hero" (`quirk.battle.no_luck_without_hero`, wiki Luck) must switch from `led_by_hero` to the real hero presence.
  - War machines: no morale; the Ballista gets luck (`wiki` War Machine, `vcmi` above).
- **R6d One cast per round (E04b).** `HERO_SPELL_CASTS_PER_COMBAT_TURN` = 1, reset in `nextRound` ([gameConfig.json L822-827](https://github.com/vcmi/vcmi/blob/1.7.5/config/gameConfig.json#L822-L827); [CBattleInfoCallback.cpp L140-151](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/CBattleInfoCallback.cpp#L140-L151); [BattleInfo.cpp L651-656](https://github.com/vcmi/vcmi/blob/1.7.5/lib/battle/BattleInfo.cpp#L651-L656)). `wiki` (Combat): "the hero can cast a spell (once per round)".

---

## 7. Outline for the seam: E04b (magic) and E04c (sieges)

**Spell effect kinds.** `vcmi` has the effect classes in `lib/spells/effects/`: Damage, Timed (buffs and debuffs), Heal (heal / resurrect, temporary or permanent), Summon, DemonSummon, Clone, Dispel, Obstacle, RemoveObstacle, Sacrifice, Teleport, Catapult, Moat, UnitEffect. The spells by group:
- **Damage** ([config/spells/offensive.json](https://github.com/vcmi/vcmi/blob/1.7.5/config/spells/offensive.json)):
  - single target: Magic Arrow, Ice Bolt, Lightning Bolt, Implosion, Titan Bolt;
  - chain: Chain Lightning;
  - area: Frost Ring, Fireball, Inferno, Meteor Shower;
  - every stack: Death Ripple, Destroy Undead, Armageddon.
- **Timed** ([config/spells/timed.json](https://github.com/vcmi/vcmi/blob/1.7.5/config/spells/timed.json)):
  - Shield, Air Shield, Fire Shield;
  - Protection from Air / Fire / Water / Earth;
  - Anti-Magic, Magic Mirror;
  - Bless, Curse, Bloodlust, Precision, Weakness, Stone Skin, Disrupting Ray, Prayer;
  - Mirth, Sorrow, Fortune, Misfortune;
  - Haste, Slow, Slayer, Frenzy, Counterstrike;
  - mind spells Berserk, Hypnotize, Forgetfulness, Blind.
- **Other** ([config/spells/other.json](https://github.com/vcmi/vcmi/blob/1.7.5/config/spells/other.json)):
  - obstacles: Quicksand, Land Mine, Force Field, Fire Wall;
  - Earthquake (E04c);
  - Dispel, Cure;
  - Resurrection, Animate Dead, Sacrifice;
  - Teleport, Remove Obstacle, Clone;
  - the four Elementals (summons).
- **Hidden obstacles:** Quicksand `"hidden": true` ([other.json L15](https://github.com/vcmi/vcmi/blob/1.7.5/config/spells/other.json#L15)), Land Mine `true` ([L122](https://github.com/vcmi/vcmi/blob/1.7.5/config/spells/other.json#L122)), and the Tower town's land-mine moat `true` ([moats.json L210](https://github.com/vcmi/vcmi/blob/1.7.5/config/spells/moats.json#L210)).
- **Visible obstacles:** Force Field `false` ([L180](https://github.com/vcmi/vcmi/blob/1.7.5/config/spells/other.json#L180)), Fire Wall `false` ([L300](https://github.com/vcmi/vcmi/blob/1.7.5/config/spells/other.json#L300)).
- **Mass versions** come from the school's Expert level.

**Mana and casting rules.**
- The hero starts the battle with its *current* spell points. Points spent are kept off after the battle; any mana gained during it is capped at the initial value ([GameStatePackVisitor.cpp L1480-1486](https://github.com/vcmi/vcmi/blob/1.7.5/lib/gameState/GameStatePackVisitor.cpp#L1480-L1486)).
- One cast per round (R6d). No casting during tactics (R1.2e).
- No casting without a spell book or under BLOCK_ALL_MAGIC (CBattleInfoCallback.cpp L144-149).
- Spell-level limits from artifacts and terrain: `BLOCK_MAGIC_ABOVE/BELOW` (L2371-2406). Cursed ground has the message `GENRLTXT.TXT` l.~630 "cannot cast spells over level one here".
- Opening battle spells from artifacts ([BattleFlowProcessor.cpp L198-228](https://github.com/vcmi/vcmi/blob/1.7.5/server/battles/BattleFlowProcessor.cpp#L198-L228)).
- The Eagle Eye history (§4.6).
- Temporary resurrection must be tracked for casualties (§4.1c).

**What `BattleSetup` must carry for E04b** (reserve in ADR-0013):
- the spell book;
- current mana;
- spell power, knowledge;
- school levels (Fire, Air, Water, Earth Magic);
- Sorcery %;
- Resistance %;
- spell-damage artifacts and specialties;
- magic-blocking flags (hero artifacts, garrison anti-magic, battlefield type);
- opening-spell effects.

**Siege inputs (E04c).**
- **Fort level**, from VCMI ([config/buildingsLibrary.json L22-45](https://github.com/vcmi/vcmi/blob/1.7.5/config/buildingsLibrary.json#L22-L45)):
  - Fort: walls HP 2;
  - Citadel: keep HP 2 and a moat;
  - Castle: walls HP 3 and upper and lower towers HP 2 each.
- **Moat** spell by faction ([config/spells/moats.json](https://github.com/vcmi/vcmi/blob/1.7.5/config/spells/moats.json)). `wiki` Moat: 70 damage for Castle, Rampart, Stronghold, Necropolis and Conflux; 90 for Dungeon, Inferno and Fortress (Fortress 2 hexes wide); Tower has land mines. Units "are forced to stop on the moat" and have "defense reduced by 3".
- **Tower damage** depends on the number of town buildings `n`: towers `(6+n)..2(6+n)`, keep `(10+2n)..2(10+2n)` ([lib/mapObjects/CGTownInstance.cpp L407-436](https://github.com/vcmi/vcmi/blob/1.7.5/lib/mapObjects/CGTownInstance.cpp#L407-L436)).
- **Gate state**.
- **Defender's Escape Tunnel** (retreat, R2a).
- The attacker's **Catapult** and **Ballistics** level (ballist.txt); the defender's **Artillery** controls the towers.
- No defender Ballista in a town (VCMI).
- Town-specific defender bonuses (faction buildings).
- +500 XP for capturing the town (§4.2).
- No random obstacles in a fort town (BattleInfo.cpp L200).

---

## 8. Seam inputs

### 8.1 `BattleSetup` / `BattleHero` (E04a), all derived before the seam

| Item | Why | Source |
|---|---|---|
| `heroes: [Option<BattleHero>; 2]` replacing `led_by_hero` | Many rules depend on hero presence: Q-0003 luck, retreat, surrender, +500 XP | R2a, R3a, §6 |
| `BattleHero.id` (HeroId handle) | events, outcome | — |
| effective attack and defense (after artifacts and specialty; cap rule decided in T4/T5) | applied to all own stacks and war machines; Ballista damage factor (attack + 1) | R6a, R5.3b |
| morale and luck modifier sums of hero origin (Leadership, Luck skill, artifacts, specialty, one-battle map bonuses) | side modifiers; clamps unchanged | R6c |
| tactics: reposition value, block value, `tactics_formation_enabled` | phase owner and distance; the block counts even when disabled (VCMI) | R1.1 |
| Artillery level (0..3) and the resulting shots per turn and double-damage chance | Ballista control | R5.3 |
| First Aid level (0..3) and heal amount (including the +5%/level specialty) | Tent control | R5.4 |
| Ballistics level (E04c) | Catapult | R5.4c |
| Archery, Offense, Armorer % (T6) | damage factors | SSTRAITS |
| `no_retreat` (Shackles of War worn) | retreat and surrender; the SoD scope is D-6 | R2a |
| war-machine specialty effects (Ballista attack/defense per level) | T10 | HeroSpec.txt |
| reserved for E04b: spell book, mana, spell power, knowledge, school levels, Sorcery, Resistance, magic-block flags | — | §7 |
| `war_machines` (existing; derived from WM1..WM4; Catapult only for an attacker at a walled town) | — | R5.1 |
| battle kind or layout: open field / creature bank (no tactics, obstacles or war machines on the field, passive Ammo Cart) / siege (E04c) / ship (E04c) | tactics and obstacle switches | R1.1d, §4.7 |
| retreat context per side: besieged defender, Escape Tunnel present | R2a | R2a |
| surrender context: enemy hero present, besieged defender, **gold available and discount %**, or a precomputed "can afford" check | The battle must know whether Surrender is legal (action mask). The cost changes with casualties, so a static flag is not enough; see design note D-1/D-2 | R3 |

**Commands to add.**
- Tactics: `TacticsMove { stack, hex }` and `EndTactics`.
- `Retreat` and `Surrender` (in tactics or on the side's turn).
- `SurrenderAnswer { accept }` if D-5 confirms the enemy may refuse.
- War machines: the Ballista uses the normal `Shoot`; the Tent gets `Heal { target }`; whether Wait and Defend are allowed is OQ-4.

### 8.2 `BattleOutcome` must report

| Field | Used for |
|---|---|
| `result`: `Victory(side)` / `Draw` / **`Retreated(side)`** / **`Surrendered(side)`**; the winner is the other side in both new variants | hero fate, +500 rule, artifact capture (only `Victory`) |
| per stack: origin (army slot / war machine kind / summoned / clone, E04b), creature type, initial count, final count, killed, temporarily resurrected (E04b) | casualties per side and type; XP; Necromancy (per stack, for D-7); army update |
| `survivors` (existing) | army update, surrender cost |
| `war_machines` destroyed per side (existing) | remove the artifacts (not the Catapult) |
| per side: had a hero; the winner has non-summoned units left | XP +500, artifact capture, Necromancy and Eagle Eye preconditions, "winner without troops retreats" |
| surrender: gold paid (if the battle computes the cost) | treasury |
| reserved for E04b: spells cast by each hero (ordered, with level), mana spent | Eagle Eye, mana |
| reserved for E04c: town captured, wall and tower state | +500, town |
| rounds played | statistics, replays |

The orchestrator then applies: casualties; XP (§4.2, with Learning and the level cap); Necromancy (§4.5, needs player-wide amplifier counts); Eagle Eye (§4.6); artifact capture (§4.3); war-machine loss (§4.4); hero fates (tavern roles, §2/§3); surrender gold.

---

## 9. Open questions (no conflict, but no rule found)

- **OQ-1** Does a hero with "Tactics formation" disabled still cancel the enemy's Tactics? VCMI: yes. No other source. *SoD experiment:* two heroes, both with Basic Tactics; disable the toggle on one; does the other get a tactics phase?
- **OQ-2** Tactics placement:
  - does a walker need a path (VCMI), or can it be placed freely?
  - can that path leave the zone?
  - can war machines be moved (VCMI: no)?
  - is the zone edge zig-zag (raw x) or straight?
  - *SoD experiment:* screenshot the highlighted zone at Basic Tactics; try to move the Ballista.
- **OQ-3** Can a stack be moved more than once and are stacks cycled in a fixed order? This is a UI matter; VCMI allows any order.
- **OQ-4** May a hero-controlled Ballista or Tent Wait or Defend? VCMI: yes (generic buttons).
- **OQ-5** War machines of a hero defeated in a normal battle: lost (VCMI) or captured? No wiki line. *SoD experiment:* defeat a hero that has a Tent; check the winner's slots.
- **OQ-6** Is the +500 XP for defeating a hero multiplied by Learning? VCMI: yes. Wiki: silent.

## 10. Disputes

| # | What | Source A | Source B | Impact | Suggested SoD experiment |
|---|---|---|---|---|---|
| D-1 | Surrender cost factor | `vcmi`: 100 % of the gold recruitment cost of alive army stacks (CBattleInfoCallback.cpp L2347-2369) | `wiki` Combat#Surrender: **half** the recruitment cost | High: the price ×2; the action mask needs the cost | Hero with 10 Pikemen (60 g each, 600 total), no Diplomacy, against an enemy hero; press Surrender and read the offered price: 300 → wiki, 600 → VCMI |
| D-2 | How surrender reductions combine | `vcmi`: additive, `(100 − Σ%)` (Diplomacy 20/40/60 + 10 per artifact) | wiki and original texts: silent on combining | Medium | Same army with Basic Diplomacy + Statesman's Medal: raw 600 → wiki-half additive 210 / multiplicative 216; full additive 420 / multiplicative 432 |
| D-3 | XP for killed war machines | `vcmi`: counted (casualties include war machines, BattleResultProcessor.cpp L660-689, L212-225) | `wiki` Experience: "war machines … do not count" | Medium (Ballista 250 XP) | Enemy hero with 1 Peasant and a Ballista; kill both; the message "For valor in combat, %s receives %d experience" shows 501 (wiki) or 751 (VCMI); no Learning |
| D-4 | Shots of a Ballista with **Basic** Artillery | `vcmi` and `wiki`: 1 shot (extra attacks 0 at Basic) | `original-data` SSTRAITS: "allowing two shots" at Basic | Medium (Ballista DPS ×2) | Hero with Basic Artillery and a Ballista; count shots in the Ballista's first turn (combat log) |
| D-5 | Can the enemy refuse a surrender? | `vcmi`: no consent step | `wiki`: "The enemy may refuse … In HotA the enemy cannot refuse" | Medium; changes the seam (offer/answer) | Surrender against an AI hero several times with varied armies; note any refusal |
| D-6 | Shackles of War when the enemy has **no** hero | `vcmi`: the check only applies if the enemy has a hero, i.e. retreat vs neutrals is allowed (CBattleInfoEssentials.cpp L311) | `wiki`: "In HotA, the effect is active in hero-to-hero battles only", implying SoD blocks retreat always; `original-data` ARTRAITS: "neither you nor your opponent may retreat or surrender" | Low–medium | Hero wearing Shackles attacks a neutral stack; is Retreat available? |
| D-7 | Necromancy rounding | `vcmi`: one sum over creature types, `floor`, minimum 1 (CGHeroInstance.cpp L1057-1071) | `wiki`: SoD applies the rules "to every killed stack individually … and always rounded towards 1" | Low–medium (many small stacks) | Basic Necromancy (10 %); kill two separate enemy stacks of 9 Pikemen (10 HP each): VCMI 1, per-stack floor 0 (or 2 if each stack rounds up to 1); then one stack of 15: 1 |
| D-8 | Ballista ammunition | `vcmi` and E03 BAT-WAR-002: 24 shots (special.json L79) | `wiki` War Machine: "a Ballista has unlimited shots (fixed in HotA)" | Low; grows with Artillery (2 shots per turn, so empty after 12 rounds) | No Ammo Cart; after the Ballista fires, right-click it and check whether the shots counter falls |
| D-9 *(E04b)* | Necromancy on summoned units and clones | `vcmi`: excluded (`summoned` stacks are not casualties) | `wiki`: "includes … cloned creatures, summoned elementals" | Low | Later (E04b batch) |
| D-10 *(E04b)* | Eagle Eye preconditions | `vcmi`: no Wisdom check; only spells cast by the enemy *hero* | `wiki`: Wisdom required; `original-data`: "any … spell used in combat" | Low | Later (E04b batch) |
| D-11 *(E04b)* | Winner left with only summoned units | `vcmi`: second pass gives a **draw** (both heroes removed, then sent to the tavern) | `wiki` Combat: that hero "is also considered as retreated" (the other side presumably loses) | Low | Later (E04b batch) |
| D-12 | First Aid heal amount with the skill | `vcmi` and `original-data` text: fixed 50/75/100 | `wiki` (SoD branch): random 1–50 / 1–75 / 1–100 | Low–medium; pairs with Q26 (base 1–25 vs VCMI 10) | Hero with Basic First Aid; heal a top creature missing more than 50 HP a dozen times; record amounts (fixed 50 vs varying) |

**Quirk candidates noticed in passing**, for the T1 quirk survey:
- AI Necromancy always raises exactly 1 (wiki, SoD bug).
- Ballista unlimited shots (D-8).
- Primary skill overflow at 129 (wiki).
- Experience overflow (wiki).
- A defending hero in a town gets no Ballista (VCMI "bug #551", unverified).
- The "attack yourself" siege glitch (wiki Combat, E04c).

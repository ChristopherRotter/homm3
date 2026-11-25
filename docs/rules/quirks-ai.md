# Quirks ledger: ai lane

Known differences between the original's computer player (GOG Complete's `Heroes3.exe`, SoD's code; E31) and a clean design, in the rules and values of `h3_game::computer` (`CPU-` rules, `docs/rules/catalog/computer.toml`). Classes, the decision guide and profiles are in `docs/04-rules-and-sources.md` §4; the entry template in `docs/rules/quirks-ledger.md` (the battle lane's ledger). Ids are `AI-QUIRK-<n>` (`docs/08-work-process.md`, "Lanes").

**Where the candidates come from**: the survey `docs/rules/surveys/E31-computer-player.md` §5.5 classifies the 62 quirk candidates of the reading (the user at the checkpoint, AI-Q31). This ledger takes them as their ports land:

- **slip** (36): a value or choice of the computer's own that reads a wrong record, argument or test, or computes a slip, and changes only the computer's choices. All of them sit behind **AI-QUIRK-1** (`quirk.computer.original_ai_slips`, fix-optional: kept in `faithful`, valued as the rule in `strict-fixed`; the ADR draft's point 10, as BAT-QUIRK-25 for battle). The id is reserved: its entry is written with the first slip built (E31 increment 4), each slip confirmed by a blind re-read of its deciding addresses before it is built (Q30) and listed there with its test.
- **state** (6): the lane that owns the rule enters its own entry when it builds the rule (survey §5.3; the battle, adventure and town lanes' inboxes).
- **exploit-fix** (5): a crash or a corrupted state, never reproduced. The three in the computer's own code are below; the map loader's two (§5.5 #54, a 49th Seer's Hut divides by zero at the load; #60, a creature quest's counts read without a length test) are the adventure lane's reader's refusals, in its inbox (E31 T1b).
- **none** (13) and **fixed** (2: #36 and #55, the Seer's luck reward valued as luck in every profile): ported as read, or as the rule, without an entry.

## Entries

### AI-QUIRK-1 — reserved: the original's slips (E31 increment 4)

### AI-QUIRK-2 — The Angelic Alliance's alignments: a creature without a faction throws
- Class: exploit-fix
- Context / rules affected: computer, an army's values (`ai_stack_marginal_value` 0x42C830 at 0x42C8B6, `ai_stack_to_replace` 0x42C690 at 0x42C781); survey §2.3 (a), §5.5 #12; the `CPU-` rule of the army values when it is ported (E31 increment 2)
- Original behavior: with the Angelic Alliance worn by any of the player's heroes (the plan's flag), a stack's alignment is its faction's position in the nine-town set. A creature without a faction (a neutral, RoE's elementals 112 to 115) has faction −1, a position past the set's nine: the library's range check throws `std::out_of_range`. Reached by a neutral dwelling's value or an exchange with neutral creatures; whether anything catches the exception was not read (a crash if nothing does).
- Intended/expected behavior: a creature without a faction is the neutral alignment, with or without the Alliance, as `ai_army_plan_alignments` 0x42C060 counts it (the neutral and Conflux stay apart there).
- Impact: game-breaking (a crash) where reached
- Community status: unknown
- Sources: code reading (survey §2.3, "Quirk candidates" (a); `docs/exe/map.toml` 0x42C8B6)
- Flag: none (fixed in every profile)
- Defaults: faithful = fixed, strict-fixed = fixed
- Regression test: when the army values are ported (E31 increment 2): a hero of a player wearing the Alliance values a neutral stack's purchase and an exchange with one, both profiles, the neutral counted as its own alignment
- Decided by / date: the survey's second look (E31 T1b), the user at the checkpoint, 2026-11-24 (AI-Q31)

### AI-QUIRK-3 — The formation keeps a stack's count in 16 bits
- Class: exploit-fix
- Context / rules affected: computer, the formation of an army (`ai_army_arrange` 0x42D8E0: the count stored at 0x42D922, read back with its sign at 0x42DA39 and 0x42DAD9), called by the exchanges, the town visit's recruits, the hero prospects and the battle's split; survey §3.3 (b), §5.5 #27; the `CPU-` rule of the formation when it is ported (E31 increment 3)
- Original behavior: every stack is taken out into a list whose entries keep the count as a signed 16-bit value, then put back by `army_add_stack`: a stack above 32 767 comes back wrapped (40 000 → −25 536, kept negative; 65 536 → 0).
- Intended/expected behavior: the formation moves stacks between slots and keeps every count whole; in the port it is a sequence of the game's stack moves, which change no count.
- Impact: game-breaking (a negative or lost stack, a corrupted army)
- Community status: unknown
- Sources: code reading (survey §3.3, "Quirk candidates" (b); `docs/exe/map.toml` 0x42D8E0)
- Flag: none (fixed in every profile)
- Defaults: faithful = fixed, strict-fixed = fixed
- Regression test: when the formation is ported (E31 increment 3): an army with a stack of 40 000 and one of 65 536 arranged, both profiles, the counts unchanged
- Decided by / date: the survey's second look (E31 T1b), the user at the checkpoint, 2026-11-24 (AI-Q31)

### AI-QUIRK-4 — The battle's split divides by zero
- Class: exploit-fix
- Context / rules affected: computer, the army's split at a battle's start (`ai_battle_army_split` 0x42DB20, `ai_army_split_into_free_slots` 0x42DD70 at 0x42DDA1); survey §3.5 (b), §5.5 #38; the `CPU-` rule of the attacker's split when it is ported (E31 increment 4). The defender's split is a rule (§3.6 row 8, the battle lane's): its own entry when that lane builds it.
- Original behavior: each own stack's parts are its AI value × count divided by the threshold T (5 or 1 times the strongest enemy stack's value), taken as a signed 16-bit value: a T whose low half is 0 (a multiple of 65 536), or an empty enemy army (the strongest value 0), divides by zero. (That T wraps above 32 767 is a separate candidate, #37, a state one.)
- Intended/expected behavior: a threshold of 0 splits nothing.
- Impact: game-breaking (a crash)
- Community status: unknown
- Sources: code reading (survey §3.5, "Quirk candidates" (b); `docs/exe/map.toml` 0x42DDA1)
- Flag: none (fixed in every profile)
- Defaults: faithful = fixed, strict-fixed = fixed
- Regression test: when the attacker's split is ported (E31 increment 4): a split against an empty army and against a threshold of 65 536, both profiles, no stack split
- Decided by / date: the survey's second look (E31 T1b), the user at the checkpoint, 2026-11-24 (AI-Q31)

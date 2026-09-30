# Quirks ledger

Every known difference between the original game's behavior and a clean design. See `docs/04-rules-and-sources.md` for classes, the decision guide and profiles.

Classes: `exploit-fix` (always fixed, never toggleable) · `preserve` (accepted, flag on in `faithful`) · `fix-optional` (flag; default per profile).

## Entry template

```
### Q-NNNN — Short title
- Class: exploit-fix | preserve | fix-optional
- Context / rules affected: <context>, rule IDs
- Original behavior: precise description (conditions, numbers)
- Intended/expected behavior (if known):
- Impact: game-breaking | exploit | gameplay-altering | cosmetic
- Community status: e.g. "well known, accepted in tournaments" / "unknown"
- Sources: (same labelling as rule sources, with URLs)
- Flag: `quirk.<context>.<name>` (none for exploit-fix)
- Defaults: faithful = on/off, strict-fixed = on/off
- Regression test: test name (required for exploit-fix)
- Decided by / date:
```

## Entries

### Q-0001 — Spending another player's resources (exploit class)
- Class: exploit-fix
- Context / rules affected: town/economy, game (command validation); rule IDs to be assigned when the economy epic starts
- Original behavior: reported by the user: in the original game it is possible to buy things in your town using another player's resources. Exact mechanism and preconditions to be documented from sources when the economy epic begins.
- Intended behavior: a player can only spend from their own treasury.
- Impact: exploit
- Community status: known exploit
- Sources: user report (2026-09-30); details to be collected
- Flag: none (always fixed)
- Defaults: n/a
- Regression test: to be written with the first command that spends resources (must attempt the exploit through the command API and assert rejection, state unchanged)
- Decided by / date: user requirement, 2026-09-30

### Q-0002 — Information exposure beyond player knowledge (exploit class)
- Class: exploit-fix
- Context: game/projection (security)
- Original behavior: the original client holds full game state; information a player should not see can be read by a modified client (user-reported concern).
- Intended behavior: clients receive only projected observations/events.
- Impact: exploit
- Sources: user requirement (2026-09-30)
- Flag: none
- Regression test: observation-hygiene property tests per context
- Decided by / date: user requirement, 2026-09-30

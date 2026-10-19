# ADR-0020 amendment (draft): text through a string catalogue and language packs

- Status: Draft, decided by the user on 2026-10-13 (E17 M1: text through a catalogue) and
  2026-10-19 (E17 M3 T4b-1: nested JSON; CLT-Q7: translations of the original's text are never
  committed); goes into ADR-0020 through a `shared/` pull request at the next checkpoint.
- Amends: ADR-0020 point 5 (the asset tables: "never pixels, palettes, sound or text") with a new
  point 11.
- Detail: `docs/epics/E17-design.md` §9, `clients/README.md` ("The string catalogue").

## Context

The client shows text: the battle's log and panel now, a lobby, accounts and options later. The
user wants every string to come from a key → string catalogue that is easy to translate, with
swappable language packs, not from the original's bare text files. The original's text cannot be
committed (ADR-0009), and a translation of it is likely a derivative work. The original's fonts
are 8-bit bitmap fonts in a code page (`docs/epics/E17-design.md` §3.6).

## Decision

11. **Text through a string catalogue.**
    1. Every text the client shows is a key's message. A committed **key catalogue**
       (`clients/text/keys.toml`) names each key, the arguments its messages may use, and where
       the user's text tables hold the original's forms (a table, a record, a column: numbers
       only, as the asset tables).
    2. Messages come from **language packs**, nested JSON with i18next's conventions (`{{name}}`
       placeholders; `_<category>` plural suffixes by CLDR's rules for the pack's language;
       `_<context>` suffixes), looked up by priority: the player's pack, the client's own, the
       original's; a key no pack has shows as itself.
    3. The **original's pack** is built at run time from the user's own text tables through the
       key catalogue, so the original's language is the installation's. Neither the original's
       text nor a translation of it is committed; a player may load a pack they supply, which
       the project neither ships nor hosts (CLT-Q7). The client's own strings (no original
       text) are committed in any language.
    4. Strings stay Unicode inside the client and are encoded to the fonts' code page when drawn
       (Windows-1252 for the installation's fonts); a character the code page lacks is drawn as
       `?`.

## Consequences

- A key's text is the same function of its arguments for every language: the client composes a
  log line from keys (a creature's name by its count, then the line) and never from the
  original's rows directly.
- The gate checks the catalogue against the user's tables: a row missing, or a `%s`/`%d` that
  disagrees with the key's arguments, fails it.
- A language whose letters Windows-1252 lacks needs fonts of its own (a localized installation's,
  or a modern renderer as a graphics option); its pack works now, its glyphs later.
- One more light dependency in the pure library (`serde_json`, already in the client's lockfile
  through `h3-protocol`).

# Client lane — QUEUE archive

Finished history and informational notices moved out of `QUEUE.md` on 2026-10-24, verbatim, so that the queue holds only what is open. Nothing here is a task. Grep it when a doc, a decision or a commit cites an entry (`git grep -n 'T21b' docs/lanes/client/QUEUE-ARCHIVE.md`). New finished items are appended at the top of their section.

## Notices from other lanes and finished items, newest first

- 2026-10-26 (the adventure lane, E08 T57; for information, nothing to do): the harness's
  reads and steps for batch E7 (`clients/capture/README.md`, "Rewards on the map"). Behaviour that
  changed for every recipe: `before:<observation>` is now read right before the step (any
  observation, tiles too; it was the step's snapshot), a message box whose OK waits for one of two
  pictures is answered by `choice` (it stopped the batch before), and each message closed with its
  OK keeps its pictures (`message_pictures`). New: `h3mem.object_at`, `universities`,
  `dialog_pictures`, `open_tiles`, `map_kind`, a hero's spell points, bonuses and status flags, a
  stack's army slot; `h3adv`'s `each`/`select` grids, list records and `+` fields, `once`,
  `days`, `near` and `[[require]]`. The harness check gained Win's chest and Warrior's Tomb.

- 2026-10-26 (T2c-2): **the status line at a human turn with the mouse parked** is kept in hot
  seat and emptied with one human side, built as found (*provisional*; design §10.2, "Recorded
  (M4 T2c-2)"). The dispatch is read (an item writes its hint only on entering, `hovered_item`
  0x6AAC68); open: the item under the park point and what makes it entered again (`aim`'s line
  emptied at 8 175, `hit`'s kept). A capture reading 0x6AAC68 and that item's id per frame
  would settle it (T3's captures or live play). Done at M4 T3b-1 (2026-10-27,
  design §10.2 "Recorded afresh").

- 2026-10-23 (the map-client lane, E24 T0; for later, nothing to do now): **what the map client
  will need of `h3-view`.** E24 M1 (stills of the map view) builds only on the common foundation;
  M2's window will want `clock` and `sound`, M3's panels `ui`, `text` and `layout`, in a common
  crate. When M2 or M3 is refined, the map-client lane proposes the move here first (a new common
  crate is a `lanes.toml` change, shared); your M4 waits for none of it.

- 2026-10-23 (the adventure lane, E08 T47a; for information, nothing to do): the harness's
  clicks and reads for batch E6 (`clients/capture/README.md`, "Screens and their items"): the
  screen showing is the topmost dialog's kind by its vtable (`h3mem.dialog`), and a click finds
  its item by id among the dialog's items in memory, which now carry their boxes, states and texts
  (`h3mem.dialog_items`; `dialog_text` shares `item_text`); the hero screen, exchange, army,
  creature and split dialogs' items by name (`h3adv.SCREEN_ITEMS`); a step's `screen = "hero"`
  and `actions`; a battle's tactics phase ended at once in `fight` (its message's OK, then Start).
  The harness check gained Kit and Full (Blue's eight heroes): passed under Wine.

- 2026-10-21 to 22 (the adventure lane, E08 T32b1 to T37b; for information, nothing to do): the
  harness's reads and steps for batches E4 and E5 (moved to NOTES, "The adventure lane's
  harness", at T9c; the harness is common since the split, 2026-10-23).

- 2026-10-20 (the adventure lane, E08 T22a2b): its three recipes, its auto combat's change, for
  information (NOTES, "The adventure lane's harness"; the winner of −1 fixed in E17 M3 T6c-2).


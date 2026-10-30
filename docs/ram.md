# RAM

Where the running game keeps its state, as CPU addresses, for programs that
read the game from an emulator: `z1::ram`. Both US releases use the same
addresses.

## How it was measured

Every address was measured in a headless NES emulator ([jsnes]) with the
scripts of [z1r-tracker](https://github.com/jansen-wtf/z1r-tracker)
(`tools/emu/measure.js`): do one thing in the game, compare RAM before and
after. `$0600`-`$0637` (sound) and the stack page change all the time and
are left out of every comparison.

| What | Measured by |
|---|---|
| level, mode, screen | entering levels and caves, scrolling, dying, reaching Zelda (`dungeon`, `modes`, `caves`, `death`, `ending`) |
| frame counter | 100 frames playing and on the pause screen (`frames`) |
| item on the screen | other coast items patched into the ROM (`coast`) |
| inventory | each item id put in the wood sword cave's middle slot and taken (`items`) |
| dungeon items, room flags | an item put in a level's start room and taken; walls bombed; cellar items taken with and without holding them (`dungeon`, `bomb`, `cellar`) |
| room tables, level block | where the six tables and the block appear in battery RAM (`tables`) |
| a cave showing | each cave kind entered, frame by frame, against the frame its items appear in the sprites (`cavefade`) |
| start screen | the level block's start room on the start screen (`cavefade`) |
| save slot, names | two files registered with different names, each started (`name`) |

## Where Link is

| Address | Contents |
|---|---|
| `$0010` | level, 0 = overworld |
| `$0011` | the step within the game mode: in a cave 0 while it is laid out, then 1 |
| `$0012` | game mode, below |
| `$0015` | frame counter, one up per frame, pause screen included, wrapping at 256 |
| `$00AB` | the item lying on the overworld screen: the coast item on every screen, the armos item once that statue moved |
| `$00EB` | overworld screen or room; in a cave the overworld screen; down a staircase the room under the level |
| `$0407` | in a cave, 1, 2, 3 while it fades in, 0 once it shows |
| `$6BAD` | the level block's start room; on the overworld the screen Link starts on |

## Game modes (`$0012`)

| Mode | When |
|---|---|
| `FF` | right after power-on |
| `00` | title |
| `01` | file select |
| `0E` | registering a name |
| `02`-`04` | loading a level or the overworld |
| `05` | playing |
| `06`, `07`, `04` | scrolling to the next screen, in that order |
| `08` | continue / save / retry after dying |
| `09` | in a cellar, down a staircase |
| `0B` | in a cave |
| `0C` | in a take-any-road cave (the warp) |
| `10` | taking a staircase |
| `11` | Link dying |
| `12` | holding up a triforce piece |
| `13` | the ending, from reaching Zelda through the credits |

## The player

| Address | Contents |
|---|---|
| `$0016` | the save slot being played, 0-2 |
| `$0638` + 8 × slot | the slots' names, 8 font tiles each, 36 for a blank; battery RAM keeps them too, from `$6002` |

## Inventory

| Address | Item | Values |
|---|---|---|
| `$0656` | selected B item | |
| `$0657` | sword | 1 wood, 2 white, 3 magical |
| `$0658` | bombs | count |
| `$0659` | arrows | 1 wood, 2 silver |
| `$065A`-`$0666` | bow, candle (1 blue, 2 red), recorder, bait, potion (1 blue, 2 red), wand, raft, book, ring (1 blue, 2 red), ladder, magical key, power bracelet, letter | 1 when held unless noted |
| `$066C` | clock | 1 |
| `$066D` | rupees | count |
| `$066E` | keys | count |
| `$066F` | hearts | high nibble containers - 1, low nibble full hearts - 1 |
| `$0674`-`$0676` | boomerang, magical boomerang, magical shield | 1 |
| `$067C` | most bombs Link can carry | 8, then +4 per upgrade |

## Dungeon items and flags

| Address | Contents |
|---|---|
| `$0667`, `$0668` | compasses, maps: bit `level - 1` for levels 1-8 |
| `$0669`, `$066A` | compass, map of level 9: bit 0 |
| `$0671` | triforce pieces, bit `level - 1` |
| `$067F` + screen | overworld screen flags; bit 4: the cave's item is taken |
| `$06FF` + room | room flags of levels 1-6, shared; bit 5 visited, bit 4 item taken, bits 0-3 east, west, south, north wall bombed open |
| `$077F` + room | the same for levels 7-9 |

- A bombed wall sets its bit in both rooms, each for its own side.
- An item down a staircase sets bit 4 of the cellar's room (`$00EB`), also
  when Link already holds it; the inventory then stays as it was.

## Battery RAM

| Address | Contents |
|---|---|
| `$687E`-`$6B7D` | the six room tables of the overworld or of the level Link is in; the real ones, also in a race ROM whose ROM tables are scrambled |
| `$6B81`- | the level block of the level Link is in, laid out as `z1::level::Block` |

[jsnes]: https://github.com/bfirsh/jsnes

# Plan: ASCII Guitar Fretboard Display

## Goal

Display a scrolling, tempo-synced ASCII tab showing 6 guitar strings across
4 measures (4/4 time) of "Wonderful Tonight" at once, with a preview line of
the next 4 measures below it. A cursor (`x`) marks the note the player should
be playing right now, advancing beat-by-beat with a game loop. When the
current 4-measure block finishes, the "next" block bumps up to become the
current block and a new preview block is generated.

## Approach

### 1. Define guitar string frequencies

Store the standard tuning frequencies for each string:

- E2 (82.41 Hz), A2 (110.00 Hz), D3 (146.83 Hz), G3 (196.00 Hz), B3 (246.94 Hz), E4 (329.63 Hz)

### 2. Map detected frequency to string + fret

Given a detected frequency:

- Find the closest string (by frequency).
- Calculate fret: `round(12 * log2(freq / string_open_freq))`
- Clamp fret to valid range (0–24).

### 3. Song data: "Wonderful Tonight" (simplified placeholder)

Real-time play needs the song expressed as a sequence of timed note events,
not just a static diagram. Represent the song as a list of measures, each
holding one chord/root-note event per beat slot.

> Note: the progression and root-note fingerings below are a **simplified,
> generic placeholder** (one root note per measure, common ballad changes)
> used to wire up the engine — not a verified, note-for-note transcription.
> Swap in an accurate transcription later without changing the data shape.

```
tempo_bpm: 66
beats_per_measure: 4

chord_roots = {
  "G":    { string: 6, fret: 3 },  // low E, 3rd fret
  "D/F#": { string: 6, fret: 2 },  // low E, 2nd fret (F# bass)
  "Em":   { string: 6, fret: 0 },  // low E, open
  "C":    { string: 5, fret: 3 },  // A string, 3rd fret
  "D":    { string: 5, fret: 5 },  // A string, 5th fret
}

song = [
  // measures 1..4 (current block on first load)
  { chord: "G"    },
  { chord: "D/F#" },
  { chord: "C"    },
  { chord: "G"    },
  // measures 5..8 (next block on first load)
  { chord: "Em" },
  { chord: "C"  },
  { chord: "D"  },
  { chord: "G"  },
  // ...continue transcribing the rest of the song here
]
```

Each measure contributes 4 beat slots (quarter notes) to the display. The
root note is the "hit" target on beat 1 of its measure; beats 2–4 hold the
cursor on the same string while it advances across the remaining columns,
so the player has a full measure to fret and strike the note.

### 4. Two-block scrolling ASCII renderer

Render two stacked 6-string grids: the **current** 4-measure block (16 beat
columns) and, directly below it, a dimmer **next** 4-measure preview block.
Strings are rows (top to bottom: e, B, G, D, A, E), columns are beat slots,
and measures are separated by `|`.

```
Now:
e||----|----|----|----|
B||----|----|----|----|
G||----|----|----|----|
D||----|----|----|----|
A||----|----|----|----|
E||x---|xx--|----|x---|

Next:
e||----|----|----|----|
B||----|----|----|----|
G||----|----|----|----|
D||----|xxx-|xxxx|----|
A||----|----|----|----|
E||xxxx|----|----|xxxx|

Frequency: 82.41 Hz (E2)   String: 6  Fret: 3
Measure 1 / 4   Beat 1 / 4
```

Only one row has an active `x` per column (the target string/fret for that
beat), since the song data is currently monophonic root notes. The renderer
just walks `song` in slices of 4 measures at a time to build each block —
no change needed if the data later gains chord tones on multiple strings
per beat, since each cell is independently addressable by (string, column).

### 5. Cursor and current-note display

Below both grids, print the detected note info, exactly as before, plus a
measure/beat counter so the player can see where they are in the block:

```
Frequency: 110.00 Hz (A2)
String: 5, Fret: 0
Measure 2 / 4   Beat 3 / 4
```

This line updates in-place (overwrites with `\r`) without redrawing the
grids above.

### 6. Block bump / scroll logic

When the beat cursor passes the last column of the current block (end of
measure 4):

1. The **next** block's data becomes the **current** block's data.
2. A new **next** block is pulled from `song` (the following 4 measures).
3. The beat/measure counters reset to `1/4`.
4. If `song` runs out of measures, either loop back to the start or stop
   the game loop and show an end-of-song state.

This is a simple index-window slide over `song` (`block_start += 4`), so no
data is duplicated — only the render window moves.

### 7. Game loop

The whole thing runs on a tempo-driven tick loop rather than a one-shot
render:

```
on start:
  clear screen
  current_block = song[0..4]
  next_block    = song[4..8]
  beat = 0            // 0..15 within the current block
  tick_interval = 60_000 / tempo_bpm   // ms per beat

loop:
  wait until next tick_interval elapses
  beat += 1
  if beat == 16:
      current_block = next_block
      next_block    = song[block_start+8 .. block_start+12]
      block_start  += 4
      beat = 0
  redraw grids (move cursor to top-left, update x position + note line)
  poll audio input for detected frequency
  compare detected string/fret to target for this beat
  update hit/miss feedback (optional, future work)
```

The loop owns timing (tick interval derived from BPM) and drives both the
cursor position and the block-bump transition. Audio detection stays a
side channel that's polled each tick and compared against the current
target note, so scoring/feedback can be layered on later without changing
the render or timing logic.

### 8. Terminal control

- On start: clear the screen, render both grids anchored at (0,0).
- On each tick: move cursor to the grid area, update only the marker
  character(s) that changed (previous beat's `x` and the new one), then
  reprint the note/measure line below.
- Use ANSI escape codes (`\x1b[H`, `\x1b[<row>;<col>H`, `\x1b[<n>A`) for
  precise cursor positioning, same as the static version but now driven
  by the tick loop instead of only on note detection.

### 9. Module structure

| File       | Responsibility                                                                   |
| ---------- | --------------------------------------------------------------------------------- |
| `main.rs`  | Stream setup, game loop entry point, top-level orchestration                      |
| `pitch.rs` | `freq_to_note()`, frequency-to-string/fret mapping, tuning constants              |
| `song.rs`  | Song data model, chord-root table, `song` sequence for "Wonderful Tonight"        |
| `tab.rs`   | Two-block grid renderer — draws current + next 4-measure blocks, cursor updates   |
| `game.rs`  | Game loop: tick timing (BPM), beat/measure counters, block bump/scroll logic      |
| `audio.rs` | Device selection (`get_scarlet`)                                                  |

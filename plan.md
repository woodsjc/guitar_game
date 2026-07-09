# Plan: ASCII Guitar Fretboard Display

## Goal
Display a static ASCII fretboard diagram at the top-left of the terminal showing which string/fret the user is playing, with the current note printed below.

## Approach

### 1. Define guitar string frequencies
Store the standard tuning frequencies for each string:
- E2 (82.41 Hz), A2 (110.00 Hz), D3 (146.83 Hz), G3 (196.00 Hz), B3 (246.94 Hz), E4 (329.63 Hz)

### 2. Map detected frequency to string + fret
Given a detected frequency:
- Find the closest string (by frequency).
- Calculate fret: `round(12 * log2(freq / string_open_freq))`
- Clamp fret to valid range (0–24).

### 3. Static ASCII fretboard renderer
Render a fixed fretboard diagram anchored at the top-left of the terminal. It is drawn once and only the fret marker updates in place.

```
e||-x---x---x---x---x---|
B||-x---x---x---x---x---|
G||-x---x---x---x---x---|
D||-x---x---x---x---x---|
A||-x---x---x---x---x---|
E||-x---x---x---x---x---|
```

Where `x` marks the active fret on the correct string. Fret columns are labelled at the bottom.

### 4. No scrolling history
The diagram is static — only the current played note is indicated. No scrollable tab history.

### 5. Current note display
Below the fretboard, print the detected note info:

```
Frequency: 110.00 Hz (A2)
String: 5, Fret: 0
```

This line updates in-place (overwrites with `\r`) without redrawing the fretboard above.

### 6. Terminal control
- On start: clear the screen, render the fretboard at (0,0).
- On each note: move cursor to the fretboard area, update only the marker character, then print the note line below.
- Use ANSI escape codes (`\x1b[H`, `\x1b[<row>;<col>H`, `\x1b[<n>A`) for precise cursor positioning.

### 7. Module structure

| File | Responsibility |
|------|---------------|
| `main.rs` | Stream setup, detection loop, top-level orchestration |
| `pitch.rs` | `freq_to_note()`, frequency-to-string/fret mapping, tuning constants |
| `tab.rs` | Fretboard renderer — static diagram with cursor-based marker updates |
| `audio.rs` | Device selection (`get_scarlet`) |

use crate::pitch;
use crate::song::{Measure, Song};

const STRING_NAMES: [&str; 6] = ["e", "B", "G", "D", "A", "E"];
const COL_W: usize = 4;
const BEATS_PER_MEASURE: usize = 4;
const MEASURES_PER_WINDOW: usize = 4;
const BEATS_PER_WINDOW: usize = BEATS_PER_MEASURE * MEASURES_PER_WINDOW;
/// Width, in columns, of the "e ||" style string-name prefix on each row.
const PREFIX_LEN: usize = 4;

const ROW_NOW_HEADER: usize = 1;
const ROW_NOW_STRINGS_START: usize = 2; // occupies 6 rows: 2..=7
const ROW_MARKER: usize = 8;
const ROW_ANNOTATION: usize = 9;
const ROW_NEXT_HEADER: usize = 11;
const ROW_NEXT_STRINGS_START: usize = 12; // occupies 6 rows: 12..=17
const ROW_NOTE_INFO: usize = 19;

pub enum Advance {
    Continue,
    Bumped,
    SongComplete,
}

pub struct TabView {
    song: Song,
    /// Index into `song.measures` where the current 4-measure window begins.
    window_start_measure: usize,
    /// Beat position within the current window, 0..BEATS_PER_WINDOW.
    beat_in_window: usize,
}

impl TabView {
    pub fn new(song: Song) -> Self {
        Self {
            song,
            window_start_measure: 0,
            beat_in_window: 0,
        }
    }

    pub fn is_song_complete(&self) -> bool {
        self.window_start_measure >= self.song.measures.len()
    }

    pub fn expected_beat(&self) -> Option<crate::song::Beat> {
        self.song
            .beat_at(self.window_start_measure, self.beat_in_window)
    }

    /// Draws both the "NOW PLAYING" and "NEXT UP" blocks from scratch, plus
    /// the marker and annotation line for the current beat. Call this on
    /// startup and whenever the window bumps.
    pub fn render_full(&self) {
        print!("\x1b[2J\x1b[H");

        let now = self.window_measures(self.window_start_measure);
        let next = self.window_measures(self.window_start_measure + MEASURES_PER_WINDOW);

        self.render_block("NOW PLAYING", &now, ROW_NOW_HEADER, ROW_NOW_STRINGS_START);
        self.render_block("NEXT UP", &next, ROW_NEXT_HEADER, ROW_NEXT_STRINGS_START);

        self.render_marker();
        print!("\x1b[{};1H", ROW_NOTE_INFO + 1);
        use std::io::Write;
        let _ = std::io::stdout().flush();
    }

    pub fn render_song_complete(&self) {
        print!("\x1b[{};1H\x1b[2K", ROW_ANNOTATION);
        print!("Song complete!");
        print!("\x1b[{};1H", ROW_NOTE_INFO + 1);
        use std::io::Write;
        let _ = std::io::stdout().flush();
    }

    /// Advance one beat. Handles moving the marker within the current
    /// window, or bumping to the next window (and loading a fresh preview)
    /// once 4 measures are complete.
    pub fn advance(&mut self) -> Advance {
        self.beat_in_window += 1;

        if self.beat_in_window >= BEATS_PER_WINDOW {
            self.window_start_measure += MEASURES_PER_WINDOW;
            self.beat_in_window = 0;

            if self.is_song_complete() {
                self.render_song_complete();
                return Advance::SongComplete;
            }

            self.render_full();
            return Advance::Bumped;
        }

        self.render_marker();
        Advance::Continue
    }

    /// Prints the live frequency / detected-note / match-status line without
    /// touching the tab blocks above it.
    pub fn render_note_info(
        &self,
        frequency: f64,
        detected: Option<(usize, u8)>,
        expected: Option<crate::song::Beat>,
    ) {
        let note = pitch::freq_to_note(frequency);
        let detected_desc = match detected {
            Some((s, f)) => format!("{} string, fret {}", STRING_NAMES[s], f),
            None => "no clear pitch".to_string(),
        };
        let expected_desc = match expected {
            Some(b) => match (b.string, b.fret) {
                (Some(s), Some(f)) => format!("{} string, fret {}", STRING_NAMES[s], f),
                _ => "rest".to_string(),
            },
            None => "-".to_string(),
        };
        let is_match = match (detected, expected) {
            (Some((s, f)), Some(b)) => b.string == Some(s) && b.fret == Some(f),
            _ => false,
        };

        print!("\x1b[{};1H\x1b[2K", ROW_NOTE_INFO);
        print!("Frequency: {:.2} Hz ({})", frequency, note);
        print!("\x1b[{};1H\x1b[2K", ROW_NOTE_INFO + 1);
        print!(
            "Detected: {}   Expected: {}   {}",
            detected_desc,
            expected_desc,
            if is_match { "MATCH" } else { "" }
        );
        print!("\x1b[{};1H", ROW_NOTE_INFO + 2);
        use std::io::Write;
        let _ = std::io::stdout().flush();
    }

    /// Returns up to `MEASURES_PER_WINDOW` measures starting at `start`,
    /// padding with `None` if the song runs out (e.g. near the end).
    fn window_measures(&self, start: usize) -> [Option<&Measure>; MEASURES_PER_WINDOW] {
        let mut out: [Option<&Measure>; MEASURES_PER_WINDOW] = [None; MEASURES_PER_WINDOW];
        for i in 0..MEASURES_PER_WINDOW {
            out[i] = self.song.measures.get(start + i);
        }
        out
    }

    fn render_block(
        &self,
        title: &str,
        measures: &[Option<&Measure>; MEASURES_PER_WINDOW],
        header_row: usize,
        strings_start_row: usize,
    ) {
        print!("\x1b[{};1H\x1b[2K", header_row);
        let labels: Vec<String> = measures
            .iter()
            .map(|m| m.and_then(|m| m.label).unwrap_or("").to_string())
            .collect();
        print!("== {} == [{}]", title, labels.join(" | "));

        for (row_offset, string_idx) in (0..6).enumerate() {
            let row = strings_start_row + row_offset;
            print!("\x1b[{};1H\x1b[2K", row);
            print!("{}", self.string_row(measures, string_idx));
        }
    }

    fn string_row(&self, measures: &[Option<&Measure>; MEASURES_PER_WINDOW], string_idx: usize) -> String {
        let mut s = format!("{:<2}||", STRING_NAMES[string_idx]);
        for m in measures {
            match m {
                Some(measure) => {
                    for beat in &measure.beats {
                        s.push_str(&Self::cell(beat, string_idx));
                    }
                }
                None => {
                    for _ in 0..BEATS_PER_MEASURE {
                        s.push_str(&"-".repeat(COL_W));
                    }
                }
            }
            s.push('|');
        }
        s
    }

    fn cell(beat: &crate::song::Beat, string_idx: usize) -> String {
        if beat.string == Some(string_idx) {
            if let Some(fret) = beat.fret {
                return format!("{:-<width$}", fret.to_string(), width = COL_W);
            }
        }
        "-".repeat(COL_W)
    }

    /// Redraws only the marker row + annotation line, positioning `x` under
    /// the current beat's column. Used for in-window advancement so the tab
    /// blocks themselves don't need to be redrawn every beat.
    fn render_marker(&self) {
        let col = self.marker_col();

        print!("\x1b[{};1H\x1b[2K", ROW_MARKER);
        print!("\x1b[{};{}Hx", ROW_MARKER, col);

        print!("\x1b[{};1H\x1b[2K", ROW_ANNOTATION);
        let measure_num = self.window_start_measure + self.beat_in_window / BEATS_PER_MEASURE + 1;
        let beat_num = self.beat_in_window % BEATS_PER_MEASURE + 1;
        match self.expected_beat() {
            Some(b) => match (b.string, b.fret) {
                (Some(s), Some(f)) => print!(
                    "(beat {} of measure {}: play {} string, fret {})",
                    beat_num, measure_num, STRING_NAMES[s], f
                ),
                _ => print!("(beat {} of measure {}: rest)", beat_num, measure_num),
            },
            None => print!("(beat {} of measure {})", beat_num, measure_num),
        }
    }

    fn marker_col(&self) -> usize {
        let measure_idx = self.beat_in_window / BEATS_PER_MEASURE;
        let beat_in_measure = self.beat_in_window % BEATS_PER_MEASURE;
        let offset = measure_idx * (BEATS_PER_MEASURE * COL_W + 1) + beat_in_measure * COL_W;
        PREFIX_LEN + offset + 1
    }
}

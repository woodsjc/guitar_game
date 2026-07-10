use crate::pitch;
use crate::song::Song;
use crate::tab::{Advance, TabView};

pub enum StepResult {
    /// Detected pitch didn't match the expected beat; marker held in place.
    NoMatch,
    /// Matched; marker advanced within the current window.
    Advanced,
    /// Matched, completed the current 4-measure window; NEXT UP was
    /// promoted to NOW PLAYING and a new preview window was loaded.
    WindowBumped,
    /// Matched the final beat of the song.
    SongComplete,
}

/// Owns game state and drives it forward one detected pitch at a time.
/// Rendering itself lives in `TabView`; `Game` just decides match/no-match
/// and calls into it.
pub struct Game {
    tab: TabView,
}

impl Game {
    pub fn new(song: Song) -> Self {
        let tab = TabView::new(song);
        tab.render_full();
        Self { tab }
    }

    pub fn is_complete(&self) -> bool {
        self.tab.is_song_complete()
    }

    /// The beat currently expected to be played. Exposed mainly for testing
    /// the loop end-to-end without live audio input.
    pub fn expected_beat(&self) -> Option<crate::song::Beat> {
        self.tab.expected_beat()
    }

    /// Feed one detected frequency into the game loop.
    pub fn on_pitch(&mut self, frequency: f64) -> StepResult {
        if self.tab.is_song_complete() {
            return StepResult::SongComplete;
        }

        let detected = pitch::freq_to_string_fret(frequency);
        let expected = self.tab.expected_beat();

        self.tab.render_note_info(frequency, detected, expected);

        let is_match = match (detected, expected) {
            (Some((s, f)), Some(beat)) => beat.string == Some(s) && beat.fret == Some(f),
            _ => false,
        };

        if !is_match {
            return StepResult::NoMatch;
        }

        match self.tab.advance() {
            Advance::Continue => StepResult::Advanced,
            Advance::Bumped => StepResult::WindowBumped,
            Advance::SongComplete => StepResult::SongComplete,
        }
    }
}

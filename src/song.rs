/// A single 4/4 beat: which string/fret should be played, if any.
/// `string` uses the same 0..=5 indexing as `pitch::TUNING`
/// (0 = high e ... 5 = low E), so detector output can be compared directly.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Beat {
    pub string: Option<usize>,
    pub fret: Option<u8>,
}

impl Beat {
    pub const fn rest() -> Self {
        Beat {
            string: None,
            fret: None,
        }
    }

    pub const fn note(string: usize, fret: u8) -> Self {
        Beat {
            string: Some(string),
            fret: Some(fret),
        }
    }
}

/// One measure of 4/4 time: exactly 4 beats, plus an optional chord label
/// shown in the tab header for context.
#[derive(Clone, Debug)]
pub struct Measure {
    pub beats: [Beat; 4],
    pub label: Option<&'static str>,
}

impl Measure {
    pub const fn new(beats: [Beat; 4], label: Option<&'static str>) -> Self {
        Measure { beats, label }
    }
}

pub struct Song {
    pub title: &'static str,
    pub measures: Vec<Measure>,
}

impl Song {
    pub fn beat_at(&self, window_start_measure: usize, beat_in_window: usize) -> Option<Beat> {
        let measure_idx = window_start_measure + beat_in_window / 4;
        let beat_idx = beat_in_window % 4;
        self.measures.get(measure_idx).map(|m| m.beats[beat_idx])
    }
}

/// A simplified practice arrangement for "Wonderful Tonight", built from the
/// song's well-known chord tones (key of G: G, D/F#, C, G/B, Am, D7). This is
/// NOT a transcription of any published tab -- it's placeholder data for
/// exercising the two-window renderer / game loop. Swap in your own
/// arrangement here.
pub fn wonderful_tonight() -> Song {
    let measures = vec![
        Measure::new(
            [
                Beat::note(5, 3),
                Beat::note(1, 3),
                Beat::note(2, 0),
                Beat::note(3, 0),
            ],
            Some("G"),
        ),
        Measure::new(
            [
                Beat::note(3, 0),
                Beat::note(4, 2),
                Beat::note(3, 2),
                Beat::note(5, 3),
            ],
            Some("D/F#"),
        ),
        Measure::new(
            [
                Beat::note(4, 3),
                Beat::note(3, 2),
                Beat::note(2, 0),
                Beat::note(5, 0),
            ],
            Some("C"),
        ),
        Measure::new(
            [
                Beat::note(5, 0),
                Beat::note(2, 0),
                Beat::note(1, 0),
                Beat::note(5, 0),
            ],
            Some("G/B"),
        ),
        Measure::new(
            [
                Beat::note(4, 0),
                Beat::note(5, 0),
                Beat::note(4, 2),
                Beat::note(5, 0),
            ],
            Some("Am"),
        ),
        Measure::new(
            [
                Beat::note(4, 0),
                Beat::note(3, 0),
                Beat::note(4, 2),
                Beat::note(3, 1),
            ],
            Some("D7"),
        ),
        Measure::new(
            [
                Beat::note(5, 3),
                Beat::note(1, 3),
                Beat::note(2, 0),
                Beat::note(3, 0),
            ],
            Some("G"),
        ),
        Measure::new(
            [
                Beat::note(2, 0),
                Beat::note(5, 3),
                Beat::note(3, 0),
                Beat::note(2, 0),
            ],
            Some("G"),
        ),
    ];

    Song {
        title: "Wonderful Tonight (practice arrangement)",
        measures,
    }
}

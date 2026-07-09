pub const TUNING: &[(&str, f64); 6] = &[
    ("E4", 329.63),
    ("B3", 246.94),
    ("G3", 196.00),
    ("D3", 146.83),
    ("A2", 110.00),
    ("E2", 82.41),
];

pub fn freq_to_note(freq: f64) -> String {
    let a4 = 440.0;
    let semitone = 12.0 * (freq / a4).log2();
    let midi = (69.0 + semitone).round() as i32;
    let notes = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let note = notes[((midi % 12 + 12) % 12) as usize];
    let octave = (midi / 12) - 1;
    format!("{}{}", note, octave)
}

pub fn freq_to_string_fret(freq: f64) -> Option<(usize, u8)> {
    let mut best: Option<(usize, u8, f64)> = None;
    for (i, &(_, open)) in TUNING.iter().enumerate() {
        let semitones = 12.0 * (freq / open).log2();
        let fret = semitones.round() as i32;
        if (0..=24).contains(&fret) {
            let expected = open * 2.0_f64.powf(fret as f64 / 12.0);
            let err = (freq - expected).abs();
            if best.map_or(true, |(_, _, b)| err < b) {
                best = Some((i, fret as u8, err));
            }
        }
    }
    best.map(|(s, f, _)| (s, f))
}

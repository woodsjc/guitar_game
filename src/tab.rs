const STRING_NAMES: [&str; 6] = ["e", "B", "G", "D", "A", "E"];
const NUM_FRETS: usize = 12;
const COL_W: usize = 3;
const PREFIX: usize = 4;

pub struct Fretboard {
    active: Option<(usize, usize)>,
}

impl Fretboard {
    pub fn new() -> Self {
        Self { active: None }
    }

    pub fn render_initial(&self) {
        print!("\x1b[2J\x1b[H");

        for f in 0..NUM_FRETS {
            print!("{:>3}", f + 1);
        }
        println!();

        for name in &STRING_NAMES {
            print!("{}||", name);
            for _ in 0..NUM_FRETS {
                print!("---");
            }
            println!("|");
        }
    }

    pub fn mark(&mut self, string: usize, fret: usize) {
        if let Some((s, f)) = self.active {
            let row = s + 2;
            let col = PREFIX + f * COL_W + 1;
            print!("\x1b[{};{}H-", row, col);
        }
        if fret < NUM_FRETS {
            let row = string + 2;
            let col = PREFIX + fret * COL_W + 1;
            print!("\x1b[{};{}Hx", row, col);
        }
        self.active = Some((string, fret));
    }

    pub fn to_note_line(&self) {
        print!("\x1b[9;1H\x1b[K");
    }
}

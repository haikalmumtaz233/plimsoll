pub const HEIGHT: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyph {
    width: u32,
    rows: [u8; 5],
}

impl Glyph {
    const fn new(width: u32, rows: [u8; 5]) -> Self {
        Self { width, rows }
    }

    #[must_use]
    pub const fn width(self) -> u32 {
        self.width
    }

    #[must_use]
    pub fn is_lit(self, column: u32, row: u32) -> bool {
        column < self.width
            && usize::try_from(row)
                .ok()
                .and_then(|row| self.rows.get(row))
                .is_some_and(|bits| (bits >> (self.width - 1 - column)) & 1 == 1)
    }
}

#[must_use]
pub const fn glyph(character: char) -> Option<Glyph> {
    let glyph = match character {
        '0' => Glyph::new(3, [0b111, 0b101, 0b101, 0b101, 0b111]),
        '1' => Glyph::new(2, [0b01, 0b11, 0b01, 0b01, 0b01]),
        '2' => Glyph::new(3, [0b111, 0b001, 0b111, 0b100, 0b111]),
        '3' => Glyph::new(3, [0b111, 0b001, 0b011, 0b001, 0b111]),
        '4' => Glyph::new(3, [0b101, 0b101, 0b111, 0b001, 0b001]),
        '5' => Glyph::new(3, [0b111, 0b100, 0b111, 0b001, 0b111]),
        '6' => Glyph::new(3, [0b111, 0b100, 0b111, 0b101, 0b111]),
        '7' => Glyph::new(3, [0b111, 0b001, 0b010, 0b010, 0b010]),
        '8' => Glyph::new(3, [0b111, 0b101, 0b111, 0b101, 0b111]),
        '9' => Glyph::new(3, [0b111, 0b101, 0b111, 0b001, 0b111]),
        'k' => Glyph::new(3, [0b100, 0b101, 0b110, 0b101, 0b101]),
        'M' => Glyph::new(5, [0b10001, 0b11011, 0b10101, 0b10001, 0b10001]),
        '-' => Glyph::new(3, [0b000, 0b000, 0b111, 0b000, 0b000]),
        _ => return None,
    };
    Some(glyph)
}

#[cfg(test)]
mod tests {
    use super::{HEIGHT, glyph};

    fn picture(character: char) -> Vec<String> {
        let glyph = glyph(character).expect("known glyph");
        (0..HEIGHT)
            .map(|row| {
                (0..glyph.width())
                    .map(|column| if glyph.is_lit(column, row) { '#' } else { '.' })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn draws_digits_from_the_top_left() {
        assert_eq!(picture('7'), ["###", "..#", ".#.", ".#.", ".#."]);
        assert_eq!(picture('1'), [".#", "##", ".#", ".#", ".#"]);
    }

    #[test]
    fn covers_every_character_the_tray_labels_use() {
        for character in "0123456789kM-".chars() {
            assert!(glyph(character).is_some(), "{character}");
        }
        assert_eq!(glyph('%'), None);
        assert_eq!(glyph(' '), None);
    }

    #[test]
    fn every_row_fits_inside_the_glyph_width() {
        for character in "0123456789kM-".chars() {
            let glyph = glyph(character).expect("known glyph");
            assert!(
                glyph
                    .rows
                    .iter()
                    .all(|row| u32::from(*row) < 1 << glyph.width())
            );
            assert!(glyph.rows.iter().any(|row| *row != 0));
        }
    }

    #[test]
    fn out_of_range_pixels_are_dark() {
        let glyph = glyph('8').expect("known glyph");
        assert!(!glyph.is_lit(3, 0));
        assert!(!glyph.is_lit(0, HEIGHT));
    }
}

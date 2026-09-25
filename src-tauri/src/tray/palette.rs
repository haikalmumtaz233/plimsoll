use crate::domain::severity::Severity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub [u8; 3]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub background: Rgb,
    pub foreground: Rgb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Neutral,
    Severity(Severity),
}

const WHITE: Rgb = Rgb([0xFF, 0xFF, 0xFF]);
const INK: Rgb = Rgb([0x11, 0x11, 0x11]);

impl Tone {
    #[must_use]
    pub const fn palette(self) -> Palette {
        match self {
            Self::Neutral => Palette {
                background: Rgb([0x3D, 0x3D, 0x3D]),
                foreground: WHITE,
            },
            Self::Severity(Severity::Normal) => Palette {
                background: Rgb([0x1B, 0x6E, 0x3A]),
                foreground: WHITE,
            },
            Self::Severity(Severity::Elevated) => Palette {
                background: Rgb([0xF5, 0xB4, 0x00]),
                foreground: INK,
            },
            Self::Severity(Severity::High) => Palette {
                background: Rgb([0xC2, 0x41, 0x0C]),
                foreground: WHITE,
            },
            Self::Severity(Severity::Critical) => Palette {
                background: Rgb([0xB9, 0x1C, 0x1C]),
                foreground: WHITE,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Rgb, Tone};
    use crate::domain::severity::Severity;

    const TONES: [Tone; 5] = [
        Tone::Neutral,
        Tone::Severity(Severity::Normal),
        Tone::Severity(Severity::Elevated),
        Tone::Severity(Severity::High),
        Tone::Severity(Severity::Critical),
    ];

    fn luminance(Rgb(channels): Rgb) -> f64 {
        let [red, green, blue] = channels.map(|channel| {
            let value = f64::from(channel) / 255.0;
            if value <= 0.040_45 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        });
        0.2126 * red + 0.7152 * green + 0.0722 * blue
    }

    fn contrast(first: Rgb, second: Rgb) -> f64 {
        let (a, b) = (luminance(first), luminance(second));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    #[test]
    fn every_tone_meets_wcag_aa_text_contrast() {
        for tone in TONES {
            let palette = tone.palette();
            let ratio = contrast(palette.foreground, palette.background);
            assert!(ratio >= 4.5, "{tone:?} has contrast {ratio:.2}");
        }
    }

    #[test]
    fn every_tone_has_a_distinct_background() {
        for (index, tone) in TONES.iter().enumerate() {
            for other in &TONES[index + 1..] {
                assert_ne!(tone.palette().background, other.palette().background);
            }
        }
    }
}

use super::glyph::{self, Glyph};
use super::palette::{Palette, Rgb};

pub const MIN_SIZE: u32 = 16;
pub const MAX_SIZE: u32 = 64;

const SIZES: [u32; 7] = [16, 20, 24, 32, 40, 48, 64];
const BASE_SIZE: f64 = 16.0;
const ROOMY_SIZE: u32 = 24;
const MARGIN_DIVISOR: u32 = 8;
const OPAQUE: u8 = 0xFF;
const TRANSPARENT: [u8; 4] = [0, 0, 0, 0];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bitmap {
    pub size: u32,
    pub rgba: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Layout {
    left: u32,
    top: u32,
    scale_x: u32,
    scale_y: u32,
}

#[must_use]
pub fn icon_size(scale_factor: f64) -> u32 {
    let wanted = BASE_SIZE * scale_factor + 0.5;
    SIZES
        .into_iter()
        .rfind(|size| f64::from(*size) <= wanted)
        .unwrap_or(MIN_SIZE)
}

#[must_use]
pub fn render(label: &str, palette: Palette, size: u32) -> Option<Bitmap> {
    if !(MIN_SIZE..=MAX_SIZE).contains(&size) {
        return None;
    }
    let glyphs: Vec<Glyph> = label.chars().map(glyph::glyph).collect::<Option<_>>()?;
    let layout = layout(&glyphs, size)?;
    let mut rgba = Vec::with_capacity(pixel_count(size));
    for y in 0..size {
        for x in 0..size {
            let pixel = if !inside_badge(x, y, size) {
                TRANSPARENT
            } else if text_lit(&glyphs, layout, x, y) {
                opaque(palette.foreground)
            } else {
                opaque(palette.background)
            };
            rgba.extend_from_slice(&pixel);
        }
    }
    Some(Bitmap { size, rgba })
}

fn layout(glyphs: &[Glyph], size: u32) -> Option<Layout> {
    let units = text_units(glyphs)?;
    [margin(size), 1, 0]
        .into_iter()
        .find_map(|margin| fit(units, size, margin))
}

const fn margin(size: u32) -> u32 {
    if size < ROOMY_SIZE {
        1
    } else {
        size / MARGIN_DIVISOR
    }
}

fn fit(units: u32, size: u32, margin: u32) -> Option<Layout> {
    let room = size.checked_sub(2 * margin)?;
    let scale_x = (room / units).min(room / glyph::HEIGHT);
    if scale_x == 0 {
        return None;
    }
    let scale_y = (room / glyph::HEIGHT).min(scale_x * 2);
    Some(Layout {
        left: (size - units * scale_x) / 2,
        top: (size - glyph::HEIGHT * scale_y) / 2,
        scale_x,
        scale_y,
    })
}

fn text_units(glyphs: &[Glyph]) -> Option<u32> {
    let gaps = u32::try_from(glyphs.len()).ok()?.checked_sub(1)?;
    Some(glyphs.iter().map(|glyph| glyph.width()).sum::<u32>() + gaps)
}

fn text_lit(glyphs: &[Glyph], layout: Layout, x: u32, y: u32) -> bool {
    let (Some(dx), Some(dy)) = (x.checked_sub(layout.left), y.checked_sub(layout.top)) else {
        return false;
    };
    let (column, row) = (dx / layout.scale_x, dy / layout.scale_y);
    let mut start = 0;
    for glyph in glyphs {
        if column < start {
            return false;
        }
        if column < start + glyph.width() {
            return glyph.is_lit(column - start, row);
        }
        start += glyph.width() + 1;
    }
    false
}

fn inside_badge(x: u32, y: u32, size: u32) -> bool {
    let radius = size / 8;
    let far = size - 1 - radius;
    let dx = radius.saturating_sub(x) + x.saturating_sub(far);
    let dy = radius.saturating_sub(y) + y.saturating_sub(far);
    dx * dx + dy * dy <= radius * radius
}

const fn opaque(Rgb([red, green, blue]): Rgb) -> [u8; 4] {
    [red, green, blue, OPAQUE]
}

fn pixel_count(size: u32) -> usize {
    usize::try_from(size * size * 4).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{Bitmap, MAX_SIZE, MIN_SIZE, icon_size, render};
    use crate::tray::palette::{Palette, Rgb};

    const PALETTE: Palette = Palette {
        background: Rgb([0x10, 0x20, 0x30]),
        foreground: Rgb([0xFF, 0xFF, 0xFF]),
    };

    fn ascii(bitmap: &Bitmap) -> Vec<String> {
        bitmap
            .rgba
            .chunks(4)
            .map(|pixel| match pixel {
                [_, _, _, 0] => ' ',
                [0xFF, 0xFF, 0xFF, 0xFF] => '#',
                _ => '.',
            })
            .collect::<Vec<char>>()
            .chunks(usize::try_from(bitmap.size).expect("size"))
            .map(|row| row.iter().collect())
            .collect()
    }

    fn lit_bounds(bitmap: &Bitmap) -> (usize, usize, usize, usize) {
        let rows = ascii(bitmap);
        let lit: Vec<(usize, usize)> = rows
            .iter()
            .enumerate()
            .flat_map(|(y, row)| {
                row.chars()
                    .enumerate()
                    .filter(|(_, pixel)| *pixel == '#')
                    .map(move |(x, _)| (x, y))
            })
            .collect();
        let min_x = lit.iter().map(|(x, _)| *x).min().expect("lit pixels");
        let max_x = lit.iter().map(|(x, _)| *x).max().expect("lit pixels");
        let min_y = lit.iter().map(|(_, y)| *y).min().expect("lit pixels");
        let max_y = lit.iter().map(|(_, y)| *y).max().expect("lit pixels");
        (min_x, max_x, min_y, max_y)
    }

    #[test]
    fn picks_the_largest_standard_size_for_the_display_scale() {
        assert_eq!(icon_size(1.0), 16);
        assert_eq!(icon_size(1.25), 20);
        assert_eq!(icon_size(1.5), 24);
        assert_eq!(icon_size(1.75), 24);
        assert_eq!(icon_size(2.0), 32);
        assert_eq!(icon_size(3.0), 48);
        assert_eq!(icon_size(4.0), 64);
        assert_eq!(icon_size(0.5), MIN_SIZE);
        assert_eq!(icon_size(f64::NAN), MIN_SIZE);
    }

    #[test]
    fn renders_two_digits_large_and_centred_at_sixteen_pixels() {
        let bitmap = render("42", PALETTE, 16).expect("fits");
        assert_eq!(bitmap.rgba.len(), 16 * 16 * 4);
        assert_eq!(
            ascii(&bitmap),
            [
                "  ............  ",
                " .............. ",
                "................",
                ".##..##..######.",
                ".##..##..######.",
                ".##..##......##.",
                ".##..##......##.",
                ".######..######.",
                ".######..######.",
                ".....##..##.....",
                ".....##..##.....",
                ".....##..######.",
                ".....##..######.",
                "................",
                " .............. ",
                "  ............  ",
            ]
            .map(str::to_owned)
        );
    }

    #[test]
    fn three_digit_labels_still_fit_at_sixteen_pixels() {
        let bitmap = render("100", PALETTE, 16).expect("fits");
        let (min_x, max_x, min_y, max_y) = lit_bounds(&bitmap);
        assert_eq!((min_x, max_x), (3, 12));
        assert_eq!((min_y, max_y), (3, 12));
    }

    #[test]
    fn token_labels_use_the_full_width_when_needed() {
        let bitmap = render("999k", PALETTE, 16).expect("fits");
        let (min_x, max_x, _, _) = lit_bounds(&bitmap);
        assert_eq!((min_x, max_x), (0, 14));
    }

    #[test]
    fn larger_icons_scale_the_text_up() {
        let small = lit_bounds(&render("42", PALETTE, 16).expect("fits"));
        let large = lit_bounds(&render("42", PALETTE, 48).expect("fits"));
        assert!(large.1 - large.0 > 2 * (small.1 - small.0));
        assert!(large.3 - large.2 > 2 * (small.3 - small.2));
    }

    #[test]
    fn larger_icons_keep_an_eighth_of_the_badge_around_short_labels() {
        for size in [24, 32, 40, 48, 64] {
            for label in ["-", "7", "42", "100"] {
                let bitmap = render(label, PALETTE, size).expect("fits");
                let (min_x, max_x, min_y, max_y) = lit_bounds(&bitmap);
                let margin = usize::try_from(size / 8).expect("size");
                let last = usize::try_from(size - 1).expect("size");
                assert!(
                    min_x >= margin && max_x <= last - margin,
                    "{label} at {size}"
                );
                assert!(
                    min_y >= margin && max_y <= last - margin,
                    "{label} at {size}"
                );
            }
        }
    }

    #[test]
    fn corners_are_transparent_and_the_badge_is_opaque() {
        for size in [MIN_SIZE, 24, 32, MAX_SIZE] {
            let bitmap = render("-", PALETTE, size).expect("fits");
            let rows = ascii(&bitmap);
            assert!(rows[0].starts_with(' '), "{size}");
            let middle = usize::try_from(size / 2).expect("size");
            assert!(!rows[middle].contains(' '), "{size}");
        }
    }

    #[test]
    fn rejects_unknown_characters_empty_labels_and_odd_sizes() {
        assert_eq!(render("4%", PALETTE, 16), None);
        assert_eq!(render("", PALETTE, 16), None);
        assert_eq!(render("42", PALETTE, 8), None);
        assert_eq!(render("42", PALETTE, 128), None);
        assert_eq!(render("99999999", PALETTE, 16), None);
    }
}

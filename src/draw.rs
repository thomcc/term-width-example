//! This is where the code the blog is about lives, although I've cleaned it up some.
use super::{
    term::{Clear, Terminal},
    Result,
};
// use std::io::Write;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const BOX_COL: u8 = 1;

// it's easier to determine how wron something is without the box drawing chars,
// so only use them if --feature="box_drawing" is passed.
#[cfg(feature = "box_drawing")]
mod drawing {
    pub const HLINE: &str = "─";
    pub const VLINE: &str = "│";
    pub const CORNERS: &[char] = &['┌', '┐', '└', '┘'];
}
#[cfg(not(feature = "box_drawing"))]
mod drawing {
    pub const HLINE: &str = "-";
    pub const VLINE: &str = "|";
    pub const CORNERS: &[char] = &['+'; 4];
}

fn wrong_draw_common(t: &mut Terminal, x: u16, y: u16, s: &str, w: usize) -> Result<()> {
    t.move_to(x, y)?;
    let line = drawing::HLINE.repeat(w);
    t.write_colored(
        BOX_COL,
        &format!("{}{}{}", drawing::CORNERS[0], line, drawing::CORNERS[1]),
    )?;
    // write!(t, "{}{}{}", drawing::CORNERS[0], line, drawing::CORNERS[1])?;
    t.move_to(x, y + 1)?;
    t.write_colored(BOX_COL, drawing::VLINE)?;
    t.write(s)?;
    t.write_colored(BOX_COL, drawing::VLINE)?;
    t.move_to(x, y + 2)?;
    t.write_colored(
        BOX_COL,
        &format!("{}{}{}", drawing::CORNERS[0], line, drawing::CORNERS[1]),
    )?;
    Ok(())
}

pub fn byte_len(t: &mut Terminal, x: u16, y: u16, s: &str) -> Result<()> {
    wrong_draw_common(t, x, y, s, s.len())
}

pub fn codepoints(t: &mut Terminal, x: u16, y: u16, s: &str) -> Result<()> {
    wrong_draw_common(t, x, y, s, s.chars().count())
}

pub fn nfc_codepoints(t: &mut Terminal, x: u16, y: u16, s: &str) -> Result<()> {
    let s = s.nfc().collect::<String>();
    wrong_draw_common(t, x, y, &s, s.nfc().count())
}

pub fn graphemes(t: &mut Terminal, x: u16, y: u16, s: &str) -> Result<()> {
    wrong_draw_common(t, x, y, s, s.graphemes(true).count())
}

pub fn system_wcwidth(t: &mut Terminal, x: u16, y: u16, s: &str) -> Result<()> {
    let width = s
        .chars()
        .map(|c| crate::wcwidths::system_wcwidth(c).unwrap_or_default())
        .sum();
    wrong_draw_common(t, x, y, s, width)
}

pub fn unicode_width(t: &mut Terminal, x: u16, y: u16, s: &str) -> Result<()> {
    wrong_draw_common(t, x, y, s, s.width())
}

pub fn nfc_unicode_width(t: &mut Terminal, x: u16, y: u16, s: &str) -> Result<()> {
    let s = s.nfc().collect::<String>();
    wrong_draw_common(t, x, y, &s, s.width())
}

pub fn widecharwidth_fish(t: &mut Terminal, x: u16, y: u16, s: &str) -> Result<()> {
    let width = s.chars().map(crate::wcwidths::widecharwidth_fish).sum();
    wrong_draw_common(t, x, y, &s, width)
}

pub fn widecharwidth_recommended(t: &mut Terminal, x: u16, y: u16, s: &str) -> Result<()> {
    let width = s
        .chars()
        .map(crate::wcwidths::widecharwidth_recommended)
        .sum();
    wrong_draw_common(t, x, y, &s, width)
}

pub fn termwiz_ish(t: &mut Terminal, x: u16, y: u16, s: &str) -> Result<()> {
    let s = s.nfc().collect::<String>();
    let width = s
        .graphemes(true)
        .map(|g| {
            let is_emoji_sequence = g.chars().any(|c| {
                // This is incomplete, but you could imagine a version which
                // follows https://unicode.org/reports/tr51.
                unic_emoji_char::is_emoji_modifier(c)
                    || unic_emoji_char::is_emoji_modifier_base(c)
                    // regional indicator sequence.
                    || (0x1F1E6..=0x1F1FF).contains(&(c as u32))
            });
            if is_emoji_sequence {
                2
            } else {
                g.width()
            }
        })
        .sum();
    wrong_draw_common(t, x, y, &s, width)
}

pub fn read_pos(t: &mut Terminal, x: u16, y: u16, s: &str) -> Result<()> {
    t.move_to(x, y + 1)?;
    t.write_colored(BOX_COL, drawing::VLINE)?;
    t.write(s)?;
    t.write_colored(BOX_COL, drawing::VLINE)?;
    let (ex, ey) = t.get_pos()?;
    let (sx, sy) = (x, y + 1);
    // Handle if we go off the end. In the sample code we can't, but the big
    // downside of the approach is that you may have to handle that, since when
    // you write some text, you won't know how long it will be.
    let width = if ey != sy {
        t.move_to(t.size().0 - 1, sy)?;
        t.write_colored(BOX_COL, drawing::VLINE)?;
        t.clear(Clear::ToEndOfScreen)?;
        t.size().0.saturating_sub(sx)
    } else {
        debug_assert!(ex > sx);
        ex - sx
    };
    let width = (width as usize).saturating_sub(2);
    let line = drawing::HLINE.repeat(width);
    t.move_to(x, y)?;
    t.write_colored(
        BOX_COL,
        &format!("{}{}{}", drawing::CORNERS[0], line, drawing::CORNERS[1]),
    )?;
    t.move_to(x, y + 2)?;
    t.write_colored(
        BOX_COL,
        &format!("{}{}{}", drawing::CORNERS[2], line, drawing::CORNERS[3]),
    )?;
    Ok(())
}

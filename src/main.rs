#![allow(dead_code)]

mod term;
use std::io::Write;
use term::Terminal;
type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
type Result<T, E = Error> = std::result::Result<T, E>;

fn draw_boxed_byte_len(t: &mut Terminal, x: u16, y: u16, s: &str) -> Result<()> {
    t.move_to(x, y)?;
    let line = "─".repeat(s.len());
    write!(t, "┌{}┐", line)?;
    t.move_to(x, y + 1)?;
    write!(t, "│{}│", s)?;
    t.move_to(x, y + 2)?;
    write!(t, "└{}┘", line)?;
    Ok(())
}

fn draw_boxed_char_count(t: &mut Terminal, x: u16, y: u16, s: &str) -> Result<()> {
    t.move_to(x, y)?;
    let line = "─".repeat(s.chars().count());
    write!(t, "┌{}┐", line)?;
    t.move_to(x, y + 1)?;
    write!(t, "│{}│", s)?;
    t.move_to(x, y + 2)?;
    write!(t, "└{}┘", line)?;
    Ok(())
}

fn draw_boxed_graphemes(t: &mut Terminal, x: u16, y: u16, s: &str) -> Result<()> {
    use unicode_segmentation::UnicodeSegmentation;
    t.move_to(x, y)?;
    let line = "─".repeat(s.graphemes(true).count());
    write!(t, "┌{}┐", line)?;
    t.move_to(x, y + 1)?;
    write!(t, "│{}│", s)?;
    t.move_to(x, y + 2)?;
    write!(t, "└{}┘", line)?;
    Ok(())
}

fn draw_boxed_unicode_width(t: &mut Terminal, x: u16, y: u16, s: &str) -> Result<()> {
    use unicode_width::UnicodeWidthStr;
    t.move_to(x, y)?;
    let line = "─".repeat(s.width());
    write!(t, "┌{}┐", line)?;
    t.move_to(x, y + 1)?;
    write!(t, "│{}│", s)?;
    t.move_to(x, y + 2)?;
    write!(t, "└{}┘", line)?;
    Ok(())
}

pub fn hybrid_width(s: &str) -> usize {
    use unicode_segmentation::UnicodeSegmentation;
    use unicode_width::UnicodeWidthStr;
    s.graphemes(true)
        .map(|g| {
            let is_emoji_sequence = g.chars().any(|c| {
                // This is incomplete, but you could imagine a version which
                // follows https://unicode.org/reports/tr51. At a minimum this
                // should probably
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
        .sum()
}

fn draw_boxed_hybrid_width_graphemes(t: &mut Terminal, x: u16, y: u16, s: &str) -> Result<()> {
    t.move_to(x, y)?;
    let line = "─".repeat(hybrid_width(s));
    write!(t, "┌{}┐", line)?;
    t.move_to(x, y + 1)?;
    write!(t, "│{}│", s)?;
    t.move_to(x, y + 2)?;
    write!(t, "└{}┘", line)?;
    Ok(())
}

fn draw_boxed_fixed(t: &mut Terminal, x: u16, y: u16, s: &str) -> Result<()> {
    t.move_to(x, y + 1)?;
    write!(t, "│{}│", s)?;
    t.flush()?;
    let (ex, ey) = t.get_pos()?;
    let (sx, sy) = (x, y + 1);
    let width = if ey != sy {
        t.move_to(t.size().0 - 2, sy)?;
        write!(t, "│")?;
        t.clear(term::Clear::ToEndOfScreen)?;
        (t.size().0 - 2).saturating_sub(sx)
    } else {
        debug_assert!(ex > sx);
        ex - sx
    };
    let line = "─".repeat((width as usize).saturating_sub(2));
    t.move_to(x, y)?;
    write!(t, "┌{}┐", line)?;
    t.move_to(x, y + 2)?;
    write!(t, "└{}┘", line)?;
    Ok(())
}

fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        eprintln!("Usage: {} text", args[0]);
        std::process::exit(1);
    }
    let impls: &[(&str, fn(&mut Terminal, u16, u16, &str) -> Result<()>)] = &[
        ("bytes", draw_boxed_byte_len),
        ("chars", draw_boxed_char_count),
        ("graphemes", draw_boxed_graphemes),
        ("unicode-width", draw_boxed_unicode_width),
        ("hybrid/heuristic", draw_boxed_hybrid_width_graphemes),
        ("fixed", draw_boxed_fixed),
    ];

    let mut term = Terminal::open(true)?;
    let mut pos = term.get_pos()?;
    let size = term.size();
    if pos.1 as usize + (impls.len() * 4) >= size.1 as usize {
        term.clear(term::Clear::FullScreen)?;
        pos = (1, 1);
        term.move_to(pos.0, pos.1)?;
    } else {
        term.scroll(impls.len() as u16 * 4)?;
        pos = (1, pos.1.checked_sub(impls.len() as u16 * 4).unwrap_or(1));
        term.move_to(pos.0, pos.1)?;
        term.clear(term::Clear::ToEndOfScreen)?;
    };
    let (x, mut y) = pos;
    let mut continue_after = Some(1);
    while let Some(arg_start) = continue_after.take() {
        if arg_start >= args.len() {
            break;
        }
        for &(name, func) in impls {
            term.move_to(x, y)?;
            if y + 3 >= size.1 {
                term.scroll(4)?;
                y -= 4;
                term.move_to(x, y)?;
            }
            write!(term, "{}", name)?;
            let mut x = x;
            for (i, arg) in args[arg_start..].iter().enumerate() {
                func(&mut term, x, y + 1, arg)?;
                x += arg.len() as u16 + 5;
                if x > term.size().0 {
                    continue_after = Some(i + 1);
                    break;
                }
            }
            y += 4;
            term.move_to(x, y + 4)?;
        }
    }
    term.flush()?;
    drop(term);
    println!();
    Ok(())
}

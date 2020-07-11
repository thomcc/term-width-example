#![allow(dead_code)]

mod draw;
mod term;
mod wcwidths;
use std::io::Write;
use term::Terminal;
type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
type Result<T, E = Error> = std::result::Result<T, E>;

type DrawFunc = (
    &'static str,
    fn(&mut Terminal, u16, u16, &str) -> Result<()>,
);

const IMPLS: &[DrawFunc] = &[
    ("byte_len", draw::byte_len),
    ("codepoints", draw::codepoints),
    ("nfc_codepoints", draw::nfc_codepoints),
    ("graphemes", draw::graphemes),
    ("unicode_width", draw::unicode_width),
    ("nfc_unicode_width", draw::nfc_unicode_width),
    ("system_wcwidth", draw::system_wcwidth),
    ("widecharwidth_rec", draw::widecharwidth_recommended),
    ("widecharwidth_fish", draw::widecharwidth_fish),
    ("termwiz_ish", draw::termwiz_ish),
    ("read_pos", draw::read_pos),
];

fn list() {
    for i in IMPLS {
        println!("- `{}`", i.0);
    }
}

#[derive(argh::FromArgs)]
/// Try to draw boxes around text.
struct Args {
    #[argh(switch)]
    /// list all tests
    list: bool,
    #[argh(switch)]
    /// disable drawing the boxes colored (this is done so that they can stand
    /// out).
    no_color: bool,
    #[argh(switch)]
    /// bypass isatty check
    force: bool,
    #[argh(switch)]
    /// allow arguments longer than terminal width. only `read_pos` has been
    /// programmed to handle this, so you probably need `-t read-pos` too.
    allow_overlong: bool,
    /// only include the specified tests, may repeat, default is all
    #[argh(option, short = 't')]
    test: Vec<String>,
    /// list of phrases to draw boxed.
    #[argh(positional)]
    phrases: Vec<String>,
}

fn main() -> Result<()> {
    crate::wcwidths::init_once();
    let mut args: Args = argh::from_env();
    if args.list {
        list();
        return Ok(());
    }
    if args.phrases.is_empty() {
        args.phrases.push("🏳️‍🌈 space communism".to_string());
    }
    if !args.force && !term::is_terminal() {
        eprintln!("doesn't look like this is a terminal. this test requires that.");
        std::process::exit(1);
    }

    let filters = args
        .test
        .iter()
        .map(|i| i.trim().to_ascii_lowercase().replace('-', "_"))
        .collect::<std::collections::HashSet<_>>();

    let selected_tests = if filters.len() == 0 {
        IMPLS.iter().collect()
    } else {
        IMPLS
            .iter()
            .filter(|f| filters.contains(f.0))
            .collect::<Vec<_>>()
    };

    if selected_tests.is_empty() && !filters.is_empty() {
        eprintln!("Warning: all implementations filtered. Printing options.");
        list();
        std::process::exit(1);
    }

    let mut term = Terminal::open(true, args.no_color)?;
    let size = term.size();

    term.clear(term::Clear::FullScreen)?;
    term.move_to(1, 1)?;

    // Ensure we won't try to make a column that goes off the end. If we would,
    // we just do set of rows.
    let mut passes = vec![(0, vec![])];
    for word in args.phrases {
        let mut cur = passes.pop().unwrap();
        if cur.0 + word.len() + 5 >= size.0 as usize && !cur.1.is_empty() {
            passes.push(std::mem::replace(&mut cur, (0, vec![])));
        }
        cur.0 += word.len() + 4;
        cur.1.push(word);
        passes.push(cur)
    }

    let (x, mut y) = (1, 1);
    for (_, pass) in passes {
        for &test in &selected_tests {
            term.move_to(x, y)?;
            if y + 4 >= size.1 {
                term.scroll(4)?;
                y -= 4;
                term.move_to(x, y)?;
            }
            term.write_colored(term::Color::Yellow, test.0)?;
            y += 1;
            let mut x = x;
            for word in &pass {
                (test.1)(&mut term, x, y, word)?;
                x += word.len() as u16 + 5;
            }
            y += 3;
            // term.move_to(x, y + 3)?;
        }
    }
    term.flush()?;
    drop(term);
    println!();
    Ok(())
}

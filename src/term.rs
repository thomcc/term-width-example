// Small terminal library. unix-only, and hardcodes escapes rather than using
// terminfo, although I believe any of this could be implemented to do the right
// thing fairly easily.
use super::Result;
use libc::termios;
use std::fs::File;
use std::io::Write;
use std::os::unix::io::AsRawFd;

pub struct Terminal {
    size: (u16, u16),
    prev: Option<termios>,
    tty: File,
}

impl Terminal {
    pub fn open(raw: bool) -> Result<Self> {
        let tty = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")?;
        let size = size()?;
        let prev = if raw {
            unsafe {
                let fd = tty.as_raw_fd();
                let prev = get_termios(fd)?;
                let mut raw = prev;
                libc::cfmakeraw(&mut raw);
                if libc::tcsetattr(fd, 0, &raw) == -1 {
                    eprintln!("tcsetattr failed");
                    return Err(std::io::Error::last_os_error().into());
                }
                Some(prev)
            }
        } else {
            None
        };
        Ok(Self { prev, tty, size })
    }

    pub fn size(&self) -> (u16, u16) {
        self.size
    }

    pub fn write(&mut self, s: &str) -> Result<()> {
        self.tty.write(s.as_bytes())?;
        Ok(())
    }

    // write `\E[6n`, get back `\E[{y};{x}R`. We do this in a dumb way but a
    // serious application would have this code written.
    pub fn get_pos(&mut self) -> Result<(u16, u16)> {
        use std::io::Read;
        self.tty.flush()?;
        self.tty.write_all(b"\x1b[6n")?;
        self.tty.flush()?;
        let mut v = String::new();
        let mut saw_esc = false;
        loop {
            // this is a dumb way to read this but we don't want to read past
            // what we need to, which could block.
            let mut buf = [0u8; 1];
            let size = self.tty.read(&mut buf)?;
            if size == 0 {
                continue;
            }
            if !saw_esc {
                saw_esc = buf[0] == 0x1b;
            } else {
                if buf[0] == b'R' {
                    break;
                }
                v.push(buf[0] as char);
            }
        }
        let mut fields = v.split(';');
        // v should be `[row;col`
        let row = fields.next().unwrap()[1..].parse::<u16>()?;
        let col = fields.next().unwrap().parse::<u16>()?;
        Ok((col, row))
    }

    pub fn move_to(&mut self, x: u16, y: u16) -> Result<()> {
        // real code should look up `cup` in terminfo
        self.tty
            .write_all(format!("\x1b[{};{}H", y.max(1), x.max(1)).as_bytes())?;
        Ok(())
    }
    pub fn clear(&mut self, clear: Clear) -> Result<()> {
        self.tty.write_all(match clear {
            Clear::ToEndOfScreen => b"\x1b[0J",
            Clear::ToStartOfScreen => b"\x1b[1J",
            Clear::FullScreen => b"\x1b[2J",
            Clear::ToStartOfLine => b"\x1b[0K",
            Clear::ToEndOfLine => b"\x1b[1K",
            Clear::FullLine => b"\x1b[2K",
        })?;
        Ok(())
    }
    pub fn save(&mut self) -> Result<()> {
        self.tty.write_all(b"\x1b[s")?;
        Ok(())
    }

    pub fn restore(&mut self) -> Result<()> {
        self.tty.write_all(b"\x1b[u")?;
        Ok(())
    }
    pub fn scroll(&mut self, n: u16) -> Result<()> {
        write!(self.tty, "\x1b[{}S", n)?;
        self.tty.flush()?;
        Ok(())
    }
}

fn size() -> Result<(u16, u16)> {
    Ok((tput::<u16>("cols")?, tput::<u16>("lines")?))
}

// This is a lot more reliable in subshells and such than using the libc
// api. yeah, really. I mean, I probably should fall back to the other
// thing in case the user doesn't have a terminfo entry for their `TERM`
fn tput<T: std::str::FromStr>(cmd: &str) -> Result<T> {
    Ok(
        String::from_utf8_lossy(&std::process::Command::new("tput").arg(cmd).output()?.stdout)
            .trim()
            .parse::<T>()
            .map_err(|_| format!("Failed to parse tput {} output", cmd))?,
    )
}
impl Write for Terminal {
    fn write(&mut self, s: &[u8]) -> std::io::Result<usize> {
        self.tty.write(s)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.tty.flush()
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        if let Some(prev) = self.prev {
            unsafe {
                if libc::tcsetattr(self.tty.as_raw_fd(), 0, &prev) == -1 {
                    eprintln!("tcsetattr failed: {:?}", std::io::Error::last_os_error());
                }
            }
        }
    }
}

fn get_termios(fd: libc::c_int) -> Result<termios> {
    unsafe {
        let mut tios = std::mem::zeroed();
        if libc::tcgetattr(fd, &mut tios) == -1 {
            eprintln!("tcgetattr failed");
            Err(std::io::Error::last_os_error().into())
        } else {
            Ok(tios)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Clear {
    FullScreen,
    ToStartOfScreen,
    ToEndOfScreen,
    FullLine,
    ToStartOfLine,
    ToEndOfLine,
}

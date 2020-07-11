// Port of https://github.com/ridiculousfish/widecharwidth/
mod widecharwidth;
use widecharwidth::WCWidth;

pub fn init_once() {
    static LOCALE_INIT: std::sync::Once = std::sync::Once::new();
    LOCALE_INIT.call_once(|| unsafe {
        // Set to default locale
        libc::setlocale(libc::LC_ALL, b"\0".as_ptr() as *const libc::c_char);
        // fetch current locale
        let cur = core::ptr::NonNull::new(libc::setlocale(libc::LC_ALL, core::ptr::null()));
        // Detect if it's utf8
        let change = cur.map_or(true, |p| {
            let cstr = std::ffi::CStr::from_ptr(p.as_ptr());
            let locale = cstr.to_string_lossy().to_ascii_lowercase();
            !locale.contains("utf-8") && !locale.contains("utf8")
        });
        if change {
            libc::setlocale(
                libc::LC_ALL,
                b"en_US.UTF-8\0".as_ptr() as *const libc::c_char,
            );
        }
    });
}

mod lib_c {
    // Surprisingly, these aren't in the libc crate.
    extern "C" {
        pub fn wcwidth(c: libc::wchar_t) -> libc::c_int;
    }
}

/// libc's wcwidth
pub fn system_wcwidth(c: char) -> Result<usize, libc::c_int> {
    match unsafe { lib_c::wcwidth(c as libc::wchar_t) } {
        n if n >= 0 => Ok(n as usize),
        n => Err(n),
    }
}

/// widecharwidth_wcwidth with the settings recommended on it's github page
pub fn widecharwidth_recommended(c: char) -> usize {
    match widecharwidth::wcwidth(c) {
        WCWidth::Width(n) => n,
        WCWidth::Nonprint => 0,
        WCWidth::Combining => 0,
        WCWidth::Ambiguous => 1,
        WCWidth::PrivateUse => 1,
        WCWidth::Unassigned => 0,
        WCWidth::WidenedIn9 => 2,
    }
}

/// equivalent to what fish does on my machine
pub fn widecharwidth_fish(c: char) -> usize {
    match c {
        // VS16 emoji selection
        '\u{fe0f}' => 1,
        '\u{fe0e}' => 0,
        // Korean Hangul Jamo median vowels and final consonants
        '\u{1160}'..='\u{11ff}' => 0,
        _ => match widecharwidth::wcwidth(c) {
            WCWidth::Width(n) => n,
            WCWidth::Ambiguous => 1,
            WCWidth::PrivateUse => 1,
            WCWidth::WidenedIn9 => 2,
            WCWidth::Nonprint | WCWidth::Combining | WCWidth::Unassigned => {
                system_wcwidth(c).unwrap_or_default()
            }
        },
    }
}

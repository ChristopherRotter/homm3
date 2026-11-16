//! A dialog's edit (E27 M2 T7; `docs/epics/E27-design.md` §16): `H3DlgEdit`'s text, caret and
//! focus, and its keys as the executable takes them (`edit_key` 0x5BAC50), typed through the
//! original's own key table. The shell's multiplayer names first; the scenario screen's save name
//! and the high score's entry later (E27 M3, M4).
//!
//! - **A key** reaches an edit only with the focus (+0x6D), as its scan code (set 1) and whether
//!   Shift is held ([`Scan`]: the input manager's event +4 and +0xC's bits 0 and 1). Escape, Tab
//!   and Enter are not the edit's (its +0x40, 0x5BBA20): its dialog takes them. Every other key
//!   is taken, whatever it does ([`Edit::key`]).
//! - **The caret keys**, by scan code (0x5BAD1A): Home to the start, Left back one, Right on one
//!   up to the end, End to the end, Delete removes the character at the caret.
//! - **Typing** (0x5BAE1A): the key's character by the input manager's table ([`character`]: a
//!   fixed US layout by scan code, 0x4EC8C0 and 0x4EC6F0, whatever the system's layout; Caps Lock
//!   not read); BackSpace (0x7F) removes the character before the caret; another character is
//!   put in at the caret unless the text with its terminator would reach the limit (a limit of 21
//!   holds 20 characters, `hotseat-long`), or it is none (0, F1 to F12: 0x5BABA0) or a brace.
//! - **Not built** (*provisional*): the check that undoes a character once the text before it
//!   takes more lines than one in the box (0x5BB05D, its off-by-one included: it measures the text
//!   without the new character), and the scroll of a text wider than its box (the edit's +0x68 of
//!   3, never set: 0x5BB7E7): every edit built holds its limit of the widest glyph in its box
//!   (20 × `M`, 260 px of `smalfont.fnt`, in boxes of 281 and 334 px). The caret's blink state
//!   (+0x6C, 360 ms through 0x698998) is written and never read: the caret does not blink
//!   (`edit-keys`' `later` shots).
//!
//! The edit draws as a label with its caret ([`Edit::place`]: [`Draw::Edit`]).

use alloc::string::String;

use crate::ui::Draw;

/// Escape's scan code (not the edit's: its dialog's Cancel).
pub const ESCAPE: u8 = 0x01;
/// BackSpace's scan code (the table's 0x7F).
pub const BACKSPACE: u8 = 0x0E;
/// Tab's scan code (not the edit's).
pub const TAB: u8 = 0x0F;
/// Enter's scan code (not the edit's).
pub const RETURN: u8 = 0x1C;
/// Home's scan code (the keypad's 7 shares it).
pub const HOME: u8 = 0x47;
/// The up arrow's scan code (the keypad's 8 shares it).
pub const UP: u8 = 0x48;
/// The left arrow's scan code (the keypad's 4 shares it).
pub const LEFT: u8 = 0x4B;
/// The right arrow's scan code (the keypad's 6 shares it).
pub const RIGHT: u8 = 0x4D;
/// End's scan code (the keypad's 1 shares it).
pub const END: u8 = 0x4F;
/// The down arrow's scan code (the keypad's 2 shares it).
pub const DOWN: u8 = 0x50;
/// Delete's scan code (the keypad's full stop shares it).
pub const DELETE: u8 = 0x53;

/// The character BackSpace types (the table's, 0x4EC964).
const RUB_OUT: u8 = 0x7F;

/// A key down as the original's input manager gives it to an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scan {
    /// Its scan code (set 1, as `DirectInput`'s and the keyboard's own).
    pub code: u8,
    /// A Shift key is held (the event's +0xC & 3).
    pub shift: bool,
}

impl Scan {
    /// A key `code` without Shift.
    #[must_use]
    pub const fn plain(code: u8) -> Self {
        Self { code, shift: false }
    }
}

/// The input manager's table (0x4EC8C0, the manager's +0x848: a word a scan code, its low byte
/// the character; codes past 0x58 none): the US layout's unshifted keys, upper-case letters.
/// The function keys' words (0x3B to 0x44, 0x57, 0x58) are no character here (0x5BABA0).
const TABLE: [u8; 0x59] = {
    let mut t = [0u8; 0x59];
    let rows: [(usize, &[u8]); 8] = [
        (0x01, b"\x1b1234567890-=\x7f\x09"),
        (0x10, b"QWERTYUIOP[]\x0a"),
        (0x1E, b"ASDFGHJKL;'"),
        (0x2B, b"\\ZXCVBNM,./"),
        (0x37, b"*"),
        (0x39, b" "),
        (0x4A, b"-"),
        (0x4E, b"+"),
    ];
    let mut r = 0;
    while r < rows.len() {
        let (at, keys) = rows[r];
        let mut i = 0;
        while i < keys.len() {
            t[at + i] = keys[i];
            i += 1;
        }
        r += 1;
    }
    t
};

/// The character a key types (0x4EC6F0): the table's ([`TABLE`]); without Shift a letter in
/// lower case, with Shift the US layout's shifted digits and marks (`[`, `]`, `*`, `+` and the
/// letters unchanged); 0 for none.
#[must_use]
pub fn character(scan: Scan) -> u8 {
    let c = TABLE.get(usize::from(scan.code)).copied().unwrap_or(0);
    if !scan.shift {
        return if c.is_ascii_uppercase() { c + 0x20 } else { c };
    }
    match c {
        b'\'' => b'"',
        b',' => b'<',
        b'-' => b'_',
        b'.' => b'>',
        b'/' => b'?',
        b'0' => b')',
        b'1' => b'!',
        b'2' => b'@',
        b'3' => b'#',
        b'4' => b'$',
        b'5' => b'%',
        b'6' => b'^',
        b'7' => b'&',
        b'8' => b'*',
        b'9' => b'(',
        b';' => b':',
        b'=' => b'+',
        b'\\' => b'|',
        c => c,
    }
}

/// The key that types `c` ([`character`]'s inverse: the US layout, as the harness's `xdotool type`
/// types on it): the first code without Shift, else the first with it; none for a character no
/// key types.
#[must_use]
pub fn scan_of(c: u8) -> Option<Scan> {
    [false, true].into_iter().find_map(|shift| {
        (0..=0x58u8)
            .map(|code| Scan { code, shift })
            .find(|&scan| c != 0 && character(scan) == c)
    })
}

/// An edit's state: its text, its caret, its limit and its focus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    text: String,
    caret: usize,
    /// The text's room with its terminator (`H3DlgEdit::Create`'s length, +0x5A).
    limit: usize,
    focused: bool,
}

impl Edit {
    /// An edit holding `text`, its room `limit` characters with the terminator, the caret at the
    /// text's end (0x5BAABF), without the focus.
    #[must_use]
    pub fn new(text: &str, limit: usize) -> Self {
        Self {
            text: String::from(text),
            caret: text.len(),
            limit,
            focused: false,
        }
    }

    /// The text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The caret's index.
    #[must_use]
    pub const fn caret(&self) -> usize {
        self.caret
    }

    /// Whether it has the focus.
    #[must_use]
    pub const fn focused(&self) -> bool {
        self.focused
    }

    /// The focus given or taken (+0x38, 0x5BAB50).
    pub const fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// A key down (`edit_key` 0x5BAC50): whether the edit took it (its dialog's hotkeys get the
    /// others, and every key while it has no focus).
    pub fn key(&mut self, scan: Scan) -> bool {
        if !self.focused || matches!(scan.code, ESCAPE | TAB | RETURN) {
            return false;
        }
        let len = self.text.len();
        self.caret = self.caret.min(len);
        match scan.code {
            HOME => self.caret = 0,
            LEFT => self.caret = self.caret.saturating_sub(1),
            RIGHT if self.caret < len => self.caret += 1,
            END => self.caret = len,
            DELETE if self.caret < len => {
                self.text.remove(self.caret);
            }
            RIGHT | DELETE => {}
            _ => match character(scan) {
                RUB_OUT if self.caret > 0 => {
                    self.caret -= 1;
                    self.text.remove(self.caret);
                }
                RUB_OUT | 0 | b'{' | b'}' => {}
                _ if len + 1 >= self.limit => {}
                c => {
                    self.text.insert(self.caret, char::from(c));
                    self.caret += 1;
                }
            },
        }
        true
    }

    /// The edit's item `id` in `draws` (its dialog's, where the edit is a label) drawn as the
    /// edit: its text, and its caret with the focus ([`Draw::Edit`]).
    pub fn place(&self, id: u16, draws: &mut [Draw]) {
        for draw in draws.iter_mut() {
            if let Draw::Label { item, x, y, .. } = draw
                && item.id == id
            {
                *draw = Draw::Edit {
                    item: item.clone(),
                    x: *x,
                    y: *y,
                    text: self.text.clone(),
                    caret: self
                        .focused
                        .then(|| u32::try_from(self.caret).unwrap_or(u32::MAX)),
                };
            }
        }
    }
}

#[cfg(test)]
mod tests;

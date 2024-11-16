use std::collections::HashSet;

use sdl2::keyboard::Keycode;

pub fn to_ascii_char(keycode: &Keycode, pressed_keys: &HashSet<Keycode>) -> Option<char> {
    match *keycode {
        Keycode::SPACE => Some(' '),
        Keycode::TAB => Some('\t'),
        Keycode::PERIOD => Some('.'),
        Keycode::EXCLAIM => Some('!'),
        Keycode::QUOTEDBL => Some('"'),
        Keycode::HASH => Some('#'),
        Keycode::DOLLAR => Some('$'),
        Keycode::PERCENT => Some('%'),
        Keycode::AMPERSAND => Some('&'),
        Keycode::QUOTE => Some('\''),
        Keycode::LEFTPAREN => Some('('),
        Keycode::RIGHTPAREN => Some(')'),
        Keycode::ASTERISK => Some('*'),
        Keycode::PLUS => Some('+'),
        Keycode::COMMA => Some(','),
        Keycode::MINUS => Some('-'),
        Keycode::SLASH => Some('/'),
        Keycode::NUM_0 => Some('0'),
        Keycode::NUM_1 => Some('1'),
        Keycode::NUM_2 => Some('2'),
        Keycode::NUM_3 => Some('3'),
        Keycode::NUM_4 => Some('4'),
        Keycode::NUM_5 => Some('5'),
        Keycode::NUM_6 => Some('6'),
        Keycode::NUM_7 => Some('7'),
        Keycode::NUM_8 => Some('8'),
        Keycode::NUM_9 => Some('9'),
        Keycode::COLON => Some(':'),
        Keycode::SEMICOLON => Some(';'),
        Keycode::LESS => Some('<'),
        Keycode::EQUALS => Some('='),
        Keycode::GREATER => Some('>'),
        Keycode::QUESTION => Some('?'),
        Keycode::AT => Some('@'),
        Keycode::LEFTBRACKET => Some('{'),
        Keycode::BACKSLASH => Some('\\'),
        Keycode::RIGHTBRACKET => Some('}'),
        Keycode::CARET => Some('^'),
        Keycode::UNDERSCORE => Some('_'),
        Keycode::BACKQUOTE => Some('`'),
        // Keycode::TILDE => Some('~'),
        Keycode::A
        | Keycode::B
        | Keycode::C
        | Keycode::D
        | Keycode::E
        | Keycode::F
        | Keycode::G
        | Keycode::H
        | Keycode::I
        | Keycode::J
        | Keycode::K
        | Keycode::L
        | Keycode::M
        | Keycode::N
        | Keycode::O
        | Keycode::P
        | Keycode::Q
        | Keycode::R
        | Keycode::S
        | Keycode::T
        | Keycode::U
        | Keycode::V
        | Keycode::W
        | Keycode::X
        | Keycode::Y
        | Keycode::Z => {
            let char = keycode.name().chars().next().unwrap();

            let mut uppercase =
                pressed_keys.contains(&Keycode::LShift) || pressed_keys.contains(&Keycode::RShift);

            if pressed_keys.contains(&Keycode::CapsLock) {
                uppercase = !uppercase;
            }

            if uppercase {
                Some(char)
            } else {
                let lowercase = char.to_lowercase().next().unwrap();

                Some(lowercase)
            }
        }
        _ => None,
    }
}

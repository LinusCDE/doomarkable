use super::{ButtonAction, Element, Layout, LayoutId};
use libremarkable::framebuffer::common;

struct KeyDefinition {
    weighted_width: f32,
    title: &'static str,
    primary_letter: char,
}

impl KeyDefinition {
    fn new(weighted_width: f32, title: &'static str, primary_letter: char) -> Self {
        Self {
            weighted_width,
            title,
            primary_letter,
        }
    }
}

fn create_keyboard_definitions() -> Vec<Vec<KeyDefinition>> {
    vec![
        // Row 1
        vec![
            KeyDefinition::new(61.0, "^", '^'),
            KeyDefinition::new(61.0, "1", '1'),
            KeyDefinition::new(61.0, "2", '2'),
            KeyDefinition::new(61.0, "3", '3'),
            KeyDefinition::new(61.0, "4", '4'),
            KeyDefinition::new(61.0, "5", '5'),
            KeyDefinition::new(61.0, "6", '6'),
            KeyDefinition::new(61.0, "7", '7'),
            KeyDefinition::new(61.0, "8", '8'),
            KeyDefinition::new(61.0, "9", '9'),
            KeyDefinition::new(61.0, "0", '0'),
            KeyDefinition::new(61.0, "-", '-'),
            KeyDefinition::new(61.0, "+", '+'),
            KeyDefinition::new(121.0, "Backspace", '\x7f'),
        ],
        // Row 2
        vec![
            KeyDefinition::new(91.0, "Tab", '\x09'),
            KeyDefinition::new(61.0, "q", 'q'),
            KeyDefinition::new(61.0, "w", 'w'),
            KeyDefinition::new(61.0, "e", 'e'),
            KeyDefinition::new(61.0, "r", 'r'),
            KeyDefinition::new(61.0, "t", 't'),
            KeyDefinition::new(61.0, "y", 'y'),
            KeyDefinition::new(61.0, "u", 'u'),
            KeyDefinition::new(61.0, "i", 'i'),
            KeyDefinition::new(61.0, "o", 'o'),
            KeyDefinition::new(61.0, "p", 'p'),
            KeyDefinition::new(61.0, "" /*"{"*/, '\0'),
            KeyDefinition::new(61.0, "" /*"}"*/, '\0'),
            KeyDefinition::new(91.0, "" /*"|"*/, '\0'),
        ],
        // Row 3
        vec![
            KeyDefinition::new(106.0, "Caps Lock", 0xba as char),
            KeyDefinition::new(61.0, "a", 'a'),
            KeyDefinition::new(61.0, "s", 's'),
            KeyDefinition::new(61.0, "d", 'd'),
            KeyDefinition::new(61.0, "f", 'f'),
            KeyDefinition::new(61.0, "g", 'g'),
            KeyDefinition::new(61.0, "h", 'h'),
            KeyDefinition::new(61.0, "j", 'j'),
            KeyDefinition::new(61.0, "k", 'k'),
            KeyDefinition::new(61.0, "l", 'l'),
            KeyDefinition::new(61.0, ":", ':'),
            KeyDefinition::new(61.0, "\"", '\"'),
            KeyDefinition::new(136.0, "Enter", '\x0d'),
        ],
        // Row 4
        vec![
            KeyDefinition::new(136.0, "Shift", '\x0e'), // Shift in??
            KeyDefinition::new(61.0, "z", 'z'),
            KeyDefinition::new(61.0, "x", 'x'),
            KeyDefinition::new(61.0, "c", 'c'),
            KeyDefinition::new(61.0, "v", 'v'),
            KeyDefinition::new(61.0, "b", 'b'),
            KeyDefinition::new(61.0, "n", 'n'),
            KeyDefinition::new(61.0, "m", 'm'),
            KeyDefinition::new(61.0, "<", '<'),
            KeyDefinition::new(61.0, ">", '>'),
            KeyDefinition::new(61.0, "?", '?'),
            KeyDefinition::new(166.0, "Shift", 0xb6 as char), // Shift in??
        ],
        // Row 5
        vec![
            KeyDefinition::new(91.0, "Ctrl", 0x9d as char),
            KeyDefinition::new(61.0, /*"Super"*/ "", '\0'),
            KeyDefinition::new(91.0, "Alt", 0xb8 as char),
            KeyDefinition::new(361.0, "Space", ' '),
            KeyDefinition::new(91.0, "Alt", 0xb8 as char),
            KeyDefinition::new(61.0, "" /*"Super"*/, '\0'),
            KeyDefinition::new(61.0, "" /*"Menu"*/, '\0'),
            KeyDefinition::new(91.0, "Ctrl", 0x9d as char),
        ],
    ]
}

pub fn create() -> Layout {
    let mut buttons = vec![Element::Button {
        rect: common::mxcfb_rect {
            left: 1404 - 62 - 100,
            top: 1400 - 300 - 10 - 10,
            width: 100,
            height: 50,
        },
        label: "Back",
        label_size: 25.0,
        action: ButtonAction::SwitchLayout(LayoutId::Controls),
    }];

    let keys = create_keyboard_definitions();
    let weighted_height = 61f32;
    //let height_weight_sum = weighted_height * keys.len() as f32;
    let width_weight_sum = keys[0].iter().map(|k| k.weighted_width).sum::<f32>();

    let mut y = (1400 - 300 - 10 - 10 + 50 + 100) as f32;

    let width_factor = (common::DISPLAYWIDTH as f32 - (62 * 2) as f32) / width_weight_sum;
    //let height_factor = ((common::DISPLAYHEIGHT as u32 - 62) as f32 - y) / height_weight_sum;
    let height_factor = width_factor; // Square keys

    for row in keys {
        let mut x = 62f32;
        for key in row {
            buttons.push(Element::Button {
                rect: common::mxcfb_rect {
                    left: x as u32,
                    top: y as u32,
                    width: (key.weighted_width * width_factor) as u32,
                    height: (weighted_height * height_factor) as u32,
                },
                label: key.title,
                label_size: 25.0,
                action: ButtonAction::DoomKey(key.primary_letter as u8),
            });
            x += key.weighted_width * width_factor;
        }

        y += weighted_height * height_factor;
    }

    Layout::new(buttons)
}

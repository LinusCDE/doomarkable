use super::{ButtonAction, Element, Layout, LayoutId};
use doomgeneric::input::keys;
use libremarkable::framebuffer::common;

pub fn create() -> Layout {
    let buttons = vec![
        Element::Button {
            rect: common::mxcfb_rect {
                left: 722,
                top: 1400,
                width: 200,
                height: 200 + 10 + 200,
            },
            label: "<",
            label_size: 100.0,
            action: ButtonAction::DoomKey(*keys::KEY_LEFT),
        },
        Element::Button {
            rect: common::mxcfb_rect {
                left: 722 + 200 + 10,
                top: 1400,
                width: 200,
                height: 200,
            },
            label: "^",
            label_size: 100.0,
            action: ButtonAction::DoomKey(*keys::KEY_UP),
        },
        Element::Button {
            rect: common::mxcfb_rect {
                left: 722 + 200 + 10,
                top: 1400 + 200 + 10,
                width: 200,
                height: 200,
            },
            label: "v",
            label_size: 100.0,
            action: ButtonAction::DoomKey(*keys::KEY_DOWN),
        },
        Element::Button {
            rect: common::mxcfb_rect {
                left: 722 + 200 + 10 + 200 + 10,
                top: 1400,
                width: 200,
                height: 200 + 10 + 200,
            },
            label: ">",
            label_size: 100.0,
            action: ButtonAction::DoomKey(*keys::KEY_RIGHT),
        },
        Element::Button {
            rect: common::mxcfb_rect {
                left: 62,
                top: 1400,
                width: 300,
                height: 200 + 10 + 200,
            },
            label: "Strafe",
            label_size: 25.0,
            action: ButtonAction::DoomKey(*keys::KEY_STRAFE),
        },
        Element::Button {
            rect: common::mxcfb_rect {
                left: 62 + 300 + 10,
                top: 1400,
                width: 300,
                height: 200 + 10 + 200,
            },
            label: "Fire",
            label_size: 25.0,
            action: ButtonAction::DoomKey(*keys::KEY_FIRE),
        },
        Element::Button {
            rect: common::mxcfb_rect {
                left: 62,
                top: 1400 - 10 - 150 - 10 - 150,
                width: 300,
                height: 150,
            },
            label: "ESC",
            label_size: 25.0,
            action: ButtonAction::DoomKey(keys::KEY_ESCAPE),
        },
        Element::Button {
            rect: common::mxcfb_rect {
                left: 62,
                top: 1400 - 150 - 10,
                width: 300,
                height: 150,
            },
            label: "Enter",
            label_size: 25.0,
            action: ButtonAction::DoomKey(keys::KEY_ENTER),
        },
        Element::Button {
            rect: common::mxcfb_rect {
                left: 62 + 300 + 10,
                top: 1400 - 300 - 10 - 10,
                width: 300,
                height: 150 + 10 + 150,
            },
            label: "Use",
            label_size: 25.0,
            action: ButtonAction::DoomKey(*keys::KEY_USE),
        },
        Element::Button {
            rect: common::mxcfb_rect {
                left: 1404 - 62 - 100,
                top: 1400 - 300 - 10 - 10,
                width: 100,
                height: 50,
            },
            label: "Settings",
            label_size: 25.0,
            action: ButtonAction::SwitchLayout(LayoutId::Settings),
        },
    ];

    Layout::new(buttons)
}

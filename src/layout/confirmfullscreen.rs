use super::{ButtonAction, Element, Layout, LayoutId};
use libremarkable::framebuffer::common;

pub fn create() -> Layout {
    let buttons = vec![
        Element::Text {
            rect: common::mxcfb_rect {
                left: 0,
                top: 1400 - 300 - 10 - 10,
                width: common::DISPLAYWIDTH as u32,
                height: 100,
            },
            text: "Go Fullscreen?",
            size: 100.0,
        },
        Element::Text {
            rect: common::mxcfb_rect {
                left: 0,
                top: 1400 - 300 - 10 - 10 + 100,
                width: common::DISPLAYWIDTH as u32,
                height: 100,
            },
            text: "You'll need to attach a physical keyboard to play!",
            size: 50.0,
        },
        Element::Text {
            rect: common::mxcfb_rect {
                left: 0,
                top: 1400 - 300 - 10 - 10 + 100 + 75,
                width: common::DISPLAYWIDTH as u32,
                height: 100,
            },
            text: "To exit fullscreen, just touch anywhere.",
            size: 50.0,
        },
        Element::Button {
            rect: common::mxcfb_rect {
                left: (common::DISPLAYWIDTH as u32 - (300 + 50 + 300)) / 2,
                top: 1400 - 300 - 10 - 10 + 100 + 75 + 75 + 50,
                width: 300,
                height: 150,
            },
            label: "OK",
            label_size: 75.0,
            action: ButtonAction::EnterFullscreen,
        },
        Element::Button {
            rect: common::mxcfb_rect {
                left: (common::DISPLAYWIDTH as u32 - (300 + 50 + 300)) / 2 + 300 + 50,
                top: 1400 - 300 - 10 - 10 + 100 + 75 + 75 + 50,
                width: 300,
                height: 150,
            },
            label: "Back",
            label_size: 75.0,
            action: ButtonAction::SwitchLayout(LayoutId::Settings),
        },
    ];

    Layout::new(buttons)
}

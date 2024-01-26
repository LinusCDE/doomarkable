use super::{ButtonAction, Element, Layout, LayoutId};
use crate::FB;
use libremarkable::framebuffer::{common, FramebufferRefresh};

pub fn create() -> Layout {
    let buttons = vec![
        Element::Button {
            rect: common::mxcfb_rect {
                left: 1404 - 62 - 100,
                top: 1400 - 300 - 10 - 10,
                width: 100,
                height: 50,
            },
            label: "Back",
            label_size: 25.0,
            action: ButtonAction::SwitchLayout(LayoutId::Controls),
        },
        Element::Text {
            rect: common::mxcfb_rect {
                left: 0,
                top: 1400 - 300 - 10 - 10,
                width: common::DISPLAYWIDTH as u32,
                height: 100,
            },
            text: "Settings",
            size: 100.0,
        },
        Element::Button {
            rect: common::mxcfb_rect {
                left: 62,
                top: 1400 - 300 - 10 + 100 + 10 + (100 + 10) * 0,
                width: 400,
                height: 100,
            },
            label: "Full refresh",
            label_size: 50.0,
            action: ButtonAction::Function(Box::new(|| {
                FB.lock().unwrap().full_refresh(
                    common::waveform_mode::WAVEFORM_MODE_GC16,
                    common::display_temp::TEMP_USE_MAX,
                    common::dither_mode::EPDC_FLAG_USE_REMARKABLE_DITHER,
                    0,
                    true,
                );
            })),
        },
        Element::Button {
            rect: common::mxcfb_rect {
                left: 62,
                top: 1400 - 300 - 10 + 100 + 10 + (100 + 10) * 1,
                width: 400,
                height: 100,
            },
            label: "Fullscreen",
            label_size: 50.0,
            action: ButtonAction::SwitchLayout(LayoutId::ConfirmFullscreen),
        },
        Element::Button {
            rect: common::mxcfb_rect {
                left: 62,
                top: 1400 - 300 - 10 + 100 + 10 + (100 + 10) * 2,
                width: 400,
                height: 100,
            },
            label: "Exit",
            label_size: 50.0,
            action: ButtonAction::SwitchLayout(LayoutId::ConfirmExit),
        },
    ];

    Layout::new(buttons)
}

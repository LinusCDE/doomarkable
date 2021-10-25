use crate::FB;
use doomgeneric::input::{keys, KeyData};
use libremarkable::cgmath::{Point2, Vector2};
use libremarkable::framebuffer::common;
use libremarkable::framebuffer::core::Framebuffer;
use libremarkable::framebuffer::{
    refresh::PartialRefreshMode, FramebufferDraw, FramebufferIO, FramebufferRefresh,
};
use libremarkable::input::{multitouch::Finger, multitouch::MultitouchEvent, InputEvent};

pub enum InputOutcome {
    KeyData(KeyData),
    SwitchLayout(LayoutId),
}

pub struct LayoutManager {
    layouts: fxhash::FxHashMap<LayoutId, Layout>,
    current_layout_id: LayoutId,
}

impl LayoutManager {
    pub fn new() -> Self {
        let mut layouts: fxhash::FxHashMap<LayoutId, Layout> = Default::default();

        layouts.insert(LayoutId::Controls, create_layout_controls());

        let instance = Self {
            layouts,
            current_layout_id: Default::default(),
        };
        instance.current_layout().render(&mut FB.lock().unwrap());
        instance.current_layout().refresh(&mut FB.lock().unwrap());

        instance
    }

    pub fn current_layout(&self) -> &Layout {
        self.layouts.get(&self.current_layout_id).unwrap()
    }

    pub fn current_layout_mut(&mut self) -> &mut Layout {
        self.layouts.get_mut(&self.current_layout_id).unwrap()
    }

    pub fn switch_layout(&mut self, new_layout: LayoutId) {
        if new_layout == self.current_layout_id {
            return;
        }
        let mut fb = FB.lock().unwrap();

        self.current_layout().clear(&mut fb);
        self.current_layout().refresh(&mut fb);

        self.current_layout_id = new_layout;

        self.current_layout().clear(&mut fb);
        self.current_layout().render(&mut fb);
        self.current_layout().refresh(&mut fb);
        self.current_layout_id = new_layout;
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum LayoutId {
    Controls,
}

impl Default for LayoutId {
    fn default() -> Self {
        LayoutId::Controls
    }
}

pub struct Layout {
    buttons: Vec<Button>,

    // Input tracking
    fingers: fxhash::FxHashMap<i32, Finger>,
    pressed_indices: fxhash::FxHashSet<usize>,
}

impl Layout {
    fn new(buttons: Vec<Button>) -> Self {
        Self {
            buttons,
            fingers: Default::default(),
            pressed_indices: Default::default(),
        }
    }

    pub fn get_area(&self) -> common::mxcfb_rect {
        let mut left = 9999;
        let mut top = 9999;
        let mut bottom = 0;
        let mut right = 0;

        for button in &self.buttons {
            left = left.min(button.rect.left);
            top = top.min(button.rect.top);
            right = right.max(button.rect.left + button.rect.width);
            bottom = bottom.max(button.rect.top + button.rect.height);
        }

        assert!(left < right);
        assert!(top < bottom);

        common::mxcfb_rect {
            left,
            top,
            width: right - left,
            height: bottom - top,
        }
    }

    pub fn render(&self, fb: &mut Framebuffer) {
        for button in &self.buttons {
            fb.draw_rect(
                Point2 {
                    x: button.rect.left as i32,
                    y: button.rect.top as i32,
                },
                Vector2 {
                    x: button.rect.width,
                    y: button.rect.height,
                },
                3,
                common::color::BLACK,
            );

            let rect = fb.draw_text(
                Point2 { x: 0f32, y: 500f32 },
                &button.label,
                button.label_size,
                common::color::BLACK,
                true,
            );
            fb.draw_text(
                Point2 {
                    x: (button.rect.left as f32 + (button.rect.width - rect.width) as f32 / 2.0),
                    y: (button.rect.top as f32 + (button.rect.height + rect.height) as f32 / 2.0),
                },
                &button.label,
                button.label_size,
                common::color::BLACK,
                false,
            );
        }
    }

    pub fn refresh(&self, fb: &mut Framebuffer) {
        fb.partial_refresh(
            &self.get_area(),
            PartialRefreshMode::Wait,
            common::waveform_mode::WAVEFORM_MODE_GC16_FAST,
            common::display_temp::TEMP_USE_MAX,
            common::dither_mode::EPDC_FLAG_USE_REMARKABLE_DITHER,
            0,
            false,
        );
    }

    pub fn clear(&self, fb: &mut Framebuffer) {
        // Turn area white
        fb.restore_region(
            self.get_area(),
            &vec![0xFF; self.get_area().width as usize * 2 * self.get_area().height as usize],
        )
        .unwrap();
    }

    pub fn handle_input(&mut self, event: InputEvent) -> Vec<InputOutcome> {
        match event {
            InputEvent::MultitouchEvent { event } => match event {
                MultitouchEvent::Press { finger } => {
                    self.fingers.insert(finger.tracking_id, finger);
                    self.find_updates()
                }
                MultitouchEvent::Move { finger } => {
                    self.fingers.insert(finger.tracking_id, finger);
                    self.find_updates()
                }
                MultitouchEvent::Release { finger } => {
                    self.fingers.remove(&finger.tracking_id);
                    self.find_updates()
                }
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }

    fn find_updates(&mut self) -> Vec<InputOutcome> {
        let mut events = vec![];
        let last_pressed_indices = self.pressed_indices.clone();

        self.pressed_indices.clear();
        for finger in self.fingers.values() {
            for (i, button) in self.buttons.iter().enumerate() {
                if finger.pos.x as u32 >= button.rect.left
                    && finger.pos.x as u32 <= button.rect.left + button.rect.width
                    && finger.pos.y as u32 >= button.rect.top
                    && finger.pos.y as u32 <= button.rect.top + button.rect.height
                {
                    self.pressed_indices.insert(i);
                    break;
                }
            }
        }

        for key_up_index in last_pressed_indices.difference(&self.pressed_indices) {
            let button = &self.buttons[*key_up_index];
            match button.action {
                ButtonAction::DoomKey(key) => {
                    #[rustfmt::skip]
                    events.push(InputOutcome::KeyData(KeyData { key, pressed: false }));
                }
                _ => unimplemented!(),
            }
        }

        for key_down_index in self.pressed_indices.difference(&last_pressed_indices) {
            let button = &self.buttons[*key_down_index];
            match button.action {
                ButtonAction::DoomKey(key) => {
                    events.push(InputOutcome::KeyData(KeyData { key, pressed: true }));
                }
                _ => unimplemented!(),
            }
        }

        events
    }
}

struct Button {
    rect: common::mxcfb_rect,
    label: &'static str,
    label_size: f32,
    action: ButtonAction,
}

enum ButtonAction {
    DoomKey(u8),
    Function(Box<dyn Fn()>),
    SwitchLayout(LayoutId),
}

fn create_layout_controls() -> Layout {
    let buttons = vec![
        Button {
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
        Button {
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
        Button {
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
        Button {
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
        Button {
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
        Button {
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
        Button {
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
        Button {
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
        Button {
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
    ];

    Layout::new(buttons)
}

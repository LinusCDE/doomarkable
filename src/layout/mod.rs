//! Mostly the ui-like stuff below the game

use doomgeneric::input::KeyData;
use libremarkable::cgmath::{Point2, Vector2};
use libremarkable::framebuffer::core::Framebuffer;
use libremarkable::framebuffer::{common, PartialRefreshMode};
use libremarkable::framebuffer::{FramebufferDraw, FramebufferIO, FramebufferRefresh};
use libremarkable::input::{Finger, InputEvent, MultitouchEvent};

mod confirmexit;
mod confirmfullscreen;
mod controls;
mod keyboard;
mod settings;

pub enum InputOutcome {
    KeyData(KeyData),
    SwitchLayout(LayoutId),
    EnterFullscreen,
}

pub struct LayoutManager {
    layouts: fxhash::FxHashMap<LayoutId, Layout>,
    current_layout_id: LayoutId,
}

fn combined_rect(mut rect_iter: impl Iterator<Item = common::mxcfb_rect>) -> common::mxcfb_rect {
    let mut left = 9999;
    let mut top = 9999;
    let mut bottom = 0;
    let mut right = 0;

    while let Some(rect) = rect_iter.next() {
        left = left.min(rect.left);
        top = top.min(rect.top);
        right = right.max(rect.left + rect.width);
        bottom = bottom.max(rect.top + rect.height);
    }

    assert!(left < right);
    assert!(top < bottom);
    assert!(right <= common::DISPLAYWIDTH as u32);
    assert!(bottom <= common::DISPLAYHEIGHT as u32);

    common::mxcfb_rect {
        left,
        top,
        width: right - left,
        height: bottom - top,
    }
}

impl LayoutManager {
    pub fn new(fb: &mut Framebuffer) -> Self {
        let mut layouts: fxhash::FxHashMap<LayoutId, Layout> = Default::default();

        // Create and add layouts
        layouts.insert(LayoutId::Controls, controls::create());
        layouts.insert(LayoutId::Settings, settings::create());
        layouts.insert(LayoutId::ConfirmExit, confirmexit::create());
        layouts.insert(LayoutId::Keyboard, keyboard::create());
        layouts.insert(LayoutId::ConfirmFullscreen, confirmfullscreen::create());

        let instance = Self {
            layouts,
            current_layout_id: Default::default(),
        };
        instance.current_layout().render(fb);
        instance.refresh(&instance.current_layout().get_area(), fb);

        instance
    }

    pub fn current_layout(&self) -> &Layout {
        self.layouts.get(&self.current_layout_id).unwrap()
    }

    pub fn current_layout_mut(&mut self) -> &mut Layout {
        self.layouts.get_mut(&self.current_layout_id).unwrap()
    }

    pub fn switch_layout(&mut self, new_layout: LayoutId, fb: &mut Framebuffer) {
        self.current_layout().clear(fb);
        let old_area = self.current_layout().get_area();

        self.current_layout_id = new_layout;

        self.current_layout().clear(fb);
        self.current_layout().render(fb);
        let new_ara = self.current_layout().get_area();
        self.refresh(&combined_rect([old_area, new_ara].iter().map(|r| *r)), fb);
        self.current_layout_id = new_layout;
    }

    fn refresh(&self, area: &common::mxcfb_rect, fb: &mut Framebuffer) {
        fb.partial_refresh(
            area,
            PartialRefreshMode::Wait,
            common::waveform_mode::WAVEFORM_MODE_GC16_FAST,
            common::display_temp::TEMP_USE_AMBIENT,
            common::dither_mode::EPDC_FLAG_USE_DITHERING_PASSTHROUGH,
            0,
            false,
        );
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum LayoutId {
    Controls,
    Settings,
    ConfirmExit,
    Keyboard,
    ConfirmFullscreen,
}

impl Default for LayoutId {
    fn default() -> Self {
        LayoutId::Controls
    }
}

pub struct Layout {
    elements: Vec<Element>,

    // Input tracking
    fingers: fxhash::FxHashMap<i32, Finger>,
    pressed_indices: fxhash::FxHashSet<usize>,
}

impl Layout {
    fn new(elements: Vec<Element>) -> Self {
        Self {
            elements,
            fingers: Default::default(),
            pressed_indices: Default::default(),
        }
    }

    pub fn get_area(&self) -> common::mxcfb_rect {
        combined_rect(self.elements.iter().map(|el| *el.rect()))
    }

    pub fn render(&self, fb: &mut Framebuffer) {
        for element in &self.elements {
            match element {
                Element::Button {
                    label,
                    label_size,
                    rect,
                    ..
                } => {
                    fb.draw_rect(
                        Point2 {
                            x: rect.left as i32 + 2,
                            y: rect.top as i32 + 2,
                        },
                        Vector2 {
                            x: rect.width - 4,
                            y: rect.height - 4,
                        },
                        3,
                        common::color::BLACK,
                    );

                    let text_rect = fb.draw_text(
                        Point2 { x: 0f32, y: 500f32 },
                        label,
                        *label_size,
                        common::color::BLACK,
                        true,
                    );

                    fb.draw_text(
                        Point2 {
                            x: (rect.left as f32 + (rect.width - text_rect.width) as f32 / 2.0),
                            y: (rect.top as f32 + (rect.height - text_rect.height) as f32 / 2.0)
                                + text_rect.height as f32,
                        },
                        label,
                        *label_size,
                        common::color::BLACK,
                        false,
                    );
                }
                Element::Text { text, size, rect } => {
                    let text_rect = fb.draw_text(
                        Point2 { x: 0f32, y: 500f32 },
                        text,
                        *size,
                        common::color::BLACK,
                        true,
                    );

                    fb.draw_text(
                        Point2 {
                            x: (rect.left as f32 + (rect.width - text_rect.width) as f32 / 2.0),
                            y: (rect.top as f32 + (rect.height - text_rect.height) as f32 / 2.0)
                                + text_rect.height as f32,
                        },
                        text,
                        *size,
                        common::color::BLACK,
                        false,
                    );
                }
            }
        }
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
        let mut outcomes = match event {
            InputEvent::MultitouchEvent { event } => match event {
                MultitouchEvent::Press { finger } => {
                    self.fingers.insert(finger.tracking_id, finger);
                    self.process_fingers()
                }
                MultitouchEvent::Move { finger } => {
                    self.fingers.insert(finger.tracking_id, finger);
                    self.process_fingers()
                }
                MultitouchEvent::Release { finger } => {
                    self.fingers.remove(&finger.tracking_id);
                    self.process_fingers()
                }
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        };

        // Fake all fingers released before switching a layout to prevent stuck keys
        let mut i = 0;
        while i < outcomes.len() {
            if let InputOutcome::SwitchLayout(_) = &outcomes[i] {
                self.fingers.clear();
                for outcome in self.process_fingers() {
                    outcomes.insert(i, outcome);
                    i += 1;
                }
            }
            i += 1;
        }

        outcomes
    }

    fn process_fingers(&mut self) -> Vec<InputOutcome> {
        let mut outcomes = vec![];
        let last_pressed_indices = self.pressed_indices.clone();

        self.pressed_indices.clear();
        for finger in self.fingers.values() {
            for (i, element) in self.elements.iter().enumerate() {
                if finger.pos.x as u32 >= element.rect().left
                    && finger.pos.x as u32 <= element.rect().left + element.rect().width
                    && finger.pos.y as u32 >= element.rect().top
                    && finger.pos.y as u32 <= element.rect().top + element.rect().height
                {
                    self.pressed_indices.insert(i);
                    break;
                }
            }
        }

        for key_up_index in last_pressed_indices.difference(&self.pressed_indices) {
            if let Element::Button { action, .. } = &self.elements[*key_up_index] {
                match action {
                    ButtonAction::DoomKey(key) => {
                        outcomes.push(InputOutcome::KeyData(KeyData {
                            key: *key,
                            pressed: false,
                        }));
                    }
                    ButtonAction::Function(func) => {
                        func();
                    }
                    ButtonAction::SwitchLayout(layout_id) => {
                        outcomes.push(InputOutcome::SwitchLayout(*layout_id));
                    }
                    ButtonAction::EnterFullscreen => {
                        outcomes.push(InputOutcome::EnterFullscreen);
                    }
                }
            }
        }

        for key_down_index in self.pressed_indices.difference(&last_pressed_indices) {
            if let Element::Button { action, .. } = &self.elements[*key_down_index] {
                match action {
                    ButtonAction::DoomKey(key) => {
                        outcomes.push(InputOutcome::KeyData(KeyData {
                            key: *key,
                            pressed: true,
                        }));
                    }

                    ButtonAction::Function(_) => {}
                    ButtonAction::SwitchLayout(_) => {}
                    ButtonAction::EnterFullscreen => {}
                }
            }
        }

        outcomes
    }
}

enum Element {
    Button {
        rect: common::mxcfb_rect,
        label: &'static str,
        label_size: f32,
        action: ButtonAction,
    },
    Text {
        rect: common::mxcfb_rect,
        text: &'static str,
        size: f32,
    },
}

impl Element {
    fn rect(&self) -> &common::mxcfb_rect {
        match self {
            Element::Button { rect, .. } => rect,
            Element::Text { rect, .. } => rect,
        }
    }
}

enum ButtonAction {
    DoomKey(u8),
    Function(Box<dyn Fn()>),
    SwitchLayout(LayoutId),
    EnterFullscreen,
}

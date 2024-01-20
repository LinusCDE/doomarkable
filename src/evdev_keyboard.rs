use std::{path::Path, sync::mpsc::Sender};

use doomgeneric::input::KeyData;
use evdev::Key;

const DEV_INPUT_DIR: &str = "/dev/input";

pub fn init(keydata_tx: Sender<KeyData>) {
    scan_for_existing_keyboards(&keydata_tx);
    spawn_keyboard_watcher(keydata_tx);
}

fn scan_for_existing_keyboards(keydata_tx: &Sender<KeyData>) {
    // Find existing unknown evdev devices in /dev/input
    for entry in std::fs::read_dir(DEV_INPUT_DIR).expect("Listing files of input devices dir") {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let filename = entry.file_name().to_str().unwrap().to_owned();
        let path = Path::new(DEV_INPUT_DIR).join(&filename);
        if path.is_dir()
            || !filename.starts_with("event")
            || libremarkable::input::scan::SCANNED.gpio_path == path
            || libremarkable::input::scan::SCANNED.wacom_path == path
            || libremarkable::input::scan::SCANNED.multitouch_path == path
        {
            continue; // Skip directories or known input devices (gpio, mt, wacom)
        }
        info!("Existing evdev device detected: {path:?}");
        spawn_evdev_keyboard(path, keydata_tx.clone());
    }
}

fn spawn_keyboard_watcher(keydata_tx: Sender<KeyData>) {
    // Listen for new devices in /dev/input to allow hotplugging keyboards
    std::thread::spawn(move || {
        let mut inotify = match inotify::Inotify::init() {
            Ok(inotify) => {
                if let Err(err) = inotify
                    .watches()
                    .add(DEV_INPUT_DIR, inotify::WatchMask::CREATE)
                {
                    error!("Could initialize inotify, however adding a watch for new files in {DEV_INPUT_DIR:?} failed: {err:?}");
                    return;
                }
                inotify
            }
            Err(err) => {
                error!("Could not initialize inotify: {err:?}");
                return;
            }
        };

        let mut inotify_buffer = [0u8; 4096];
        loop {
            let inotify_events = match inotify.read_events_blocking(&mut inotify_buffer) {
                Ok(events) => events,
                Err(err) => {
                    error!("Encountered an issue while reading inotify events! Keyboard hotplugging will not work anymore! {:?}", err);
                    return;
                }
            };

            for event in inotify_events {
                if !event.mask.contains(inotify::EventMask::CREATE)
                    || event.mask.contains(inotify::EventMask::ISDIR)
                {
                    continue; // Make sure this is a new file being created
                }

                let filename = event.name.unwrap().to_str().unwrap();
                if !filename.starts_with("event") {
                    continue; // Ignore things like "touchscreen0", "mouse0", etc.
                }
                let path = Path::new(DEV_INPUT_DIR).join(filename);
                info!("New evdev device detected: {path:?}");
                spawn_evdev_keyboard(path, keydata_tx.clone());
            }
        }
    });
}

// Check if device is a keyboard, spawn new thread and listen for keystrokes and send them to keydata_tx
fn spawn_evdev_keyboard(path: impl AsRef<Path>, keydata_tx: std::sync::mpsc::Sender<KeyData>) {
    // Check if this device is a valid keyboard
    let mut device = match evdev::Device::open(&path) {
        Ok(device) => device,
        Err(err) => {
            error!("Failed opening evdev device {:?}! {:?}", path.as_ref(), err);
            return;
        }
    };
    if !device.supported_events().contains(evdev::EventType::KEY)
        || [
            evdev::Key::KEY_Q,
            evdev::Key::KEY_W,
            evdev::Key::KEY_E,
            evdev::Key::KEY_R,
            evdev::Key::KEY_T,
            evdev::Key::KEY_Y,
        ]
        .iter()
        .any(|key| {
            !device
                .supported_keys()
                .map(|keys| keys.contains(*key))
                .unwrap_or(false)
        })
    {
        info!("The evdev device {:?} is not a keyboard.", path.as_ref());
        return;
    }

    let path = path.as_ref().to_path_buf();
    // Listen for keys in new thread
    std::thread::spawn(move || {
        let name = device.name().unwrap().to_owned();
        info!("Keyboard at {path:?} detected: {name}");

        loop {
            let evs: Vec<evdev::InputEvent> = match device.fetch_events() {
                Ok(evs) => evs,
                Err(err) => {
                    debug!("Lost connection to keyboard {name} ({path:?}). It likely got disconnected. Error: {err}");
                    info!("Keyboard disconnected: {name}");
                    return;
                }
            }.collect();

            for ev in evs {
                if let evdev::InputEventKind::Key(key) = ev.kind() {
                    if ev.value() != 0 && ev.value() != 1 {
                        continue; // Ignore key being held (value == 2) and other potential values.
                    }

                    if let Ok((keycode, scancodes)) = device.get_scancode_by_index(key.0) {
                        debug!(
                            "{} ({:?}, keycode: {keycode}, scancodes: {scancodes:?}) => {}",
                            key.0,
                            key,
                            ev.value()
                        );
                    } else {
                        debug!("{} ({:?}) => {}", key.0, key, ev.value());
                    }

                    if let Some(doom_key_code) = map_evdev_key_to_doom(key) {
                        keydata_tx
                            .send(KeyData {
                                key: doom_key_code,
                                pressed: ev.value() == 1,
                            })
                            .ok();
                    }
                }
            }
        }
    });
}

fn map_evdev_key_to_doom(key: Key) -> Option<u8> {
    // https://github.com/ozkl/doomgeneric/blob/613f870b6fa83ede448a247de5a2571092fa729d/doomgeneric/doomkeys.h
    Some(match key {
        Key::KEY_A => 'a' as u8,
        Key::KEY_B => 'b' as u8,
        Key::KEY_C => 'c' as u8,
        Key::KEY_D => 'd' as u8,
        Key::KEY_E => 'e' as u8,
        Key::KEY_F => 'f' as u8,
        Key::KEY_G => 'g' as u8,
        Key::KEY_H => 'h' as u8,
        Key::KEY_I => 'i' as u8,
        Key::KEY_J => 'j' as u8,
        Key::KEY_K => 'k' as u8,
        Key::KEY_L => 'l' as u8,
        Key::KEY_M => 'm' as u8,
        Key::KEY_N => 'n' as u8,
        Key::KEY_O => 'o' as u8,
        Key::KEY_P => 'p' as u8,
        Key::KEY_Q => 'q' as u8,
        Key::KEY_R => 'r' as u8,
        Key::KEY_S => 's' as u8,
        Key::KEY_T => 't' as u8,
        Key::KEY_U => 'u' as u8,
        Key::KEY_V => 'v' as u8,
        Key::KEY_W => 'w' as u8,
        Key::KEY_X => 'x' as u8,
        Key::KEY_Y => 'y' as u8,
        Key::KEY_Z => 'z' as u8,
        Key::KEY_0 => '0' as u8,
        Key::KEY_1 => '1' as u8,
        Key::KEY_2 => '2' as u8,
        Key::KEY_3 => '3' as u8,
        Key::KEY_4 => '4' as u8,
        Key::KEY_5 => '5' as u8,
        Key::KEY_6 => '6' as u8,
        Key::KEY_7 => '7' as u8,
        Key::KEY_8 => '8' as u8,
        Key::KEY_9 => '9' as u8,

        Key::KEY_RIGHT => 0xae,
        Key::KEY_LEFT => 0xac,
        Key::KEY_UP => 0xad,
        Key::KEY_DOWN => 0xaf,
        Key::KEY_DOT => 0xa0,   // Strafe left
        Key::KEY_COMMA => 0xa1, // Strafe right
        // USE
        // FIRE
        Key::KEY_ESC => 27,
        Key::KEY_ENTER => 13,
        Key::KEY_TAB => 9,
        Key::KEY_F1 => 0x80 + 0x3b,
        Key::KEY_F2 => 0x80 + 0x3c,
        Key::KEY_F3 => 0x80 + 0x3d,
        Key::KEY_F4 => 0x80 + 0x3e,
        Key::KEY_F5 => 0x80 + 0x3f,
        Key::KEY_F6 => 0x80 + 0x40,
        Key::KEY_F7 => 0x80 + 0x41,
        Key::KEY_F8 => 0x80 + 0x42,
        Key::KEY_F9 => 0x80 + 0x43,
        Key::KEY_F10 => 0x80 + 0x44,
        Key::KEY_F11 => 0x80 + 0x57,
        Key::KEY_F12 => 0x80 + 0x58,

        Key::KEY_BACKSPACE => 0x7f,
        Key::KEY_PAUSE => 0xff,

        Key::KEY_EQUAL => 0x3d,
        Key::KEY_MINUS => 0x2d,

        Key::KEY_LEFTSHIFT => 0x80 + 0x36, // Does this have a different key for doom?
        Key::KEY_RIGHTSHIFT => 0x80 + 0x36,
        Key::KEY_LEFTCTRL => 0x80 + 0x1d, // Does this have a different key for doom?
        Key::KEY_RIGHTCTRL => 0x80 + 0x1d,
        Key::KEY_LEFTALT => 0x80 + 0x38u8,
        Key::KEY_RIGHTALT => 0x80 + 0x38,

        Key::KEY_CAPSLOCK => 0x80 + 0x3a,
        Key::KEY_NUMLOCK => 0x80 + 0x45,
        Key::KEY_SCROLLLOCK => 0x80 + 0x46,
        Key::KEY_PRINT => 0x80 + 0x59, // Is Print == PRINTSCR?

        Key::KEY_HOME => 0x80 + 0x47,
        Key::KEY_END => 0x80 + 0x4f,
        Key::KEY_PAGEUP => 0x80 + 0x49,
        Key::KEY_PAGEDOWN => 0x80 + 0x51,
        Key::KEY_INSERT => 0x80 + 0x52,
        Key::KEY_DELETE => 0x80 + 0x53,

        Key::KEY_KP0 => 0,
        Key::KEY_KP1 => return map_evdev_key_to_doom(Key::KEY_END),
        Key::KEY_KP2 => return map_evdev_key_to_doom(Key::KEY_DOWN),
        Key::KEY_KP3 => return map_evdev_key_to_doom(Key::KEY_PAGEDOWN),
        Key::KEY_KP4 => return map_evdev_key_to_doom(Key::KEY_LEFT),
        Key::KEY_KP5 => '5' as u8,
        Key::KEY_KP6 => return map_evdev_key_to_doom(Key::KEY_RIGHT),
        Key::KEY_KP7 => return map_evdev_key_to_doom(Key::KEY_HOME),
        Key::KEY_KP8 => return map_evdev_key_to_doom(Key::KEY_UP),
        Key::KEY_KP9 => return map_evdev_key_to_doom(Key::KEY_PAGEUP),

        Key::KEY_KPSLASH => '/' as u8,
        Key::KEY_KPPLUS => '+' as u8,
        Key::KEY_KPMINUS => '-' as u8,
        Key::KEY_KPASTERISK => '*' as u8,
        Key::KEY_KPDOT => 0,
        Key::KEY_KPENTER => return map_evdev_key_to_doom(Key::KEY_ENTER),

        _ => return None,
    })
}

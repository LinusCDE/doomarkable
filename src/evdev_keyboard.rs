use std::{path::Path, sync::mpsc::Sender};

use doomgeneric::input::KeyData;
use evdev::KeyCode;

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
        debug!("Existing evdev device detected: {path:?}");
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
                debug!("New evdev device detected: {path:?}");
                spawn_evdev_keyboard(path, keydata_tx.clone());
            }
        }
    });
}

// Check if device is a keyboard, spawn new thread and listen for keystrokes and send them to keydata_tx
fn spawn_evdev_keyboard(path: impl AsRef<Path>, keydata_tx: Sender<KeyData>) {
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
            KeyCode::KEY_Q,
            KeyCode::KEY_W,
            KeyCode::KEY_E,
            KeyCode::KEY_R,
            KeyCode::KEY_T,
            KeyCode::KEY_Y,
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
                if let evdev::EventSummary::Key(_event, key, value) = ev.destructure() {
                    if value != 0 && value != 1 {
                        continue; // Ignore key being held (value == 2) and other potential values.
                    }

                    if let Ok((keycode, scancodes)) = device.get_scancode_by_index(key.0) {
                        debug!("{} ({key:?}, keycode: {keycode}, scancodes: {scancodes:?}) => {value}", key.0);
                    } else {
                        debug!("{} ({key:?}) => {value}", key.0);
                    }

                    if let Some(doom_key_code) = map_evdev_keycode_to_doom(key) {
                        keydata_tx
                            .send(KeyData {
                                key: doom_key_code,
                                pressed: value == 1,
                            })
                            .ok();
                    } else {
                        debug!("No mapping for key found. Last keypress is not forwarded to game.")
                    }
                }
            }
        }
    });
}

fn map_evdev_keycode_to_doom(key: KeyCode) -> Option<u8> {
    // https://github.com/ozkl/doomgeneric/blob/613f870b6fa83ede448a247de5a2571092fa729d/doomgeneric/doomkeys.h
    Some(match key {
        KeyCode::KEY_A => 'a' as u8,
        KeyCode::KEY_B => 'b' as u8,
        KeyCode::KEY_C => 'c' as u8,
        KeyCode::KEY_D => 'd' as u8,
        KeyCode::KEY_E => 'e' as u8,
        KeyCode::KEY_F => 'f' as u8,
        KeyCode::KEY_G => 'g' as u8,
        KeyCode::KEY_H => 'h' as u8,
        KeyCode::KEY_I => 'i' as u8,
        KeyCode::KEY_J => 'j' as u8,
        KeyCode::KEY_K => 'k' as u8,
        KeyCode::KEY_L => 'l' as u8,
        KeyCode::KEY_M => 'm' as u8,
        KeyCode::KEY_N => 'n' as u8,
        KeyCode::KEY_O => 'o' as u8,
        KeyCode::KEY_P => 'p' as u8,
        KeyCode::KEY_Q => 'q' as u8,
        KeyCode::KEY_R => 'r' as u8,
        KeyCode::KEY_S => 's' as u8,
        KeyCode::KEY_T => 't' as u8,
        KeyCode::KEY_U => 'u' as u8,
        KeyCode::KEY_V => 'v' as u8,
        KeyCode::KEY_W => 'w' as u8,
        KeyCode::KEY_X => 'x' as u8,
        KeyCode::KEY_Y => 'y' as u8,
        KeyCode::KEY_Z => 'z' as u8,
        KeyCode::KEY_0 => '0' as u8,
        KeyCode::KEY_1 => '1' as u8,
        KeyCode::KEY_2 => '2' as u8,
        KeyCode::KEY_3 => '3' as u8,
        KeyCode::KEY_4 => '4' as u8,
        KeyCode::KEY_5 => '5' as u8,
        KeyCode::KEY_6 => '6' as u8,
        KeyCode::KEY_7 => '7' as u8,
        KeyCode::KEY_8 => '8' as u8,
        KeyCode::KEY_9 => '9' as u8,

        KeyCode::KEY_RIGHT => 0xae,
        KeyCode::KEY_LEFT => 0xac,
        KeyCode::KEY_UP => 0xad,
        KeyCode::KEY_DOWN => 0xaf,
        KeyCode::KEY_DOT => 0xa0,      // Strafe left
        KeyCode::KEY_COMMA => 0xa1,    // Strafe right
        KeyCode::KEY_SPACE => 0xa2,    // Use
        KeyCode::KEY_LEFTCTRL => 0xa3, // Fire... i think
        KeyCode::KEY_ESC => 27,
        KeyCode::KEY_ENTER => 13,
        KeyCode::KEY_TAB => 9,
        KeyCode::KEY_F1 => 0x80 + 0x3b,
        KeyCode::KEY_F2 => 0x80 + 0x3c,
        KeyCode::KEY_F3 => 0x80 + 0x3d,
        KeyCode::KEY_F4 => 0x80 + 0x3e,
        KeyCode::KEY_F5 => 0x80 + 0x3f,
        KeyCode::KEY_F6 => 0x80 + 0x40,
        KeyCode::KEY_F7 => 0x80 + 0x41,
        KeyCode::KEY_F8 => 0x80 + 0x42,
        KeyCode::KEY_F9 => 0x80 + 0x43,
        KeyCode::KEY_F10 => 0x80 + 0x44,
        KeyCode::KEY_F11 => 0x80 + 0x57,
        KeyCode::KEY_F12 => 0x80 + 0x58,

        KeyCode::KEY_BACKSPACE => 0x7f,
        KeyCode::KEY_PAUSE => 0xff,

        KeyCode::KEY_EQUAL => 0x3d,
        KeyCode::KEY_MINUS => 0x2d,

        KeyCode::KEY_LEFTSHIFT => 0x80 + 0x36, // Does this have a different key for doom?
        KeyCode::KEY_RIGHTSHIFT => 0x80 + 0x36,
        KeyCode::KEY_RIGHTCTRL => 0x80 + 0x1d,
        KeyCode::KEY_LEFTALT => 0x80 + 0x38u8,
        KeyCode::KEY_RIGHTALT => 0x80 + 0x38,

        KeyCode::KEY_CAPSLOCK => 0x80 + 0x3a,
        KeyCode::KEY_NUMLOCK => 0x80 + 0x45,
        KeyCode::KEY_SCROLLLOCK => 0x80 + 0x46,
        KeyCode::KEY_PRINT => 0x80 + 0x59, // Is Print == PRINTSCR?

        KeyCode::KEY_HOME => 0x80 + 0x47,
        KeyCode::KEY_END => 0x80 + 0x4f,
        KeyCode::KEY_PAGEUP => 0x80 + 0x49,
        KeyCode::KEY_PAGEDOWN => 0x80 + 0x51,
        KeyCode::KEY_INSERT => 0x80 + 0x52,
        KeyCode::KEY_DELETE => 0x80 + 0x53,

        KeyCode::KEY_KP0 => 0,
        KeyCode::KEY_KP1 => return map_evdev_keycode_to_doom(KeyCode::KEY_END),
        KeyCode::KEY_KP2 => return map_evdev_keycode_to_doom(KeyCode::KEY_DOWN),
        KeyCode::KEY_KP3 => return map_evdev_keycode_to_doom(KeyCode::KEY_PAGEDOWN),
        KeyCode::KEY_KP4 => return map_evdev_keycode_to_doom(KeyCode::KEY_LEFT),
        KeyCode::KEY_KP5 => '5' as u8,
        KeyCode::KEY_KP6 => return map_evdev_keycode_to_doom(KeyCode::KEY_RIGHT),
        KeyCode::KEY_KP7 => return map_evdev_keycode_to_doom(KeyCode::KEY_HOME),
        KeyCode::KEY_KP8 => return map_evdev_keycode_to_doom(KeyCode::KEY_UP),
        KeyCode::KEY_KP9 => return map_evdev_keycode_to_doom(KeyCode::KEY_PAGEUP),

        KeyCode::KEY_KPSLASH => '/' as u8,
        KeyCode::KEY_KPPLUS => '+' as u8,
        KeyCode::KEY_KPMINUS => '-' as u8,
        KeyCode::KEY_KPASTERISK => '*' as u8,
        KeyCode::KEY_KPDOT => 0,
        KeyCode::KEY_KPENTER => return map_evdev_keycode_to_doom(KeyCode::KEY_ENTER),

        _ => return None,
    })
}

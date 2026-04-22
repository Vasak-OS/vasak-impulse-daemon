use evdev::{Device, KeyCode};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

const INPUT_DIR: &str = "/dev/input";

#[derive(Debug)]
pub struct KeyboardDevice {
    pub path: PathBuf,
    pub name: String,
    pub device: Device,
}

pub fn discover_keyboard_devices() -> Result<Vec<KeyboardDevice>, Box<dyn Error>> {
    let mut keyboards = Vec::new();

    for entry in fs::read_dir(INPUT_DIR)? {
        let entry = entry?;
        let path = entry.path();

        if !is_event_node(&path) {
            continue;
        }

        match Device::open(&path) {
            Ok(device) if looks_like_keyboard(&device) => {
                let name = device.name().unwrap_or("unknown keyboard").to_string();
                keyboards.push(KeyboardDevice { path, name, device });
            }
            Ok(_) => {}
            Err(error) => {
                eprintln!("⚠ No se pudo abrir {}: {}", path.display(), error);
            }
        }
    }

    keyboards.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(keyboards)
}

fn is_event_node(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("event"))
}

fn looks_like_keyboard(device: &Device) -> bool {
    let supported_keys = match device.supported_keys() {
        Some(keys) => keys,
        None => return false,
    };

    let signature_keys = [
        KeyCode::KEY_A,
        KeyCode::KEY_Z,
        KeyCode::KEY_F1,
        KeyCode::KEY_ESC,
        KeyCode::KEY_ENTER,
        KeyCode::KEY_LEFTSHIFT,
        KeyCode::KEY_RIGHTSHIFT,
    ];

    signature_keys
        .iter()
        .copied()
        .any(|key| supported_keys.contains(key))
}

use std::collections::{BTreeSet, HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

use evdev::{Device, EventSummary, KeyCode};
use serde::{Deserialize, Serialize};

const INPUT_DIR: &str = "/dev/input";
const DEFAULT_BINDINGS_FILE: &str = "bindings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BindingConfig {
    bindings: Vec<BindingRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BindingRule {
    combo: String,
    action: String,
}

#[derive(Debug)]
struct KeyboardDevice {
    path: PathBuf,
    name: String,
    device: Device,
}

fn main() -> Result<(), Box<dyn Error>> {
    let bindings_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_BINDINGS_FILE.to_string());

    let bindings = load_bindings(&bindings_path)?;
    let devices = discover_keyboard_devices()?;

    if devices.is_empty() {
        eprintln!("No se encontraron teclados en {INPUT_DIR}");
        return Ok(());
    }

    println!("Bindings cargados: {}", bindings.bindings.len());
    println!("Teclados detectados: {}", devices.len());

    let mut handles = Vec::new();

    for keyboard in devices {
        let bindings = bindings.clone();
        let handle = thread::spawn(move || {
            if let Err(error) = watch_keyboard(keyboard, bindings) {
                eprintln!("Error en hilo de teclado: {error}");
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }

    Ok(())
}

fn load_bindings(path: impl AsRef<Path>) -> Result<BindingConfig, Box<dyn Error>> {
    let path = path.as_ref();

    if !path.exists() {
        return Ok(BindingConfig { bindings: Vec::new() });
    }

    let raw = fs::read_to_string(path)?;
    let config: BindingConfig = serde_json::from_str(&raw)?;
    Ok(config)
}

fn discover_keyboard_devices() -> Result<Vec<KeyboardDevice>, Box<dyn Error>> {
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
                eprintln!("No se pudo abrir {}: {error}", path.display());
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

fn watch_keyboard(mut keyboard: KeyboardDevice, bindings: BindingConfig) -> Result<(), Box<dyn Error>> {
    println!(
        "Escuchando {} ({}) en {}",
        keyboard.name.as_str(),
        keyboard.device.name().unwrap_or("unknown keyboard"),
        keyboard.path.display()
    );

    let binding_lookup: HashMap<String, String> = bindings
        .bindings
        .into_iter()
        .map(|binding| (normalize_combo_string(&binding.combo), binding.action))
        .collect();

    let mut active_keys = HashSet::new();

    loop {
        for event in keyboard.device.fetch_events()? {
            match event.destructure() {
                EventSummary::Key(_, key_code, 1) => {
                    active_keys.insert(key_code);
                    let combo = current_combo(&active_keys);
                    if let Some(action) = binding_lookup.get(&combo) {
                        println!(
                            "[{}] combo {} => {}",
                            keyboard.name,
                            combo,
                            action
                        );
                    } else {
                        println!("[{}] tecla presionada: {}", keyboard.name, key_code_name(key_code));
                    }
                }
                EventSummary::Key(_, key_code, 0) => {
                    active_keys.remove(&key_code);
                    println!("[{}] tecla liberada: {}", keyboard.name, key_code_name(key_code));
                }
                EventSummary::Key(_, key_code, 2) => {
                    let combo = current_combo(&active_keys);
                    println!("[{}] repeticion: {} [{}]", keyboard.name, key_code_name(key_code), combo);
                }
                _ => {}
            }
        }
    }
}

fn current_combo(active_keys: &HashSet<KeyCode>) -> String {
    let ordered: BTreeSet<String> = active_keys.iter().map(|key| key_code_name(*key)).collect();
    ordered.into_iter().collect::<Vec<_>>().join("+")
}

fn normalize_combo_string(combo: &str) -> String {
    let mut parts: Vec<String> = combo
        .split('+')
        .map(|part| part.trim().to_uppercase())
        .filter(|part| !part.is_empty())
        .collect();

    parts.sort();
    parts.join("+")
}

fn key_code_name(key_code: KeyCode) -> String {
    format!("{:?}", key_code)
}

use std::collections::{BTreeSet, HashSet};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{mpsc, Arc, RwLock};
use std::thread;

use evdev::{Device, EventSummary, KeyCode};
use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};

const INPUT_DIR: &str = "/dev/input";
const SHORTCUT_CONFIG_DIR: &str = ".config/vasak";
const SHORTCUT_CONFIG_FILE: &str = "shortcut.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ShortcutBindingFile {
    keys: String,
    action: String,
    target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ShortcutBinding {
    keys: String,
    action: String,
    target: String,
}

#[derive(Debug)]
struct KeyboardDevice {
    path: PathBuf,
    name: String,
    device: Device,
}

fn main() -> Result<(), Box<dyn Error>> {
    let config_path = shortcut_config_path()?;
    ensure_shortcut_config(&config_path)?;

    let bindings = Arc::new(RwLock::new(load_bindings(&config_path)?));
    let watcher_handle = spawn_config_watcher(config_path.clone(), Arc::clone(&bindings));
    let devices = discover_keyboard_devices()?;

    if devices.is_empty() {
        eprintln!("No se encontraron teclados en {INPUT_DIR}");
        return Ok(());
    }

    let loaded_bindings = bindings
        .read()
        .map(|guard| guard.len())
        .unwrap_or_default();
    println!("Bindings cargados: {}", loaded_bindings);
    println!("Teclados detectados: {}", devices.len());

    let mut handles = Vec::new();
    handles.push(watcher_handle);

    for keyboard in devices {
        let bindings = Arc::clone(&bindings);
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

fn shortcut_config_path() -> Result<PathBuf, std::io::Error> {
    let home = std::env::var_os("HOME")
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "HOME no esta definido"))?;

    Ok(PathBuf::from(home)
        .join(SHORTCUT_CONFIG_DIR)
        .join(SHORTCUT_CONFIG_FILE))
}

fn ensure_shortcut_config(path: &Path) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if !path.exists() {
        fs::write(path, "[]\n")?;
        println!("Creado archivo de shortcuts en {}", path.display());
    }

    Ok(())
}

fn load_bindings(path: impl AsRef<Path>) -> Result<Vec<ShortcutBinding>, Box<dyn Error>> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path)?;

    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }

    let file_bindings: Vec<ShortcutBindingFile> = serde_json::from_str(&raw)?;
    Ok(file_bindings
        .into_iter()
        .map(|binding| ShortcutBinding {
            keys: normalize_combo_string(&binding.keys),
            action: binding.action,
            target: binding.target,
        })
        .collect())
}

fn spawn_config_watcher(
    config_path: PathBuf,
    bindings: Arc<RwLock<Vec<ShortcutBinding>>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let parent_dir = match config_path.parent() {
            Some(parent) => parent.to_path_buf(),
            None => {
                eprintln!("No se pudo resolver el directorio de configuracion");
                return;
            }
        };

        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

        let mut watcher = match recommended_watcher(move |result| {
            let _ = tx.send(result);
        }) {
            Ok(watcher) => watcher,
            Err(error) => {
                eprintln!("No se pudo inicializar el watcher de configuracion: {error}");
                return;
            }
        };

        if let Err(error) = watcher.watch(&parent_dir, RecursiveMode::NonRecursive) {
            eprintln!("No se pudo observar {}: {error}", parent_dir.display());
            return;
        }

        println!("Observando cambios en {}", config_path.display());

        for result in rx {
            match result {
                Ok(event) if should_reload_config(&event) => {
                    if let Err(error) = reload_bindings(&config_path, &bindings) {
                        eprintln!("No se pudo recargar {}: {error}", config_path.display());
                    }
                }
                Ok(_) => {}
                Err(error) => eprintln!("Error del watcher: {error}"),
            }
        }
    })
}

fn should_reload_config(event: &Event) -> bool {
    matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) | EventKind::Any | EventKind::Other
    )
}

fn reload_bindings(
    config_path: &Path,
    bindings: &Arc<RwLock<Vec<ShortcutBinding>>>,
) -> Result<(), Box<dyn Error>> {
    let new_bindings = load_bindings(config_path)?;

    if let Ok(mut guard) = bindings.write() {
        *guard = new_bindings;
        println!("Configuracion recargada: {} bindings", guard.len());
    }

    Ok(())
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

fn watch_keyboard(
    mut keyboard: KeyboardDevice,
    bindings: Arc<RwLock<Vec<ShortcutBinding>>>,
) -> Result<(), Box<dyn Error>> {
    println!(
        "Escuchando {} ({}) en {}",
        keyboard.name.as_str(),
        keyboard.device.name().unwrap_or("unknown keyboard"),
        keyboard.path.display()
    );

    let mut active_keys = HashSet::new();
    let mut last_triggered_combo: Option<String> = None;

    loop {
        for event in keyboard.device.fetch_events()? {
            match event.destructure() {
                EventSummary::Key(_, key_code, 1) => {
                    active_keys.insert(key_code);
                    let combo = current_combo(&active_keys);

                    if last_triggered_combo.as_deref() != Some(combo.as_str()) {
                        if let Some(binding) = find_binding(&bindings, &combo) {
                            println!(
                                "[{}] combo {} => {} ({})",
                                keyboard.name,
                                combo,
                                binding.action,
                                binding.target
                            );
                            run_target(&binding.target)?;
                            last_triggered_combo = Some(combo);
                        } else {
                            println!("[{}] tecla presionada: {}", keyboard.name, key_code_name(key_code));
                        }
                    }
                }
                EventSummary::Key(_, key_code, 0) => {
                    active_keys.remove(&key_code);
                    println!("[{}] tecla liberada: {}", keyboard.name, key_code_name(key_code));
                    last_triggered_combo = None;
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

fn find_binding(bindings: &Arc<RwLock<Vec<ShortcutBinding>>>, combo: &str) -> Option<ShortcutBinding> {
    bindings
        .read()
        .ok()
        .and_then(|guard| guard.iter().find(|binding| binding.keys == combo).cloned())
}

fn run_target(target: &str) -> Result<(), Box<dyn Error>> {
    let mut command_parts = target.split_whitespace();
    let program = match command_parts.next() {
        Some(program) => program,
        None => return Ok(()),
    };

    let mut command = Command::new(program);
    command.args(command_parts);
    let _child = command.spawn()?;
    Ok(())
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

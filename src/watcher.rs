use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Watcher};
use std::error::Error;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, RwLock};
use std::thread;

use serde_json;

use crate::bindings::ShortcutBinding;
use crate::config::Config;

pub fn spawn_config_watcher(
    config: Arc<Config>,
    bindings: Arc<RwLock<Vec<ShortcutBinding>>>,
) -> thread::JoinHandle<()> {
    let config_path = config.config_path.clone();
    let config_dir = config.config_dir().to_path_buf();

    thread::spawn(move || {
        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

        let mut watcher = match recommended_watcher(move |result| {
            let _ = tx.send(result);
        }) {
            Ok(watcher) => watcher,
            Err(error) => {
                eprintln!("✗ No se pudo inicializar el watcher: {}", error);
                return;
            }
        };

        if let Err(error) = watcher.watch(&config_dir, RecursiveMode::NonRecursive) {
            eprintln!("✗ No se pudo observar {}: {}", config_dir.display(), error);
            return;
        }

        println!("✓ Observando cambios en {}", config_path.display());

        for result in rx {
            match result {
                Ok(event) if should_reload_config(&event) => {
                    if let Err(error) = reload_bindings(&config_path, &bindings) {
                        eprintln!(
                            "⚠ No se pudo recargar {}: {}",
                            config_path.display(),
                            error
                        );
                    }
                }
                Ok(_) => {}
                Err(error) => eprintln!("⚠ Error del watcher: {}", error),
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
    config_path: &PathBuf,
    bindings: &Arc<RwLock<Vec<ShortcutBinding>>>,
) -> Result<(), Box<dyn Error>> {
    let raw = std::fs::read_to_string(config_path)?;

    if raw.trim().is_empty() {
        return Ok(());
    }

    let file_bindings: Vec<crate::bindings::ShortcutBindingFile> = serde_json::from_str(&raw)?;
    let new_bindings: Vec<ShortcutBinding> = file_bindings
        .into_iter()
        .map(ShortcutBinding::from_file)
        .collect();

    if let Ok(mut guard) = bindings.write() {
        *guard = new_bindings;
        println!(
            "✓ Configuracion recargada: {} bindings",
            guard.len()
        );
    }

    Ok(())
}

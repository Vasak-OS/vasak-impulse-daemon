use std::collections::HashSet;
use std::error::Error;
use std::sync::{Arc, RwLock};
use std::thread;

use evdev::EventSummary;

use vasak_impulse_daemon::{
    bindings::{current_combo, ShortcutBinding},
    config::Config,
    executor::run_target,
    keyboard::{discover_keyboard_devices, KeyboardDevice},
    watcher::spawn_config_watcher,
};

fn main() -> Result<(), Box<dyn Error>> {
    println!("▸ vasak-impulse-daemon iniciando...\n");

    // Inicializar configuracion (detecta usuario real incluso si se ejecuta como root)
    let config = Arc::new(Config::new()?);
    println!(
        "✓ Configuracion encontrada: {}",
        config.config_path.display()
    );

    // Cargar bindings iniciales
    let bindings = Arc::new(RwLock::new(config.load_bindings()?));
    let loaded_bindings = bindings
        .read()
        .map(|guard| guard.len())
        .unwrap_or_default();
    println!("✓ Bindings cargados: {}\n", loaded_bindings);

    // Descubrir dispositivos de entrada
    let devices = discover_keyboard_devices()?;
    if devices.is_empty() {
        eprintln!("✗ No se encontraron teclados en /dev/input");
        eprintln!("  Sugerencia: ejecuta el daemon como root o con permisos suficientes");
        return Ok(());
    }
    println!("✓ Teclados detectados: {}\n", devices.len());

    // Iniciar watcher de configuracion
    let watcher_handle = spawn_config_watcher(Arc::clone(&config), Arc::clone(&bindings));

    // Iniciar hilos para cada teclado
    let mut handles = vec![watcher_handle];
    for keyboard in devices {
        let bindings = Arc::clone(&bindings);
        let handle = thread::spawn(move || {
            if let Err(error) = watch_keyboard(keyboard, bindings) {
                eprintln!("✗ Error en hilo de teclado: {}", error);
            }
        });
        handles.push(handle);
    }

    println!("▸ Daemon ejecutándose (presiona Ctrl+C para salir)\n");

    // Esperar a que terminen todos los hilos
    for handle in handles {
        let _ = handle.join();
    }

    Ok(())
}

fn watch_keyboard(
    mut keyboard: KeyboardDevice,
    bindings: Arc<RwLock<Vec<ShortcutBinding>>>,
) -> Result<(), Box<dyn Error>> {
    println!(
        "▸ Teclado: {} ({})",
        keyboard.name, keyboard.path.display()
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
                                "[{}] ► {} → {} ({})",
                                keyboard.name, combo, binding.action, binding.target
                            );
                            run_target(&binding.target)?;
                            last_triggered_combo = Some(combo);
                        }
                    }
                }
                EventSummary::Key(_, key_code, 0) => {
                    active_keys.remove(&key_code);
                    last_triggered_combo = None;
                }
                EventSummary::Key(_, _, 2) => {
                    // Repeticion de tecla, ignorar
                }
                _ => {}
            }
        }
    }
}

fn find_binding(
    bindings: &Arc<RwLock<Vec<ShortcutBinding>>>,
    combo: &str,
) -> Option<ShortcutBinding> {
    bindings
        .read()
        .ok()
        .and_then(|guard| guard.iter().find(|binding| binding.keys == combo).cloned())
}

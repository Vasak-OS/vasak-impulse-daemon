use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

fn canonical_key_name(raw: &str) -> String {
    let key = raw.trim().to_uppercase();
    match key.as_str() {
        "CTRL" | "CONTROL" | "KEY_LEFTCTRL" | "KEY_RIGHTCTRL" => "CTRL".to_string(),
        "SHIFT" | "KEY_LEFTSHIFT" | "KEY_RIGHTSHIFT" => "SHIFT".to_string(),
        "ALT" | "KEY_LEFTALT" | "KEY_RIGHTALT" => "ALT".to_string(),
        "SUPER" | "META" | "WIN" | "KEY_LEFTMETA" | "KEY_RIGHTMETA" => "SUPER".to_string(),
        _ => key,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutBindingFile {
    pub keys: String,
    pub action: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutBinding {
    pub keys: String,
    pub action: String,
    pub target: String,
}

impl ShortcutBinding {
    pub fn from_file(file_binding: ShortcutBindingFile) -> Self {
        ShortcutBinding {
            keys: normalize_combo_string(&file_binding.keys),
            action: file_binding.action,
            target: file_binding.target,
        }
    }
}

pub fn normalize_combo_string(combo: &str) -> String {
    let parts: BTreeSet<String> = combo
        .split('+')
        .map(canonical_key_name)
        .filter(|part| !part.is_empty())
        .collect();

    parts.into_iter().collect::<Vec<_>>().join("+")
}

pub fn current_combo(active_keys: &std::collections::HashSet<evdev::KeyCode>) -> String {
    let ordered: BTreeSet<String> = active_keys
        .iter()
        .map(|key| canonical_key_name(&format!("{:?}", key)))
        .collect();
    ordered.into_iter().collect::<Vec<_>>().join("+")
}

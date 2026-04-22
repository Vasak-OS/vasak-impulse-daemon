use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

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
    let mut parts: Vec<String> = combo
        .split('+')
        .map(|part| part.trim().to_uppercase())
        .filter(|part| !part.is_empty())
        .collect();

    parts.sort();
    parts.join("+")
}

pub fn current_combo(active_keys: &std::collections::HashSet<evdev::KeyCode>) -> String {
    let ordered: BTreeSet<String> = active_keys.iter().map(|key| format!("{:?}", key)).collect();
    ordered.into_iter().collect::<Vec<_>>().join("+")
}

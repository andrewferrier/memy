use crate::hooks_generated;

pub fn get_hook_content(name: &str) -> Option<&'static str> {
    hooks_generated::HOOKS.get(name).copied()
}

pub fn get_hook_list() -> Vec<&'static str> {
    hooks_generated::HOOKS.keys().copied().collect()
}

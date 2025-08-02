use std::collections::HashMap;

static PLUGINS: std::sync::LazyLock<HashMap<&'static str, &'static str>> =
    std::sync::LazyLock::new(|| {
        let mut map = HashMap::new();
        map.insert("lfrc", "cmd on-cd &{{\n    memy note ${PWD} &\n}}\n");
        map
    });

pub fn get_plugin_content(name: &str) -> Option<&'static str> {
    PLUGINS.get(name).copied()
}

pub fn get_plugin_list() -> Vec<&'static str> {
    PLUGINS.keys().copied().collect()
}

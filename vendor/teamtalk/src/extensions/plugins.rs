#[cfg(feature = "plugins")]
use libloading::{Library, Symbol};
#[cfg(feature = "plugins")]
use std::collections::HashMap;
#[cfg(feature = "plugins")]
use std::path::{Path, PathBuf};

#[cfg(feature = "plugins")]
pub struct PluginManager {
    plugins: HashMap<String, PluginHandle>,
}

#[cfg(feature = "plugins")]
struct PluginHandle {
    _lib: Library,
    path: PathBuf,
}

#[cfg(feature = "plugins")]
type PluginInit = unsafe extern "C" fn();

#[cfg(feature = "plugins")]
impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn load(&mut self, name: &str, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref().to_path_buf();
        let lib = unsafe { Library::new(&path).map_err(|e| e.to_string())? };
        unsafe {
            let init: Symbol<PluginInit> = lib.get(b"tt_plugin_init").map_err(|e| e.to_string())?;
            init();
        }
        self.plugins
            .insert(name.to_string(), PluginHandle { _lib: lib, path });
        Ok(())
    }

    pub fn reload(&mut self, name: &str) -> Result<(), String> {
        let path = self
            .plugins
            .get(name)
            .ok_or_else(|| "plugin not found".to_string())?
            .path
            .clone();
        self.unload(name)?;
        self.load(name, path)
    }

    pub fn unload(&mut self, name: &str) -> Result<(), String> {
        self.plugins
            .remove(name)
            .ok_or_else(|| "plugin not found".to_string())?;
        Ok(())
    }
}

#[cfg(feature = "plugins")]
impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

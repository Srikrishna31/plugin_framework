use crate::errors::*;
use crate::plugin::Plugin;
use libloading::{Library, Symbol};
use log::{debug, trace};
use std::ffi::OsStr;

/// This structure manages all the plugins that are loaded, and calls the appropriate functions at
/// the appropriate time, while also keeping track of their lifetimes.
///
/// # Note
/// Something we need to keep in mind is that any `Library` we load will need to outlive our plugins.
/// This is because they contain the code for executing the various `Plugin` methods, so if the
/// `Library` is dropped too early our plugins' vtable could end up pointing at garbage... Which would be bad.
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    loaded_libraries: Vec<Library>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            loaded_libraries: Vec::new(),
        }
    }

    pub unsafe fn load_plugin<P: AsRef<OsStr>>(&mut self, filename: P) -> Result<()> {
        type PluginCreate<'a> = unsafe fn() -> &'a mut dyn Plugin;

        let lib = Library::new(filename.as_ref()).chain_err(|| "Unable to load the plugin")?;

        // We need to keep the library around, otherwise our plugin's vtable will point to garbage.
        // We do this little dance to make sure the library doesn't end up getting moved.
        self.loaded_libraries.push(lib);

        let lib = self.loaded_libraries.last().unwrap();

        let constructor: Symbol<PluginCreate> = lib
            .get(b"_plugin_create")
            .chain_err(|| "The `_plugin_create` symbol wasn't found.")?;
        let boxed_raw = constructor();

        let plugin = Box::from_raw(boxed_raw);
        debug!("Loaded Plugin: {}", plugin.name());
        plugin.on_plugin_load();
        self.plugins.push(plugin);

        Ok(())
    }
}

use crate::errors::*;
use crate::plugin::Plugin;
use libloading::{Library, Symbol};
use log::{debug, trace};
use std::ffi::OsStr;

/// This structure manages all the plugins that are loaded, and calls the appropriate functions at
/// the appropriate time, while also keeping track of their lifetimes.
///
/// # Note
///
/// Something we need to keep in mind is that any `Library` we load will need to outlive our plugins.
/// This is because they contain the code for executing the various `Plugin` methods, so if the
/// `Library` is dropped too early our plugins' vtable could end up pointing at garbage... Which would be bad.
///
/// # Note on Destroy
///
/// A thing to keep in mind is something called [panic_on_drop](https://www.reddit.com/r/rust/comments/4a9vu6/what_are_the_semantics_of_panicondrop/).
/// Basically, if the program is panicking it'll unwind the stack, calling destructors when necessary.
/// However, because our `PluginManager` tries to unload plugins if it hasn't already, a `Plugin` who's
/// `unload()` method also panics will result in a second panic. This usually results in aborting the
/// entire program because your program is most probably FUBAR.
///
/// # Note on Customization
///
/// This is a bare minimum plugin manager, with just the capability to load and unload plugins. This is
/// the reason why the members are made public. For any application wishing to support plugins, it'd
/// have to extend this PluginManager with it's own, possibly calling additional functions on all the
/// plugins, or rejecting a plugin library if it doesn't contain the expected set of functions beyond
/// the ones defined in the `Plugin` trait provided with this library.
/// An example of this can be seen in the [rust_ffi_example repo](https://github.com/Srikrishna31/rust_ffi_example)
pub struct PluginManager {
    pub plugins: Vec<Box<dyn Plugin>>,
    pub loaded_libraries: Vec<Library>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            loaded_libraries: Vec::new(),
        }
    }

    /// Load a single plugin, provided the path to the shared library plugin on the system.
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

    /// Load a set of plugins, provided a path to the folder containing shared library plugins.
    pub unsafe fn load_plugins<P: AsRef<OsStr>>(&mut self, file_path: P) -> Result<()> {
        todo!()
    }

    /// Unload all plugins and loaded plugin libraries, making sure to fire their `on_plugin_unload()`
    /// methods so they can do any necessary cleanup.
    pub fn unload(&mut self) {
        debug!("Unloading plugins");

        for plugin in self.plugins.drain(..) {
            trace!("Firing on_plugin_unload for {:?}", plugin.name());
            plugin.on_plugin_unload();
        }

        for lib in self.loaded_libraries.drain(..) {
            drop(lib);
        }
    }
}

/// We implement `Drop` for PluginManager, so that plugins are always unloaded when the `PluginManager`
/// gets dropped. This gives them a chance to do any necessary cleanup.
impl Drop for PluginManager {
    fn drop(&mut self) {
        if !self.plugins.is_empty() || !self.loaded_libraries.is_empty() {
            self.unload();
        }
    }
}

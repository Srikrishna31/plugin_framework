use std::any::Any;

/// A plugin which allows you to add extra functionality to any conforming application
pub trait Plugin: Any + Send + Sync {
    /// Get a name describing the `Plugin`
    fn name(&self) -> &'static str;

    /// A callback fired immediately after the plugin is loaded. Usually used for initialization.
    fn on_plugin_load(&self) {}

    /// A callback fired immediately before the plugin is unloaded. Use this if you need to do
    /// any cleanup.
    fn on_plugin_unload(&self) {}
}

/// Declare a plugin type and its constructor.
///
/// # Notes
///
/// This works by automatically generating an `extern "C"` function with a pre-defined signature and
/// symbol name. Therefore you will only be able to declare one plugin per library.
#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type: ty, $constructor: path) => {
        #[no_mangle]
        pub extern "C" fn _plugin_create() -> *mut $crate::Plugin {
            // make sure the constructor is the correct type.
            let constructor: fn() -> $plugin_type = $constructor;

            let object = constructor();
            let boxed: Box<$crate::Plugin> = Box::new(object);
            Box::into_raw(boxed)
        }
    };
}

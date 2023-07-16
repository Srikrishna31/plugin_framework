mod errors;
mod plugin;
mod plugin_manager;

#[macro_use]
extern crate error_chain;

pub use plugin::Plugin;
pub use plugin_manager::PluginManager;

use libloading::os::windows::{
    LOAD_LIBRARY_SEARCH_DEFAULT_DIRS, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR, Library,
};
use llvm_fixture::events::{DamageCallback, REGISTER_PLUGIN_V1, RegisterPlugin};
use mixin_runtime::Result;
use std::ffi::c_void;
use std::path::Path;

#[derive(Default)]
pub struct DamageEvents {
    callbacks: Vec<DamageCallback>,
    libraries: Vec<Library>,
}

impl DamageEvents {
    pub fn load(&mut self, path: &Path) -> Result<()> {
        let plugin_error = |error| format!("{}: {error}", path.display());
        let absolute = path
            .canonicalize()
            .map_err(|error| plugin_error(error.to_string()))?;
        // Plugins are trusted native code. The versioned entry point uses the
        // shared game ABI, and the library stays loaded for every callback.
        let library = unsafe {
            Library::load_with_flags(
                &absolute,
                LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS,
            )
        }
        .map_err(|error| plugin_error(error.to_string()))?;
        let mut subscriptions = Vec::<DamageCallback>::new();
        unsafe {
            let register = library
                .get::<RegisterPlugin>(REGISTER_PLUGIN_V1)
                .map_err(|error| plugin_error(format!("cannot find mixin_register_v1: {error}")))?;
            register(
                std::ptr::from_mut(&mut subscriptions).cast(),
                subscribe_damage,
            );
        }
        if subscriptions.is_empty() {
            return Err(plugin_error("plugin did not subscribe to on_damage".into()));
        }
        self.libraries.push(library);
        self.callbacks.extend(subscriptions);
        Ok(())
    }

    pub fn emit(&self, damage: f64) -> f64 {
        self.callbacks
            .iter()
            .fold(damage, |value, callback| callback(value))
    }
}

unsafe extern "C" fn subscribe_damage(context: *mut c_void, callback: DamageCallback) {
    // Registration is synchronous, and the plugin passes back the context
    // supplied by DamageEvents::load.
    let subscriptions = unsafe { &mut *context.cast::<Vec<DamageCallback>>() };
    subscriptions.push(callback);
}

use crate::bundle::ModManifest;
use crate::{Api, Diagnostic, DiagnosticResult, Jit, Plan, Result, ScalarValue, Stage};
use std::path::{Path, PathBuf};
use std::thread::{self, Scope, ScopedJoinHandle};

/// Single-threaded replacement of a component's ordered mod list.
///
/// Replace between component calls. The host owns persistent game state.
/// Each JIT owns its code and static data. Mods are trusted native code
/// and must follow the host's lifetime contract.
pub struct HotReload<'a> {
    api: &'a Api,
    base: &'a [u8],
    libraries: &'a [&'a Path],
    active: Jit<'a>,
}

impl<'a> HotReload<'a> {
    /// Start from verified game bitcode and an ordered list of mod directories.
    pub fn new(api: &'a Api, base: &'a [u8], mods: &[PathBuf]) -> DiagnosticResult<Self> {
        // SAFETY: the empty list adds no native symbol providers.
        unsafe { Self::with_libraries(api, base, mods, &[]) }
    }

    /// Start a reloadable component with native libraries for symbol resolution.
    ///
    /// # Safety
    /// The libraries must be trusted and match the bitcode ABI for every generation.
    pub unsafe fn with_libraries(
        api: &'a Api,
        base: &'a [u8],
        mods: &[PathBuf],
        libraries: &'a [&'a Path],
    ) -> DiagnosticResult<Self> {
        let active = Self::compile(api, base, mods, libraries)?;
        Ok(Self {
            api,
            base,
            libraries,
            active,
        })
    }

    /// Inspect package selectors without compiling a generation.
    pub fn plan(api: &Api, base: &[u8], mods: &[PathBuf]) -> DiagnosticResult<Plan> {
        let bitcode = read_mods(base, mods)?;
        let modules: Vec<_> = bitcode.iter().map(Vec::as_slice).collect();
        Jit::plan_many(api, base, &modules).map_err(|error| with_package(error, mods))
    }

    /// Apply the full desired mod list from the original game bitcode.
    /// An empty list restores base behavior. A failed build keeps the active JIT.
    pub fn replace(&mut self, mods: &[PathBuf]) -> DiagnosticResult<()> {
        let candidate = Self::compile(self.api, self.base, mods, self.libraries)?;
        drop(std::mem::replace(&mut self.active, candidate));
        Ok(())
    }

    /// Call the active generation. Scalar results can outlive a replacement.
    pub fn call(&self, target: &str, args: &[ScalarValue]) -> Result<ScalarValue> {
        self.active.call(target, args)
    }

    /// Use a scoped worker for preparation while keeping calls on the host thread.
    pub fn background<'scope, 'env>(
        self,
        scope: &'scope Scope<'scope, 'env>,
    ) -> BackgroundReload<'scope, 'env>
    where
        'a: 'scope,
    {
        BackgroundReload {
            runtime: self,
            scope,
            pending: None,
        }
    }

    fn compile(
        api: &'a Api,
        base: &[u8],
        mods: &[PathBuf],
        libraries: &[&Path],
    ) -> DiagnosticResult<Jit<'a>> {
        let bitcode = read_mods(base, mods)?;
        let modules: Vec<_> = bitcode.iter().map(Vec::as_slice).collect();
        // SAFETY: with_libraries requires trusted providers with matching ABI.
        unsafe { Jit::compile_with_libraries(api, base, &modules, libraries) }
            .map_err(|error| with_package(error, mods))
    }
}

/// Hot reload with one pending request. The host publishes it between calls.
pub struct BackgroundReload<'scope, 'env> {
    runtime: HotReload<'scope>,
    scope: &'scope Scope<'scope, 'env>,
    pending: Option<ScopedJoinHandle<'scope, DiagnosticResult<PreparedJit<'scope>>>>,
}

// Only the preparation worker can create this wrapper. No game entry has
// been called. Materialization and generated TypeId probes have finished,
// and the ORC instance owns its thread-safe context, code and static data.
// Ownership transfers once through join. Jit itself remains !Send + !Sync.
struct PreparedJit<'a>(Jit<'a>);

// SAFETY: LLVM's in-process ORC session can be used and disposed on the
// receiving thread. No LLVM builder or thread-local game state is transferred.
unsafe impl Send for PreparedJit<'_> {}

impl BackgroundReload<'_, '_> {
    /// Start preparing the full desired list. An empty list restores the base.
    /// The caller can retry a busy request after consuming the pending result.
    pub fn request(&mut self, mods: Vec<PathBuf>) -> DiagnosticResult<()> {
        if self.pending.is_some() {
            return Err(Diagnostic::new(
                Stage::Reload,
                "a reload is already pending",
            ));
        }
        let (api, base, libraries) = (self.runtime.api, self.runtime.base, self.runtime.libraries);
        let worker = thread::Builder::new()
            .name("mixin-prepare".into())
            .spawn_scoped(self.scope, move || {
                HotReload::compile(api, base, &mods, libraries).map(PreparedJit)
            })
            .map_err(|error| {
                Diagnostic::new(
                    Stage::Reload,
                    format!("could not start reload worker: {error}"),
                )
            })?;
        self.pending = Some(worker);
        Ok(())
    }

    /// At a safe point, publish a finished request and release the previous JIT.
    /// Returns false while preparation is running or when no request is pending.
    pub fn try_activate(&mut self) -> DiagnosticResult<bool> {
        if !self
            .pending
            .as_ref()
            .is_some_and(ScopedJoinHandle::is_finished)
        {
            return Ok(false);
        }
        self.finish()
    }

    /// Wait and activate at a safe point. Useful for hosts that can block.
    pub fn finish(&mut self) -> DiagnosticResult<bool> {
        let Some(worker) = self.pending.take() else {
            return Ok(false);
        };
        let PreparedJit(candidate) = worker
            .join()
            .map_err(|_| Diagnostic::new(Stage::Reload, "reload worker panicked"))??;
        drop(std::mem::replace(&mut self.runtime.active, candidate));
        Ok(true)
    }

    /// Run a call against the currently published generation.
    pub fn call(&self, target: &str, args: &[ScalarValue]) -> Result<ScalarValue> {
        self.runtime.call(target, args)
    }
}

impl Drop for BackgroundReload<'_, '_> {
    fn drop(&mut self) {
        if let Some(worker) = self.pending.take() {
            drop(worker.join());
        }
    }
}

fn read_mods(base: &[u8], mods: &[PathBuf]) -> DiagnosticResult<Vec<Vec<u8>>> {
    mods.iter()
        .enumerate()
        .map(|(index, path)| {
            let error = |message| {
                Diagnostic::new(Stage::ModManifest, format!("{}: {message}", path.display()))
                    .with_mod(Some(index))
            };
            if path
                .join("plugin.dll")
                .try_exists()
                .map_err(|e| error(e.to_string()))?
            {
                return Err(error("hot reload accepts bitcode-only mod packages".into()));
            }
            ModManifest::read(path, base).map_err(error)
        })
        .collect()
}

fn with_package(mut error: Diagnostic, mods: &[PathBuf]) -> Diagnostic {
    if let Some(path) = error.mod_index.and_then(|index| mods.get(index)) {
        error.message = format!("{}: {}", path.display(), error.message);
    }
    error
}

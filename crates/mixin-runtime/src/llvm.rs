//! Small bindings to the official LLVM 21.1.3 C API, not a Rust parser.
//! Input bitcode is trusted native code. The C API cannot sandbox arbitrary input.
#![allow(non_snake_case)]

use crate::metadata::{Handler, Index, Operation, PlannedPatch, Record, ReturnHook};
use crate::{Diagnostic, DiagnosticResult, ScalarType, ScalarValue, Stage};
use libloading::Library;
use serde::Serialize;
use std::any::TypeId;
use std::collections::BTreeMap;
use std::ffi::{CStr, CString, c_char, c_int, c_uint, c_void};
use std::marker::PhantomData;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::path::Path;
use std::ptr;
use std::rc::Rc;
use std::time::Instant;

type P = *mut c_void;
type ScalarEntry = extern "C" fn(u64, u64, u64, u64) -> u64;
type TypedEntry = unsafe extern "C" fn(u64, u64, u64, u64, P) -> bool;
type Result<T> = std::result::Result<T, String>;

macro_rules! api {
    ($(fn $name:ident($($arg:ty),*) -> $ret:ty;)+) => {
        pub struct Api {
            $(pub $name: unsafe extern "C" fn($($arg),*) -> $ret,)+
            // Unloading this LLVM DLL reproduced a process-teardown access
            // violation. Pin it for the process lifetime; the precise teardown
            // dependency remains to be audited. Each JIT is still reclaimed.
            _library: ManuallyDrop<Library>,
        }
        impl Api {
            pub fn load(path: &Path) -> Result<Self> {
                // SAFETY: caller selects the pinned, trusted native LLVM DLL.
                // The library remains owned by Api while any function is used.
                let library = unsafe { Library::new(path) }.map_err(|e| e.to_string())?;
                unsafe {
                    Ok(Self {
                        $($name: *library.get::<unsafe extern "C" fn($($arg),*) -> $ret>(
                            concat!(stringify!($name), "\0").as_bytes()
                        ).map_err(|e| format!("{}: {e}", stringify!($name)))?,)+
                        _library: ManuallyDrop::new(library),
                    })
                }
            }
        }
    }
}

api! {
    fn LLVMGetVersion(*mut c_uint, *mut c_uint, *mut c_uint) -> ();
    fn LLVMInitializeX86TargetInfo() -> ();
    fn LLVMInitializeX86Target() -> ();
    fn LLVMInitializeX86TargetMC() -> ();
    fn LLVMInitializeX86AsmPrinter() -> ();
    fn LLVMContextCreate() -> P;
    fn LLVMContextSetDiagnosticHandler(P, Option<unsafe extern "C" fn(P, P)>, P) -> ();
    fn LLVMGetDiagInfoDescription(P) -> *mut c_char;
    fn LLVMOrcCreateNewThreadSafeContextFromLLVMContext(P) -> P;
    fn LLVMOrcDisposeThreadSafeContext(P) -> ();
    fn LLVMCreateMemoryBufferWithMemoryRangeCopy(*const c_char, usize, *const c_char) -> P;
    fn LLVMDisposeMemoryBuffer(P) -> ();
    fn LLVMParseBitcodeInContext2(P, P, *mut P) -> c_int;
    fn LLVMDisposeModule(P) -> ();
    fn LLVMLinkModules2(P, P) -> c_int;
    fn LLVMVerifyModule(P, c_int, *mut *mut c_char) -> c_int;
    fn LLVMDisposeMessage(*mut c_char) -> ();
    fn LLVMPrintModuleToString(P) -> *mut c_char;
    fn LLVMGetTarget(P) -> *const c_char;
    fn LLVMGetNamedFunction(P, *const c_char) -> P;
    fn LLVMGetFirstFunction(P) -> P;
    fn LLVMGetNextFunction(P) -> P;
    fn LLVMGetFirstGlobal(P) -> P;
    fn LLVMGetNextGlobal(P) -> P;
    fn LLVMGetInitializer(P) -> P;
    fn LLVMIsAConstantDataArray(P) -> P;
    fn LLVMGetAsString(P, *mut usize) -> *const c_char;
    fn LLVMGetEnumAttributeKindForName(*const c_char, usize) -> c_uint;
    fn LLVMGetEnumAttributeAtIndex(P, c_uint, c_uint) -> P;
    fn LLVMAddCallSiteAttribute(P, c_uint, P) -> ();
    fn LLVMRemoveEnumAttributeAtIndex(P, c_uint, c_uint) -> ();
    fn LLVMGetValueName2(P, *mut usize) -> *const c_char;
    fn LLVMGetFirstBasicBlock(P) -> P;
    fn LLVMGetNextBasicBlock(P) -> P;
    fn LLVMInsertBasicBlockInContext(P, P, *const c_char) -> P;
    fn LLVMGetFirstInstruction(P) -> P;
    fn LLVMGetNextInstruction(P) -> P;
    fn LLVMInstructionEraseFromParent(P) -> ();
    fn LLVMIsACallInst(P) -> P;
    fn LLVMGetCalledValue(P) -> P;
    fn LLVMGetOperand(P, c_uint) -> P;
    fn LLVMGetParam(P, c_uint) -> P;
    fn LLVMGlobalGetValueType(P) -> P;
    fn LLVMGetReturnType(P) -> P;
    fn LLVMCountParamTypes(P) -> c_uint;
    fn LLVMGetParamTypes(P, *mut P) -> ();
    fn LLVMIsFunctionVarArg(P) -> c_int;
    fn LLVMGetFunctionCallConv(P) -> c_uint;
    fn LLVMDoubleTypeInContext(P) -> P;
    fn LLVMFloatTypeInContext(P) -> P;
    fn LLVMIntTypeInContext(P, c_uint) -> P;
    fn LLVMStructTypeInContext(P, *mut P, c_uint, c_int) -> P;
    fn LLVMVoidTypeInContext(P) -> P;
    fn LLVMPointerTypeInContext(P, c_uint) -> P;
    fn LLVMTypeOf(P) -> P;
    fn LLVMPrintTypeToString(P) -> *mut c_char;
    fn LLVMCreateBuilderInContext(P) -> P;
    fn LLVMDisposeBuilder(P) -> ();
    fn LLVMPositionBuilderBefore(P, P) -> ();
    fn LLVMPositionBuilderAtEnd(P, P) -> ();
    fn LLVMBuildCall2(P, P, P, *mut P, c_uint, *const c_char) -> P;
    fn LLVMBuildExtractValue(P, P, c_uint, *const c_char) -> P;
    fn LLVMBuildAlloca(P, P, *const c_char) -> P;
    fn LLVMBuildLoad2(P, P, P, *const c_char) -> P;
    fn LLVMBuildCondBr(P, P, P, P) -> P;
    fn LLVMBuildRet(P, P) -> P;
    fn LLVMBuildRetVoid(P) -> P;
    fn LLVMGetUndef(P) -> P;
    fn LLVMReplaceAllUsesWith(P, P) -> ();
    fn LLVMSetOperand(P, c_uint, P) -> ();
    fn LLVMGetTargetFromTriple(*const c_char, *mut P, *mut *mut c_char) -> c_int;
    fn LLVMCreateTargetMachine(P, *const c_char, *const c_char, *const c_char, c_int, c_int, c_int) -> P;
    fn LLVMDisposeTargetMachine(P) -> ();
    fn LLVMCreatePassBuilderOptions() -> P;
    fn LLVMDisposePassBuilderOptions(P) -> ();
    fn LLVMRunPasses(P, *const c_char, P, P) -> P;
    fn LLVMGetErrorMessage(P) -> *mut c_char;
    fn LLVMDisposeErrorMessage(*mut c_char) -> ();
    fn LLVMOrcJITTargetMachineBuilderCreateFromTargetMachine(P) -> P;
    fn LLVMOrcCreateLLJITBuilder() -> P;
    fn LLVMOrcLLJITBuilderSetJITTargetMachineBuilder(P, P) -> ();
    fn LLVMOrcCreateLLJIT(*mut P, P) -> P;
    fn LLVMOrcDisposeLLJIT(P) -> P;
    fn LLVMOrcLLJITGetMainJITDylib(P) -> P;
    fn LLVMOrcLLJITGetGlobalPrefix(P) -> c_char;
    fn LLVMOrcCreateDynamicLibrarySearchGeneratorForPath(*mut P, *const c_char, c_char, Option<unsafe extern "C" fn(P, P) -> c_int>, P) -> P;
    fn LLVMOrcJITDylibAddGenerator(P, P) -> ();
    fn LLVMOrcCreateNewThreadSafeModule(P, P) -> P;
    fn LLVMOrcLLJITAddLLVMIRModule(P, P, P) -> P;
    fn LLVMOrcLLJITLookup(P, *mut u64, *const c_char) -> P;
}

impl Api {
    pub fn initialize(&self) -> Result<[u32; 3]> {
        unsafe {
            let (mut major, mut minor, mut patch) = (0, 0, 0);
            (self.LLVMGetVersion)(&mut major, &mut minor, &mut patch);
            let version = [major, minor, patch];
            if version != [21, 1, 3] {
                return Err(format!("LLVM version mismatch: {version:?}"));
            }
            static INITIALIZE: std::sync::Once = std::sync::Once::new();
            INITIALIZE.call_once(|| {
                (self.LLVMInitializeX86TargetInfo)();
                (self.LLVMInitializeX86Target)();
                (self.LLVMInitializeX86TargetMC)();
                (self.LLVMInitializeX86AsmPrinter)();
            });
            Ok(version)
        }
    }

    fn error(&self, error: P) -> Result<()> {
        if error.is_null() {
            return Ok(());
        }
        unsafe {
            let message = (self.LLVMGetErrorMessage)(error);
            let text = CStr::from_ptr(message).to_string_lossy().into_owned();
            (self.LLVMDisposeErrorMessage)(message);
            Err(text)
        }
    }

    fn name(&self, value: P) -> String {
        unsafe {
            let mut len = 0;
            let bytes = (self.LLVMGetValueName2)(value, &mut len);
            if len == 0 {
                return String::new();
            }
            String::from_utf8_lossy(std::slice::from_raw_parts(bytes.cast(), len)).into_owned()
        }
    }
}

struct OwnedModule<'a> {
    api: &'a Api,
    context: P,
    raw_context: P,
    module: P,
    index: Index,
    handler_mods: BTreeMap<String, usize>,
}

struct ResolvedPatch<'a> {
    metadata: &'a Handler,
    report: PlannedPatch,
    original: P,
    handler: P,
    entry: P,
}

unsafe extern "C" fn capture_llvm_diagnostic(info: P, data: P) {
    // The caller installs this callback only during a synchronous LLVM operation.
    unsafe {
        let (api, message) = &mut *data.cast::<(&Api, Option<String>)>();
        let description = (api.LLVMGetDiagInfoDescription)(info);
        if !description.is_null() {
            *message = Some(CStr::from_ptr(description).to_string_lossy().into_owned());
            (api.LLVMDisposeMessage)(description);
        }
    }
}

fn capture_llvm_failure(
    api: &Api,
    context: P,
    fallback: &str,
    operation: impl FnOnce() -> c_int,
) -> Result<()> {
    let mut diagnostic = (api, None);
    // LLVM's default error handler exits the process. Keep the callback data
    // alive for this operation and clear the callback before reusing the context.
    let failed = unsafe {
        (api.LLVMContextSetDiagnosticHandler)(
            context,
            Some(capture_llvm_diagnostic),
            (&mut diagnostic as *mut (&Api, Option<String>)).cast(),
        );
        let failed = operation();
        (api.LLVMContextSetDiagnosticHandler)(context, None, ptr::null_mut());
        failed
    };
    if failed == 0 {
        Ok(())
    } else {
        Err(diagnostic.1.unwrap_or_else(|| fallback.into()))
    }
}

impl Drop for OwnedModule<'_> {
    fn drop(&mut self) {
        unsafe {
            if !self.module.is_null() {
                (self.api.LLVMDisposeModule)(self.module);
            }
            (self.api.LLVMOrcDisposeThreadSafeContext)(self.context);
        }
    }
}

impl<'a> OwnedModule<'a> {
    fn parse(api: &'a Api, game: &[u8], handlers: &[&[u8]]) -> DiagnosticResult<Self> {
        unsafe {
            let raw_context = (api.LLVMContextCreate)();
            let context = (api.LLVMOrcCreateNewThreadSafeContextFromLLVMContext)(raw_context);
            let mut owned = Self {
                api,
                context,
                raw_context,
                module: ptr::null_mut(),
                index: Index::default(),
                handler_mods: BTreeMap::new(),
            };
            for (origin, bytes) in std::iter::once(game)
                .chain(handlers.iter().copied())
                .enumerate()
            {
                let mod_index = origin.checked_sub(1);
                let buffer = (api.LLVMCreateMemoryBufferWithMemoryRangeCopy)(
                    bytes.as_ptr().cast(),
                    bytes.len(),
                    c"retained-rust".as_ptr(),
                );
                let mut module = ptr::null_mut();
                let parsed =
                    capture_llvm_failure(api, raw_context, "Cannot parse rustc bitcode", || {
                        (api.LLVMParseBitcodeInContext2)(raw_context, buffer, &mut module)
                    });
                (api.LLVMDisposeMemoryBuffer)(buffer);
                parsed.map_err(|error| Diagnostic::new(Stage::Parse, error).with_mod(mod_index))?;
                let triple = CStr::from_ptr((api.LLVMGetTarget)(module));
                if triple != c"x86_64-pc-windows-msvc" {
                    let message =
                        format!("Unexpected bitcode target: {}", triple.to_string_lossy());
                    (api.LLVMDisposeModule)(module);
                    return Err(Diagnostic::new(Stage::Parse, message).with_mod(mod_index));
                }
                let records = match Self::records(api, module) {
                    Ok(records) => records,
                    Err(e) => {
                        (api.LLVMDisposeModule)(module);
                        return Err(Diagnostic::new(Stage::Index, e).with_mod(mod_index));
                    }
                };
                let first_handler = owned.index.handlers.len();
                if let Err(e) = owned.index.append_detailed(records, origin == 0) {
                    (api.LLVMDisposeModule)(module);
                    return Err(e.with_mod(mod_index));
                }
                if let Some(mod_index) = mod_index {
                    for handler in &owned.index.handlers[first_handler..] {
                        owned.handler_mods.insert(handler.id.clone(), mod_index);
                    }
                }
                if owned.module.is_null() {
                    owned.module = module;
                } else {
                    // LLVMLinkModules2 consumes the source even on failure.
                    capture_llvm_failure(
                        api,
                        raw_context,
                        "Cannot link game and mod bitcode",
                        || (api.LLVMLinkModules2)(owned.module, module),
                    )
                    .map_err(|error| Diagnostic::new(Stage::Link, error).with_mod(mod_index))?;
                }
            }
            owned
                .verify()
                .map_err(|error| Diagnostic::new(Stage::Verify, error))?;
            if owned.index.targets.is_empty() && owned.index.typed_return_targets.is_empty() {
                return Err(Diagnostic::new(
                    Stage::Index,
                    "missing compiled Mixin metadata",
                ));
            }
            Ok(owned)
        }
    }

    fn records(api: &Api, module: P) -> Result<Vec<Record>> {
        unsafe {
            let mut global = (api.LLVMGetFirstGlobal)(module);
            let mut result = Vec::new();
            while !global.is_null() {
                if api.name(global).starts_with("__rsmx_meta_") {
                    let init = (api.LLVMGetInitializer)(global);
                    if init.is_null() || (api.LLVMIsAConstantDataArray)(init).is_null() {
                        return Err("metadata must be a constant byte array".into());
                    }
                    let mut len = 0;
                    let ptr = (api.LLVMGetAsString)(init, &mut len);
                    if len == 0 || len > 65536 || result.len() >= 256 {
                        return Err("metadata budget exceeded".into());
                    }
                    let bytes = std::slice::from_raw_parts(ptr.cast(), len);
                    result
                        .push(serde_json::from_slice(bytes).map_err(|e| format!("metadata: {e}"))?);
                }
                global = (api.LLVMGetNextGlobal)(global);
            }
            Ok(result)
        }
    }

    fn named(&self, name: &str, args: &[ScalarType], result: ScalarType) -> Result<P> {
        let name = CString::new(name).map_err(|_| "NUL in symbol")?;
        self.function(&name, args, result)
    }

    fn handler_error(
        &self,
        stage: Stage,
        handler: &Handler,
        actual: Option<usize>,
        message: impl Into<String>,
    ) -> Diagnostic {
        Diagnostic::new(stage, message)
            .with_mod(self.handler_mods.get(&handler.id).copied())
            .with_handler(handler, actual)
    }

    fn validate_index(&self) -> DiagnosticResult<()> {
        {
            let mut symbols = std::collections::BTreeSet::new();
            for t in self.index.targets.values() {
                let target_error =
                    |error| Diagnostic::new(Stage::Index, format!("{}: {error}", t.logical_id));
                self.named(&t.symbol, t.parameter_types(), t.return_type())
                    .map_err(target_error)?;
                if t.schema == 1 {
                    self.named(&t.entry, t.parameter_types(), t.return_type())
                } else {
                    self.named(&t.entry, &[ScalarType::U64; 4], ScalarType::U64)
                }
                .map_err(target_error)?;
                if !symbols.insert(t.symbol.clone()) || !symbols.insert(t.entry.clone()) {
                    return Err(target_error("duplicate indexed symbol".into()));
                }
                if let Some(hook) = &t.return_hook {
                    self.validate_return_hook(&t.symbol, hook, &mut symbols)
                        .map_err(target_error)?;
                }
            }
            for t in self.index.typed_return_targets.values() {
                let target_error =
                    |error| Diagnostic::new(Stage::Index, format!("{}: {error}", t.logical_id));
                self.validate_return_hook(&t.symbol, &t.return_hook, &mut symbols)
                    .map_err(target_error)?;
                let mut args = vec![self.llvm_type(ScalarType::U64); 4];
                args.push(self.pointer_type());
                let entry =
                    CString::new(t.entry.as_str()).map_err(|e| target_error(e.to_string()))?;
                self.function_with_llvm_types(&entry, &args, self.llvm_type(ScalarType::Bool))
                    .map_err(target_error)?;
                for symbol in [&t.symbol, &t.entry] {
                    if !symbols.insert(symbol.clone()) {
                        return Err(target_error("duplicate indexed symbol".into()));
                    }
                }
            }
            for h in &self.index.handlers {
                self.handler_function(h)
                    .map_err(|error| self.handler_error(Stage::Index, h, None, error))?;
                if !symbols.insert(h.symbol.clone()) {
                    return Err(self.handler_error(
                        Stage::Index,
                        h,
                        None,
                        "duplicate handler symbol",
                    ));
                }
                if let Some(probe) = &h.return_type_probe {
                    self.pointer_callback(probe)
                        .and_then(|_| {
                            if symbols.insert(probe.clone()) {
                                Ok(())
                            } else {
                                Err("duplicate return type probe".into())
                            }
                        })
                        .map_err(|error| self.handler_error(Stage::Index, h, None, error))?;
                }
            }
            Ok(())
        }
    }

    fn verify(&self) -> Result<()> {
        unsafe {
            let mut message = ptr::null_mut();
            let failed = (self.api.LLVMVerifyModule)(self.module, 2, &mut message);
            let text = if message.is_null() {
                String::new()
            } else {
                let text = CStr::from_ptr(message).to_string_lossy().into_owned();
                (self.api.LLVMDisposeMessage)(message);
                text
            };
            if failed != 0 { Err(text) } else { Ok(()) }
        }
    }

    fn llvm_type(&self, kind: ScalarType) -> P {
        unsafe {
            let api = self.api;
            let ctx = self.raw_context;
            let bits = match kind {
                ScalarType::F32 => return (api.LLVMFloatTypeInContext)(ctx),
                ScalarType::F64 => return (api.LLVMDoubleTypeInContext)(ctx),
                ScalarType::Unit => return (api.LLVMVoidTypeInContext)(ctx),
                ScalarType::Vec2 => {
                    // Vec2 is two f32 fields in the pinned Windows x64 Rust ABI.
                    let mut fields = [(api.LLVMFloatTypeInContext)(ctx); 2];
                    return (api.LLVMStructTypeInContext)(ctx, fields.as_mut_ptr(), 2, 0);
                }
                ScalarType::Bool => 1,
                ScalarType::I8 | ScalarType::U8 => 8,
                ScalarType::I16 | ScalarType::U16 => 16,
                ScalarType::I32
                | ScalarType::U32
                | ScalarType::EntityId
                | ScalarType::OptionEntityId => 32,
                ScalarType::I64 | ScalarType::U64 | ScalarType::Isize | ScalarType::Usize => 64,
            };
            (api.LLVMIntTypeInContext)(ctx, bits)
        }
    }

    fn function(&self, name: &CStr, args: &[ScalarType], result: ScalarType) -> Result<P> {
        let args: Vec<_> = args.iter().map(|kind| self.llvm_type(*kind)).collect();
        self.function_with_llvm_types(name, &args, self.llvm_type(result))
    }

    fn handler_function(&self, handler: &Handler) -> Result<P> {
        if handler.return_type_probe.is_some() {
            return self.pointer_callback(&handler.symbol);
        }
        if let Some(signature) = &handler.head_signature {
            let mut args: Vec<_> = signature
                .parameters
                .iter()
                .map(|kind| self.llvm_type(*kind))
                .collect();
            if signature.result != ScalarType::Unit {
                args.push(self.pointer_type());
            }
            let name = CString::new(handler.symbol.as_str()).map_err(|_| "NUL in symbol")?;
            return self.function_with_llvm_types(&name, &args, self.llvm_type(ScalarType::Bool));
        }
        let kind = handler.value_type();
        let args = match kind {
            ScalarType::Unit => &[][..],
            ScalarType::Vec2 => &[ScalarType::F32, ScalarType::F32],
            _ => std::slice::from_ref(&kind),
        };
        self.named(&handler.symbol, args, kind)
    }

    fn pointer_type(&self) -> P {
        unsafe { (self.api.LLVMPointerTypeInContext)(self.raw_context, 0) }
    }

    fn pointer_callback(&self, name: &str) -> Result<P> {
        let name = CString::new(name).map_err(|_| "NUL in symbol")?;
        self.function_with_llvm_types(
            &name,
            &[self.pointer_type()],
            self.llvm_type(ScalarType::Unit),
        )
    }

    fn find_function(&self, name: &CStr) -> Result<P> {
        let function = unsafe { (self.api.LLVMGetNamedFunction)(self.module, name.as_ptr()) };
        if function.is_null() {
            return Err(format!("Missing function: {name:?}"));
        }
        Ok(function)
    }

    fn function_with_llvm_types(&self, name: &CStr, args: &[P], result: P) -> Result<P> {
        unsafe {
            let api = self.api;
            let function = self.find_function(name)?;
            let ty = (api.LLVMGlobalGetValueType)(function);
            let mut actual = vec![ptr::null_mut(); (api.LLVMCountParamTypes)(ty) as usize];
            (api.LLVMGetParamTypes)(ty, actual.as_mut_ptr());
            let actual_result = (api.LLVMGetReturnType)(ty);
            if actual_result != result {
                let describe = |ty| {
                    let message = (api.LLVMPrintTypeToString)(ty);
                    let text = CStr::from_ptr(message).to_string_lossy().into_owned();
                    (api.LLVMDisposeMessage)(message);
                    text
                };
                return Err(format!(
                    "LLVM signature mismatch for {name:?}: expected return {}, got {}",
                    describe(result),
                    describe(actual_result)
                ));
            }
            if actual != args
                || (api.LLVMIsFunctionVarArg)(ty) != 0
                || (api.LLVMGetFunctionCallConv)(function) != 0
            {
                return Err(format!("C ABI signature mismatch: {name:?}"));
            }
            Ok(function)
        }
    }

    fn calls(&self, function: P) -> Vec<P> {
        unsafe {
            let mut result = Vec::new();
            let mut block = (self.api.LLVMGetFirstBasicBlock)(function);
            while !block.is_null() {
                let mut instruction = (self.api.LLVMGetFirstInstruction)(block);
                while !instruction.is_null() {
                    if !(self.api.LLVMIsACallInst)(instruction).is_null() {
                        result.push(instruction);
                    }
                    instruction = (self.api.LLVMGetNextInstruction)(instruction);
                }
                block = (self.api.LLVMGetNextBasicBlock)(block);
            }
            result
        }
    }

    fn validate_return_hook(
        &self,
        symbol: &str,
        hook: &ReturnHook,
        symbols: &mut std::collections::BTreeSet<String>,
    ) -> Result<()> {
        self.return_site(symbol, hook)?;
        self.pointer_callback(&hook.type_probe)?;
        self.return_body(hook)?;
        for name in [&hook.symbol, &hook.type_probe, &hook.body] {
            if !symbols.insert(name.clone()) {
                return Err("duplicate indexed return symbol".into());
            }
        }
        Ok(())
    }

    fn return_body(&self, hook: &ReturnHook) -> Result<P> {
        let name = CString::new(hook.body.as_str()).map_err(|_| "NUL in symbol")?;
        self.find_function(&name)
    }

    fn return_site(&self, symbol: &str, hook: &ReturnHook) -> Result<(P, P)> {
        let symbol = CString::new(symbol).map_err(|_| "NUL in symbol")?;
        let function = self.find_function(&symbol)?;
        let hook = self.pointer_callback(&hook.symbol)?;
        let sites: Vec<_> = self
            .calls(function)
            .into_iter()
            .filter(|call| unsafe { (self.api.LLVMGetCalledValue)(*call) == hook })
            .collect();
        match sites.as_slice() {
            [site] => Ok((function, *site)),
            _ => Err(format!("expected one return hook, found {}", sites.len())),
        }
    }

    fn index(&self) -> Vec<FunctionIndex> {
        unsafe {
            let mut result = Vec::new();
            let mut function = (self.api.LLVMGetFirstFunction)(self.module);
            while !function.is_null() {
                result.push(FunctionIndex {
                    function: self.api.name(function),
                    direct_calls: self
                        .calls(function)
                        .into_iter()
                        .map(|call| self.api.name((self.api.LLVMGetCalledValue)(call)))
                        .collect(),
                });
                function = (self.api.LLVMGetNextFunction)(function);
            }
            result
        }
    }

    fn resolve_plan(&self) -> DiagnosticResult<Vec<ResolvedPatch<'_>>> {
        self.validate_index()?;
        let mut handlers: Vec<_> = self.index.handlers.iter().collect();
        // Stable sort retains input-mod order, then per-mod ID order.
        handlers.sort_by_key(|h| h.priority);
        let mut resolved = Vec::new();
        unsafe {
            let api = self.api;
            for h in handlers {
                let selector_error =
                    |message| self.handler_error(Stage::ResolveSelector, h, None, message);
                if h.operation == Operation::ReturnResult {
                    resolved.push(self.resolve_return(h)?);
                    continue;
                }
                let t = self.index.target(&h.target).map_err(selector_error)?;
                let target = self
                    .named(&t.symbol, t.parameter_types(), t.return_type())
                    .map_err(selector_error)?;
                let handler = self.handler_function(h).map_err(selector_error)?;
                let (operation, callee, callee_symbol, sites, expected_type) = if let Some(
                    signature,
                ) =
                    &h.head_signature
                {
                    if signature.parameters != t.parameter_types()
                        || signature.result != t.return_type()
                    {
                        return Err(selector_error(format!(
                            "entry handler signature mismatch: expected ({}) -> {}, got ({}) -> {}",
                            format_types(t.parameter_types()),
                            t.return_type(),
                            format_types(&signature.parameters),
                            signature.result,
                        )));
                    }
                    let sites = if (api.LLVMGetFirstBasicBlock)(target).is_null() {
                        Vec::new()
                    } else {
                        vec![target]
                    };
                    (
                        Operation::InjectHead,
                        "head".into(),
                        String::new(),
                        sites,
                        t.return_type(),
                    )
                } else if h.operation == Operation::Parameter {
                    let parameter = h
                        .parameter
                        .ok_or_else(|| selector_error("missing parameter".into()))?;
                    if parameter >= t.arity {
                        return Err(self.handler_error(
                            Stage::ResolveSelector,
                            h,
                            Some(0),
                            format!(
                                "parameter {parameter} out of range for {} parameters",
                                t.arity
                            ),
                        ));
                    }
                    (
                        Operation::Parameter,
                        format!("parameter:{parameter}"),
                        String::new(),
                        vec![(api.LLVMGetParam)(target, parameter as u32)],
                        t.parameter_types()[parameter],
                    )
                } else if h.operation == Operation::Head {
                    if t.arity == 0 {
                        return Err(self.handler_error(
                            Stage::ResolveSelector,
                            h,
                            Some(0),
                            "modify_head target needs at least one parameter",
                        ));
                    }
                    (
                        Operation::Head,
                        "head".into(),
                        String::new(),
                        vec![(api.LLVMGetParam)(target, 0)],
                        t.parameter_types()[0],
                    )
                } else {
                    let callee_id = h
                        .callee
                        .as_deref()
                        .ok_or_else(|| selector_error("missing callee".into()))?;
                    let c = self.index.target(callee_id).map_err(selector_error)?;
                    let callee = self
                        .named(&c.symbol, c.parameter_types(), c.return_type())
                        .map_err(selector_error)?;
                    let body = if let Some(hook) = &t.return_hook {
                        self.return_body(hook).map_err(selector_error)?
                    } else {
                        target
                    };
                    (
                        Operation::CallResult,
                        callee_id.into(),
                        c.symbol.clone(),
                        self.calls(body)
                            .into_iter()
                            .filter(|call| (api.LLVMGetCalledValue)(*call) == callee)
                            .collect::<Vec<_>>(),
                        c.return_type(),
                    )
                };
                if sites.len() != h.require_matches || h.ordinal >= sites.len() {
                    return Err(self.handler_error(
                        Stage::ResolveSelector,
                        h,
                        Some(sites.len()),
                        format!(
                            "{}: expected {} original matches in {}, found {}; ordinal {}",
                            h.id,
                            h.require_matches,
                            h.target,
                            sites.len(),
                            h.ordinal
                        ),
                    ));
                }
                let value_type = h
                    .head_signature
                    .as_ref()
                    .map_or_else(|| h.value_type(), |signature| signature.result);
                if value_type != expected_type {
                    return Err(self.handler_error(
                        Stage::ResolveSelector,
                        h,
                        Some(sites.len()),
                        format!(
                            "handler type mismatch: expected {expected_type}, got {value_type}"
                        ),
                    ));
                }
                let block = (api.LLVMGetFirstBasicBlock)(target);
                let entry = if block.is_null() {
                    ptr::null_mut()
                } else {
                    (api.LLVMGetFirstInstruction)(block)
                };
                if matches!(operation, Operation::Parameter | Operation::Head) && entry.is_null() {
                    return Err(self.handler_error(
                        Stage::ResolveSelector,
                        h,
                        Some(0),
                        "parameter target has no body",
                    ));
                }
                let report = PlannedPatch {
                    id: h.id.clone(),
                    target: h.target.clone(),
                    target_symbol: t.symbol.clone(),
                    operation,
                    callee,
                    callee_symbol,
                    handler_symbol: h.symbol.clone(),
                    ordinal: h.ordinal,
                    matches: sites.len(),
                    priority: h.priority,
                    parameter: h.parameter,
                    value_type: Some(value_type),
                    head_signature: h.head_signature.clone(),
                };
                resolved.push(ResolvedPatch {
                    metadata: h,
                    report,
                    original: sites[h.ordinal],
                    handler,
                    entry,
                });
            }
            Ok(resolved)
        }
    }

    fn resolve_return<'b>(&self, handler: &'b Handler) -> DiagnosticResult<ResolvedPatch<'b>> {
        let error =
            |actual, message| self.handler_error(Stage::ResolveSelector, handler, actual, message);
        if handler.return_type_probe.is_none() {
            return Err(error(None, "missing return type probe".into()));
        }
        let (symbol, hook) = self.index.return_hook(&handler.target).ok_or_else(|| {
            let message = if self.index.targets.contains_key(&handler.target) {
                format!(
                    "{} has no return hook; rebuild the game with the current SDK",
                    handler.target
                )
            } else {
                format!("logical target not found: {}", handler.target)
            };
            error(None, message)
        })?;
        let (_, site) = self
            .return_site(symbol, hook)
            .map_err(|reason| error(None, reason))?;
        if handler.require_matches != 1 || handler.ordinal != 0 {
            return Err(error(
                Some(1),
                "modify_return_result targets have exactly one hook".into(),
            ));
        }
        Ok(ResolvedPatch {
            metadata: handler,
            report: PlannedPatch {
                id: handler.id.clone(),
                target: handler.target.clone(),
                target_symbol: symbol.into(),
                operation: Operation::ReturnResult,
                callee: "return".into(),
                callee_symbol: hook.symbol.clone(),
                handler_symbol: handler.symbol.clone(),
                ordinal: 0,
                matches: 1,
                priority: handler.priority,
                parameter: None,
                value_type: self
                    .index
                    .targets
                    .get(&handler.target)
                    .map(|target| target.return_type()),
                head_signature: None,
            },
            original: site,
            handler: self
                .handler_function(handler)
                .map_err(|reason| error(Some(1), reason))?,
            entry: ptr::null_mut(),
        })
    }

    fn apply_plan(&self, resolved: &[ResolvedPatch<'_>]) -> DiagnosticResult<()> {
        unsafe {
            let api = self.api;
            let mut current: BTreeMap<usize, P> = BTreeMap::new();
            for &ResolvedPatch {
                metadata,
                original,
                handler,
                entry,
                ref report,
            } in resolved
            {
                let operation = report.operation;
                if operation == Operation::InjectHead {
                    continue;
                }
                let patch_error = |stage, message| {
                    self.handler_error(stage, metadata, Some(report.matches), message)
                };
                let transform_error = |error| patch_error(Stage::Transform, error);
                if operation == Operation::ReturnResult {
                    let slot = (api.LLVMGetOperand)(original, 0);
                    self.call_before(&mut [slot], original, handler);
                    self.verify()
                        .map_err(|error| patch_error(Stage::Verify, error))?;
                    continue;
                }
                let value_type = report.value_type.expect("validated scalar patch");
                let input = current
                    .get(&(original as usize))
                    .copied()
                    .unwrap_or(original);
                if value_type == ScalarType::Unit {
                    let insertion = self.next_instruction(original).map_err(transform_error)?;
                    self.call_before(&mut [], insertion, handler);
                } else {
                    let result = if matches!(operation, Operation::Parameter | Operation::Head) {
                        if input == original {
                            self.wrap_before(input, entry, handler, value_type)
                        } else {
                            self.wrap(input, handler, value_type)
                                .map_err(transform_error)?
                        }
                    } else {
                        self.wrap(input, handler, value_type)
                            .map_err(transform_error)?
                    };
                    current.insert(original as usize, result);
                }
                self.verify()
                    .map_err(|error| patch_error(Stage::Verify, error))?;
            }
            // Prepend in reverse order so callbacks see the original parameters
            // and run in plan order before any parameter rewriting.
            for patch in resolved.iter().rev() {
                if let Some(signature) = &patch.metadata.head_signature {
                    self.inject_head(patch.original, patch.handler, signature);
                    self.verify().map_err(|error| {
                        self.handler_error(
                            Stage::Verify,
                            patch.metadata,
                            Some(patch.report.matches),
                            error,
                        )
                    })?;
                }
            }
            Ok(())
        }
    }

    fn release_retention_barriers(&self) -> Result<()> {
        unsafe {
            let kind = (self.api.LLVMGetEnumAttributeKindForName)(c"noinline".as_ptr(), 8);
            if kind == 0 {
                return Err("LLVM noinline attribute kind unavailable".into());
            }
            for target in self.index.targets.values() {
                let function = if let Some(hook) = &target.return_hook {
                    self.release_return_hook(&target.symbol, hook, kind)?
                } else {
                    self.named(
                        &target.symbol,
                        target.parameter_types(),
                        target.return_type(),
                    )?
                };
                // Only macro-generated, explicitly recorded barriers are removed.
                (self.api.LLVMRemoveEnumAttributeAtIndex)(function, u32::MAX, kind);
            }
            for target in self.index.typed_return_targets.values() {
                let function =
                    self.release_return_hook(&target.symbol, &target.return_hook, kind)?;
                (self.api.LLVMRemoveEnumAttributeAtIndex)(function, u32::MAX, kind);
            }
            Ok(())
        }
    }

    fn release_return_hook(&self, symbol: &str, hook: &ReturnHook, noinline: c_uint) -> Result<P> {
        let (function, site) = self.return_site(symbol, hook)?;
        let body = self.return_body(hook)?;
        unsafe {
            (self.api.LLVMInstructionEraseFromParent)(site);
            (self.api.LLVMRemoveEnumAttributeAtIndex)(body, u32::MAX, noinline);
        }
        Ok(function)
    }

    fn next_instruction(&self, original: P) -> Result<P> {
        let next = unsafe { (self.api.LLVMGetNextInstruction)(original) };
        if next.is_null() {
            return Err("Unsupported call position".into());
        }
        Ok(next)
    }

    fn wrap(&self, original: P, handler: P, kind: ScalarType) -> Result<P> {
        Ok(self.wrap_before(original, self.next_instruction(original)?, handler, kind))
    }

    fn wrap_before(&self, original: P, insertion: P, handler: P, kind: ScalarType) -> P {
        unsafe {
            let api = self.api;
            if kind == ScalarType::Vec2 {
                let (wrapped, extracts) = self.call_vec2_before(original, insertion, handler);
                (api.LLVMReplaceAllUsesWith)(original, wrapped);
                // The new extracts must still read the value produced before wrapping.
                for extract in extracts {
                    (api.LLVMSetOperand)(extract, 0, original);
                }
                return wrapped;
            }
            // Build with undef first, so RAUW does not turn the handler input
            // into a self-reference. Then set the sole argument to original.
            let input = (api.LLVMGetUndef)((api.LLVMTypeOf)(original));
            let wrapped = self.call_before(&mut [input], insertion, handler);
            (api.LLVMReplaceAllUsesWith)(original, wrapped);
            (api.LLVMSetOperand)(wrapped, 0, original);
            wrapped
        }
    }

    fn call_before(&self, args: &mut [P], insertion: P, handler: P) -> P {
        unsafe {
            let api = self.api;
            let builder = (api.LLVMCreateBuilderInContext)(self.raw_context);
            (api.LLVMPositionBuilderBefore)(builder, insertion);
            let call = self.build_call(builder, handler, args);
            (api.LLVMDisposeBuilder)(builder);
            call
        }
    }

    fn call_vec2_before(&self, vector: P, insertion: P, handler: P) -> (P, [P; 2]) {
        unsafe {
            let api = self.api;
            let builder = (api.LLVMCreateBuilderInContext)(self.raw_context);
            (api.LLVMPositionBuilderBefore)(builder, insertion);
            let x = (api.LLVMBuildExtractValue)(builder, vector, 0, c"mixin.x".as_ptr());
            let y = (api.LLVMBuildExtractValue)(builder, vector, 1, c"mixin.y".as_ptr());
            let result = self.build_call(builder, handler, &mut [x, y]);
            (api.LLVMDisposeBuilder)(builder);
            (result, [x, y])
        }
    }

    fn build_call(&self, builder: P, handler: P, args: &mut [P]) -> P {
        unsafe {
            let api = self.api;
            let function_type = (api.LLVMGlobalGetValueType)(handler);
            let name = if (api.LLVMGetReturnType)(function_type) == self.llvm_type(ScalarType::Unit)
            {
                c""
            } else {
                c"mixin.result"
            };
            let result = (api.LLVMBuildCall2)(
                builder,
                function_type,
                handler,
                args.as_mut_ptr(),
                args.len() as u32,
                name.as_ptr(),
            );
            // Small integer return values and arguments can carry C ABI extensions.
            for attribute in [c"signext", c"zeroext"] {
                let kind = (api.LLVMGetEnumAttributeKindForName)(
                    attribute.as_ptr(),
                    attribute.to_bytes().len(),
                );
                for index in 0..=args.len() as u32 {
                    let value = (api.LLVMGetEnumAttributeAtIndex)(handler, index, kind);
                    if !value.is_null() {
                        (api.LLVMAddCallSiteAttribute)(result, index, value);
                    }
                }
            }
            result
        }
    }

    fn inject_head(&self, target: P, handler: P, signature: &crate::metadata::Signature) {
        unsafe {
            let api = self.api;
            let next = (api.LLVMGetFirstBasicBlock)(target);
            let head =
                (api.LLVMInsertBasicBlockInContext)(self.raw_context, next, c"mixin.head".as_ptr());
            let cancel = (api.LLVMInsertBasicBlockInContext)(
                self.raw_context,
                next,
                c"mixin.cancel".as_ptr(),
            );
            let builder = (api.LLVMCreateBuilderInContext)(self.raw_context);
            (api.LLVMPositionBuilderAtEnd)(builder, head);
            let mut args: Vec<_> = (0..signature.parameters.len() as u32)
                .map(|index| (api.LLVMGetParam)(target, index))
                .collect();
            let output = if signature.result == ScalarType::Unit {
                None
            } else {
                let result = self.llvm_type(signature.result);
                let slot = (api.LLVMBuildAlloca)(builder, result, c"mixin.output".as_ptr());
                args.push(slot);
                Some((result, slot))
            };
            let stop = self.build_call(builder, handler, &mut args);
            (api.LLVMBuildCondBr)(builder, stop, cancel, next);
            (api.LLVMPositionBuilderAtEnd)(builder, cancel);
            if let Some((result, slot)) = output {
                let value = (api.LLVMBuildLoad2)(builder, result, slot, c"mixin.value".as_ptr());
                (api.LLVMBuildRet)(builder, value);
            } else {
                (api.LLVMBuildRetVoid)(builder);
            }
            (api.LLVMDisposeBuilder)(builder);
        }
    }

    fn ir(&self) -> String {
        unsafe {
            let message = (self.api.LLVMPrintModuleToString)(self.module);
            let text = CStr::from_ptr(message).to_string_lossy().into_owned();
            (self.api.LLVMDisposeMessage)(message);
            text
        }
    }
}

fn format_types(types: &[ScalarType]) -> String {
    types
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

#[derive(Debug, Serialize)]
pub struct FunctionIndex {
    pub function: String,
    pub direct_calls: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Timing {
    pub parse_link_ms: f64,
    pub select_patch_ms: f64,
    pub optimize_ms: f64,
    pub create_jit_ms: f64,
    pub materialize_ms: f64,
    pub total_ms: f64,
}

pub struct Evidence {
    pub before: Option<String>,
    pub after: Option<String>,
    pub index: Vec<FunctionIndex>,
    pub logical_index: Index,
    pub plan: Vec<PlannedPatch>,
}

#[derive(Serialize)]
pub struct Plan {
    pub index: Index,
    pub patches: Vec<PlannedPatch>,
}

pub struct Jit<'a> {
    api: &'a Api,
    raw: P,
    index: Index,
    _single_thread: PhantomData<Rc<()>>,
}

impl Drop for Jit<'_> {
    fn drop(&mut self) {
        // All calls are synchronous and completed before the owner is dropped.
        if let Err(error) = unsafe { self.api.error((self.api.LLVMOrcDisposeLLJIT)(self.raw)) } {
            eprintln!("JIT disposal: {error}");
        }
    }
}

impl<'a> Jit<'a> {
    /// Parse game and mod bitcode, validate all selectors, and return the
    /// resolved plan without changing the LLVM module or creating a JIT.
    pub fn plan_many(api: &'a Api, game: &[u8], handlers: &[&[u8]]) -> DiagnosticResult<Plan> {
        let mut owned = OwnedModule::parse(api, game, handlers)?;
        let patches = owned
            .resolve_plan()?
            .into_iter()
            .map(|patch| patch.report)
            .collect();
        Ok(Plan {
            index: std::mem::take(&mut owned.index),
            patches,
        })
    }

    pub fn compile(
        api: &'a Api,
        game: &[u8],
        handler: &[u8],
        patched: bool,
        capture: bool,
    ) -> Result<(Self, Timing, Evidence)> {
        Self::compile_many(api, game, &[handler], patched, capture)
    }

    pub fn compile_many(
        api: &'a Api,
        game: &[u8],
        handlers: &[&[u8]],
        patched: bool,
        capture: bool,
    ) -> Result<(Self, Timing, Evidence)> {
        Self::compile_many_detailed(api, game, handlers, patched, capture)
            .map_err(|error| error.message)
    }

    /// Compile with structured diagnostics for each runtime stage.
    pub fn compile_many_detailed(
        api: &'a Api,
        game: &[u8],
        handlers: &[&[u8]],
        patched: bool,
        capture: bool,
    ) -> DiagnosticResult<(Self, Timing, Evidence)> {
        Self::compile_with_index(api, game, handlers, patched, capture, &[], |_| {})
    }

    /// Compile a patched component using the supplied native libraries to resolve imports.
    ///
    /// # Safety
    /// The libraries must be trusted and provide the exact ABI expected by the
    /// bitcode. LLVM keeps the libraries loaded for the process lifetime.
    pub unsafe fn compile_with_libraries(
        api: &'a Api,
        game: &[u8],
        handlers: &[&[u8]],
        libraries: &[&Path],
    ) -> DiagnosticResult<Self> {
        Self::compile_with_index(api, game, handlers, true, false, libraries, |_| {})
            .map(|(jit, _, _)| jit)
    }

    fn compile_with_index(
        api: &'a Api,
        game: &[u8],
        handlers: &[&[u8]],
        patched: bool,
        capture: bool,
        libraries: &[&Path],
        edit: impl FnOnce(&mut Index),
    ) -> DiagnosticResult<(Self, Timing, Evidence)> {
        let start = Instant::now();
        let mut owned = OwnedModule::parse(api, game, handlers)?;
        edit(&mut owned.index);
        let parse_end = Instant::now();
        // SAFETY: all API references belong to owned's live context/module;
        // only verified C signatures are converted into callable pointers.
        unsafe {
            let resolved = owned.resolve_plan()?;
            if patched {
                owned.apply_plan(&resolved)?;
            }
            let plan = resolved.into_iter().map(|patch| patch.report).collect();
            let logical_index = owned.index.clone();
            let index = owned.index();
            owned
                .release_retention_barriers()
                .map_err(|error| Diagnostic::new(Stage::Transform, error))?;
            let select_end = Instant::now();
            let before = capture.then(|| owned.ir());
            let optimization_start = Instant::now();
            let mut native_target = ptr::null_mut();
            let mut message = ptr::null_mut();
            if (api.LLVMGetTargetFromTriple)(
                c"x86_64-pc-windows-msvc".as_ptr(),
                &mut native_target,
                &mut message,
            ) != 0
            {
                let text = CStr::from_ptr(message).to_string_lossy().into_owned();
                (api.LLVMDisposeMessage)(message);
                return Err(Diagnostic::new(Stage::Optimize, text));
            }
            let machine = (api.LLVMCreateTargetMachine)(
                native_target,
                c"x86_64-pc-windows-msvc".as_ptr(),
                c"x86-64".as_ptr(),
                c"".as_ptr(),
                3,
                2,
                1,
            );
            if machine.is_null() {
                return Err(Diagnostic::new(
                    Stage::Optimize,
                    "Cannot create target machine",
                ));
            }
            let options = (api.LLVMCreatePassBuilderOptions)();
            let error =
                (api.LLVMRunPasses)(owned.module, c"default<O3>".as_ptr(), machine, options);
            (api.LLVMDisposePassBuilderOptions)(options);
            if !error.is_null() {
                (api.LLVMDisposeTargetMachine)(machine);
                api.error(error)
                    .map_err(|error| Diagnostic::new(Stage::Optimize, error))?;
            }
            // This operation consumes the TargetMachine.
            let jtmb = (api.LLVMOrcJITTargetMachineBuilderCreateFromTargetMachine)(machine);
            let builder = (api.LLVMOrcCreateLLJITBuilder)();
            (api.LLVMOrcLLJITBuilderSetJITTargetMachineBuilder)(builder, jtmb);
            let optimization_end = Instant::now();
            let mut raw = ptr::null_mut();
            // The LLJIT builder is consumed, even on failure.
            api.error((api.LLVMOrcCreateLLJIT)(&mut raw, builder))
                .map_err(|error| Diagnostic::new(Stage::Jit, error))?;
            // Own raw immediately, so later verification/materialization failures release it.
            let jit = Self {
                api,
                raw,
                index: logical_index.clone(),
                _single_thread: PhantomData,
            };
            owned
                .verify()
                .map_err(|error| Diagnostic::new(Stage::Verify, error))?;
            let after = capture.then(|| owned.ir());
            let creation_end = Instant::now();
            let dylib = (api.LLVMOrcLLJITGetMainJITDylib)(raw);
            for path in libraries {
                let path = path
                    .to_str()
                    .and_then(|path| CString::new(path).ok())
                    .ok_or_else(|| {
                        Diagnostic::new(
                            Stage::Jit,
                            format!("Invalid native library path: {}", path.display()),
                        )
                    })?;
                let mut generator = ptr::null_mut();
                api.error((api.LLVMOrcCreateDynamicLibrarySearchGeneratorForPath)(
                    &mut generator,
                    path.as_ptr(),
                    (api.LLVMOrcLLJITGetGlobalPrefix)(raw),
                    None,
                    ptr::null_mut(),
                ))
                .map_err(|error| {
                    Diagnostic::new(Stage::Jit, format!("{}: {error}", path.to_string_lossy()))
                })?;
                (api.LLVMOrcJITDylibAddGenerator)(dylib, generator);
            }
            let tsm = (api.LLVMOrcCreateNewThreadSafeModule)(owned.module, owned.context);
            owned.module = ptr::null_mut(); // transferred to tsm
            api.error((api.LLVMOrcLLJITAddLLVMIRModule)(raw, dylib, tsm))
                .map_err(|error| Diagnostic::new(Stage::Jit, error))?;
            // Force materialization of all declared scalar entries before publication.
            for target in jit.index.targets.values() {
                jit.lookup_raw(&target.entry).map_err(|error| {
                    Diagnostic::new(Stage::Jit, format!("{}: {error}", target.logical_id))
                })?;
            }
            for target in jit.index.typed_return_targets.values() {
                jit.lookup_raw(&target.entry).map_err(|error| {
                    Diagnostic::new(Stage::Jit, format!("{}: {error}", target.logical_id))
                })?;
                jit.lookup_raw(&target.return_hook.type_probe)
                    .map_err(|error| {
                        Diagnostic::new(Stage::Jit, format!("{}: {error}", target.logical_id))
                    })?;
            }
            for handler in &jit.index.handlers {
                jit.lookup_raw(&handler.symbol)
                    .map_err(|error| owned.handler_error(Stage::Jit, handler, None, error))?;
                if let Some(probe) = &handler.return_type_probe {
                    let (_, hook) = jit
                        .index
                        .return_hook(&handler.target)
                        .expect("resolved return hook");
                    let type_error =
                        |message| owned.handler_error(Stage::Jit, handler, Some(1), message);
                    if jit.probe_type(probe).map_err(type_error)?
                        != jit.probe_type(&hook.type_probe).map_err(type_error)?
                    {
                        return Err(type_error(
                            "typed return mismatch between game and mod".into(),
                        ));
                    }
                }
            }
            let finish = Instant::now();
            let ms = |a: Instant, b: Instant| b.duration_since(a).as_secs_f64() * 1000.0;
            Ok((
                jit,
                Timing {
                    parse_link_ms: ms(start, parse_end),
                    select_patch_ms: ms(parse_end, select_end),
                    optimize_ms: ms(optimization_start, optimization_end),
                    create_jit_ms: ms(optimization_end, creation_end),
                    materialize_ms: ms(creation_end, finish),
                    total_ms: ms(start, finish),
                },
                Evidence {
                    before,
                    after,
                    index,
                    logical_index,
                    plan,
                },
            ))
        }
    }

    /// Look up a native entry owned by this JIT.
    ///
    /// # Safety
    /// Keep this JIT alive for every use of the address, use the exact ABI,
    /// and do not transfer entries to another thread.
    pub unsafe fn lookup_raw(&self, name: &str) -> Result<usize> {
        let name = CString::new(name).map_err(|_| "NUL in symbol")?;
        let mut address = 0;
        unsafe {
            self.api.error((self.api.LLVMOrcLLJITLookup)(
                self.raw,
                &mut address,
                name.as_ptr(),
            ))?;
        }
        if address == 0 {
            return Err("zero JIT entry address".into());
        }
        Ok(address as usize)
    }

    pub fn index(&self) -> &Index {
        &self.index
    }

    /// Call a validated scalar entry, checking arity and each Rust type.
    pub fn call(&self, logical_id: &str, args: &[ScalarValue]) -> Result<ScalarValue> {
        let target = self.index.target(logical_id)?;
        let bits = encode_arguments(target.parameter_types(), args)?;
        // The validated index is immutable and the JIT owns code for the duration
        // of the call. Each transmute matches the bridge validated for its schema.
        let address = unsafe { self.lookup_raw(&target.entry)? };
        if target.schema == 1 {
            return Ok(ScalarValue::F64(call_legacy(address, args)));
        }
        let entry = unsafe { std::mem::transmute::<usize, ScalarEntry>(address) };
        ScalarValue::from_bits(
            target.return_type(),
            entry(bits[0], bits[1], bits[2], bits[3]),
        )
    }

    // The generated probe uses the pinned compiler's TypeId layout and writes
    // an initialized TypeId using a previously checked pointer callback ABI.
    fn probe_type(&self, symbol: &str) -> Result<TypeId> {
        let mut result = MaybeUninit::<TypeId>::uninit();
        unsafe {
            let address = self.lookup_raw(symbol)?;
            let probe = std::mem::transmute::<usize, unsafe extern "C" fn(P)>(address);
            probe(result.as_mut_ptr().cast());
            Ok(result.assume_init())
        }
    }

    /// Call a target with a non-scalar return using its shared Rust type.
    ///
    /// # Safety
    /// Keep this JIT alive for all uses, including destruction, of any part of
    /// the result that depends on its code or data. Respect the game's ownership
    /// and thread rules, including when extracting or copying parts of the value.
    pub unsafe fn call_typed<T: 'static>(
        &self,
        logical_id: &str,
        args: &[ScalarValue],
    ) -> Result<T> {
        let target = self
            .index
            .typed_return_targets
            .get(logical_id)
            .ok_or_else(|| format!("typed return target not found: {logical_id}"))?;
        let bits = encode_arguments(&target.parameters, args)?;
        if self.probe_type(&target.return_hook.type_probe)? != TypeId::of::<T>() {
            return Err(format!(
                "{logical_id}: requested Rust return type does not match the game"
            ));
        }
        let mut result = MaybeUninit::<T>::uninit();
        unsafe {
            let address = self.lookup_raw(&target.entry)?;
            let entry = std::mem::transmute::<usize, TypedEntry>(address);
            if !entry(
                bits[0],
                bits[1],
                bits[2],
                bits[3],
                result.as_mut_ptr().cast(),
            ) {
                return Err(format!("{logical_id}: invalid typed entry arguments"));
            }
            Ok(result.assume_init())
        }
    }

    pub fn call_two(&self, logical_id: &str, a: f64, b: f64) -> Result<f64> {
        if self.index.target(logical_id)?.return_type() != ScalarType::F64 {
            return Err("entry return type is not f64".into());
        }
        match self.call(logical_id, &[ScalarValue::F64(a), ScalarValue::F64(b)])? {
            ScalarValue::F64(result) => Ok(result),
            _ => unreachable!("validated f64 return type"),
        }
    }
}

fn encode_arguments(parameters: &[ScalarType], args: &[ScalarValue]) -> Result<[u64; 4]> {
    if parameters.len() != args.len() {
        return Err(format!(
            "expected {} arguments, got {}",
            parameters.len(),
            args.len()
        ));
    }
    let mut bits = [0; 4];
    for (index, (&expected, provided)) in parameters.iter().zip(args).enumerate() {
        if expected != provided.kind() {
            return Err(format!(
                "argument {index}: expected {expected}, got {}",
                provided.kind()
            ));
        }
        bits[index] = provided.bits();
    }
    Ok(bits)
}

fn call_legacy(address: usize, args: &[ScalarValue]) -> f64 {
    use ScalarValue::F64;
    // Legacy entries have a distinct native signature for each arity.
    unsafe {
        match *args {
            [] => std::mem::transmute::<usize, extern "C" fn() -> f64>(address)(),
            [F64(a)] => std::mem::transmute::<usize, extern "C" fn(f64) -> f64>(address)(a),
            [F64(a), F64(b)] => {
                std::mem::transmute::<usize, extern "C" fn(f64, f64) -> f64>(address)(a, b)
            }
            [F64(a), F64(b), F64(c)] => {
                std::mem::transmute::<usize, extern "C" fn(f64, f64, f64) -> f64>(address)(a, b, c)
            }
            [F64(a), F64(b), F64(c), F64(d)] => std::mem::transmute::<
                usize,
                extern "C" fn(f64, f64, f64, f64) -> f64,
            >(address)(a, b, c, d),
            _ => unreachable!("validated legacy signature"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_rust_bitcode_validates_transforms_executes_and_rejects_bad_selectors() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let (_, game, handler) =
            crate::tests_support::read_fixture(&root.join("artifacts")).unwrap();
        let api = Api::load(&root.join(".tools/llvm-21.1.3/bin/LLVM-C.dll")).unwrap();
        api.initialize().unwrap();
        let mut owned = OwnedModule::parse(&api, &game, &[&handler]).unwrap();
        {
            assert!(
                owned
                    .function(c"missing", &[ScalarType::F64], ScalarType::F64)
                    .unwrap_err()
                    .contains("Missing")
            );
            let target = owned
                .index
                .target("demo::combat::apply_damage")
                .unwrap()
                .symbol
                .clone();
            assert!(
                owned
                    .named(&target, &[ScalarType::F64], ScalarType::F64)
                    .unwrap_err()
                    .contains("signature")
            );
        }
        let before = owned.ir();
        owned.index.handlers[0].callee = Some("demo::combat::caller".into());
        let error = owned.resolve_plan().err().expect("selector should fail");
        assert_eq!(error.stage, Stage::ResolveSelector);
        assert_eq!(error.mod_index, Some(0));
        let handler_error = error.handler.unwrap();
        assert_eq!(handler_error.handler_id, "adjust_damage");
        assert_eq!(handler_error.operation, Operation::CallResult);
        assert_eq!(handler_error.expected_matches, 1);
        assert_eq!(handler_error.actual_matches, Some(0));
        assert_eq!(before, owned.ir());
        drop(owned);
        let parse_error = Jit::plan_many(&api, &game, &[b"not bitcode"])
            .err()
            .expect("invalid bitcode should return a diagnostic");
        assert_eq!(parse_error.stage, Stage::Parse);
        assert_eq!(parse_error.mod_index, Some(0));
        let preview = Jit::plan_many(&api, &game, &[&handler]).unwrap();
        let (base, _, _) = Jit::compile(&api, &game, &handler, false, false).unwrap();
        let (patched, _, evidence) = Jit::compile(&api, &game, &handler, true, true).unwrap();
        assert_eq!(
            serde_json::to_value(preview.patches).unwrap(),
            serde_json::to_value(&evidence.plan).unwrap()
        );
        assert_eq!(
            base.call_two("demo::combat::apply_damage", 20.0, 8.0)
                .unwrap(),
            16.0
        );
        assert_eq!(
            patched
                .call_two("demo::combat::apply_damage", 20.0, 8.0)
                .unwrap(),
            9.5
        );
        assert_eq!(evidence.logical_index.targets.len(), 4);
        let handler_call = format!("call double @{}(", evidence.plan[0].handler_symbol);
        assert!(evidence.before.unwrap().contains(&handler_call));
        let optimized = evidence.after.unwrap();
        // This fixture's handler was inlined into the target by the real optimizer.
        assert!(!optimized.contains(&handler_call));
        assert_eq!(
            patched.call_two("demo::combat::caller", 20.0, 8.0).unwrap(),
            10.5
        );
    }

    #[test]
    fn ordinal_uses_original_calls_and_failure_preserves_an_existing_instance() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let (_, game, handler) =
            crate::tests_support::read_fixture(&root.join("artifacts")).unwrap();
        let api = Api::load(&root.join(".tools/llvm-21.1.3/bin/LLVM-C.dll")).unwrap();
        api.initialize().unwrap();
        let (base, _, _) = Jit::compile_many(&api, &game, &[], false, false).unwrap();
        for (ordinal, expected) in [(0, 27.5), (1, 26.5)] {
            let (jit, _, plan) =
                Jit::compile_with_index(&api, &game, &[&handler], true, false, &[], |index| {
                    index.handlers[0].target = "demo::combat::two_hits".into();
                    index.handlers[0].ordinal = ordinal;
                    index.handlers[0].require_matches = 2;
                })
                .unwrap();
            assert_eq!(
                jit.call_two("demo::combat::two_hits", 20.0, 8.0).unwrap(),
                expected
            );
            assert_eq!(
                jit.call_two("demo::combat::apply_damage", 20.0, 8.0)
                    .unwrap(),
                16.0
            );
            assert_eq!(plan.plan[0].matches, 2);
        }
        for case in 0..4 {
            let error =
                Jit::compile_with_index(&api, &game, &[&handler], true, false, &[], |index| {
                    match case {
                        0 => index.handlers[0].target = "unknown::target".into(),
                        1 => index.handlers[0].require_matches = 2,
                        2 => index.handlers[0].ordinal = 1,
                        _ => {
                            index
                                .targets
                                .get_mut("demo::combat::apply_damage")
                                .unwrap()
                                .arity = 1
                        }
                    }
                })
                .err()
                .unwrap();
            assert_eq!(
                error.stage,
                if case == 3 {
                    Stage::Index
                } else {
                    Stage::ResolveSelector
                }
            );
            assert!(!error.message.is_empty());
            assert_eq!(
                base.call_two("demo::combat::apply_damage", 20.0, 8.0)
                    .unwrap(),
                16.0
            );
        }
    }

    #[test]
    fn independent_mods_compose_in_order_and_duplicates_are_rejected_before_linking() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let (_, game, handler) =
            crate::tests_support::read_fixture(&root.join("artifacts")).unwrap();
        let half = std::fs::read(root.join("artifacts/half_mod.bc")).unwrap();
        let parameter = std::fs::read(root.join("artifacts/parameter_mod.bc")).unwrap();
        let api = Api::load(&root.join(".tools/llvm-21.1.3/bin/LLVM-C.dll")).unwrap();
        api.initialize().unwrap();
        let (a, _, plan) = Jit::compile_many(&api, &game, &[&handler, &half], true, false).unwrap();
        let (b, _, _) = Jit::compile_many(&api, &game, &[&half, &handler], true, false).unwrap();
        assert_eq!(
            a.call_two("demo::combat::apply_damage", 20.0, 8.0).unwrap(),
            4.75
        );
        assert_eq!(
            b.call_two("demo::combat::apply_damage", 20.0, 8.0).unwrap(),
            4.0
        );
        let (parameter_only, _, parameter_plan) =
            Jit::compile_many(&api, &game, &[&parameter], true, false).unwrap();
        assert_eq!(
            parameter_only
                .call_two("demo::combat::apply_damage", 20.0, 8.0)
                .unwrap(),
            20.0
        );
        assert_eq!(parameter_plan.plan[0].operation, Operation::Parameter);
        let (parameter_and_half, _, _) =
            Jit::compile_many(&api, &game, &[&parameter, &half], true, false).unwrap();
        assert_eq!(
            parameter_and_half
                .call_two("demo::combat::apply_damage", 20.0, 8.0)
                .unwrap(),
            10.0
        );
        let (head, _, head_plan) =
            Jit::compile_with_index(&api, &game, &[&half], true, false, &[], |index| {
                index.handlers[0].operation = Operation::Head;
                index.handlers[0].callee = None;
                index.handlers[0].parameter = None;
            })
            .unwrap();
        assert_eq!(
            head.call_two("demo::combat::apply_damage", 20.0, 8.0)
                .unwrap(),
            6.0
        );
        assert_eq!(head_plan.plan[0].operation, Operation::Head);
        assert_eq!(
            plan.plan.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(),
            ["adjust_damage", "half_damage"]
        );
        let (prioritized, _, _) =
            Jit::compile_with_index(&api, &game, &[&handler, &half], true, false, &[], |index| {
                index.handlers[0].priority = 10;
            })
            .unwrap();
        assert_eq!(
            prioritized
                .call_two("demo::combat::apply_damage", 20.0, 8.0)
                .unwrap(),
            4.0
        );
        let error = Jit::compile_many_detailed(&api, &game, &[&handler, &handler], true, false)
            .err()
            .unwrap();
        assert_eq!(error.stage, Stage::Index);
        assert_eq!(error.mod_index, Some(1));
        assert_eq!(error.handler.unwrap().handler_id, "adjust_damage");
    }
}
#[test]
fn typed_returns_use_the_same_hook_for_arrays_tuples_shared_types_and_static_borrows() {
    use ScalarValue::{F32, U8};

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (_, game, handler) =
        crate::tests_support::read_fixture(&root.join("artifacts/scalar")).unwrap();
    let api = Api::load(&root.join(".tools/llvm-21.1.3/bin/LLVM-C.dll")).unwrap();
    api.initialize().unwrap();
    let (base, _, _) = Jit::compile_many(&api, &game, &[], false, false).unwrap();
    let (patched, _, evidence) = Jit::compile(&api, &game, &handler, true, true).unwrap();
    let bytes = "demo::returns::bytes";
    let vector = "demo::returns::vector";
    let loot = "demo::returns::loot_first_byte";
    let message = "demo::returns::message";
    let payload = "demo::returns::payload";

    // The array and tuple contain no addresses. The borrowed values are
    // inspected before dropping the JIT that owns their backing storage.
    unsafe {
        assert_eq!(
            base.call_typed::<[u8; 16]>(bytes, &[U8(7)]).unwrap(),
            [7; 16]
        );
        assert_eq!(
            base.call_typed::<[u8; 16]>(bytes, &[U8(0)]).unwrap(),
            [1; 16]
        );
        let mut expected = [7; 16];
        expected[0] = 43;
        assert_eq!(
            patched.call_typed::<[u8; 16]>(bytes, &[U8(7)]).unwrap(),
            expected
        );
        assert_eq!(
            patched.call_typed::<[u8; 16]>(bytes, &[U8(0)]).unwrap()[..2],
            [43, 1]
        );
        assert_eq!(
            patched
                .call_typed::<(f32, f32)>(vector, &[F32(2.0)])
                .unwrap(),
            (4.0, 3.0)
        );
        assert_eq!(base.call_typed::<&str>(message, &[]).unwrap(), "game");
        assert_eq!(patched.call_typed::<&str>(message, &[]).unwrap(), "mod");
        assert_eq!(
            patched.call_typed::<&[u8]>(payload, &[]).unwrap(),
            b"mod payload"
        );
        assert!(
            patched
                .call_typed::<[i8; 16]>(bytes, &[U8(7)])
                .unwrap_err()
                .contains("does not match")
        );
    }
    assert_eq!(base.call(loot, &[U8(7)]).unwrap(), U8(7));
    assert_eq!(patched.call(loot, &[U8(7)]).unwrap(), U8(42));
    assert_eq!(base.call(loot, &[U8(0)]).unwrap(), U8(255));
    assert_eq!(patched.call(loot, &[U8(0)]).unwrap(), U8(0));
    assert!(
        patched
            .call(bytes, &[U8(7)])
            .unwrap_err()
            .contains("call_typed")
    );
    assert_eq!(
        evidence
            .plan
            .iter()
            .filter(|patch| patch.target == bytes)
            .map(|patch| patch.id.as_str())
            .collect::<Vec<_>>(),
        ["set_bytes", "increment_bytes"],
    );

    // str and [u8] have identical pointer-and-length layouts. Retarget the
    // genuine str handler to prove that type identity, not layout, is checked.
    let err = Jit::compile_with_index(&api, &game, &[&handler], true, false, &[], |index| {
        index
            .handlers
            .iter_mut()
            .find(|h| h.id == "change_message")
            .unwrap()
            .target = payload.into();
    })
    .err()
    .unwrap();
    assert_eq!(err.stage, Stage::Jit);
    assert!(err.message.contains("typed return mismatch"));
    unsafe {
        assert_eq!(
            patched.call_typed::<&[u8]>(payload, &[]).unwrap(),
            b"mod payload"
        );
    }
}

#[test]
fn scalar_bitcode_validates_and_executes_typed_entries() {
    use ScalarValue::*;

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (_, game, handler) =
        crate::tests_support::read_fixture(&root.join("artifacts/scalar")).unwrap();
    let api = Api::load(&root.join(".tools/llvm-21.1.3/bin/LLVM-C.dll")).unwrap();
    api.initialize().unwrap();
    let preview = Jit::plan_many(&api, &game, &[&handler]).unwrap();
    assert_eq!(
        preview
            .patches
            .iter()
            .find(|patch| patch.id == "extra_drop")
            .unwrap()
            .value_type,
        Some(ScalarType::I32)
    );
    let (base, _, _) = Jit::compile_many(&api, &game, &[], false, false).unwrap();
    let (patched, _, _) = Jit::compile(&api, &game, &handler, true, false).unwrap();
    let loot = "demo::loot::drop_count";
    let args = [I32(2), Bool(false), F32(1.0), U64(u64::MAX)];
    assert_eq!(base.call(loot, &args).unwrap(), I32(3));
    assert_eq!(patched.call(loot, &args).unwrap(), I32(4));
    assert!(
        patched
            .call(loot, &args[..3])
            .unwrap_err()
            .contains("expected 4 arguments")
    );
    assert!(
        patched
            .call(loot, &[U32(2), args[1], args[2], args[3]])
            .unwrap_err()
            .contains("expected i32, got u32")
    );
    assert!(patched.call_two(loot, 2.0, 0.0).is_err());
    assert_eq!(
        patched.call("demo::loot::gate", &[Bool(true)]).unwrap(),
        Bool(false)
    );
    assert_eq!(
        patched.call("demo::loot::gate", &[Bool(false)]).unwrap(),
        Bool(true)
    );
    assert_eq!(
        patched.call("demo::loot::narrow", &[I8(-100)]).unwrap(),
        I8(-102)
    );
    assert_eq!(
        patched.call("demo::loot::float", &[F32(5.0)]).unwrap(),
        F32(-2.5)
    );
    assert_eq!(
        patched
            .call("demo::loot::adjust", &[F64(5.5), I32(-1)])
            .unwrap(),
        F64(4.5)
    );
    assert_eq!(
        patched.call("demo::loot::max_cap", &[]).unwrap(),
        U64(u64::MAX)
    );
    for (id, value) in [
        ("signed16", I16(i16::MIN)),
        ("signed64", I64(i64::MIN)),
        ("signed_size", Isize(isize::MIN)),
        ("unsigned8", U8(u8::MAX)),
        ("unsigned16", U16(u16::MAX)),
        ("unsigned_size", Usize(usize::MAX)),
    ] {
        assert_eq!(
            patched
                .call(&format!("demo::loot::{id}"), &[value])
                .unwrap(),
            value
        );
    }

    // LLVM uses i32 for both types, but the Rust handler must match exactly.
    let error = Jit::compile_with_index(&api, &game, &[&handler], true, false, &[], |index| {
        let discount = index
            .handlers
            .iter_mut()
            .find(|handler| handler.id == "discount")
            .unwrap();
        discount.target = loot.into();
    })
    .err()
    .unwrap();
    assert_eq!(error.stage, Stage::Jit);
    assert!(error.message.contains("typed return mismatch"));
}

#[cfg(test)]
#[test]
fn vec2_return_preserves_both_fields_and_rejects_integer_layout() {
    use crate::Vec2;
    use ScalarValue::F32;

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (_, game, handler) =
        crate::tests_support::read_fixture(&root.join("artifacts/scalar")).unwrap();
    let api = Api::load(&root.join(".tools/llvm-21.1.3/bin/LLVM-C.dll")).unwrap();
    api.initialize().unwrap();
    let (base, _, _) = Jit::compile_many(&api, &game, &[], false, false).unwrap();
    let (patched, _, _) = Jit::compile(&api, &game, &handler, true, false).unwrap();
    let target = "demo::physics::knockback";
    assert_eq!(
        base.call(target, &[F32(2.0)]).unwrap(),
        ScalarValue::Vec2(Vec2 { x: 2.0, y: -1.0 })
    );
    for (name, force, x, y) in [
        ("knockback", 2.0, 4.0, 3.0),
        ("apply_knockback", 2.0, -3.0, 4.0),
        ("vertical_knockback", -3.0, 1.0, 2.0),
        ("vertical_knockback", 3.0, 0.0, 3.0),
    ] {
        assert_eq!(
            patched
                .call(&format!("demo::physics::{name}"), &[F32(force)])
                .unwrap(),
            ScalarValue::Vec2(Vec2 { x, y })
        );
    }
    let ScalarValue::Vec2(value) = base.call(target, &[F32(-0.0)]).unwrap() else {
        panic!("expected Vec2");
    };
    assert_eq!(value.x.to_bits(), (-0.0_f32).to_bits());

    let error = Jit::compile_with_index(&api, &game, &[&handler], true, false, &[], |index| {
        index.targets.get_mut(target).unwrap().symbol = "integer_knockback".into();
    })
    .err()
    .unwrap();
    assert_eq!(error.stage, Stage::Index);
    assert!(
        error
            .message
            .contains("expected return { float, float }, got { i32, i32 }")
    );
}

#[cfg(test)]
#[test]
fn entity_returns_preserve_replace_or_clear_the_game_target() {
    use crate::EntityId;
    use ScalarValue::{OptionEntityId, U32};

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (_, game, handler) =
        crate::tests_support::read_fixture(&root.join("artifacts/scalar")).unwrap();
    let api = Api::load(&root.join(".tools/llvm-21.1.3/bin/LLVM-C.dll")).unwrap();
    api.initialize().unwrap();

    let plan = Jit::plan_many(&api, &game, &[&handler]).unwrap();
    let filter = plan
        .patches
        .iter()
        .find(|patch| patch.id == "filter_target")
        .unwrap();
    assert_eq!(filter.value_type, Some(ScalarType::OptionEntityId));
    let (base, _, _) = Jit::compile_many(&api, &game, &[], false, false).unwrap();
    let (patched, _, _) = Jit::compile(&api, &game, &handler, true, false).unwrap();
    for (candidate, expected) in [(0, 0), (1, 1), (2, 42), (3, 0), (u32::MAX, u32::MAX)] {
        let args = [U32(candidate)];
        assert_eq!(
            base.call("demo::targeting::find_target", &args).unwrap(),
            OptionEntityId(EntityId::new(candidate))
        );
        for target in [
            "demo::targeting::find_target",
            "demo::targeting::fallback_target",
        ] {
            assert_eq!(
                patched.call(target, &args).unwrap(),
                OptionEntityId(EntityId::new(expected))
            );
        }
        assert_eq!(
            patched
                .call("demo::targeting::select_target", &args)
                .unwrap(),
            OptionEntityId(EntityId::new(if expected == 0 { 99 } else { expected }))
        );
    }

    let arg = ScalarValue::EntityId(EntityId::new(2).unwrap());
    let active_target = "demo::targeting::active_target";
    assert_eq!(base.call(active_target, &[arg]).unwrap(), arg);
    assert_eq!(
        patched.call(active_target, &[arg]).unwrap(),
        ScalarValue::EntityId(EntityId::new(84).unwrap())
    );
    assert!(
        patched
            .call(active_target, &[U32(2)])
            .unwrap_err()
            .contains("expected EntityId, got u32")
    );

    let error = Jit::compile_with_index(&api, &game, &[&handler], true, false, &[], |index| {
        let filter = index
            .handlers
            .iter_mut()
            .find(|handler| handler.id == "filter_target")
            .unwrap();
        filter.target = active_target.into();
    })
    .err()
    .unwrap();
    assert_eq!(error.stage, Stage::Jit);
    assert!(error.message.contains("typed return mismatch"));
}

#[cfg(test)]
#[test]
fn entry_callback_cancels_side_effects_and_composes_with_a_game_hook() {
    use ScalarValue::{I32, U32, Unit};

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (_, game, handler) =
        crate::tests_support::read_fixture(&root.join("artifacts/scalar")).unwrap();
    let api = Api::load(&root.join(".tools/llvm-21.1.3/bin/LLVM-C.dll")).unwrap();
    api.initialize().unwrap();
    let (base, _, _) = Jit::compile_many(&api, &game, &[], false, false).unwrap();
    let (patched, _, evidence) = Jit::compile(&api, &game, &handler, true, true).unwrap();
    for (jit, skill_id, mana, effects) in [
        (&base, 7, 90, 1),
        (&patched, 7, 100, 0),
        (&patched, 8, 95, 1),
        (&patched, 7, 95, 1),
    ] {
        assert_eq!(
            jit.call("demo::skills::cast_skill", &[U32(skill_id)])
                .unwrap(),
            Unit
        );
        assert_eq!(jit.call("demo::skills::mana", &[]).unwrap(), U32(mana));
        assert_eq!(
            jit.call("demo::skills::effect_count", &[]).unwrap(),
            U32(effects)
        );
    }
    for (skill_id, power) in [(7, -9), (8, -8), (9, 18)] {
        assert_eq!(
            patched
                .call("demo::skills::skill_power", &[U32(skill_id)])
                .unwrap(),
            I32(power)
        );
    }

    let before = evidence.before.unwrap();
    for (id, argument_prefix) in [("observe_cast", "ptr "), ("observe_spawn", ")")] {
        let observer = evidence.plan.iter().find(|patch| patch.id == id).unwrap();
        assert!(before.contains(&format!(
            "call void @{}({argument_prefix}",
            observer.handler_symbol
        )));
    }

    let error = Jit::compile_with_index(&api, &game, &[&handler], true, false, &[], |index| {
        let entry = index
            .handlers
            .iter_mut()
            .find(|handler| handler.id == "block_skill")
            .unwrap();
        entry.target = "demo::skills::skill_power".into();
    })
    .err()
    .unwrap();
    assert_eq!(error.stage, Stage::ResolveSelector);
    assert!(
        error
            .message
            .contains("expected (u32) -> i32, got (u32) -> ()")
    );
}

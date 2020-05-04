//! JIT compilation.

use crate::{CodeMemory, CompiledModule};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use wasm_common::entity::PrimaryMap;
use wasm_common::{FunctionType, LocalFunctionIndex, MemoryIndex, SignatureIndex, TableIndex};
use wasmer_compiler::{Compilation, CompileError, FunctionBody, Target};
#[cfg(feature = "compiler")]
use wasmer_compiler::{Compiler, CompilerConfig};
use wasmer_engine::{
    CompiledModule as BaseCompiledModule, DeserializeError, Engine, InstantiationError, Resolver,
    SerializeError, Tunables,
};
use wasmer_runtime::{
    InstanceHandle, MemoryPlan, Module, SignatureRegistry, TablePlan, VMFunctionBody,
    VMSharedSignatureIndex, VMTrampoline,
};

/// A WebAssembly `JIT` Engine.
#[derive(Clone)]
pub struct JITEngine {
    inner: Arc<RefCell<JITEngineInner>>,
    tunables: Arc<Box<dyn Tunables>>,
}

impl JITEngine {
    const MAGIC_HEADER: &'static [u8] = b"\0wasmer-jit";

    /// Create a new `JITEngine` with the given config
    #[cfg(feature = "compiler")]
    pub fn new<C: CompilerConfig>(config: &C, tunables: impl Tunables + 'static) -> Self
    where
        C: ?Sized,
    {
        let compiler = config.compiler();
        Self {
            inner: Arc::new(RefCell::new(JITEngineInner {
                compiler: Some(compiler),
                trampolines: HashMap::new(),
                code_memory: CodeMemory::new(),
                signatures: SignatureRegistry::new(),
            })),
            tunables: Arc::new(Box::new(tunables)),
        }
    }

    /// Create a headless `JITEngine`
    ///
    /// A headless engine is an engine without any compiler attached.
    /// This is useful for assuring a minimal runtime for running
    /// WebAssembly modules.
    ///
    /// For example, for running in IoT devices where compilers are very
    /// expensive, or also to optimize startup speed.
    ///
    /// # Important
    ///
    /// Headless engines can't compile or validate any modules,
    /// they just take already processed Modules (via `Module::serialize`).
    pub fn headless(tunables: impl Tunables + 'static) -> Self {
        Self {
            inner: Arc::new(RefCell::new(JITEngineInner {
                #[cfg(feature = "compiler")]
                compiler: None,
                trampolines: HashMap::new(),
                code_memory: CodeMemory::new(),
                signatures: SignatureRegistry::new(),
            })),
            tunables: Arc::new(Box::new(tunables)),
        }
    }

    pub(crate) fn compiler(&self) -> std::cell::Ref<'_, JITEngineInner> {
        self.inner.borrow()
    }

    pub(crate) fn compiler_mut(&self) -> std::cell::RefMut<'_, JITEngineInner> {
        self.inner.borrow_mut()
    }

    /// Check if the provided bytes look like a serialized
    /// module by the `JITEngine` implementation.
    pub fn is_deserializable(bytes: &[u8]) -> bool {
        bytes.starts_with(Self::MAGIC_HEADER)
    }
}

impl Engine for JITEngine {
    /// Get the tunables
    fn tunables(&self) -> &dyn Tunables {
        &**self.tunables
    }

    /// Register a signature
    fn register_signature(&self, func_type: &FunctionType) -> VMSharedSignatureIndex {
        let compiler = self.compiler();
        compiler.signatures().register(func_type)
    }

    /// Lookup a signature
    fn lookup_signature(&self, sig: VMSharedSignatureIndex) -> Option<FunctionType> {
        let compiler = self.compiler();
        compiler.signatures().lookup(sig)
    }

    /// Retrieves a trampoline given a signature
    fn trampoline(&self, sig: VMSharedSignatureIndex) -> Option<VMTrampoline> {
        self.compiler().trampoline(sig)
    }

    /// Validates a WebAssembly module
    fn validate(&self, binary: &[u8]) -> Result<(), CompileError> {
        self.compiler().validate(binary)
    }

    /// Compile a WebAssembly binary
    fn compile(&self, binary: &[u8]) -> Result<Arc<dyn BaseCompiledModule>, CompileError> {
        Ok(Arc::new(CompiledModule::new(&self, binary)?))
    }

    /// Instantiates a WebAssembly module
    unsafe fn instantiate(
        &self,
        compiled_module: &Arc<dyn BaseCompiledModule>,
        resolver: &dyn Resolver,
    ) -> Result<InstanceHandle, InstantiationError> {
        let compiled_module = compiled_module.downcast_ref::<CompiledModule>().unwrap();
        unsafe { compiled_module.instantiate(&self, resolver, Box::new(())) }
    }

    /// Serializes a WebAssembly module
    fn serialize(
        &self,
        compiled_module: &Arc<dyn BaseCompiledModule>,
    ) -> Result<Vec<u8>, SerializeError> {
        let compiled_module = compiled_module.downcast_ref::<CompiledModule>().unwrap();
        // We append the header
        let mut serialized = Self::MAGIC_HEADER.to_vec();
        serialized.extend(compiled_module.serialize()?);
        Ok(serialized)
    }

    /// Deserializes a WebAssembly module
    fn deserialize(&self, bytes: &[u8]) -> Result<Arc<dyn BaseCompiledModule>, DeserializeError> {
        if !Self::is_deserializable(bytes) {
            return Err(DeserializeError::Incompatible(
                "The provided bytes are not wasmer-jit".to_string(),
            ));
        }
        Ok(Arc::new(CompiledModule::deserialize(
            &self,
            &bytes[Self::MAGIC_HEADER.len()..],
        )?))
    }
}

/// The inner contents of `JITEngine`
pub struct JITEngineInner {
    /// The compiler
    #[cfg(feature = "compiler")]
    compiler: Option<Box<dyn Compiler>>,
    /// Pointers to trampoline functions used to enter particular signatures
    trampolines: HashMap<VMSharedSignatureIndex, VMTrampoline>,
    /// The code memory is responsible of publishing the compiled
    /// functions to memory.
    code_memory: CodeMemory,
    /// The signature registry is used mainly to operate with trampolines
    /// performantly.
    signatures: SignatureRegistry,
}

impl JITEngineInner {
    /// Gets the compiler associated to this JIT
    #[cfg(feature = "compiler")]
    pub fn compiler(&self) -> Result<&dyn Compiler, CompileError> {
        if self.compiler.is_none() {
            return Err(CompileError::Codegen("The JITEngine is operating in headless mode, so it can only execute already compiled Modules.".to_string()));
        }
        Ok(&**self.compiler.as_ref().unwrap())
    }

    /// Validate the module
    #[cfg(feature = "compiler")]
    pub fn validate<'data>(&self, data: &'data [u8]) -> Result<(), CompileError> {
        self.compiler()?.validate_module(data)
    }

    /// Validate the module
    #[cfg(not(feature = "compiler"))]
    pub fn validate<'data>(&self, data: &'data [u8]) -> Result<(), CompileError> {
        Err(CompileError::Validate(
            "Validation is only enabled with the compiler feature".to_string(),
        ))
    }

    /// Compile the given function bodies.
    pub(crate) fn allocate<'data>(
        &mut self,
        module: &Module,
        functions: &PrimaryMap<LocalFunctionIndex, FunctionBody>,
        trampolines: &PrimaryMap<SignatureIndex, FunctionBody>,
    ) -> Result<PrimaryMap<LocalFunctionIndex, *mut [VMFunctionBody]>, CompileError> {
        // Allocate all of the compiled functions into executable memory,
        // copying over their contents.
        let allocated_functions =
            self.code_memory
                .allocate_functions(&functions)
                .map_err(|message| {
                    CompileError::Resource(format!(
                        "failed to allocate memory for functions: {}",
                        message
                    ))
                })?;

        for (sig_index, compiled_function) in trampolines.iter() {
            let func_type = module.signatures.get(sig_index).unwrap();
            let index = self.signatures.register(&func_type);
            if self.trampolines.contains_key(&index) {
                // We don't need to allocate the trampoline in case
                // it's signature is already allocated.
                continue;
            }
            let ptr = self
                .code_memory
                .allocate_for_function(&compiled_function)
                .map_err(|message| {
                    CompileError::Resource(format!(
                        "failed to allocate memory for trampolines: {}",
                        message
                    ))
                })?
                .as_ptr();
            let trampoline =
                unsafe { std::mem::transmute::<*const VMFunctionBody, VMTrampoline>(ptr) };
            self.trampolines.insert(index, trampoline);
        }
        Ok(allocated_functions)
    }

    /// Make memory containing compiled code executable.
    pub(crate) fn publish_compiled_code(&mut self) {
        self.code_memory.publish();
    }

    /// Shared signature registry.
    pub fn signatures(&self) -> &SignatureRegistry {
        &self.signatures
    }

    /// Gets the trampoline pre-registered for a particular signature
    pub fn trampoline(&self, sig: VMSharedSignatureIndex) -> Option<VMTrampoline> {
        self.trampolines.get(&sig).cloned()
    }
}

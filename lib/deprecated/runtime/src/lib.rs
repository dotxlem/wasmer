//! Wasmer-runtime is a library that makes embedding WebAssembly
//! in your application easy, efficient, and safe.
//!
//! # How to use Wasmer-Runtime
//!
//! The easiest way is to use the [`instantiate`] function to create an [`Instance`].
//! Then you can use [`call`] or [`func`] and then [`call`][func.call] to call an exported function safely.
//!
//! [`instantiate`]: fn.instantiate.html
//! [`Instance`]: struct.Instance.html
//! [`call`]: struct.Instance.html#method.call
//! [`func`]: struct.Instance.html#method.func
//! [func.call]: struct.Function.html#method.call
//!
//! ## Example
//!
//! Given this WebAssembly:
//!
//! ```wat
//! (module
//!   (type $t0 (func (param i32) (result i32)))
//!   (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
//!     get_local $p0
//!     i32.const 1
//!     i32.add))
//! ```
//!
//! compiled into wasm bytecode, we can call the exported `add_one` function:
//!
//! ```rust
//! static WASM: &'static [u8] = &[
//!    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x06, 0x01, 0x60,
//!    0x01, 0x7f, 0x01, 0x7f, 0x03, 0x02, 0x01, 0x00, 0x07, 0x0b, 0x01, 0x07,
//!    0x61, 0x64, 0x64, 0x5f, 0x6f, 0x6e, 0x65, 0x00, 0x00, 0x0a, 0x09, 0x01,
//!    0x07, 0x00, 0x20, 0x00, 0x41, 0x01, 0x6a, 0x0b, 0x00, 0x1a, 0x04, 0x6e,
//!    0x61, 0x6d, 0x65, 0x01, 0x0a, 0x01, 0x00, 0x07, 0x61, 0x64, 0x64, 0x5f,
//!    0x6f, 0x6e, 0x65, 0x02, 0x07, 0x01, 0x00, 0x01, 0x00, 0x02, 0x70, 0x30,
//! ];
//!
//! use wasmer_runtime::{
//!     instantiate,
//!     Value,
//!     imports,
//!     Func,
//! };
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let import_object = imports! {};
//!     let mut instance = instantiate(WASM, &import_object)?;
//!
//!     let add_one: Func<i32, i32> = instance.exports.get("add_one")?;
//!
//!     let value = add_one.call(42)?;
//!     assert_eq!(value, 43);
//!
//!     Ok(())
//! }
//! ```

use std::{error::Error, fmt};

pub use wasmer_runtime_core::{
    backend::Backend,
    compile, compile_with,
    export::{Export, RuntimeExport},
    func,
    global::Global,
    import::{ImportObject, LikeNamespace},
    imports,
    instance::{DynFunc, Instance},
    load_cache_with,
    memory::ptr::{Array, Item, WasmPtr},
    memory::Memory,
    module::Module,
    table::Table,
    typed_func::{DynamicFunc, Func},
    types::Value,
    validate,
    vm::Ctx,
    wat2wasm,
};

pub mod memory {
    //! The memory module contains the implementation data structures
    //! and helper functions used to manipulate and access wasm
    //! memory.
    pub use wasmer_runtime_core::memory::{Atomically, Memory, MemoryView};
}

pub mod wasm {
    //! Various types exposed by the Wasmer Runtime.
    pub use wasmer_runtime_core::{
        global::Global,
        table::Table,
        types::{FuncSig, GlobalDescriptor, MemoryDescriptor, TableDescriptor, Type, Value},
    };
}

pub mod error {
    //! The error module contains the data structures and helper
    //! functions used to implement errors that are produced and
    //! returned from the wasmer runtime.
    pub use wasmer_runtime_core::error::*;
}

pub mod units {
    //! Various unit types.
    pub use wasmer_runtime_core::units::{Bytes, Pages};
}

pub mod types {
    //! Types used in the Wasm runtime and conversion functions.
    pub use wasmer_runtime_core::types::*;
}

pub mod cache {
    //! The cache module provides the common data structures used by
    //! compiler backends to allow serializing compiled wasm code to a
    //! binary format. The binary format can be persisted, and loaded
    //! to allow skipping compilation and fast startup.
    pub use wasmer_runtime_core::cache::*;
}

#[derive(Debug)]
pub enum InstantiateError {
    CompileError(Box<dyn Error>),
    InstantiationError(wasmer_runtime_core::error::InstantiationError),
}

impl Error for InstantiateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CompileError(e) => e.source(),
            Self::InstantiationError(e) => e.source(),
        }
    }
}

impl fmt::Display for InstantiateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CompileError(e) => e.fmt(f),
            Self::InstantiationError(e) => e.fmt(f),
        }
    }
}

pub fn instantiate(
    wasm: &[u8],
    import_object: &ImportObject,
) -> Result<Instance, InstantiateError> {
    let module = compile(wasm).map_err(InstantiateError::CompileError)?;

    module
        .instantiate(import_object)
        .map_err(InstantiateError::InstantiationError)
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

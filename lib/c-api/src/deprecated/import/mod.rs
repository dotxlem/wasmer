//! Create, read, destroy import definitions (function, global, memory
//! and table) on an instance.

use crate::deprecated::{
    export::{wasmer_import_export_kind, wasmer_import_export_value},
    get_global_store,
    instance::{wasmer_instance_context_t, CAPIInstance},
    module::wasmer_module_t,
    value::wasmer_value_tag,
    wasmer_byte_array, wasmer_result_t,
};
use crate::error::{update_last_error, CApiError};
use libc::c_uint;
use std::ffi::{c_void, CStr};
use std::os::raw::c_char;
use std::ptr::{self, NonNull};
use std::slice;
use std::sync::{Arc, Mutex};
// use std::convert::TryFrom,
use std::collections::HashMap;
use wasmer::{
    ChainableNamedResolver, Exports, Extern, Function, FunctionType, Global, ImportObject,
    ImportObjectIterator, ImportType, Memory, Module, NamedResolver, RuntimeError, Table, Val,
    ValType, WasmerEnv,
};

#[repr(C)]
pub struct wasmer_import_t {
    pub module_name: wasmer_byte_array,
    pub import_name: wasmer_byte_array,
    pub tag: wasmer_import_export_kind,
    pub value: wasmer_import_export_value,
}

// An opaque wrapper around `CAPIImportObject`
#[repr(C)]
pub struct wasmer_import_object_t;

/// A wrapper around a Wasmer ImportObject with extra data to maintain the existing API
pub(crate) struct CAPIImportObject {
    /// The real wasmer `ImportObject`-like thing.
    pub(crate) import_object: Box<dyn NamedResolver>,
    /// List of pointers of memories that are imported into this import object. This is
    /// required because the old Wasmer API allowed accessing these from the Instance
    /// but the new/current API does not. So we store them here to pass them to the Instance
    /// to allow functions to access this data for backwards compatibilty.
    pub(crate) imported_memories: Vec<*mut Memory>,
    /// List of pointers to `LegacyEnv`s used to patch imported functions to be able to
    /// pass a the "vmctx" as the first argument.
    /// Needed here because of extending import objects.
    pub(crate) instance_pointers_to_update: Vec<Arc<Mutex<LegacyEnv>>>,
}

#[repr(C)]
#[derive(Clone)]
pub struct wasmer_import_func_t;

#[repr(C)]
#[derive(Clone)]
pub struct wasmer_import_descriptor_t;

#[repr(C)]
#[derive(Clone)]
pub struct wasmer_import_descriptors_t;

#[repr(C)]
#[derive(Clone)]
pub struct wasmer_import_object_iter_t;

/// Creates a new empty import object.
/// See also `wasmer_import_object_append`
#[allow(clippy::cast_ptr_alignment)]
#[no_mangle]
pub extern "C" fn wasmer_import_object_new() -> NonNull<wasmer_import_object_t> {
    let import_object_inner: Box<dyn NamedResolver> = Box::new(ImportObject::new());
    let import_object = Box::new(CAPIImportObject {
        import_object: import_object_inner,
        imported_memories: vec![],
        instance_pointers_to_update: vec![],
    });

    // TODO: use `Box::into_raw_non_null` when it becomes stable
    unsafe { NonNull::new_unchecked(Box::into_raw(import_object) as *mut wasmer_import_object_t) }
}

#[cfg(feature = "wasi")]
mod wasi;

#[cfg(feature = "wasi")]
pub use self::wasi::*;

#[cfg(feature = "emscripten")]
mod emscripten;

#[cfg(feature = "emscripten")]
pub use self::emscripten::*;

/// Gets an entry from an ImportObject at the name and namespace.
/// Stores `name`, `namespace`, and `import_export_value` in `import`.
/// Thus these must remain valid for the lifetime of `import`.
///
/// The caller owns all data involved.
/// `import_export_value` will be written to based on `tag`.
#[no_mangle]
pub unsafe extern "C" fn wasmer_import_object_get_import(
    import_object: *const wasmer_import_object_t,
    namespace: wasmer_byte_array,
    name: wasmer_byte_array,
    import: *mut wasmer_import_t,
    import_export_value: *mut wasmer_import_export_value,
    tag: u32,
) -> wasmer_result_t {
    todo!("Disabled until ImportObject APIs are updated")
    /*
    let tag: wasmer_import_export_kind = if let Ok(t) = TryFrom::try_from(tag) {
        t
    } else {
        update_last_error(CApiError {
            msg: "wasmer_import_export_tag out of range".to_string(),
        });
        return wasmer_result_t::WASMER_ERROR;
    };
    let import_object: &mut ImportObject = &mut *(import_object as *mut ImportObject);
    let namespace_str = if let Ok(ns) = namespace.as_str() {
        ns
    } else {
        update_last_error(CApiError {
            msg: "error converting namespace to UTF-8 string".to_string(),
        });
        return wasmer_result_t::WASMER_ERROR;
    };
    let name_str = if let Ok(name) = name.as_str() {
        name
    } else {
        update_last_error(CApiError {
            msg: "error converting name to UTF-8 string".to_string(),
        });
        return wasmer_result_t::WASMER_ERROR;
    };
    if import.is_null() || import_export_value.is_null() {
        update_last_error(CApiError {
            msg: "pointers to import and import_export_value must not be null".to_string(),
        });
        return wasmer_result_t::WASMER_ERROR;
    }
    let import_out = &mut *import;
    let import_export_value_out = &mut *import_export_value;
    if let Some(export) = import_object.get_export(namespace_str, name_str) {
        match export {
            Extern::Function(function) => {
                if tag != wasmer_import_export_kind::WASM_FUNCTION {
                    update_last_error(CApiError {
                        msg: format!("Found function, expected {}", tag.to_str()),
                    });
                    return wasmer_result_t::WASMER_ERROR;
                }
                import_out.tag = wasmer_import_export_kind::WASM_FUNCTION;
                let writer = import_export_value_out.func as *mut Function;
                *writer = function.clone();
            }
            Extern::Memory(memory) => {
                if tag != wasmer_import_export_kind::WASM_MEMORY {
                    update_last_error(CApiError {
                        msg: format!("Found memory, expected {}", tag.to_str()),
                    });
                    return wasmer_result_t::WASMER_ERROR;
                }
                import_out.tag = wasmer_import_export_kind::WASM_MEMORY;
                let writer = import_export_value_out.func as *mut Memory;
                *writer = memory.clone();
            }
            Extern::Table(table) => {
                if tag != wasmer_import_export_kind::WASM_TABLE {
                    update_last_error(CApiError {
                        msg: format!("Found table, expected {}", tag.to_str()),
                    });
                    return wasmer_result_t::WASMER_ERROR;
                }
                import_out.tag = wasmer_import_export_kind::WASM_TABLE;
                let writer = import_export_value_out.func as *mut Table;
                *writer = table.clone();
            }
            Extern::Global(global) => {
                if tag != wasmer_import_export_kind::WASM_GLOBAL {
                    update_last_error(CApiError {
                        msg: format!("Found global, expected {}", tag.to_str()),
                    });
                    return wasmer_result_t::WASMER_ERROR;
                }
                import_out.tag = wasmer_import_export_kind::WASM_GLOBAL;
                let writer = import_export_value_out.func as *mut Global;
                *writer = global.clone();
            }
        }

        import_out.value = *import_export_value;
        import_out.module_name = namespace;
        import_out.import_name = name;

        wasmer_result_t::WASMER_OK
    } else {
        update_last_error(CApiError {
            msg: format!("Extern {} {} not found", namespace_str, name_str),
        });
        wasmer_result_t::WASMER_ERROR
    }
    */
}

/// private wrapper data type used for casting
#[repr(C)]
struct WasmerImportObjectIterator(
    std::iter::Peekable<Box<dyn Iterator<Item = <ImportObjectIterator as Iterator>::Item>>>,
);

/// Create an iterator over the functions in the import object.
/// Get the next import with `wasmer_import_object_iter_next`
/// Free the iterator with `wasmer_import_object_iter_destroy`
#[no_mangle]
pub unsafe extern "C" fn wasmer_import_object_iterate_functions(
    import_object: *const wasmer_import_object_t,
) -> *mut wasmer_import_object_iter_t {
    todo!("Disabled until ImportObject APIs are updated")
    /*
    if import_object.is_null() {
        update_last_error(CApiError {
            msg: "import_object must not be null".to_owned(),
        });
        return std::ptr::null_mut();
    }
    let import_object: &ImportObject = &*(import_object as *const ImportObject);
    let iter_inner = Box::new(import_object.clone().into_iter().filter(|((_, _), e)| {
        if let Extern::Function(_) = e {
            true
        } else {
            false
        }
    })) as Box<dyn Iterator<Item = <ImportObjectIterator as Iterator>::Item>>;
    let iterator = Box::new(WasmerImportObjectIterator(iter_inner.peekable()));

    Box::into_raw(iterator) as *mut wasmer_import_object_iter_t
    */
}

/// Writes the next value to `import`.  `WASMER_ERROR` is returned if there
/// was an error or there's nothing left to return.
///
/// To free the memory allocated here, pass the import to `wasmer_import_object_imports_destroy`.
/// To check if the iterator is done, use `wasmer_import_object_iter_at_end`.
#[no_mangle]
pub unsafe extern "C" fn wasmer_import_object_iter_next(
    import_object_iter: *mut wasmer_import_object_iter_t,
    import: *mut wasmer_import_t,
) -> wasmer_result_t {
    todo!("Disabled until ImportObject APIs are updated")
    /*
    if import_object_iter.is_null() || import.is_null() {
        update_last_error(CApiError {
            msg: "import_object_iter and import must not be null".to_owned(),
        });
        return wasmer_result_t::WASMER_ERROR;
    }

    let iter = &mut *(import_object_iter as *mut WasmerImportObjectIterator);
    let out = &mut *import;
    // TODO: the copying here can be optimized away, we just need to use a different type of
    // iterator internally
    if let Some(((namespace, name), export)) = iter.0.next() {
        let ns = {
            let mut n = namespace.clone();
            n.shrink_to_fit();
            n.into_bytes()
        };
        let ns_bytes = wasmer_byte_array {
            bytes: ns.as_ptr(),
            bytes_len: ns.len() as u32,
        };

        let name = {
            let mut n = name.clone();
            n.shrink_to_fit();
            n.into_bytes()
        };
        let name_bytes = wasmer_byte_array {
            bytes: name.as_ptr(),
            bytes_len: name.len() as u32,
        };

        out.module_name = ns_bytes;
        out.import_name = name_bytes;

        std::mem::forget(ns);
        std::mem::forget(name);

        match export {
            Extern::Function(function) => {
                let func = Box::new(function.clone());

                out.tag = wasmer_import_export_kind::WASM_FUNCTION;
                out.value = wasmer_import_export_value {
                    func: Box::into_raw(func) as *mut _ as *const _,
                };
            }
            Extern::Global(global) => {
                let glbl = Box::new(global.clone());

                out.tag = wasmer_import_export_kind::WASM_GLOBAL;
                out.value = wasmer_import_export_value {
                    global: Box::into_raw(glbl) as *mut _ as *const _,
                };
            }
            Extern::Memory(memory) => {
                let mem = Box::new(memory.clone());

                out.tag = wasmer_import_export_kind::WASM_MEMORY;
                out.value = wasmer_import_export_value {
                    memory: Box::into_raw(mem) as *mut _ as *const _,
                };
            }
            Extern::Table(table) => {
                let tbl = Box::new(table.clone());

                out.tag = wasmer_import_export_kind::WASM_TABLE;
                out.value = wasmer_import_export_value {
                    memory: Box::into_raw(tbl) as *mut _ as *const _,
                };
            }
        }

        wasmer_result_t::WASMER_OK
    } else {
        wasmer_result_t::WASMER_ERROR
    }
    */
}

/// Returns true if further calls to `wasmer_import_object_iter_next` will
/// not return any new data
#[no_mangle]
pub unsafe extern "C" fn wasmer_import_object_iter_at_end(
    import_object_iter: Option<NonNull<wasmer_import_object_iter_t>>,
) -> bool {
    let mut import_object_iter = if let Some(import_object_iter) = import_object_iter {
        import_object_iter.cast::<WasmerImportObjectIterator>()
    } else {
        update_last_error(CApiError {
            msg: "import_object_iter must not be null".to_owned(),
        });
        return true;
    };
    let iter = import_object_iter.as_mut();

    iter.0.peek().is_none()
}

/// Frees the memory allocated by `wasmer_import_object_iterate_functions`
#[no_mangle]
pub unsafe extern "C" fn wasmer_import_object_iter_destroy(
    import_object_iter: Option<NonNull<wasmer_import_object_iter_t>>,
) {
    if let Some(import_object_iter) = import_object_iter {
        let _ = Box::from_raw(
            import_object_iter
                .cast::<WasmerImportObjectIterator>()
                .as_ptr(),
        );
    }
}

/// Frees the memory allocated in `wasmer_import_object_iter_next`
///
/// This function does not free the memory in `wasmer_import_object_t`;
/// it only frees memory allocated while querying a `wasmer_import_object_t`.
#[no_mangle]
pub unsafe extern "C" fn wasmer_import_object_imports_destroy(
    imports: Option<NonNull<wasmer_import_t>>,
    imports_len: u32,
) {
    let imports = if let Some(imp) = imports {
        imp
    } else {
        return;
    };
    let imports: &[wasmer_import_t] =
        &*slice::from_raw_parts_mut(imports.as_ptr(), imports_len as usize);
    for import in imports {
        let _namespace: Vec<u8> = Vec::from_raw_parts(
            import.module_name.bytes as *mut u8,
            import.module_name.bytes_len as usize,
            import.module_name.bytes_len as usize,
        );
        let _name: Vec<u8> = Vec::from_raw_parts(
            import.import_name.bytes as *mut u8,
            import.import_name.bytes_len as usize,
            import.import_name.bytes_len as usize,
        );
        match import.tag {
            wasmer_import_export_kind::WASM_FUNCTION => {
                let function_wrapper: Box<FunctionWrapper> =
                    Box::from_raw(import.value.func as *mut _);
                let _: Box<Function> = Box::from_raw(function_wrapper.func.as_ptr());
            }
            wasmer_import_export_kind::WASM_GLOBAL => {
                let _: Box<Global> = Box::from_raw(import.value.global as *mut _);
            }
            wasmer_import_export_kind::WASM_MEMORY => {
                let _: Box<Memory> = Box::from_raw(import.value.memory as *mut _);
            }
            wasmer_import_export_kind::WASM_TABLE => {
                let _: Box<Table> = Box::from_raw(import.value.table as *mut _);
            }
        }
    }
}

/// Extends an existing import object with new imports
#[allow(clippy::cast_ptr_alignment)]
#[no_mangle]
pub unsafe extern "C" fn wasmer_import_object_extend(
    import_object: *mut wasmer_import_object_t,
    imports: *const wasmer_import_t,
    imports_len: c_uint,
) -> wasmer_result_t {
    let import_object: &mut CAPIImportObject = &mut *(import_object as *mut CAPIImportObject);

    let mut import_data: HashMap<String, Exports> = HashMap::new();
    let imports: &[wasmer_import_t] = slice::from_raw_parts(imports, imports_len as usize);
    for import in imports {
        let module_name = slice::from_raw_parts(
            import.module_name.bytes,
            import.module_name.bytes_len as usize,
        );
        let module_name = if let Ok(s) = std::str::from_utf8(module_name) {
            s
        } else {
            update_last_error(CApiError {
                msg: "error converting module name to string".to_string(),
            });
            return wasmer_result_t::WASMER_ERROR;
        };
        let import_name = slice::from_raw_parts(
            import.import_name.bytes,
            import.import_name.bytes_len as usize,
        );
        let import_name = if let Ok(s) = std::str::from_utf8(import_name) {
            s
        } else {
            update_last_error(CApiError {
                msg: "error converting import_name to string".to_string(),
            });
            return wasmer_result_t::WASMER_ERROR;
        };

        let export = match import.tag {
            wasmer_import_export_kind::WASM_MEMORY => {
                let mem = import.value.memory as *mut Memory;
                import_object.imported_memories.push(mem);
                Extern::Memory((&*mem).clone())
            }
            wasmer_import_export_kind::WASM_FUNCTION => {
                // TODO: investigate consistent usage of `FunctionWrapper` in this context
                let func_wrapper = import.value.func as *mut FunctionWrapper;
                let func_export = (*func_wrapper).func.as_ptr();
                import_object
                    .instance_pointers_to_update
                    .push((*func_wrapper).legacy_env.clone());
                Extern::Function((&*func_export).clone())
            }
            wasmer_import_export_kind::WASM_GLOBAL => {
                let global = import.value.global as *mut Global;
                Extern::Global((&*global).clone())
            }
            wasmer_import_export_kind::WASM_TABLE => {
                let table = import.value.table as *mut Table;
                Extern::Table((&*table).clone())
            }
        };

        let export_entry = import_data
            .entry(module_name.to_string())
            .or_insert_with(Exports::new);
        export_entry.insert(import_name.to_string(), export);
    }

    let mut new_import_object = ImportObject::new();
    for (k, v) in import_data {
        new_import_object.register(k, v);
    }

    // We use `Option<Box<T>>` here to perform a swap (move value out and move a new value in)
    // Thus this code will break if `size_of::<Option<Box<T>>>` != `size_of::<Box<T>>`.
    debug_assert_eq!(
        std::mem::size_of::<Option<Box<dyn NamedResolver>>>(),
        std::mem::size_of::<Box<dyn NamedResolver>>()
    );

    let import_object_inner: &mut Option<Box<dyn NamedResolver>> =
        &mut *(&mut import_object.import_object as *mut Box<dyn NamedResolver>
            as *mut Option<Box<dyn NamedResolver>>);

    let import_object_value = import_object_inner.take().unwrap();
    let new_resolver: Box<dyn NamedResolver> =
        Box::new(import_object_value.chain_front(new_import_object));
    *import_object_inner = Some(new_resolver);

    return wasmer_result_t::WASMER_OK;
}

/// Gets import descriptors for the given module
///
/// The caller owns the object and should call `wasmer_import_descriptors_destroy` to free it.
#[allow(clippy::cast_ptr_alignment)]
#[no_mangle]
pub unsafe extern "C" fn wasmer_import_descriptors(
    module: Option<&wasmer_module_t>,
    import_descriptors: *mut *mut wasmer_import_descriptors_t,
) {
    let module = if let Some(module) = module {
        &*(module as *const wasmer_module_t as *const Module)
    } else {
        return;
    };
    let descriptors = module.imports().collect::<Vec<ImportType>>();

    let named_import_descriptors: Box<NamedImportDescriptors> =
        Box::new(NamedImportDescriptors(descriptors));
    *import_descriptors =
        Box::into_raw(named_import_descriptors) as *mut wasmer_import_descriptors_t;
}

pub struct NamedImportDescriptors(Vec<ImportType>);

/// Frees the memory for the given import descriptors
#[allow(clippy::cast_ptr_alignment)]
#[no_mangle]
pub extern "C" fn wasmer_import_descriptors_destroy(
    import_descriptors: Option<NonNull<wasmer_import_descriptors_t>>,
) {
    if let Some(id) = import_descriptors {
        unsafe { Box::from_raw(id.cast::<NamedImportDescriptors>().as_ptr()) };
    }
}

/// Gets the length of the import descriptors
#[allow(clippy::cast_ptr_alignment)]
#[no_mangle]
pub unsafe extern "C" fn wasmer_import_descriptors_len(
    exports: Option<NonNull<wasmer_import_descriptors_t>>,
) -> c_uint {
    let exports = if let Some(exports) = exports {
        exports.cast::<NamedImportDescriptors>()
    } else {
        return 0;
    };
    exports.as_ref().0.len() as c_uint
}

/// Gets import descriptor by index
#[allow(clippy::cast_ptr_alignment)]
#[no_mangle]
pub unsafe extern "C" fn wasmer_import_descriptors_get(
    import_descriptors: Option<NonNull<wasmer_import_descriptors_t>>,
    idx: c_uint,
) -> Option<NonNull<wasmer_import_descriptor_t>> {
    let mut nid = import_descriptors?.cast::<NamedImportDescriptors>();
    let named_import_descriptors = nid.as_mut();
    Some(
        NonNull::from(&mut named_import_descriptors.0[idx as usize])
            .cast::<wasmer_import_descriptor_t>(),
    )
}

/// Gets name for the import descriptor
#[no_mangle]
#[allow(clippy::cast_ptr_alignment)]
pub unsafe extern "C" fn wasmer_import_descriptor_name(
    import_descriptor: *mut wasmer_import_descriptor_t,
) -> wasmer_byte_array {
    let named_import_descriptor = &*(import_descriptor as *mut ImportType);
    wasmer_byte_array {
        bytes: named_import_descriptor.name().as_ptr(),
        bytes_len: named_import_descriptor.name().len() as u32,
    }
}

/// Gets module name for the import descriptor
#[no_mangle]
#[allow(clippy::cast_ptr_alignment)]
pub unsafe extern "C" fn wasmer_import_descriptor_module_name(
    import_descriptor: *mut wasmer_import_descriptor_t,
) -> wasmer_byte_array {
    let named_import_descriptor = &*(import_descriptor as *mut ImportType);
    wasmer_byte_array {
        bytes: named_import_descriptor.module().as_ptr(),
        bytes_len: named_import_descriptor.module().len() as u32,
    }
}

/// Gets export descriptor kind
#[no_mangle]
#[allow(clippy::cast_ptr_alignment)]
pub unsafe extern "C" fn wasmer_import_descriptor_kind(
    export: *mut wasmer_import_descriptor_t,
) -> wasmer_import_export_kind {
    let named_import_descriptor = &*(export as *mut ImportType);
    named_import_descriptor.ty().into()
}

/// Sets the result parameter to the arity of the params of the wasmer_import_func_t
///
/// Returns `wasmer_result_t::WASMER_OK` upon success.
///
/// Returns `wasmer_result_t::WASMER_ERROR` upon failure. Use `wasmer_last_error_length`
/// and `wasmer_last_error_message` to get an error message.
#[no_mangle]
#[allow(clippy::cast_ptr_alignment)]
pub unsafe extern "C" fn wasmer_import_func_params_arity(
    func: *const wasmer_import_func_t,
    result: *mut u32,
) -> wasmer_result_t {
    todo!("Figure out how to get a usable siganture from an Function")
    /*let function = &*(func as *const Function);
    if let Extren::Function(function) = *export {
        *result = signature.params().len() as u32;
        wasmer_result_t::WASMER_OK
    } else {
        update_last_error(CApiError {
            msg: "func ptr error in wasmer_import_func_params_arity".to_string(),
        });
        wasmer_result_t::WASMER_ERROR
    }
    */
}

/// struct used to pass in context to functions (which must be back-patched)

#[derive(Debug, Default, WasmerEnv, Clone)]
pub(crate) struct LegacyEnv {
    pub(crate) instance_ptr: Option<NonNull<CAPIInstance>>,
}

/// Because this type requires unsafe to do any meaningful operation, we
/// can implement `Send` and `Sync`. All operations must be synchronized.
unsafe impl Send for LegacyEnv {}
unsafe impl Sync for LegacyEnv {}

impl LegacyEnv {
    pub(crate) fn ctx_ptr(&self) -> *mut CAPIInstance {
        self.instance_ptr
            .map(|p| p.as_ptr())
            .unwrap_or(ptr::null_mut())
    }
}

/// struct used to hold on to `LegacyEnv` pointer as well as the function.
/// we need to do this to initialize the context ptr inside of `LegacyEnv` when
/// instantiating the module.
#[derive(Debug)]
pub(crate) struct FunctionWrapper {
    pub(crate) func: NonNull<Function>,
    pub(crate) legacy_env: Arc<Mutex<LegacyEnv>>,
}

/// Creates new host function, aka imported function. `func` is a
/// function pointer, where the first argument is the famous `vm::Ctx`
/// (in Rust), or `wasmer_instance_context_t` (in C). All arguments
/// must be typed with compatible WebAssembly native types:
///
/// | WebAssembly type | C/C++ type |
/// | ---------------- | ---------- |
/// | `i32`            | `int32_t`  |
/// | `i64`            | `int64_t`  |
/// | `f32`            | `float`    |
/// | `f64`            | `double`   |
///
/// The function pointer must have a lifetime greater than the
/// WebAssembly instance lifetime.
///
/// The caller owns the object and should call
/// `wasmer_import_func_destroy` to free it.
#[no_mangle]
#[allow(clippy::cast_ptr_alignment)]
pub unsafe extern "C" fn wasmer_import_func_new(
    func: extern "C" fn(data: *mut c_void),
    params: *const wasmer_value_tag,
    params_len: c_uint,
    returns: *const wasmer_value_tag,
    returns_len: c_uint,
) -> *mut wasmer_import_func_t {
    let params: &[wasmer_value_tag] = slice::from_raw_parts(params, params_len as usize);
    let params: Vec<ValType> = params.iter().cloned().map(|x| x.into()).collect();
    let returns: &[wasmer_value_tag] = slice::from_raw_parts(returns, returns_len as usize);
    let returns: Vec<ValType> = returns.iter().cloned().map(|x| x.into()).collect();
    let func_type = FunctionType::new(params, &returns[..]);

    let store = get_global_store();

    let env = Arc::new(Mutex::new(LegacyEnv::default()));

    let func = Function::new_with_env(store, &func_type, env.clone(), move |env, args| {
        use libffi::high::call::{call, Arg};
        use libffi::low::CodePtr;
        let env_guard = env.lock().unwrap();

        let ctx_ptr = env_guard.ctx_ptr();
        let ctx_ptr_val = ctx_ptr as *const _ as isize as i64;

        let ffi_args: Vec<Arg> = {
            let mut ffi_args = Vec::with_capacity(args.len() + 1);
            ffi_args.push(Arg::new::<i64>(&ctx_ptr_val));
            ffi_args.extend(args.iter().map(|ty| match ty {
                Val::I32(v) => Arg::new::<i32>(v),
                Val::I64(v) => Arg::new::<i64>(v),
                Val::F32(v) => Arg::new::<f32>(v),
                Val::F64(v) => Arg::new::<f64>(v),
                _ => todo!("Unsupported type in C API"),
            }));

            ffi_args
        };
        let code_ptr = CodePtr::from_ptr(func as _);

        assert!(returns.len() <= 1);

        let return_value = if returns.len() == 1 {
            vec![match returns[0] {
                ValType::I32 => Val::I32(call::<i32>(code_ptr, &ffi_args)),
                ValType::I64 => Val::I64(call::<i64>(code_ptr, &ffi_args)),
                ValType::F32 => Val::F32(call::<f32>(code_ptr, &ffi_args)),
                ValType::F64 => Val::F64(call::<f64>(code_ptr, &ffi_args)),
                _ => todo!("Unsupported type in C API"),
            }]
        } else {
            call::<()>(code_ptr, &ffi_args);
            vec![]
        };

        Ok(return_value)
    });

    let function_wrapper = FunctionWrapper {
        func: NonNull::new_unchecked(Box::into_raw(Box::new(func))),
        legacy_env: env.clone(),
    };
    Box::into_raw(Box::new(function_wrapper)) as *mut wasmer_import_func_t
}

/// Stop the execution of a host function, aka imported function. The
/// function must be used _only_ inside a host function.
///
/// The pointer to `wasmer_instance_context_t` is received by the host
/// function as its first argument. Just passing it to `ctx` is fine.
///
/// The error message must have a greater lifetime than the host
/// function itself since the error is read outside the host function
/// with `wasmer_last_error_message`.
///
/// This function returns `wasmer_result_t::WASMER_ERROR` if `ctx` or
/// `error_message` are null.
///
/// This function never returns otherwise.
#[no_mangle]
#[allow(clippy::cast_ptr_alignment)]
pub unsafe extern "C" fn wasmer_trap(
    _ctx: *const wasmer_instance_context_t,
    error_message: *const c_char,
) -> wasmer_result_t {
    if error_message.is_null() {
        update_last_error(CApiError {
            msg: "error_message is null in wasmer_trap".to_string(),
        });

        return wasmer_result_t::WASMER_ERROR;
    }

    let error_message = CStr::from_ptr(error_message).to_str().unwrap();

    wasmer::raise_user_trap(Box::new(RuntimeError::new(error_message))); // never returns

    // cbindgen does not generate a binding for a function that
    // returns `!`. Since we also need to error in some cases, the
    // output type of `wasmer_trap` is `wasmer_result_t`. But the OK
    // case is not reachable because `do_early_trap` never
    // returns. That's a compromise to satisfy the Rust type system,
    // cbindgen, and get an acceptable clean code.
    #[allow(unreachable_code)]
    wasmer_result_t::WASMER_OK
}

/// Sets the params buffer to the parameter types of the given wasmer_import_func_t
///
/// Returns `wasmer_result_t::WASMER_OK` upon success.
///
/// Returns `wasmer_result_t::WASMER_ERROR` upon failure. Use `wasmer_last_error_length`
/// and `wasmer_last_error_message` to get an error message.
#[no_mangle]
#[allow(clippy::cast_ptr_alignment)]
pub unsafe extern "C" fn wasmer_import_func_params(
    func: *const wasmer_import_func_t,
    params: *mut wasmer_value_tag,
    params_len: c_uint,
) -> wasmer_result_t {
    todo!("Figure out how to get a usable signature from an `Function`")
    /*
    let function = &*(func as *const Function);
    if let Extern::Function(function) = *export {
        let params: &mut [wasmer_value_tag] =
            slice::from_raw_parts_mut(params, params_len as usize);
        for (i, item) in signature.params().iter().enumerate() {
            params[i] = item.into();
        }
        wasmer_result_t::WASMER_OK
    } else {
        update_last_error(CApiError {
            msg: "func ptr error in wasmer_import_func_params".to_string(),
        });
        wasmer_result_t::WASMER_ERROR
    }
    */
}

/// Sets the returns buffer to the parameter types of the given wasmer_import_func_t
///
/// Returns `wasmer_result_t::WASMER_OK` upon success.
///
/// Returns `wasmer_result_t::WASMER_ERROR` upon failure. Use `wasmer_last_error_length`
/// and `wasmer_last_error_message` to get an error message.
#[no_mangle]
#[allow(clippy::cast_ptr_alignment)]
pub unsafe extern "C" fn wasmer_import_func_returns(
    func: *const wasmer_import_func_t,
    returns: *mut wasmer_value_tag,
    returns_len: c_uint,
) -> wasmer_result_t {
    todo!("Figure out how to get a usable signature from an `Function`")
    /*
    let function = &*(func as *const Function);
        let returns: &mut [wasmer_value_tag] =
            slice::from_raw_parts_mut(returns, returns_len as usize);
        for (i, item) in signature.returns().iter().enumerate() {
            returns[i] = item.into();
        }
        wasmer_result_t::WASMER_OK
    */
}

/// Sets the result parameter to the arity of the returns of the wasmer_import_func_t
///
/// Returns `wasmer_result_t::WASMER_OK` upon success.
///
/// Returns `wasmer_result_t::WASMER_ERROR` upon failure. Use `wasmer_last_error_length`
/// and `wasmer_last_error_message` to get an error message.
#[no_mangle]
#[allow(clippy::cast_ptr_alignment)]
pub unsafe extern "C" fn wasmer_import_func_returns_arity(
    func: *const wasmer_import_func_t,
    result: *mut u32,
) -> wasmer_result_t {
    todo!("Figure out how to get a usable signature from an `Function`")
    /*
    let function = &*(func as *const Function);
    *result = signature.returns().len() as u32;
    wasmer_result_t::WASMER_OK
    */
}

/// Frees memory for the given Func
#[allow(clippy::cast_ptr_alignment)]
#[no_mangle]
pub unsafe extern "C" fn wasmer_import_func_destroy(func: Option<NonNull<wasmer_import_func_t>>) {
    if let Some(func) = func {
        let function_wrapper = Box::from_raw(func.cast::<FunctionWrapper>().as_ptr());
        let _function = Box::from_raw(function_wrapper.func.as_ptr());
    }
}

/// Frees memory of the given ImportObject
#[no_mangle]
pub unsafe extern "C" fn wasmer_import_object_destroy(
    import_object: Option<NonNull<wasmer_import_object_t>>,
) {
    if let Some(import_object) = import_object {
        let _resolver_box: Box<Box<dyn NamedResolver>> =
            Box::from_raw(import_object.cast::<Box<dyn NamedResolver>>().as_ptr());
    }
}

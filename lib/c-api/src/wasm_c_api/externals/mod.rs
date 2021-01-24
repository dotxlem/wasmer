mod function;
mod global;
mod memory;
mod table;

pub use function::*;
pub use global::*;
pub use memory::*;
use std::sync::Arc;
pub use table::*;
use wasmer::{Extern, Instance};

#[allow(non_camel_case_types)]
#[derive(Clone)]
pub struct wasm_extern_t {
    // this is how we ensure the instance stays alive
    pub(crate) instance: Option<Arc<Instance>>,
    pub(crate) inner: Extern,
}

wasm_declare_boxed_vec!(extern);

#[no_mangle]
pub unsafe extern "C" fn wasm_func_as_extern(
    func: Option<&wasm_func_t>,
) -> Option<Box<wasm_extern_t>> {
    let func = func?;

    Some(Box::new(wasm_extern_t {
        instance: func.instance.clone(),
        inner: Extern::Function(func.inner.clone()),
    }))
}

#[no_mangle]
pub unsafe extern "C" fn wasm_global_as_extern(
    global: Option<&wasm_global_t>,
) -> Option<Box<wasm_extern_t>> {
    let global = global?;

    Some(Box::new(wasm_extern_t {
        // TODO: update this if global does hold onto an `instance`
        instance: None,
        inner: Extern::Global(global.inner.clone()),
    }))
}

#[no_mangle]
pub unsafe extern "C" fn wasm_memory_as_extern(
    memory: Option<&wasm_memory_t>,
) -> Option<Box<wasm_extern_t>> {
    let memory = memory?;

    Some(Box::new(wasm_extern_t {
        // TODO: update this if global does hold onto an `instance`
        instance: None,
        inner: Extern::Memory(memory.inner.clone()),
    }))
}

#[no_mangle]
pub unsafe extern "C" fn wasm_table_as_extern(
    table: Option<&wasm_table_t>,
) -> Option<Box<wasm_extern_t>> {
    let table = table?;

    Some(Box::new(wasm_extern_t {
        // TODO: update this if global does hold onto an `instance`
        instance: None,
        inner: Extern::Table(table.inner.clone()),
    }))
}

#[no_mangle]
pub unsafe extern "C" fn wasm_extern_as_func(
    r#extern: Option<&wasm_extern_t>,
) -> Option<Box<wasm_func_t>> {
    let r#extern = r#extern?;

    if let Extern::Function(f) = &r#extern.inner {
        Some(Box::new(wasm_func_t {
            inner: f.clone(),
            instance: r#extern.instance.clone(),
        }))
    } else {
        None
    }
}

#[no_mangle]
pub unsafe extern "C" fn wasm_extern_as_global(
    r#extern: Option<&wasm_extern_t>,
) -> Option<Box<wasm_global_t>> {
    let r#extern = r#extern?;

    if let Extern::Global(g) = &r#extern.inner {
        Some(Box::new(wasm_global_t { inner: g.clone() }))
    } else {
        None
    }
}

#[no_mangle]
pub unsafe extern "C" fn wasm_extern_as_memory(
    r#extern: Option<&wasm_extern_t>,
) -> Option<Box<wasm_memory_t>> {
    let r#extern = r#extern?;

    if let Extern::Memory(m) = &r#extern.inner {
        Some(Box::new(wasm_memory_t { inner: m.clone() }))
    } else {
        None
    }
}

#[no_mangle]
pub unsafe extern "C" fn wasm_extern_as_table(
    r#extern: Option<&wasm_extern_t>,
) -> Option<Box<wasm_table_t>> {
    let r#extern = r#extern?;

    if let Extern::Table(t) = &r#extern.inner {
        Some(Box::new(wasm_table_t { inner: t.clone() }))
    } else {
        None
    }
}

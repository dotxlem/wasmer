//! This build script aims at:
//!
//! * generating the C header files for the C API,
//! * setting `inline-c` up.

use cbindgen::{Builder, Language};
use std::{env, fs, path::PathBuf};

const PRE_HEADER: &'static str = r#"
// Define the `ARCH_X86_X64` constant.
#if defined(MSVC) && defined(_M_AMD64)
#  define ARCH_X86_64
#elif (defined(GCC) || defined(__GNUC__) || defined(__clang__)) && defined(__x86_64__)
#  define ARCH_X86_64
#endif

// Compatibility with non-Clang compilers.
#if !defined(__has_attribute)
#  define __has_attribute(x) 0
#endif

// Compatibility with non-Clang compilers.
#if !defined(__has_declspec_attribute)
#  define __has_declspec_attribute(x) 0
#endif

// Define the `DEPRECATED` macro.
#if defined(GCC) || defined(__GNUC__) || __has_attribute(deprecated)
#  define DEPRECATED(message) __attribute__((deprecated(message)))
#elif defined(MSVC) || __has_declspec_attribute(deprecated)
#  define DEPRECATED(message) __declspec(deprecated(message))
#endif
"#;

#[allow(unused)]
const JIT_FEATURE_AS_C_DEFINE: &'static str = "WASMER_JIT_ENABLED";

#[allow(unused)]
const COMPILER_FEATURE_AS_C_DEFINE: &'static str = "WASMER_COMPILER_ENABLED";

#[allow(unused)]
const WASI_FEATURE_AS_C_DEFINE: &'static str = "WASMER_WASI_ENABLED";

#[allow(unused)]
const EMSCRIPTEN_FEATURE_AS_C_DEFINE: &'static str = "WASMER_EMSCRIPTEN_ENABLED";

macro_rules! map_feature_as_c_define {
    ($feature:expr, $c_define:ident, $accumulator:ident) => {
        #[cfg(feature = $feature)]
        {
            $accumulator.push_str(&format!(
                r#"
// The `{feature}` feature has been enabled for this build.
#define {define}
"#,
                feature = $feature,
                define = $c_define,
            ));
        }
    };
}

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();

    if building_c_api_headers() {
        build_wasm_c_api_headers(&crate_dir, &out_dir);
        build_wasmer_c_api_headers(&crate_dir, &out_dir);
    }

    build_inline_c_env_vars();
}

/// Check whether we should build the C API headers.
///
/// For the moment, it's always enabled, unless if the `DOCS_RS`
/// environment variable is present.
fn building_c_api_headers() -> bool {
    env::var("DOCS_RS").is_err()
}

/// Build the header files for the `wasm_c_api` API.
fn build_wasm_c_api_headers(crate_dir: &str, out_dir: &str) {
    let mut crate_header_file = PathBuf::from(crate_dir);
    crate_header_file.push("wasmer_wasm");

    let mut out_header_file = PathBuf::from(out_dir);
    out_header_file.push("wasmer_wasm");

    let mut pre_header = format!(
        r#"// The Wasmer C/C++ header file compatible with the `wasm-c-api` standard API.
// This file is generated by lib/c-api/build.rs.

#if !defined(WASMER_WASM_H_PRELUDE)

#define WASMER_WASM_H_PRELUDE
{pre_header}"#,
        pre_header = PRE_HEADER
    );

    map_feature_as_c_define!("jit", JIT_FEATURE_AS_C_DEFINE, pre_header);
    map_feature_as_c_define!("compiler", COMPILER_FEATURE_AS_C_DEFINE, pre_header);
    map_feature_as_c_define!("wasi", WASI_FEATURE_AS_C_DEFINE, pre_header);
    map_feature_as_c_define!("emscripten", EMSCRIPTEN_FEATURE_AS_C_DEFINE, pre_header);

    add_wasmer_version(&mut pre_header);

    // Close pre header.
    pre_header.push_str(
        r#"
#endif // WASMER_WASM_H_PRELUDE


//
// OK, here we go. The code below is automatically generated.
//
"#,
    );

    let guard = "WASMER_WASM_H";

    // C bindings.
    {
        // Generate the bindings in the `OUT_DIR`.
        out_header_file.set_extension("h");

        // Build and generate the header file.
        exclude_items_from_deprecated(new_builder(Language::C, crate_dir, guard, &pre_header))
            .with_include("wasm.h")
            .generate()
            .expect("Unable to generate C bindings")
            .write_to_file(out_header_file.as_path());

        // Copy the generated bindings from `OUT_DIR` to
        // `CARGO_MANIFEST_DIR`.
        crate_header_file.set_extension("h");

        fs::copy(out_header_file.as_path(), crate_header_file.as_path())
            .expect("Unable to copy the generated C bindings");
    }
}

/// Build the header files for the `deprecated` API.
fn build_wasmer_c_api_headers(crate_dir: &str, out_dir: &str) {
    let mut crate_header_file = PathBuf::from(crate_dir);
    crate_header_file.push("wasmer");

    let mut out_header_file = PathBuf::from(out_dir);
    out_header_file.push("wasmer");

    let mut pre_header = format!(
        r#"// The Wasmer C/C++ header file.

#if !defined(WASMER_H_PRELUDE)

#define WASMER_H_PRELUDE
{pre_header}"#,
        pre_header = PRE_HEADER
    );

    map_feature_as_c_define!("wasi", WASI_FEATURE_AS_C_DEFINE, pre_header);
    map_feature_as_c_define!("emscritpen", EMSCRIPTEN_FEATURE_AS_C_DEFINE, pre_header);

    add_wasmer_version(&mut pre_header);

    // Close pre header.
    pre_header.push_str(
        r#"
#endif // WASMER_H_PRELUDE


//
// OK, here we go. The code below is automatically generated.
//
"#,
    );

    let guard = "WASMER_H";

    // C bindings.
    {
        // Generate the bindings in the `OUT_DIR`.
        out_header_file.set_extension("h");

        // Build and generate the header file.
        exclude_items_from_wasm_c_api(new_builder(Language::C, crate_dir, guard, &pre_header))
            .generate()
            .expect("Unable to generate C bindings")
            .write_to_file(out_header_file.as_path());

        // Copy the generated bindings from `OUT_DIR` to
        // `CARGO_MANIFEST_DIR`.
        crate_header_file.set_extension("h");

        fs::copy(out_header_file.as_path(), crate_header_file.as_path())
            .expect("Unable to copy the generated C bindings");
    }

    // C++ bindings.
    {
        // Generate the bindings in the `OUT_DIR`.
        out_header_file.set_extension("hh");

        // Build and generate the header file.
        exclude_items_from_wasm_c_api(new_builder(Language::Cxx, crate_dir, guard, &pre_header))
            .generate()
            .expect("Unable to generate C++ bindings")
            .write_to_file(out_header_file.as_path());

        // Copy the generated bindings from `OUT_DIR` to
        // `CARGO_MANIFEST_DIR`.
        crate_header_file.set_extension("hh");

        fs::copy(out_header_file, crate_header_file)
            .expect("Unable to copy the generated C++ bindings");
    }
}

fn add_wasmer_version(pre_header: &mut String) {
    pre_header.push_str(&format!(
        r#"
// This file corresponds to the following Wasmer version.
#define WASMER_VERSION "{full}"
#define WASMER_VERSION_MAJOR {major}
#define WASMER_VERSION_MINOR {minor}
#define WASMER_VERSION_PATCH {patch}
#define WASMER_VERSION_PRE "{pre}"
"#,
        full = env!("CARGO_PKG_VERSION"),
        major = env!("CARGO_PKG_VERSION_MAJOR"),
        minor = env!("CARGO_PKG_VERSION_MINOR"),
        patch = env!("CARGO_PKG_VERSION_PATCH"),
        pre = env!("CARGO_PKG_VERSION_PRE"),
    ));
}

/// Create a fresh new `Builder`, already pre-configured.
fn new_builder(language: Language, crate_dir: &str, include_guard: &str, header: &str) -> Builder {
    Builder::new()
        .with_config(cbindgen::Config {
            sort_by: cbindgen::SortKey::Name,
            ..cbindgen::Config::default()
        })
        .with_language(language)
        .with_crate(crate_dir)
        .with_include_guard(include_guard)
        .with_header(header)
        .with_documentation(true)
        .with_define("target_family", "windows", "_WIN32")
        .with_define("target_arch", "x86_64", "ARCH_X86_64")
        .with_define("feature", "jit", JIT_FEATURE_AS_C_DEFINE)
        .with_define("feature", "compiler", COMPILER_FEATURE_AS_C_DEFINE)
        .with_define("feature", "wasi", WASI_FEATURE_AS_C_DEFINE)
        .with_define("feature", "emscripten", EMSCRIPTEN_FEATURE_AS_C_DEFINE)
}

/// Exclude types and functions from the `deprecated` API.
fn exclude_items_from_deprecated(builder: Builder) -> Builder {
    builder
        // List of all functions to exclude given by:
        //
        // `rg 'extern "C" fn' deprecated/` builder = builder
        .exclude_item("wasmer_compile")
        .exclude_item("wasmer_emscripten_call_main")
        .exclude_item("wasmer_emscripten_destroy_globals")
        .exclude_item("wasmer_emscripten_generate_import_object")
        .exclude_item("wasmer_emscripten_get_globals")
        .exclude_item("wasmer_emscripten_set_up")
        .exclude_item("wasmer_export_descriptor_kind")
        .exclude_item("wasmer_export_descriptor_name")
        .exclude_item("wasmer_export_descriptors")
        .exclude_item("wasmer_export_descriptors_destroy")
        .exclude_item("wasmer_export_descriptors_get")
        .exclude_item("wasmer_export_descriptors_len")
        .exclude_item("wasmer_export_func_call")
        .exclude_item("wasmer_export_func_params")
        .exclude_item("wasmer_export_func_params_arity")
        .exclude_item("wasmer_export_func_returns")
        .exclude_item("wasmer_export_func_returns_arity")
        .exclude_item("wasmer_export_kind")
        .exclude_item("wasmer_export_name")
        .exclude_item("wasmer_export_to_func")
        .exclude_item("wasmer_export_to_memory")
        .exclude_item("wasmer_exports_destroy")
        .exclude_item("wasmer_exports_get")
        .exclude_item("wasmer_exports_len")
        .exclude_item("wasmer_global_destroy")
        .exclude_item("wasmer_global_get")
        .exclude_item("wasmer_global_get_descriptor")
        .exclude_item("wasmer_global_new")
        .exclude_item("wasmer_global_set")
        .exclude_item("wasmer_import_descriptor_kind")
        .exclude_item("wasmer_import_descriptor_module_name")
        .exclude_item("wasmer_import_descriptor_name")
        .exclude_item("wasmer_import_descriptors")
        .exclude_item("wasmer_import_descriptors_destroy")
        .exclude_item("wasmer_import_descriptors_get")
        .exclude_item("wasmer_import_descriptors_len")
        .exclude_item("wasmer_import_func_destroy")
        .exclude_item("wasmer_import_func_new")
        .exclude_item("wasmer_import_func_params")
        .exclude_item("wasmer_import_func_params_arity")
        .exclude_item("wasmer_import_func_returns")
        .exclude_item("wasmer_import_func_returns_arity")
        .exclude_item("wasmer_import_object_destroy")
        .exclude_item("wasmer_import_object_extend")
        .exclude_item("wasmer_import_object_get_import")
        .exclude_item("wasmer_import_object_imports_destroy")
        .exclude_item("wasmer_import_object_iter_at_end")
        .exclude_item("wasmer_import_object_iter_destroy")
        .exclude_item("wasmer_import_object_iter_next")
        .exclude_item("wasmer_import_object_iterate_functions")
        .exclude_item("wasmer_import_object_new")
        .exclude_item("wasmer_import_object_new")
        .exclude_item("wasmer_instance_call")
        .exclude_item("wasmer_instance_context_data_get")
        .exclude_item("wasmer_instance_context_data_set")
        .exclude_item("wasmer_instance_context_get")
        .exclude_item("wasmer_instance_context_memory")
        .exclude_item("wasmer_instance_destroy")
        .exclude_item("wasmer_instance_exports")
        .exclude_item("wasmer_instantiate")
        .exclude_item("wasmer_memory_data")
        .exclude_item("wasmer_memory_data_length")
        .exclude_item("wasmer_memory_destroy")
        .exclude_item("wasmer_memory_grow")
        .exclude_item("wasmer_memory_length")
        .exclude_item("wasmer_memory_new")
        .exclude_item("wasmer_module_deserialize")
        .exclude_item("wasmer_module_destroy")
        .exclude_item("wasmer_module_import_instantiate")
        .exclude_item("wasmer_module_instantiate")
        .exclude_item("wasmer_module_serialize")
        .exclude_item("wasmer_serialized_module_bytes")
        .exclude_item("wasmer_serialized_module_destroy")
        .exclude_item("wasmer_serialized_module_from_bytes")
        .exclude_item("wasmer_table_destroy")
        .exclude_item("wasmer_table_grow")
        .exclude_item("wasmer_table_length")
        .exclude_item("wasmer_table_new")
        .exclude_item("wasmer_trampoline_buffer_builder_add_callinfo_trampoline")
        .exclude_item("wasmer_trampoline_buffer_builder_add_context_trampoline")
        .exclude_item("wasmer_trampoline_buffer_builder_build")
        .exclude_item("wasmer_trampoline_buffer_builder_new")
        .exclude_item("wasmer_trampoline_buffer_destroy")
        .exclude_item("wasmer_trampoline_buffer_get_trampoline")
        .exclude_item("wasmer_trampoline_get_context")
        .exclude_item("wasmer_trap")
        .exclude_item("wasmer_validate")
        .exclude_item("wasmer_wasi_generate_default_import_object")
        .exclude_item("wasmer_wasi_generate_import_object")
        .exclude_item("wasmer_wasi_generate_import_object_for_version")
        .exclude_item("wasmer_wasi_get_version")
        // List of all structs and enums to exclude given by:
        //
        // `rg 'pub (enum|struct|union)' deprecated/`
        .exclude_item("NamedExportDescriptors(Vec<NamedExportDescriptor>)")
        .exclude_item("NamedImportDescriptors(Vec<ImportType>)")
        .exclude_item("Version")
        .exclude_item("WasmerImportObjectIterator")
        .exclude_item("wasmer_byte_array")
        .exclude_item("wasmer_emscripten_globals_t")
        .exclude_item("wasmer_export_descriptor_t")
        .exclude_item("wasmer_export_descriptors_t")
        .exclude_item("wasmer_export_func_t")
        .exclude_item("wasmer_export_t")
        .exclude_item("wasmer_exports_t")
        .exclude_item("wasmer_global_descriptor_t")
        .exclude_item("wasmer_global_t")
        .exclude_item("wasmer_import_descriptor_t")
        .exclude_item("wasmer_import_descriptors_t")
        .exclude_item("wasmer_import_export_kind")
        .exclude_item("wasmer_import_func_t")
        .exclude_item("wasmer_import_object_iter_t")
        .exclude_item("wasmer_import_object_t")
        .exclude_item("wasmer_import_t")
        .exclude_item("wasmer_instance_context_t")
        .exclude_item("wasmer_instance_t")
        .exclude_item("wasmer_limit_option_t")
        .exclude_item("wasmer_limits_t")
        .exclude_item("wasmer_memory_t")
        .exclude_item("wasmer_module_t")
        .exclude_item("wasmer_result_t")
        .exclude_item("wasmer_serialized_module_t")
        .exclude_item("wasmer_table_t")
        .exclude_item("wasmer_trampoline_buffer_builder_t")
        .exclude_item("wasmer_trampoline_buffer_t")
        .exclude_item("wasmer_trampoline_callable_t")
        .exclude_item("wasmer_value_t")
        .exclude_item("wasmer_value_tag")
        .exclude_item("wasmer_wasi_map_dir_entry_t")
}

/// Excludes non-standard types and functions of the `wasm_c_api` API.
///
/// All items defined in `wasm.h` are ignored by cbindgen already
/// based on `cbindgen:ignore` instructions, because we don't want
/// duplications. We must exclude extra non-standard items, like the
/// ones from the WASI API.
fn exclude_items_from_wasm_c_api(builder: Builder) -> Builder {
    builder
        .exclude_item("wasi_config_arg")
        .exclude_item("wasi_config_env")
        .exclude_item("wasi_config_mapdir")
        .exclude_item("wasi_config_preopen_dir")
        .exclude_item("wasi_config_inherit_stderr")
        .exclude_item("wasi_config_inherit_stdin")
        .exclude_item("wasi_config_inherit_stdout")
        .exclude_item("wasi_config_new")
        .exclude_item("wasi_config_t")
        .exclude_item("wasi_env_delete")
        .exclude_item("wasi_env_new")
        .exclude_item("wasi_env_read_stderr")
        .exclude_item("wasi_env_read_stdout")
        .exclude_item("wasi_env_set_instance")
        .exclude_item("wasi_env_set_memory")
        .exclude_item("wasi_env_t")
        .exclude_item("wasi_get_imports")
        .exclude_item("wasi_get_imports_inner")
        .exclude_item("wasi_get_start_function")
        .exclude_item("wasi_get_wasi_version")
        .exclude_item("wasi_version_t")
        .exclude_item("wasm_config_set_compiler")
        .exclude_item("wasm_config_set_engine")
        .exclude_item("wasm_module_name")
        .exclude_item("wasm_module_set_name")
        .exclude_item("wasmer_compiler_t")
        .exclude_item("wasmer_engine_t")
        .exclude_item("wat2wasm")
}

fn build_inline_c_env_vars() {
    use std::ffi::OsStr;

    // We start from `OUT_DIR` because `cargo publish` uses a different directory
    // so traversing from `CARGO_MANIFEST_DIR` is less reliable.
    let mut shared_object_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    assert_eq!(shared_object_dir.file_name(), Some(OsStr::new("out")));
    shared_object_dir.pop();

    assert!(shared_object_dir
        .file_name()
        .as_ref()
        .unwrap()
        .to_string_lossy()
        .to_string()
        .starts_with("wasmer-c-api"));
    shared_object_dir.pop();

    assert_eq!(shared_object_dir.file_name(), Some(OsStr::new("build")));
    shared_object_dir.pop();
    shared_object_dir.pop(); // "debug" or "release"

    // We either find `target` or the target triple if cross-compiling.
    if shared_object_dir.file_name() != Some(OsStr::new("target")) {
        let target = env::var("TARGET").unwrap();
        assert_eq!(shared_object_dir.file_name(), Some(OsStr::new(&target)));
    }
    shared_object_dir.push(env::var("PROFILE").unwrap());

    let shared_object_dir = shared_object_dir.as_path().to_string_lossy();
    let include_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // The following options mean:
    //
    // * `-I`, add `include_dir` to include search path,
    // * `-L`, add `shared_object_dir` to library search path,
    // * `-D_DEBUG`, enable debug mode to enable `assert.h`.
    println!(
        "cargo:rustc-env=INLINE_C_RS_CFLAGS=-I{I} -L{L} -D_DEBUG",
        I = include_dir,
        L = shared_object_dir.clone(),
    );

    println!(
        "cargo:rustc-env=INLINE_C_RS_LDFLAGS={shared_object_dir}/{lib}",
        shared_object_dir = shared_object_dir,
        lib = if cfg!(target_os = "windows") {
            "wasmer_c_api.dll".to_string()
        } else if cfg!(target_os = "macos") {
            "libwasmer_c_api.dylib".to_string()
        } else {
            "libwasmer_c_api.so".to_string()
        }
    );
}

use crate::{instance::Exports, new};

pub use new::wasmer::{namespace, ImportObject, ImportObjectIterator, LikeNamespace};

pub struct Namespace {
    exports: Exports,
}

impl Namespace {
    pub fn new() -> Self {
        Self {
            exports: Exports::new(),
        }
    }

    pub fn insert<N, V>(&mut self, name: N, value: V)
    where
        N: Into<String>,
        V: Into<new::wasmer::Extern> + 'static,
    {
        self.exports.new_exports.insert(name, value);
    }

    pub fn contains_key<N>(&mut self, name: N) -> bool
    where
        N: Into<String>,
    {
        self.exports.new_exports.contains(name)
    }
}

impl LikeNamespace for Namespace {
    fn get_namespace_export(&self, name: &str) -> Option<new::wasmer::Export> {
        self.exports.new_exports.get_namespace_export(name)
    }

    fn get_namespace_exports(&self) -> Vec<(String, new::wasmer::Export)> {
        self.exports.new_exports.get_namespace_exports()
    }
}

#[deprecated(
    since = "__NEXT_VERSION__",
    note = "Please use the `Exportable` trait instead."
)]
pub trait IsExport {}

/// Generate an `ImportObject` easily with the `imports!` macro.
///
/// # Usage
///
/// ```
/// # use wasmer_runtime_core::{imports, func, vm::Ctx};
///
/// let import_object = imports! {
///     "env" => {
///         "foo" => func!(foo)
///     },
/// };
///
/// fn foo(_: &mut Ctx, n: i32) -> i32 {
///     n
/// }
/// ```
#[macro_export]
macro_rules! imports {
    ( $( $namespace_name:expr => $namespace:tt ),* $(,)? ) => {
        {
            let mut import_object = $crate::import::ImportObject::new();

            $({
                let namespace = $crate::import_namespace!($namespace);

                import_object.register($namespace_name, namespace);
            })*

            import_object
        }
    };

    ($state_creator:expr, $( $namespace_name:expr => $namespace:tt ),* $(,)? ) => {
        {
            compile_error!("State creation in the ImportObject is no longer supported.\nYou can achieve something similar by using Function environments in the Wasmer 1.0 API.");
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! import_namespace {
    ( { $( $import_name:expr => $import_item:expr ),* $(,)? } ) => {
        {
            let mut namespace = $crate::import::Namespace::new();

            $(
                namespace.insert($import_name, $import_item);
            )*

            namespace
        }
    };

    ( $namespace:ident ) => {
        $namespace
    };
}

#[cfg(test)]
mod test {
    use crate::{func, vm};

    fn func(_: &mut vm::Ctx, arg: i32) -> i32 {
        arg + 1
    }

    #[test]
    fn imports_macro_allows_trailing_comma_and_none() {
        let _ = imports! {
            "env" => {
                "func" => func!(func),
            },
        };
        let _ = imports! {
            "env" => {
                "func" => func!(func),
            }
        };
        let _ = imports! {
            "env" => {
                "func" => func!(func),
            },
            "abc" => {
                "def" => func!(func),
            }
        };
        let _ = imports! {
            "env" => {
                "func" => func!(func)
            },
        };
        let _ = imports! {
            "env" => {
                "func" => func!(func)
            }
        };
        let _ = imports! {
            "env" => {
                "func1" => func!(func),
                "func2" => func!(func)
            }
        };
        let _ = imports! {
            "env" => {
                "func1" => func!(func),
                "func2" => func!(func),
            }
        };
    }
}

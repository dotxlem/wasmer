#[doc(hidden)]
#[macro_export]
macro_rules! wasm_declare_vec_inner {
    ($name:ident) => {
        paste::paste! {
            #[doc = "Creates an empty vector of [`wasm_" $name "_t`].

# Example

```rust
# use inline_c::assert_c;
# fn main() {
#    (assert_c! {
# #include \"tests/wasmer_wasm.h\"
#
int main() {
    // Creates an empty vector of `wasm_" $name "_t`.
    wasm_" $name "_vec_t vector;
    wasm_" $name "_vec_new_empty(&vector);

    // Check that it is empty.
    assert(vector.size == 0);

    // Free it.
    wasm_" $name "_vec_delete(&vector);
}
#    })
#    .success();
# }
```"]
            #[no_mangle]
            pub unsafe extern "C" fn [<wasm_ $name _vec_new_empty>](out: *mut [<wasm_ $name _vec_t>]) {
                // TODO: actually implement this
                [<wasm_ $name _vec_new_uninitialized>](out, 0);
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! wasm_declare_vec {
    ($name:ident) => {
        paste::paste! {
            #[doc = "Represents a vector of `wasm_" $name "_t`.

Read the documentation of [`wasm_" $name "_t`] to see more concrete examples.

# Example

```rust
# use inline_c::assert_c;
# fn main() {
#    (assert_c! {
# #include \"tests/wasmer_wasm.h\"
#
int main() {
    // Create a vector of 2 `wasm_" $name "_t`.
    wasm_" $name "_t x;
    wasm_" $name "_t y;
    wasm_" $name "_t* items[2] = {&x, &y};

    wasm_" $name "_vec_t vector;
    wasm_" $name "_vec_new(&vector, 2, (wasm_" $name "_t*) items);

    // Check that it contains 2 items.
    assert(vector.size == 2);

    // Free it.
    wasm_" $name "_vec_delete(&vector);
}
#    })
#    .success();
# }
```"]
            #[derive(Debug)]
            #[repr(C)]
            pub struct [<wasm_ $name _vec_t>] {
                pub size: usize,
                pub data: *mut [<wasm_ $name _t>],
            }

            impl Clone for [<wasm_ $name _vec_t>] {
                fn clone(&self) -> Self {
                    if self.data.is_null() {
                        return Self {
                            size: self.size,
                            data: ::std::ptr::null_mut(),
                        };
                    }
                    let data =
                        unsafe {
                            let vec = Vec::from_raw_parts(self.data, self.size, self.size);
                            let mut vec_copy = vec.clone().into_boxed_slice();
                            let new_ptr = vec_copy.as_mut_ptr();

                            ::std::mem::forget(vec);
                            ::std::mem::forget(vec_copy);

                            new_ptr
                        };

                    Self {
                        size: self.size,
                        data,
                    }
                }
            }

            impl<'a> From<Vec<[<wasm_ $name _t>]>> for [<wasm_ $name _vec_t>] {
                fn from(mut vec: Vec<[<wasm_ $name _t>]>) -> Self {
                    vec.shrink_to_fit();

                    let length = vec.len();
                    let pointer = vec.as_mut_ptr();

                    ::std::mem::forget(vec);

                    Self {
                        size: length,
                        data: pointer,
                    }
                }
            }

            impl<'a, T: Into<[<wasm_ $name _t>]> + Clone> From<&'a [T]> for [<wasm_ $name _vec_t>] {
                fn from(other: &'a [T]) -> Self {
                    let size = other.len();
                    let mut copied_data = other
                        .iter()
                        .cloned()
                        .map(Into::into)
                        .collect::<Vec<[<wasm_ $name _t>]>>()
                        .into_boxed_slice();
                    let data = copied_data.as_mut_ptr();
                    ::std::mem::forget(copied_data);

                    Self {
                        size,
                        data,
                    }
                }
            }

            impl [<wasm_ $name _vec_t>] {
                pub unsafe fn into_slice(&self) -> Option<&[[<wasm_ $name _t>]]>{
                    if self.is_uninitialized() {
                        return None;
                    }

                    Some(::std::slice::from_raw_parts(self.data, self.size))
                }

                pub unsafe fn into_slice_mut(&mut self) -> Option<&mut [[<wasm_ $name _t>]]>{
                    if self.is_uninitialized() {
                        return None;
                    }

                    Some(::std::slice::from_raw_parts_mut(self.data, self.size))
                }

                pub fn is_uninitialized(&self) -> bool {
                    self.data.is_null()
                }
            }

            // TODO: investigate possible memory leak on `init` (owned pointer)
            #[doc = "Creates a new vector of [`wasm_" $name "_t`].

# Example

See the [`wasm_" $name "_vec_t`] type to get an example."]
            #[no_mangle]
            pub unsafe extern "C" fn [<wasm_ $name _vec_new>](out: *mut [<wasm_ $name _vec_t>], length: usize, init: *mut [<wasm_ $name _t>]) {
                let mut bytes: Vec<[<wasm_ $name _t>]> = Vec::with_capacity(length);

                for i in 0..length {
                    bytes.push(::std::ptr::read(init.add(i)));
                }

                let pointer = bytes.as_mut_ptr();
                debug_assert!(bytes.len() == bytes.capacity());

                (*out).data = pointer;
                (*out).size = length;
                ::std::mem::forget(bytes);
            }

            #[doc = "Creates a new uninitialized vector of [`wasm_" $name "_t`].

# Example

```rust
# use inline_c::assert_c;
# fn main() {
#    (assert_c! {
# #include \"tests/wasmer_wasm.h\"
#
int main() {
    // Creates an empty vector of `wasm_" $name "_t`.
    wasm_" $name "_vec_t vector;
    wasm_" $name "_vec_new_uninitialized(&vector, 3);

    // Check that it contains 3 items.
    assert(vector.size == 3);

    // Free it.
    wasm_" $name "_vec_delete(&vector);
}
#    })
#    .success();
# }
```"]
            #[no_mangle]
            pub unsafe extern "C" fn [<wasm_ $name _vec_new_uninitialized>](out: *mut [<wasm_ $name _vec_t>], length: usize) {
                let mut bytes: Vec<[<wasm_ $name _t>]> = Vec::with_capacity(length);
                let pointer = bytes.as_mut_ptr();

                (*out).data = pointer;
                (*out).size = length;

                ::std::mem::forget(bytes);
            }

            #[doc = "Performs a deep copy of a vector of [`wasm_" $name "_t`]."]
            #[no_mangle]
            pub unsafe extern "C" fn [<wasm_ $name _vec_copy>](
                out_ptr: &mut [<wasm_ $name _vec_t>],
                in_ptr: & [<wasm _$name _vec_t>])
            {
                *out_ptr = in_ptr.clone();
            }

            #[doc = "Deletes a vector of [`wasm_" $name "_t`].

# Example

See the [`wasm_" $name "_vec_t`] type to get an example."]
            #[no_mangle]
            pub unsafe extern "C" fn [<wasm_ $name _vec_delete>](ptr: Option<&mut [<wasm_ $name _vec_t>]>) {
                if let Some(vec) = ptr {
                    if !vec.data.is_null() {
                        Vec::from_raw_parts(vec.data, vec.size, vec.size);
                        vec.data = ::std::ptr::null_mut();
                        vec.size = 0;
                    }
                }
            }
        }

        wasm_declare_vec_inner!($name);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! wasm_declare_boxed_vec {
    ($name:ident) => {
        paste::paste! {
            #[doc = "Represents a vector of `wasm_" $name "_t`.

Read the documentation of [`wasm_" $name "_t`] to see more concrete examples."]
            #[derive(Debug)]
            #[repr(C)]
            pub struct [<wasm_ $name _vec_t>] {
                pub size: usize,
                pub data: *mut *mut [<wasm_ $name _t>],
            }

            impl Clone for [<wasm_ $name _vec_t>] {
                fn clone(&self) -> Self {
                    if self.data.is_null() {
                        return Self {
                            size: self.size,
                            data: ::std::ptr::null_mut(),
                        };
                    }
                    let data =
                        unsafe {
                            let data: *mut Option<Box<[<wasm_ $name _t>]>> = self.data as _;
                            let vec = Vec::from_raw_parts(data, self.size, self.size);
                            let mut vec_copy = vec.clone().into_boxed_slice();
                            let new_ptr = vec_copy.as_mut_ptr() as *mut *mut [<wasm_ $name _t>];

                            ::std::mem::forget(vec);
                            ::std::mem::forget(vec_copy);

                            new_ptr
                        };

                    Self {
                        size: self.size,
                        data,
                    }
                }
            }

            impl<'a> From<Vec<Box<[<wasm_ $name _t>]>>> for [<wasm_ $name _vec_t>] {
                fn from(other: Vec<Box<[<wasm_ $name _t>]>>) -> Self {
                    let boxed_slice: Box<[Box<[<wasm_ $name _t>]>]> = other.into_boxed_slice();
                    let mut boxed_slice: Box<[*mut [<wasm_ $name _t>]]> = unsafe { ::std::mem::transmute(boxed_slice) };
                    let size = boxed_slice.len();
                    let data = boxed_slice.as_mut_ptr();

                    ::std::mem::forget(boxed_slice);
                    Self {
                        size,
                        data,
                    }
                }
            }

            impl<'a, T: Into<[<wasm_ $name _t>]> + Clone> From<&'a [T]> for [<wasm_ $name _vec_t>] {
                fn from(other: &'a [T]) -> Self {
                    let size = other.len();
                    let mut copied_data = other
                        .iter()
                        .cloned()
                        .map(Into::into)
                        .map(Box::new)
                        .map(Box::into_raw)
                        .collect::<Vec<*mut [<wasm_ $name _t>]>>()
                        .into_boxed_slice();
                    let data = copied_data.as_mut_ptr();
                    ::std::mem::forget(copied_data);

                    Self {
                        size,
                        data,
                    }
                }
            }

            // TODO: do this properly
            impl [<wasm_ $name _vec_t>] {
                pub unsafe fn into_slice(&self) -> Option<&[Box<[<wasm_ $name _t>]>]>{
                    if self.data.is_null() {
                        return None;
                    }

                    let slice: &[*mut [<wasm_ $name _t>]] = ::std::slice::from_raw_parts(self.data, self.size);
                    let slice: &[Box<[<wasm_ $name _t>]>] = ::std::mem::transmute(slice);
                    Some(slice)
                }
            }

            // TODO: investigate possible memory leak on `init` (owned pointer)
            #[doc = "Creates a new vector of [`wasm_" $name "_t`]."]
            #[no_mangle]
            pub unsafe extern "C" fn [<wasm_ $name _vec_new>](out: *mut [<wasm_ $name _vec_t>], length: usize, init: *const *mut [<wasm_ $name _t>]) {
                let mut bytes: Vec<*mut [<wasm_ $name _t>]> = Vec::with_capacity(length);

                for i in 0..length {
                    bytes.push(*init.add(i));
                }

                let mut boxed_vec = bytes.into_boxed_slice();
                let pointer = boxed_vec.as_mut_ptr();

                (*out).data = pointer;
                (*out).size = length;

                ::std::mem::forget(boxed_vec);
            }

            #[doc = "Creates a new uninitialized vector of [`wasm_" $name "_t`].

# Example

```rust
# use inline_c::assert_c;
# fn main() {
#    (assert_c! {
# #include \"tests/wasmer_wasm.h\"
#
int main() {
    // Creates an empty vector of `wasm_" $name "_t`.
    wasm_" $name "_vec_t vector;
    wasm_" $name "_vec_new_uninitialized(&vector, 3);

    // Check that it contains 3 items.
    assert(vector.size == 3);

    // Free it.
    wasm_" $name "_vec_delete(&vector);
}
#    })
#    .success();
# }
```"]
            #[no_mangle]
            pub unsafe extern "C" fn [<wasm_ $name _vec_new_uninitialized>](out: *mut [<wasm_ $name _vec_t>], length: usize) {
                let mut bytes: Vec<*mut [<wasm_ $name _t>]> = vec![::std::ptr::null_mut(); length];
                let pointer = bytes.as_mut_ptr();

                (*out).data = pointer;
                (*out).size = length;

                ::std::mem::forget(bytes);
            }

            #[doc = "Performs a deep copy of a vector of [`wasm_" $name "_t`]."]
            #[no_mangle]
            pub unsafe extern "C" fn [<wasm_ $name _vec_copy>](
                out_ptr: &mut [<wasm_ $name _vec_t>],
                in_ptr: & [<wasm_ $name _vec_t>])
            {
                *out_ptr = in_ptr.clone();
            }

            #[doc = "Deletes a vector of [`wasm_" $name "_t`].

# Example

See the [`wasm_" $name "_vec_t`] type to get an example."]
            #[no_mangle]
            pub unsafe extern "C" fn [<wasm_ $name _vec_delete>](ptr: Option<&mut [<wasm_ $name _vec_t>]>) {
               if let Some(vec) = ptr {
                    if !vec.data.is_null() {
                        let data = vec.data as *mut Option<Box<[<wasm_ $name _t>]>>;
                        let _data: Vec<Option<Box<[<wasm_ $name _t>]>>> = Vec::from_raw_parts(data, vec.size, vec.size);

                        vec.data = ::std::ptr::null_mut();
                        vec.size = 0;
                    }
                }
            }
        }

        wasm_declare_vec_inner!($name);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! wasm_declare_ref_base {
    ($name:ident) => {
        wasm_declare_own!($name);

        paste::paste! {
            #[no_mangle]
            pub extern "C" fn [<wasm_ $name _copy>](_arg: *const [<wasm_ $name _t>]) -> *mut [<wasm_ $name _t>] {
                todo!("in generated declare ref base");
                //ptr::null_mut()
            }

            // TODO: finish this...
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! wasm_declare_own {
    ($name:ident) => {
        paste::paste! {
            #[repr(C)]
            pub struct [<wasm_ $name _t>] {}

            #[no_mangle]
            pub extern "C" fn [<wasm_ $name _delete>](_arg: *mut [<wasm_ $name _t>]) {
                todo!("in generated delete")
            }
        }
    };
}

#[macro_export]
macro_rules! c_try {
    ($expr:expr; otherwise $return:expr) => {{
        let res: Result<_, _> = $expr;
        match res {
            Ok(val) => val,
            Err(err) => {
                crate::error::update_last_error(err);
                return $return;
            }
        }
    }};
    ($expr:expr) => {{
        c_try!($expr; otherwise None)
    }};
    ($expr:expr, $e:expr) => {{
        let opt: Option<_> = $expr;
        c_try!(opt.ok_or_else(|| $e))
    }};
}

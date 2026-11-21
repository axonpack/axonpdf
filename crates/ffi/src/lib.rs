//! The C ABI over the engine: the one binding every platform calls (WebAssembly through
//! Emscripten, React Native through a TurboModule, Flutter through `dart:ffi`).
//!
//! Every function takes the handle from `axonpdf_new`. Functions that can fail return 0 on success
//! and -1 on failure; `axonpdf_last_error` then gives the message. Byte buffers handed to the
//! engine (fonts, images) must come from `axonpdf_alloc`, and the engine takes ownership of them.

use std::ffi::{CStr, CString, c_char, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;

use axonpdf::Engine;

mod callbacks;

pub use callbacks::*;

/// Receives each chunk of the PDF in order. The bytes are only valid during the call.
pub type OutputCallback = extern "C" fn(user: *mut c_void, bytes: *const u8, len: usize);

pub struct Handle {
    engine: Engine,
    error: Option<CString>,
}

#[unsafe(no_mangle)]
pub extern "C" fn axonpdf_new() -> *mut Handle {
    Box::into_raw(Box::new(Handle {
        engine: Engine::new(),
        error: None,
    }))
}

/// # Safety
/// `handle` comes from `axonpdf_new` and is not used again.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn axonpdf_free(handle: *mut Handle) {
    if !handle.is_null() {
        drop(unsafe { Box::from_raw(handle) });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn axonpdf_alloc(len: usize) -> *mut u8 {
    Box::into_raw(vec![0u8; len].into_boxed_slice()).cast()
}

/// # Safety
/// `ptr` and `len` come from one `axonpdf_alloc` call whose buffer the engine did not take.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn axonpdf_dealloc(ptr: *mut u8, len: usize) {
    drop(unsafe { take(ptr, len) });
}

/// # Safety
/// `family` is `family_len` bytes of UTF-8; `bytes` comes from `axonpdf_alloc(len)` and is taken.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn axonpdf_add_font(
    handle: *mut Handle,
    family: *const u8,
    family_len: usize,
    weight: u16,
    italic: bool,
    bytes: *mut u8,
    len: usize,
) -> i32 {
    let bytes = unsafe { take(bytes, len) };
    let family = unsafe { text(family, family_len) };
    run(handle, |engine| {
        engine
            .add_font(family?, weight, italic, bytes)
            .map_err(|e| e.to_string())
    })
}

/// # Safety
/// `bytes` comes from `axonpdf_alloc(len)` and is taken.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn axonpdf_add_image(
    handle: *mut Handle,
    id: u32,
    bytes: *mut u8,
    len: usize,
) -> i32 {
    let bytes = unsafe { take(bytes, len) };
    run(handle, |engine| {
        engine.add_image(id, bytes).map_err(|e| e.to_string())
    })
}

/// Renders the tree and hands each chunk to `output` as it is written.
///
/// # Safety
/// `json` is `json_len` bytes of UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn axonpdf_render(
    handle: *mut Handle,
    json: *const u8,
    json_len: usize,
    output: OutputCallback,
    user: *mut c_void,
) -> i32 {
    let json = unsafe { text(json, json_len) };
    run(handle, |engine| {
        let mut out = |chunk: &[u8]| output(user, chunk.as_ptr(), chunk.len());
        engine.render(json?, &mut out).map_err(|e| e.to_string())
    })
}

/// The message of the last failed call, or null. Valid until the next call on this handle.
///
/// # Safety
/// `handle` comes from `axonpdf_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn axonpdf_last_error(handle: *const Handle) -> *const c_char {
    unsafe { &*handle }
        .error
        .as_ref()
        .map_or(ptr::null(), |e| e.as_ptr())
}

pub(crate) fn run(handle: *mut Handle, f: impl FnOnce(&mut Engine) -> Result<(), String>) -> i32 {
    let handle = unsafe { &mut *handle };
    let result = catch_unwind(AssertUnwindSafe(|| f(&mut handle.engine)))
        .unwrap_or_else(|panic| Err(panic_message(&panic)));
    handle.error = result
        .err()
        .map(|e| CString::new(e.replace('\0', " ")).unwrap());
    if handle.error.is_some() { -1 } else { 0 }
}

fn panic_message(panic: &Box<dyn std::any::Any + Send>) -> String {
    let message = panic
        .downcast_ref::<&str>()
        .copied()
        .or(panic.downcast_ref::<String>().map(String::as_str));
    format!("engine panicked: {}", message.unwrap_or("unknown"))
}

/// Copies a NUL-terminated string from `axonpdf_alloc` and frees it; None for null.
pub(crate) fn take_string(ptr: *mut c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let bytes = unsafe { CStr::from_ptr(ptr) }.to_bytes();
    let (string, len) = (String::from_utf8_lossy(bytes).into_owned(), bytes.len());
    unsafe { axonpdf_dealloc(ptr.cast(), len + 1) };
    Some(string)
}

unsafe fn take(ptr: *mut u8, len: usize) -> Vec<u8> {
    unsafe { Box::from_raw(ptr::slice_from_raw_parts_mut(ptr, len)) }.into_vec()
}

pub(crate) unsafe fn text<'a>(ptr: *const u8, len: usize) -> Result<&'a str, String> {
    let bytes = if len == 0 {
        &[][..]
    } else {
        unsafe { std::slice::from_raw_parts(ptr, len) }
    };
    std::str::from_utf8(bytes).map_err(|e| e.to_string())
}

//! The C ABI over the engine: the one binding every platform calls (WebAssembly through
//! Emscripten, React Native through a TurboModule, Flutter through `dart:ffi`).
//!
//! Every function takes the handle from `axonpdf_new`. Functions that can fail return 0 on success
//! and -1 on failure; `axonpdf_last_error` then gives the message. Byte buffers handed to the
//! engine (fonts, images) must come from `axonpdf_alloc`, and the engine takes ownership of them.

use std::ffi::{CStr, CString, c_char, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;

use axonpdf::text::hyphen;
use axonpdf::{Engine, RenderArgs};

/// Called during layout for a node with a `render` id. -1 means "not known yet" (`undefined`).
/// Returns a NUL-terminated UTF-8 string allocated with `axonpdf_alloc` (the engine frees it), or
/// null for no output. The string is `{"children":[nodes],"readsTotal":bool}`.
pub type RenderCallback = extern "C" fn(
    user: *mut c_void,
    id: u32,
    page_number: u32,
    total_pages: i32,
    sub_page_number: i32,
    sub_page_total_pages: i32,
) -> *mut c_char;

/// Runs a Canvas `paint` function by id at the content box's size (NaN for both when the canvas
/// is only measured). Returns the painter calls, a JSON list of `[method, ...args]`, as a
/// NUL-terminated string from `axonpdf_alloc`, or null for none.
pub type PaintCallback =
    extern "C" fn(user: *mut c_void, id: u32, width: f32, height: f32) -> *mut c_char;

/// Runs a hyphenation function by id on one word (`word_len` bytes of UTF-8). Returns the parts,
/// a JSON list of strings, as a NUL-terminated string from `axonpdf_alloc`, or null to keep the
/// word whole.
pub type HyphenationCallback =
    extern "C" fn(user: *mut c_void, id: u32, word: *const u8, word_len: usize) -> *mut c_char;

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

/// # Safety
/// `user` stays valid for as long as the engine may call `callback`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn axonpdf_set_render_callback(
    handle: *mut Handle,
    callback: RenderCallback,
    user: *mut c_void,
) {
    let user = user as usize;
    run(handle, |engine| {
        engine.set_render_callback(Box::new(move |id, args: &RenderArgs| {
            let total = |n: Option<u32>| n.map_or(-1, |n| n as i32);
            let out = callback(
                user as *mut c_void,
                id,
                args.page_number,
                total(args.total_pages),
                total(args.sub_page_number),
                total(args.sub_page_total_pages),
            );
            take_string(out).unwrap_or_else(|| r#"{"children":[]}"#.to_owned())
        }));
        Ok(())
    });
}

/// # Safety
/// `user` stays valid for as long as the engine may call `callback`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn axonpdf_set_paint_callback(
    handle: *mut Handle,
    callback: PaintCallback,
    user: *mut c_void,
) {
    let user = user as usize;
    run(handle, |engine| {
        engine.set_paint_callback(Box::new(move |id, width, height| {
            take_string(callback(user as *mut c_void, id, width, height))
                .unwrap_or_else(|| "[]".to_owned())
        }));
        Ok(())
    });
}

/// # Safety
/// `user` stays valid for as long as the engine may call `callback`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn axonpdf_set_hyphenation_callback(
    handle: *mut Handle,
    callback: HyphenationCallback,
    user: *mut c_void,
) {
    let user = user as usize;
    run(handle, |engine| {
        engine.set_hyphenation_callback(Box::new(move |id, word: &str| {
            let out = callback(user as *mut c_void, id, word.as_ptr(), word.len());
            take_string(out).unwrap_or_else(|| serde_json::json!([word]).to_string())
        }));
        Ok(())
    });
}

/// The built-in split of a word (soft hyphens, else the English patterns), the second argument a
/// hyphenation function receives. Returns a JSON list of strings, NUL-terminated, from
/// `axonpdf_alloc`; free it with `axonpdf_dealloc(ptr, strlen + 1)`.
///
/// # Safety
/// `word` is `word_len` bytes of UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn axonpdf_hyphenate_builtin(
    word: *const u8,
    word_len: usize,
) -> *mut c_char {
    let parts = unsafe { text(word, word_len) }.map_or_else(|_| Vec::new(), hyphen::builtin);
    let json = serde_json::to_string(&parts).unwrap();
    let out = axonpdf_alloc(json.len() + 1);
    unsafe { ptr::copy_nonoverlapping(json.as_ptr(), out, json.len()) };
    out.cast()
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

fn run(handle: *mut Handle, f: impl FnOnce(&mut Engine) -> Result<(), String>) -> i32 {
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
fn take_string(ptr: *mut c_char) -> Option<String> {
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

unsafe fn text<'a>(ptr: *const u8, len: usize) -> Result<&'a str, String> {
    let bytes = if len == 0 {
        &[][..]
    } else {
        unsafe { std::slice::from_raw_parts(ptr, len) }
    };
    std::str::from_utf8(bytes).map_err(|e| e.to_string())
}

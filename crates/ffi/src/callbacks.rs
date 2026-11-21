//! The functions the engine calls back into the caller while it renders, and how to set them.

use std::ffi::{c_char, c_void};
use std::ptr;

use axonpdf::RenderArgs;
use axonpdf::text::hyphen;

use crate::{Handle, axonpdf_alloc, run, take_string, text};

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

/// Asks a `Rows` (by its `rows` id) for its next batch. Returns
/// `{"children":[nodes],"done":bool}` as a NUL-terminated string from `axonpdf_alloc`, or null
/// when it has no more.
pub type RowsCallback = extern "C" fn(user: *mut c_void, id: u32) -> *mut c_char;

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

/// # Safety
/// `user` stays valid for as long as the engine may call `callback`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn axonpdf_set_rows_callback(
    handle: *mut Handle,
    callback: RowsCallback,
    user: *mut c_void,
) {
    let user = user as usize;
    run(handle, |engine| {
        engine.set_rows_callback(Box::new(move |id| {
            take_string(callback(user as *mut c_void, id))
                .unwrap_or_else(|| r#"{"children":[],"done":true}"#.to_owned())
        }));
        Ok(())
    });
}

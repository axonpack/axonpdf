//! Rendering: in one call, or resumable for rows that arrive later (an async source).

use std::ffi::c_void;

use axonpdf::{Engine, Step};

use crate::{Handle, owned_text, run, text};

/// Receives each chunk of the PDF in order. The bytes are only valid during the call.
pub type OutputCallback = extern "C" fn(user: *mut c_void, bytes: *const u8, len: usize);

/// Renders the tree and hands each chunk to `output` as it is written.
///
/// # Safety
/// `json` comes from `axonpdf_alloc(json_len)`, holds UTF-8, and is taken: the engine reads the
/// tree in place and frees it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn axonpdf_render(
    handle: *mut Handle,
    json: *mut u8,
    json_len: usize,
    output: OutputCallback,
    user: *mut c_void,
) -> i32 {
    let json = unsafe { owned_text(json, json_len) };
    run(handle, |engine| {
        let mut out = |chunk: &[u8]| output(user, chunk.as_ptr(), chunk.len());
        engine
            .render_owned(json?, &mut out)
            .map_err(|e| e.to_string())
    })
}

/// Starts a render that stops when it needs rows that weren't pushed yet; finished pages go to
/// `output` as they are written. Returns 0 when the file is complete, 1 when it needs the next
/// batch of the `Rows` whose id it wrote to `rows_id` (push it with `axonpdf_push_rows`), or -1
/// on failure. Used when no rows callback is set.
///
/// # Safety
/// `json` is taken as for `axonpdf_render`; `rows_id` points to a writable u32.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn axonpdf_render_begin(
    handle: *mut Handle,
    json: *mut u8,
    json_len: usize,
    output: OutputCallback,
    user: *mut c_void,
    rows_id: *mut u32,
) -> i32 {
    let json = unsafe { owned_text(json, json_len) };
    unsafe {
        step(handle, rows_id, |engine| {
            let mut out = |chunk: &[u8]| output(user, chunk.as_ptr(), chunk.len());
            engine.render_begin(json?, &mut out)
        })
    }
}

/// Pushes a batch, `{"children":[<row nodes>],"done":bool}`, to the `Rows` with this id and
/// carries the render on. Returns as `axonpdf_render_begin` does.
///
/// # Safety
/// `batch` is `batch_len` bytes of UTF-8; `rows_id` points to a writable u32.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn axonpdf_push_rows(
    handle: *mut Handle,
    id: u32,
    batch: *const u8,
    batch_len: usize,
    output: OutputCallback,
    user: *mut c_void,
    rows_id: *mut u32,
) -> i32 {
    let batch = unsafe { text(batch, batch_len) };
    unsafe {
        step(handle, rows_id, |engine| {
            let mut out = |chunk: &[u8]| output(user, chunk.as_ptr(), chunk.len());
            engine.push_rows(id, batch?, &mut out)
        })
    }
}

unsafe fn step(
    handle: *mut Handle,
    rows_id: *mut u32,
    f: impl FnOnce(&mut Engine) -> Result<Step, String>,
) -> i32 {
    let mut next = None;
    let status = run(handle, |engine| {
        next = Some(f(engine)?);
        Ok(())
    });
    match next {
        _ if status != 0 => -1,
        Some(Step::NeedRows(id)) => {
            unsafe { *rows_id = id };
            1
        }
        _ => 0,
    }
}

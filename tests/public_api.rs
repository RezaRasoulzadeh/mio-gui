// public_api.rs
use std::error::Error;

use mio_gui::{ClipboardError, RenderError, RendererInitError};

fn assert_error<T: Error + Send + Sync + 'static>() {}

#[test]
fn public_error_types_are_thread_safe() {
    assert_error::<RenderError>();
    assert_error::<RendererInitError>();
    assert_error::<ClipboardError>();
}

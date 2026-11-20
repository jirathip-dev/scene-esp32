mod buf;
mod scene;
mod status;

pub use buf::{FrameBuf, HEIGHT, WIDTH};
pub use scene::{area, draw, DEFAULT_COAT};
pub use status::status;

#[cfg(test)]
mod tests;

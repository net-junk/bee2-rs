

extern crate bee2_traits;

pub use bee2_traits::{Hasher};

#[derive(Clone)]
struct BashState {
    /// Current state.
    s: [u8; 192],
    /// Copy of s1.
    s1: [u8; 192],
    /// Size(length) of buffer.
    buff_len: usize,
    /// Current position in buffer.
    pos: usize,
    // /// ?
    // stack: &[u8],
}



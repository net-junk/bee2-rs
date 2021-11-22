extern crate bee2_traits;

use crate::consts::belt_block_encr;
use crate::consts::belt_block_decr;

#[derive(Clone)]
struct BeltState {
    /// Mode (encryption or decrryption)
    mode: u8,
    /// Secret key
    key: [u32; 8],  
    /// Size(length) of buffer
    buff_len: usize,
    /// Current position in buffer
    pos: usize,    
}

#[derive(Clone)]
struct Belt {
    state: BeltState,
}

#[derive(Clone)]
pub struct BeltECB {
    belt: Belt,
}

impl Default for Belt {
    fn default() -> Self {
        Belt {
            state: BeltState {
                buff_len: 128,
                pos: 0,
                key: [0; 8],
                mode: 0,
            },
        }
    }
}

// TODO
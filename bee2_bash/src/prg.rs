extern crate bee2_traits;

use crate::consts::bash_f0;
pub use bee2_traits::*;
use std::convert::TryInto;

#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum PrgCommands {
    BASH_PRG_NULL = 0x01, /* 000000 01 */
    BASH_PRG_KEY = 0x05,  /* 000001 01 */
    BASH_PRG_DATA = 0x09, /* 000010 01 */
    BASH_PRG_TEXT = 0x0D, /* 000011 01 */
    BASH_PRG_OUT = 0x11,  /* 000100 01 */
}

#[derive(Clone)]
struct BashPrgState {
    /// Security level.
    l: usize,
    /// Capacity level.
    d: usize,
    /// Current state.
    s: [u8; 192],
    /// Copy of s1.
    t: [u8; 192],
    /// Size(length) of buffer.
    buff_len: usize,
    /// Current position in buffer.
    pos: usize,
    // /// ?
    // stack: &[u8],
}

pub struct BashPrg {
    /// State.
    state: BashPrgState,
}

impl BashPrgState {
    fn is_key_mode(&self) -> bool {
        // (192 - buf_len) ==? (l + d * l / 2) / 8
        return 16 * (192 - self.buff_len) == self.l * (2 + self.d);
    }

    #[inline]
    pub fn bash_f(s: &mut [u8; 192]) {
        if cfg!(feature = "go-faster") {
            let x: *mut [u64; 24] = s.as_mut_ptr() as *mut [u64; 24];
            bash_f0(unsafe { x.as_mut().unwrap() });
        } else {
            let mut s1: [u64; 24] = [0; 24];
            for (dst, src) in s1.iter_mut().zip(s.chunks_exact(8)) {
                *dst = u64::from_le_bytes(src.try_into().unwrap());
            }
            bash_f0(&mut s1);
            for (src, dst) in s1.iter().zip(s.chunks_exact_mut(8)) {
                dst.clone_from_slice(&src.to_le_bytes());
            }
        }
    }
}

impl BashPrg {
    /// `Commit` command.
    /// Finish previous command and start new one using `code`.
    fn prg_commit(&mut self, code: u8) {
        self.state.s[self.state.pos] ^= code;
        self.state.s[self.state.buff_len] ^= 0x80;
        BashPrgState::bash_f(&mut self.state.s);
        self.state.pos = 0;
    }
}

impl PrgStart for BashPrg {
    fn start(l: usize, d: usize, ann_: impl AsRef<[u8]>, key_: impl AsRef<[u8]>) -> Self {
        if l != 128 && l != 192 && l != 256 {
            panic!(format!("Incorrect param of security {:}", l).to_owned());
        }

        if d != 1 && d != 2 {
            panic!(format!("Incorrect param of capacity {:}", d).to_owned());
        }

        let ann = ann_.as_ref();
        let key = key_.as_ref();

        if ann.len() % 4 != 0 || ann.len() > 60 || (key.len() != 0 && key.len() < l / 8) {
            panic!(format!("Incorrect len of annotation {:}", ann.len()).to_owned());
        }

        if key.len() % 4 != 0 || key.len() > 60 {
            panic!(format!("Incorrect len of annotation {:}", ann.len()).to_owned());
        }

        // pos <- 8 + |ann| + |key|
        let pos = 1 + ann.len() + key.len();
        let mut s = [0; 192];
        // s[0..pos) <- <|ann|/2 + |key|/32>_8 || ann || key
        s[0] = (ann.len() / 4 + key.len() / 4) as u8;
        s[1..1 + ann.len()].copy_from_slice(&ann[..]);
        s[1 + ann.len()..1 + ann.len() + key.len()].copy_from_slice(&key[..]);
        // s[1472..) <- <l / 4 + d>_{64}
        s[192 - 8] = (l / 4 + d) as u8;

        // s[pos..) <- 0
        // s[pos..].iter_mut().for_each(|x| *x = 0);

        return BashPrg {
            state: BashPrgState {
                l: l,
                d: d,
                pos: pos,
                s: s,
                t: [0; 192],
                buff_len: match key.len() {
                    0 => (192 - d * l / 4),
                    _ => (192 - l * (2 + d) / 16),
                },
            },
        };
    }
}

#[inline]
fn mem_xor(a: &mut [u8], b: &[u8]) {
    // Standart Xor
    a.iter_mut().zip(b.iter()).for_each(|(x, y)| *x ^= y);
}

#[inline]
fn mem_cpy(a: &mut [u8], b: &[u8]) {
    // Standart Cpy
    a.clone_from_slice(b);
}

impl PrgRestart for BashPrg {
    fn restart(&mut self, ann_: impl AsRef<[u8]>, key_: impl AsRef<[u8]>) {
        let ann = ann_.as_ref();
        let key = key_.as_ref();

        if ann.len() % 4 != 0 || ann.len() > 60 || (key.len() != 0 && key.len() < self.state.l / 8)
        {
            panic!(format!("Incorrect len of annotation {:}", ann.len()).to_owned());
        }

        if key.len() % 4 != 0 || key.len() > 60 {
            panic!(format!("Incorrect len of annotation {:}", ann.len()).to_owned());
        }

        if key.len() != 0 {
            // TODO: remove as u8
            self.prg_commit(PrgCommands::BASH_PRG_KEY as u8);
        } else {
            // TODO: remove as u8
            self.prg_commit(PrgCommands::BASH_PRG_NULL as u8);
        }
        // pos <- 8 + |ann| + |key|
        self.state.pos = 1 + ann.len() + key.len();
        // s[0..pos) <- s[0..pos) ^ <|ann|/2 + |key|/32>_8  || ann || key
        self.state.s[0] ^= (ann.len() * 4 + key.len() / 4) as u8;
        mem_xor(&mut self.state.s[1..1 + ann.len()], ann);
        mem_xor(&mut self.state.s[1 + ann.len()..self.state.pos], key);
    }
}

impl PrgAbsorb for BashPrg {
    fn absorb_start(&mut self) {
        // TODO: remove as u8
        self.prg_commit(PrgCommands::BASH_PRG_DATA as u8)
    }

    fn absorb_step(&mut self, buf_: impl AsRef<[u8]>) {
        let buf = buf_.as_ref();
        let mut count = buf.len();

        if count < self.state.buff_len - self.state.pos {
            mem_xor(
                &mut self.state.s[self.state.pos..self.state.pos + count],
                buf,
            );
            self.state.pos += count;
            return;
        }

        mem_xor(
            &mut self.state.s[self.state.pos..self.state.buff_len],
            &buf[0..self.state.buff_len - self.state.pos],
        );
        BashPrgState::bash_f(&mut self.state.s);

        let mut copy_size = self.state.buff_len - self.state.pos;
        count -= copy_size;

        while count >= self.state.buff_len {
            mem_xor(
                &mut self.state.s,
                &buf[copy_size..copy_size + self.state.buff_len],
            );
            copy_size += self.state.buff_len;
            count -= self.state.buff_len;
            BashPrgState::bash_f(&mut self.state.s);
        }

        self.state.pos = count;
        if count != 0 {
            mem_xor(
                &mut self.state.s[0..count],
                &buf[copy_size..copy_size + count],
            );
        }
    }

    fn absorb(&mut self, buf: impl AsRef<[u8]>) {
        self.absorb_start();
        self.absorb_step(buf);
    }
}

impl PrgSqueeze for BashPrg {
    fn squeeze_start(&mut self) {
        // TODO: remove as u8
        self.prg_commit(PrgCommands::BASH_PRG_OUT as u8)
    }

    fn squeeze_step(&mut self, buf: &mut [u8]) {
        let mut count = buf.len();

        if count < self.state.buff_len - self.state.pos {
            mem_cpy(buf, &self.state.s[self.state.pos..self.state.pos + count]);
            self.state.pos += count;
            return;
        }

        mem_cpy(
            &mut buf[..self.state.buff_len - self.state.pos],
            &self.state.s[self.state.pos..self.state.buff_len],
        );
        BashPrgState::bash_f(&mut self.state.s);

        let mut copy_size = self.state.buff_len - self.state.pos;
        count -= copy_size;

        while count >= self.state.buff_len {
            mem_cpy(
                &mut buf[copy_size..copy_size + self.state.buff_len],
                &self.state.s[0..self.state.buff_len],
            );
            copy_size += self.state.buff_len;
            count -= self.state.buff_len;
            BashPrgState::bash_f(&mut self.state.s);
        }

        self.state.pos = count;
        if count != 0 {
            mem_cpy(
                &mut buf[copy_size..copy_size + count],
                &self.state.s[0..count],
            );
        }
    }

    fn squeeze(&mut self, buf: &mut [u8]) {
        self.squeeze_start();
        self.squeeze_step(buf);
    }
}

impl PrgEncr for BashPrg {
    fn encr_start(&mut self) {
        if self.state.is_key_mode() == false {
            panic!("State not in key mode");
        }
        // TODO: remove as u8
        self.prg_commit(PrgCommands::BASH_PRG_TEXT as u8)
    }

    fn encr_step(&mut self, buf: &mut [u8]) {
        let mut count = buf.len();

        if count < self.state.buff_len - self.state.pos {
            mem_xor(
                &mut self.state.s[self.state.pos..self.state.pos + count],
                buf,
            );
            mem_cpy(buf, &self.state.s[self.state.pos..self.state.pos + count]);
            self.state.pos += count;
            return;
        }

        mem_xor(
            &mut self.state.s[self.state.pos..self.state.buff_len],
            &buf[0..self.state.buff_len - self.state.pos],
        );
        mem_cpy(
            &mut buf[..self.state.buff_len - self.state.pos],
            &self.state.s[self.state.pos..self.state.buff_len],
        );
        BashPrgState::bash_f(&mut self.state.s);

        let mut copy_size = self.state.buff_len - self.state.pos;
        count -= copy_size;

        while count >= self.state.buff_len {
            mem_xor(
                &mut self.state.s,
                &buf[copy_size..copy_size + self.state.buff_len],
            );
            mem_cpy(
                &mut buf[copy_size..copy_size + self.state.buff_len],
                &self.state.s[0..self.state.buff_len],
            );
            copy_size += self.state.buff_len;
            count -= self.state.buff_len;
            BashPrgState::bash_f(&mut self.state.s);
        }

        self.state.pos = count;
        if count != 0 {
            mem_xor(
                &mut self.state.s[0..count],
                &buf[copy_size..copy_size + count],
            );
            mem_cpy(
                &mut buf[copy_size..copy_size + count],
                &self.state.s[0..count],
            );
        }
    }

    fn encr(&mut self, buf: &mut [u8]) {
        self.encr_start();
        self.encr_step(buf);
    }
}

impl PrgDecr for BashPrg {
    fn decr_start(&mut self) {
        if self.state.is_key_mode() == false {
            panic!("State not in key mode");
        }
        // TODO: remove as u8
        self.prg_commit(PrgCommands::BASH_PRG_TEXT as u8)
    }

    fn decr_step(&mut self, buf: &mut [u8]) {
        let mut count = buf.len();

        if count < self.state.buff_len - self.state.pos {
            mem_xor(buf, &self.state.s[self.state.pos..self.state.pos + count]);
            mem_xor(
                &mut self.state.s[self.state.pos..self.state.pos + count],
                buf,
            );
            self.state.pos += count;
            return;
        }
        mem_xor(
            &mut buf[..self.state.buff_len - self.state.pos],
            &self.state.s[self.state.pos..self.state.buff_len],
        );
        mem_xor(
            &mut self.state.s[self.state.pos..self.state.buff_len],
            &buf[0..self.state.buff_len - self.state.pos],
        );
        BashPrgState::bash_f(&mut self.state.s);

        let mut copy_size = self.state.buff_len - self.state.pos;
        count -= copy_size;

        while count >= self.state.buff_len {
            mem_xor(
                &mut buf[copy_size..copy_size + self.state.buff_len],
                &self.state.s[0..self.state.buff_len],
            );
            mem_xor(
                &mut self.state.s,
                &buf[copy_size..copy_size + self.state.buff_len],
            );
            copy_size += self.state.buff_len;
            count -= self.state.buff_len;
            BashPrgState::bash_f(&mut self.state.s);
        }

        self.state.pos = count;
        if count != 0 {
            mem_xor(
                &mut buf[copy_size..copy_size + count],
                &self.state.s[0..count],
            );
            mem_xor(
                &mut self.state.s[0..count],
                &buf[copy_size..copy_size + count],
            );
        }
    }

    fn decr(&mut self, buf: &mut [u8]) {
        self.decr_start();
        self.decr_step(buf);
    }
}

impl PrgRatchet for BashPrg {
    fn ratchet(&mut self) {
        mem_cpy(&mut self.state.t, &self.state.s);
        self.prg_commit(PrgCommands::BASH_PRG_NULL as u8);
        self.state.s.iter_mut().for_each(|x| *x = 0);
    }
}

impl Prg for BashPrg {
    fn output_size() -> usize {
        return 0;
    }
}

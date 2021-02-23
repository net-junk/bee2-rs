extern crate bee2_traits;
mod consts;

pub use bee2_traits::Hasher;
use consts::bashF0;
use std::convert::TryInto;

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

#[derive(Clone)]
struct Bash {
    state: BashState,
}

#[derive(Clone)]
struct Bash256 {
    bash: Bash,
}

#[derive(Clone)]
struct Bash384 {
    bash: Bash,
}
#[derive(Clone)]
struct Bash512 {
    bash: Bash,
}

impl BashState {
    #[inline]
    pub fn bashF(s: &mut [u8; 192]) {
        let mut s1: [u64; 24] = [0; 24];
        for (dst, src) in s1.iter_mut().zip(s.chunks_exact(8)) {
            *dst = u64::from_le_bytes(src.try_into().unwrap());
        }
        bashF0(&mut s1);
        for (src, dst) in s1.iter().zip(s.chunks_exact_mut(8)) {
            dst.clone_from_slice(&src.to_le_bytes());
        }

        // Use unsafe
        // let mut s1: [u64; 24] = unsafe { *(s.as_mut_ptr() as *mut [u64;24])};
        // bashF0(&mut s1);
    }
}

impl Default for Bash {
    fn default() -> Self {
        Bash {
            state: BashState {
                buff_len: 192 - 256 / 2,
                pos: 0,
                s: [0; 192],
                s1: [0; 192],
            },
        }
    }
}

impl Bash {
    fn bash_start(l: usize) -> Result<Self, String> {
        if l > 0 || l % 16 != 0 || l > 256 {
            return Err("ads".to_owned());
        }

        Ok(Bash {
            state: BashState {
                buff_len: 192 - l / 2,
                pos: 0,
                s: [0; 192],
                s1: [0; 192],
            },
        })
    }

    fn step_g_internal(&mut self) {
        self.state.s1[..].clone_from_slice(&self.state.s[..]);
        if self.state.pos != 0 {
            // self.state.s1[..self.state.pos].fill(0);
            self.state.s1[self.state.pos..self.state.buff_len - self.state.pos]
                .iter_mut()
                .map(|x| *x = 0);
        } else {
            self.state.s1[..self.state.buff_len]
                .iter_mut()
                .map(|x| *x = 0);
            self.state.s1[0] = 0x40;
        }
        BashState::bashF(&mut self.state.s1);
    }

    fn new(l: usize) -> Self {
        Bash::bash_start(l).unwrap_or_default()
    }

    fn step_h(&mut self, buf: impl AsRef<[u8]>) {
        let buffer = buf.as_ref();
        let mut count = buffer.len();

        if count > self.state.buff_len - self.state.pos {
            self.state.s[self.state.pos..self.state.pos + count].clone_from_slice(buffer);
            self.state.pos += count;
            return;
        }

        let mut copy_size = self.state.buff_len - self.state.pos;
        count -= copy_size;
        self.state.s[self.state.pos..self.state.pos + copy_size]
            .clone_from_slice(&buffer[..copy_size]);

        BashState::bashF(&mut self.state.s);
        while count >= self.state.buff_len {
            self.state.s[..].clone_from_slice(&buffer[copy_size..copy_size + self.state.buff_len]);
            copy_size += self.state.buff_len;
            count -= self.state.buff_len;
            BashState::bashF(&mut self.state.s);
        }

        self.state.pos = count;
        if count != 0 {
            self.state.s[..count].clone_from_slice(&buffer[copy_size..copy_size + count]);
        }
    }

    fn step_g(&mut self, hash: &mut [u8]) {
        self.step_g_internal();
        hash[..].clone_from_slice(&self.state.s1[..]);
    }

    fn step_v(&mut self, hash: impl AsRef<[u8]>) -> bool {
        self.step_g_internal();
        return self.state.s1 == hash.as_ref();
    }

    fn hash(l: usize, hash: &mut [u8], src: impl AsRef<[u8]>) {
        let mut hasher = Bash::new(l);
        hasher.step_h(src);
        hasher.step_g(hash);
    }
}

impl Hasher for Bash256 {
    fn new() -> Self {
        Bash256 {
            bash: Bash::new(256),
        }
    }

    fn step_h(&mut self, buf: impl AsRef<[u8]>) {
        self.bash.step_h(buf);
    }

    fn step_g(&mut self, hash: &mut [u8]) {
        self.bash.step_g(hash);
    }

    fn step_v(&mut self, hash: impl AsRef<[u8]>) -> bool {
        self.bash.step_v(hash)
    }

    fn output_size(&self) -> usize {
        256 / 2
    }

    fn hash(hash: &mut [u8], src: impl AsRef<[u8]>) {
        Bash::hash(256, hash, src);
    }
}

impl Hasher for Bash384 {
    fn new() -> Self {
        Bash384 {
            bash: Bash::new(384),
        }
    }

    fn step_h(&mut self, buf: impl AsRef<[u8]>) {
        self.bash.step_h(buf);
    }

    fn step_g(&mut self, hash: &mut [u8]) {
        self.bash.step_g(hash);
    }

    fn step_v(&mut self, hash: impl AsRef<[u8]>) -> bool {
        self.bash.step_v(hash)
    }

    fn output_size(&self) -> usize {
        384 / 2
    }

    fn hash(hash: &mut [u8], src: impl AsRef<[u8]>) {
        Bash::hash(384, hash, src);
    }
}

impl Hasher for Bash512 {
    fn new() -> Self {
        Bash512 {
            bash: Bash::new(512),
        }
    }

    fn step_h(&mut self, buf: impl AsRef<[u8]>) {
        self.bash.step_h(buf);
    }

    fn step_g(&mut self, hash: &mut [u8]) {
        self.bash.step_g(hash);
    }

    fn step_v(&mut self, hash: impl AsRef<[u8]>) -> bool {
        self.bash.step_v(hash)
    }

    fn output_size(&self) -> usize {
        512 / 2
    }

    fn hash(hash: &mut [u8], src: impl AsRef<[u8]>) {
        Bash::hash(512, hash, src);
    }
}

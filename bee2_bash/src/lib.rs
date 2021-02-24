extern crate bee2_traits;
mod consts;

pub use bee2_traits::Hasher;
use consts::bash_f0;
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
pub struct Bash256 {
    hash_size: usize,
    bash: Bash,
}

#[derive(Clone)]
pub struct Bash384 {
    hash_size: usize,
    bash: Bash,
}
#[derive(Clone)]
pub struct Bash512 {
    hash_size: usize,
    bash: Bash,
}

impl BashState {
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
        if l % 16 != 0 || l > 256 {
            return Err(format!("Incorrect param of security {:}", l).to_owned());
        }

        let mut s = [0; 192];
        s[192 - 8] = (l / 4) as u8;

        Ok(Bash {
            state: BashState {
                buff_len: 192 - l / 2,
                pos: 0,
                s: s,
                s1: [0; 192],
            },
        })
    }

    fn step_g_internal(&mut self) {
        self.state.s1[..].clone_from_slice(&self.state.s[..]);
        if self.state.pos != 0 {
            // self.state.s1[..self.state.pos].fill(0);
            self.state.s1[self.state.pos..self.state.buff_len]
                .iter_mut()
                .for_each(|x| *x = 0);
            self.state.s1[self.state.pos] = 0x40;
        } else {
            self.state.s1[..self.state.buff_len]
                .iter_mut()
                .for_each(|x| *x = 0);
            self.state.s1[0] = 0x40;
        }
        BashState::bash_f(&mut self.state.s1);
    }

    fn new(l: usize) -> Self {
        Bash::bash_start(l).unwrap()
    }

    fn step_h(&mut self, buf: impl AsRef<[u8]>) {
        let buffer = buf.as_ref();
        let mut count = buffer.len();

        if count < self.state.buff_len - self.state.pos {
            self.state.s[self.state.pos..self.state.pos + count].clone_from_slice(buffer);
            self.state.pos += count;
            return;
        }

        let mut copy_size = self.state.buff_len - self.state.pos;
        count -= copy_size;
        self.state.s[self.state.pos..self.state.pos + copy_size]
            .clone_from_slice(&buffer[..copy_size]);

        BashState::bash_f(&mut self.state.s);
        while count >= self.state.buff_len {
            self.state.s[..self.state.buff_len]
                .clone_from_slice(&buffer[copy_size..copy_size + self.state.buff_len]);
            copy_size += self.state.buff_len;
            count -= self.state.buff_len;
            BashState::bash_f(&mut self.state.s);
        }

        self.state.pos = count;
        if count != 0 {
            self.state.s[..count].clone_from_slice(&buffer[copy_size..copy_size + count]);
        }
    }

    fn step_g(&mut self, hash: &mut [u8]) {
        self.step_g_internal();
        let hash_size = hash.len();
        hash[..].clone_from_slice(&self.state.s1[..hash_size]);
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
            hash_size: 128 / 4,
            bash: Bash::new(128),
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

    fn output_size() -> usize {
        128 / 4
    }

    fn hash(hash: &mut [u8], src: impl AsRef<[u8]>) {
        Bash::hash(128, hash, src);
    }
}

impl Hasher for Bash384 {
    fn new() -> Self {
        Bash384 {
            hash_size: 192 / 4,
            bash: Bash::new(192),
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

    fn output_size() -> usize {
        192 / 4
    }

    fn hash(hash: &mut [u8], src: impl AsRef<[u8]>) {
        Bash::hash(192, hash, src);
    }
}

impl Hasher for Bash512 {
    fn new() -> Self {
        Bash512 {
            hash_size: 256 / 4,
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

    fn output_size() -> usize {
        256 / 2
    }

    fn hash(hash: &mut [u8], src: impl AsRef<[u8]>) {
        Bash::hash(256, hash, src);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const S: [u64; 24] = [
        0xB194BAC80A08F53Bu64.to_be(),
        0x366D008E584A5DE4u64.to_be(),
        0x8504FA9D1BB6C7ACu64.to_be(),
        0x252E72C202FDCE0Du64.to_be(),
        0x5BE3D61217B96181u64.to_be(),
        0xFE6786AD716B890Bu64.to_be(),
        0x5CB0C0FF33C356B8u64.to_be(),
        0x35C405AED8E07F99u64.to_be(),
        0xE12BDC1AE28257ECu64.to_be(),
        0x703FCCF095EE8DF1u64.to_be(),
        0xC1AB76389FE678CAu64.to_be(),
        0xF7C6F860D5BB9C4Fu64.to_be(),
        0xF33C657B637C306Au64.to_be(),
        0xDD4EA7799EB23D31u64.to_be(),
        0x3E98B56E27D3BCCFu64.to_be(),
        0x591E181F4C5AB793u64.to_be(),
        0xE9DEE72C8F0C0FA6u64.to_be(),
        0x2DDB49F46F739647u64.to_be(),
        0x06075316ED247A37u64.to_be(),
        0x39CBA38303A98BF6u64.to_be(),
        0x92BD9B1CE5D14101u64.to_be(),
        0x5445FBC95E4D0EF2u64.to_be(),
        0x682080AA227D642Fu64.to_be(),
        0x2687F93490405511u64.to_be(),
    ];

    #[test]
    fn hash_test_128_0() {
        let l_128_0 = [
            0x114C3DFAE373D9BCu64.to_be(),
            0xBC3602D6386F2D6Au64.to_be(),
            0x2059BA1BF9048DBAu64.to_be(),
            0xA5146A6CB775709Du64.to_be(),
        ];

        let mut hash: [u8; 32] = Default::default();
        let s: [u8; 192] = unsafe { *(S.as_ptr() as *const [u8; 192]) };

        Bash256::hash(&mut hash, &s[..0]);
        assert_eq!(hash, unsafe { *(l_128_0.as_ptr() as *const [u8; 32]) });
    }
    #[test]
    fn hash_test_128_1() {
        let l_128_1 = [
            0x3D7F4EFA00E9BA33u64.to_be(),
            0xFEED259986567DCFu64.to_be(),
            0x5C6D12D51057A968u64.to_be(),
            0xF14F06CC0F905961u64.to_be(),
        ];

        let mut hash: [u8; 32] = Default::default();
        let s: [u8; 192] = unsafe { *(S.as_ptr() as *const [u8; 192]) };

        Bash256::hash(&mut hash, &s[..127]);
        assert_eq!(hash, unsafe { *(l_128_1.as_ptr() as *const [u8; 32]) });
    }
    #[test]
    fn hash_test_128_2() {
        let l_128_2 = [
            0xD7F428311254B8B2u64.to_be(),
            0xD00F7F9EEFBD8F30u64.to_be(),
            0x25FA87C4BABD1BDDu64.to_be(),
            0xBE87E35B7AC80DD6u64.to_be(),
        ];

        let mut hash: [u8; 32] = Default::default();
        let s: [u8; 192] = unsafe { *(S.as_ptr() as *const [u8; 192]) };

        Bash256::hash(&mut hash, &s[..128]);
        assert_eq!(hash, unsafe { *(l_128_2.as_ptr() as *const [u8; 32]) });
    }
    #[test]
    fn hash_test_128_3() {
        let l_128_3 = [
            0x1393FA1B65172F2Du64.to_be(),
            0x18946AEAE576FA1Cu64.to_be(),
            0xF54FDD354A0CB297u64.to_be(),
            0x4A997DC4865D3100u64.to_be(),
        ];
        let mut hash: [u8; 32] = Default::default();
        let s: [u8; 192] = unsafe { *(S.as_ptr() as *const [u8; 192]) };

        Bash256::hash(&mut hash, &s[..135]);
        assert_eq!(hash, unsafe { *(l_128_3.as_ptr() as *const [u8; 32]) });
    }
    #[test]
    fn hash_test_194_0() {
        let l_194_0 = [
            0x64334AF830D33F63u64.to_be(),
            0xE9ACDFA184E32522u64.to_be(),
            0x103FFF5C6860110Au64.to_be(),
            0x2CD369EDBC04387Cu64.to_be(),
            0x501D8F92F749AE4Du64.to_be(),
            0xE15A8305C353D64Du64.to_be(),
        ];

        let mut hash: [u8; 48] = [0; 48];
        let s: [u8; 192] = unsafe { *(S.as_ptr() as *const [u8; 192]) };
        Bash384::hash(&mut hash, &s[..95]);
        assert_eq!(hash, unsafe { *(l_194_0.as_ptr() as *const [u8; 48]) });
    }
    #[test]
    fn hash_test_194_1() {
        let l_194_1 = [
            0xD06EFBC16FD6C088u64.to_be(),
            0x0CBFC6A4E3D65AB1u64.to_be(),
            0x01FA82826934190Fu64.to_be(),
            0xAABEBFBFFEDE93B2u64.to_be(),
            0x2B85EA72A7FB3147u64.to_be(),
            0xA133A5A8FEBD8320u64.to_be(),
        ];

        let mut hash: [u8; 48] = [0; 48];
        let s: [u8; 192] = unsafe { *(S.as_ptr() as *const [u8; 192]) };
        Bash384::hash(&mut hash, &s[..96]);
        assert_eq!(hash, unsafe { *(l_194_1.as_ptr() as *const [u8; 48]) });
    }
    #[test]
    fn hash_test_194_2() {
        let l_194_2 = [
            0xFF763296571E2377u64.to_be(),
            0xE71A1538070CC0DEu64.to_be(),
            0x88888606F32EEE6Bu64.to_be(),
            0x082788D246686B00u64.to_be(),
            0xFC05A17405C55176u64.to_be(),
            0x99DA44B7EF5F55ABu64.to_be(),
        ];

        let mut hash: [u8; 48] = [0; 48];
        let s: [u8; 192] = unsafe { *(S.as_ptr() as *const [u8; 192]) };
        Bash384::hash(&mut hash, &s[..108]);
        assert_eq!(hash, unsafe { *(l_194_2.as_ptr() as *const [u8; 48]) });
    }

    #[test]
    fn hash_test_256_0() {
        let l_256_0 = [
            0x2A66C87C189C12E2u64.to_be(),
            0x55239406123BDEDBu64.to_be(),
            0xF19955EAF0808B2Au64.to_be(),
            0xD705E249220845E2u64.to_be(),
            0x0F4786FB6765D0B5u64.to_be(),
            0xC48984B1B16556EFu64.to_be(),
            0x19EA8192B985E423u64.to_be(),
            0x3D9C09508D6339E7u64.to_be(),
        ];

        let mut hash: [u8; 64] = [0; 64];
        let s: [u8; 192] = unsafe { *(S.as_ptr() as *const [u8; 192]) };
        Bash512::hash(&mut hash, &s[..63]);
        assert_eq!(hash, unsafe { *(l_256_0.as_ptr() as *const [u8; 64]) });
    }
    #[test]
    fn hash_test_256_1() {
        let l_256_1 = [
            0x07ABBF8580E7E5A3u64.to_be(),
            0x21E9B940F667AE20u64.to_be(),
            0x9E2952CEF557978Au64.to_be(),
            0xE743DB086BAB4885u64.to_be(),
            0xB708233C3F5541DFu64.to_be(),
            0x8AAFC3611482FDE4u64.to_be(),
            0x98E58B3379A6622Du64.to_be(),
            0xAC2664C9C118A162u64.to_be(),
        ];
        let mut hash: [u8; 64] = [0; 64];
        let s: [u8; 192] = unsafe { *(S.as_ptr() as *const [u8; 192]) };
        Bash512::hash(&mut hash, &s[..64]);
        assert_eq!(hash, unsafe { *(l_256_1.as_ptr() as *const [u8; 64]) });
    }

    #[test]
    fn hash_test_256_2() {
        let l_256_2 = [
            0x526073918F97928Eu64.to_be(),
            0x9D15508385F42F03u64.to_be(),
            0xADE3211A23900A30u64.to_be(),
            0x131F8A1E3E1EE21Cu64.to_be(),
            0xC09D13CFF6981101u64.to_be(),
            0x235D895746A4643Fu64.to_be(),
            0x0AA62B0A7BC98A26u64.to_be(),
            0x9E4507A257F0D4EEu64.to_be(),
        ];
        let mut hash: [u8; 64] = [0; 64];
        let s: [u8; 192] = unsafe { *(S.as_ptr() as *const [u8; 192]) };
        Bash512::hash(&mut hash, &s[..127]);
        assert_eq!(hash, unsafe { *(l_256_2.as_ptr() as *const [u8; 64]) });
    }
    #[test]
    fn hash_test_256_3() {
        let l_256_3 = [
            0x8724C7FF8A2A83F2u64.to_be(),
            0x2E38CB9763777B96u64.to_be(),
            0xA70ABA3444F214C7u64.to_be(),
            0x63D93CD6D19FCFDEu64.to_be(),
            0x6C3D3931857C4FF6u64.to_be(),
            0xCCCD49BD99852FE9u64.to_be(),
            0xEAA7495ECCDD96B5u64.to_be(),
            0x71E0EDCF47F89768u64.to_be(),
        ];
        let mut hash: [u8; 64] = [0; 64];
        let s: [u8; 192] = unsafe { *(S.as_ptr() as *const [u8; 192]) };
        Bash512::hash(&mut hash, &s[..192]);
        assert_eq!(hash, unsafe { *(l_256_3.as_ptr() as *const [u8; 64]) });
    }
}

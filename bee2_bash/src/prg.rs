extern crate bee2_traits;

use crate::consts::bash_f0;
pub use bee2_traits::*;
use core::marker::PhantomData;
use generic_array::{
    typenum::{U1, U128, U192, U2, U256},
    ArrayLength,
};
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

#[derive(Copy, Clone)]
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

#[derive(Clone, Copy)]
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
        println!("Buff Len: {:?}", self.state.buff_len);
        println!("Commit: {:X?}", self.state.s);
        BashPrgState::bash_f(&mut self.state.s);
        self.state.pos = 0;
        println!("CommitA: {:X?}", self.state.s);
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
            panic!(format!("Incorrect len of key {:}", ann.len()).to_owned());
        }

        // pos <- 8 + |ann| + |key|
        let pos = 1 + ann.len() + key.len();
        let mut s = [0; 192];
        // s[0..pos) <- <|ann|/2 + |key|/32>_8 || ann || key
        s[0] = (ann.len() * 4 + key.len() / 4) as u8;
        s[1..1 + ann.len()].copy_from_slice(&ann[..]);
        s[1 + ann.len()..1 + ann.len() + key.len()].copy_from_slice(&key[..]);
        // s[1472..) <- <l / 4 + d>_{64}
        s[192 - 8] = (l / 4 + d) as u8;
        // s[pos..) <- 0
        // s[pos..].iter_mut().for_each(|x| *x = 0);
        println!("Start s: {:X?}, l: {:?}, d: {:?}", s, l, d);
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

impl BashPrg {
    fn new(l: usize, d: usize, ann_: impl AsRef<[u8]>, key_: impl AsRef<[u8]>) -> Self {
        return BashPrg::start(l, d, ann_, key_);
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
            panic!(format!("Incorrect len of key {:}", ann.len()).to_owned());
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
            println!("Encrypt!! {:?}", count);
            println!("Encrypts!: {:X?}", buf);
            mem_xor(
                &mut self.state.s[self.state.pos..self.state.pos + count],
                buf,
            );
            println!("Encrypt!: {:X?}", self.state.s);
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
        mem_xor(&mut self.state.s, &self.state.t);
    }
}

impl Prg for BashPrg {
    fn output_size() -> usize {
        return 0;
    }
}

/// Block 8.11 Programming
//#[allow(non_camel_case_types)]
pub fn programming(
    K: impl AsRef<[u8]>,
    I: impl AsRef<[u8]>,
    A1: impl AsRef<[u8]>,
    A2: impl AsRef<[u8]>,
    Y1: &mut [u8],
    Y2: &mut [u8],
    K1: &mut [u8],
) {
    // Step 1.
    let mut alpha = BashPrg::start(256, 2, [], K);
    // Step 2.
    alpha.absorb(I);
    // Step 3.
    alpha.ratchet();
    // Step 4.
    alpha.squeeze(K1);
    // Step 5.
    let mut beta = BashPrg::start(128, 1, A1, K1);
    // Step 6.
    let mut gamma = beta;
    // Step 7.
    gamma.restart(A2, []);
    // Step 8.
    beta.encr(Y1);
    // Step 9.
    gamma.encr(Y2);
}

#[derive(Clone)]
pub struct BashPrgHash<SecurityLevel, Capacity>
where
    SecurityLevel: ArrayLength<u8>,
    Capacity: ArrayLength<u8>,
{
    prg: BashPrg,
    l: PhantomData<SecurityLevel>,
    d: PhantomData<Capacity>,
}

impl<L, D> PrgHasher for BashPrgHash<L, D>
where
    L: ArrayLength<u8>,
    D: ArrayLength<u8>,
{
    fn new(ann: impl AsRef<[u8]>) -> Self {
        BashPrgHash {
            l: Default::default(),
            d: Default::default(),
            prg: BashPrg::start(L::to_usize(), D::to_usize() as usize, ann, []),
        }
    }

    fn hash(&mut self, data: impl AsRef<[u8]>, hash: &mut [u8]) {
        self.prg.absorb(data);
        self.prg.squeeze(hash);
        self.prg.ratchet();
    }
}

pub type BashPrgHash2561 = BashPrgHash<U128, U1>;
pub type BashPrgHash2562 = BashPrgHash<U128, U2>;
pub type BashPrgHash3841 = BashPrgHash<U192, U1>;
pub type BashPrgHash3842 = BashPrgHash<U192, U2>;
pub type BashPrgHash5121 = BashPrgHash<U256, U1>;
pub type BashPrgHash5122 = BashPrgHash<U256, U2>;

#[derive(Clone)]
pub struct BashPrgAEAD<SecurityLevel, Capacity>
where
    SecurityLevel: ArrayLength<u8>,
    Capacity: ArrayLength<u8>,
{
    prg: BashPrg,
    l: PhantomData<SecurityLevel>,
    d: PhantomData<Capacity>,
}

impl<L, D> PrgAEAD for BashPrgAEAD<L, D>
where
    L: ArrayLength<u8>,
    D: ArrayLength<u8>,
{
    fn new(ann: impl AsRef<[u8]>, key: impl AsRef<[u8]>) -> Self {
        BashPrgAEAD {
            l: Default::default(),
            d: Default::default(),
            prg: BashPrg::start(L::to_usize(), D::to_usize() as usize, ann, key),
        }
    }

    fn encrypt(
        &mut self,
        plaintext: impl AsRef<[u8]>,
        header: impl AsRef<[u8]>,
        ciphertext: &mut [u8],
        tag: &mut [u8],
    ) {
        self.prg.absorb(header);
        ciphertext.clone_from_slice(&plaintext.as_ref());
        self.prg.encr(ciphertext);
        self.prg.squeeze(tag);
    }

    fn decrypt(
        &mut self,
        ciphertext: impl AsRef<[u8]>,
        header: impl AsRef<[u8]>,
        tag_: impl AsRef<[u8]>,
        plaintext: &mut [u8],
    ) {
        self.prg.absorb(header);
        plaintext.clone_from_slice(&ciphertext.as_ref());
        self.prg.decr(plaintext);
        let tag = tag_.as_ref();
        let mut tag_get: Box<[u8]> = vec![0; tag.len()].into_boxed_slice();
        self.prg.squeeze(tag_get.as_mut());
        if tag_get.as_ref().eq(tag) == false {
            plaintext.iter_mut().for_each(|x| *x = 0);
            panic!(format!("Incorrect tag").to_owned());
        }
    }
}

pub type BashPrgAEAD2561 = BashPrgAEAD<U128, U1>;
pub type BashPrgAEAD2562 = BashPrgAEAD<U128, U2>;
pub type BashPrgAEAD3841 = BashPrgAEAD<U192, U1>;
pub type BashPrgAEAD3842 = BashPrgAEAD<U192, U2>;
pub type BashPrgAEAD5121 = BashPrgAEAD<U256, U1>;
pub type BashPrgAEAD5122 = BashPrgAEAD<U256, U2>;

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
    fn prg_test_a4() {
        let k1_ = [0x71CC358A0D508217u64.to_be(), 0x3DE04803F7E905CBu64.to_be()];

        let y1_ = [
            0x51ED3B28D345FFD1u64.to_be(),
            0xAD22815B86ECC17Cu64.to_be(),
            0x278C8FE892021400u64.to_be(),
        ];

        let y2_ = [
            0x28FE0998BFC010F1u64.to_be(),
            0x3B260685A27AFB36u64.to_be(),
            0xCCF580F753521B00u64.to_be(),
        ];

        let s: [u8; 192] = unsafe { *(S.as_ptr() as *const [u8; 192]) };
        let mut Y1: [u8; 23] = [0; 23];
        let mut Y2: [u8; 23] = [0; 23];
        let mut K1: [u8; 16] = [0; 16];
        Y1[..].copy_from_slice(&s[160..183]);
        Y2[..].copy_from_slice(&s[160..183]);

        println!("X {:X?}", Y1);
        println!("K {:X?}", &s[0..32]);
        println!("I {:X?}", &s[32..127]);
        println!("A1 {:X?}", &s[128..144]);
        println!("A2 {:X?}", &s[144..148]);
        programming(
            &s[0..32],
            &s[32..127],
            &s[128..144],
            &s[144..148],
            &mut Y1,
            &mut Y2,
            &mut K1,
        );

        println!("K1 {:X?}", K1);
        println!("Y1 {:X?}", Y1);
        println!("Y2 {:X?}", Y2);
        assert_eq!(K1, unsafe { *(k1_.as_ptr() as *const [u8; 16]) });
        assert_eq!(Y1, unsafe { *(y1_.as_ptr() as *const [u8; 23]) });
        assert_eq!(Y2, unsafe { *(y2_.as_ptr() as *const [u8; 23]) });
    }

    /// A.5 (l,d) = (128,2), m = 0
    #[test]
    fn hash_test_128_2_0() {
        let l_128_2 = [
            0x36FA075EC15721F2u64.to_be(),
            0x50B9A641A8CB99A3u64.to_be(),
            0x33A9EE7BA8586D06u64.to_be(),
            0x46CBAC3686C03DF3u64.to_be(),
        ];
        let mut hash: [u8; 32] = [0; 32];
        let s: [u8; 192] = unsafe { *(S.as_ptr() as *const [u8; 192]) };

        let mut hasher = BashPrgHash2562::new([]);
        hasher.hash(&s[..0], &mut hash);

        assert_eq!(hash, unsafe { *(l_128_2.as_ptr() as *const [u8; 32]) });
    }

    /// A.5 (l,d) = (128,2), m = 127
    #[test]
    fn hash_test_128_2_127() {
        let l_128_2_127 = [
            0xC930FF427307420Du64.to_be(),
            0xA6E4182969AA1FFCu64.to_be(),
            0x3310179B8A0EDB3Eu64.to_be(),
            0x20BEC285B568BA17u64.to_be(),
        ];
        let mut hash: [u8; 32] = [0; 32];
        let s: [u8; 192] = unsafe { *(S.as_ptr() as *const [u8; 192]) };

        let mut hasher = BashPrgHash2562::new([]);
        hasher.hash(&s[..127], &mut hash);

        assert_eq!(hash, unsafe { *(l_128_2_127.as_ptr() as *const [u8; 32]) });
    }

    /// A.5 (l,d) = (128,2), m = 128
    #[test]
    fn hash_test_128_2_128() {
        let l_128_2_128 = [
            0x92AD1402C2007191u64.to_be(),
            0xF2F7CFAD6A2F8807u64.to_be(),
            0xBB0C50F73DFF95EFu64.to_be(),
            0x1B8AF08504D54007u64.to_be(),
        ];
        let mut hash: [u8; 32] = [0; 32];
        let s: [u8; 192] = unsafe { *(S.as_ptr() as *const [u8; 192]) };

        let mut hasher = BashPrgHash2562::new([]);
        hasher.hash(&s[..128], &mut hash);

        assert_eq!(hash, unsafe { *(l_128_2_128.as_ptr() as *const [u8; 32]) });
    }

    /// A.5 (l,d) = (128,2), m = 150
    #[test]
    fn hash_test_128_2_150() {
        let l_128_2_150 = [
            0x48DB61832CA10090u64.to_be(),
            0x03BC0D8BDE67893Au64.to_be(),
            0x9DC683C48A5BC23Au64.to_be(),
            0xC884EB4613B480A6u64.to_be(),
        ];
        let mut hash: [u8; 32] = [0; 32];
        let s: [u8; 192] = unsafe { *(S.as_ptr() as *const [u8; 192]) };

        let mut hasher = BashPrgHash2562::new([]);
        hasher.hash(&s[..150], &mut hash);

        assert_eq!(hash, unsafe { *(l_128_2_150.as_ptr() as *const [u8; 32]) });
    }

    /// A.5 (l,d) = (192,1), m = 143
    #[test]
    fn hash_test_192_1_143() {
        let l_ = [
            0x6166032D6713D401u64.to_be(),
            0xA6BC687CCFFF2E60u64.to_be(),
            0x3287143A84C78D2Cu64.to_be(),
            0x62C71551E0E2FB2Au64.to_be(),
            0xF6B799EE33B5DECDu64.to_be(),
            0x7F62F190B1FBB052u64.to_be(),
        ];
        let mut hash: [u8; 48] = [0; 48];
        let s: [u8; 192] = unsafe { *(S.as_ptr() as *const [u8; 192]) };

        let mut hasher = BashPrgHash3841::new([]);
        hasher.hash(&s[..143], &mut hash);

        assert_eq!(hash, unsafe { *(l_.as_ptr() as *const [u8; 48]) });
    }

    /// A.5 (l,d) = (192,1), m = 144
    #[test]
    fn hash_test_192_1_144() {
        let l_ = [
            0x8D84C82ECD0AB646u64.to_be(),
            0x8CC451CFC5EEB3B2u64.to_be(),
            0x98DFD381D200DA69u64.to_be(),
            0xFBED5AE67D26BAD5u64.to_be(),
            0xC727E2652A225BF4u64.to_be(),
            0x65993043039E338Bu64.to_be(),
        ];
        let mut hash: [u8; 48] = [0; 48];
        let s: [u8; 192] = unsafe { *(S.as_ptr() as *const [u8; 192]) };

        let mut hasher = BashPrgHash3841::new([]);
        hasher.hash(&s[..144], &mut hash);

        assert_eq!(hash, unsafe { *(l_.as_ptr() as *const [u8; 48]) });
    }

    /// A.5 (l,d) = (192,1), m = 150
    #[test]
    fn hash_test_192_1_150() {
        let l_ = [
            0x47529F9D499AB6ABu64.to_be(),
            0x8AD72B1754C90C39u64.to_be(),
            0xE7DA237BEB16CDFCu64.to_be(),
            0x00FE87934F5AFC11u64.to_be(),
            0x01862DFA50560F06u64.to_be(),
            0x2A4DAC859CC13DBCu64.to_be(),
        ];
        let mut hash: [u8; 48] = [0; 48];
        let s: [u8; 192] = unsafe { *(S.as_ptr() as *const [u8; 192]) };

        let mut hasher = BashPrgHash3841::new([]);
        hasher.hash(&s[..150], &mut hash);

        assert_eq!(hash, unsafe { *(l_.as_ptr() as *const [u8; 48]) });
    }

    /// A.6 (l,d) = (256,1)
    #[test]
    fn aead_test() {
        let y_ = [
            0x690673766C3E848Cu64.to_be(),
            0xAC7C05169FFB7B77u64.to_be(),
            0x51E52A011040E560u64.to_be(),
            0x2573FAF991044A00u64.to_be(),
            0x4329EEF7BED8E687u64.to_be(),
            0x5830A91854D1BD2Eu64.to_be(),
            0xDC6FC2FF37851DBAu64.to_be(),
            0xC249DF400A0549EAu64.to_be(),
            0x2E0C811D499E1FF1u64.to_be(),
            0xE5E32FAE7F0532FAu64.to_be(),
            0x4051D0F9E300D9B1u64.to_be(),
            0xDBF119AC8CFFC48Du64.to_be(),
            0xD3CBF1CA0DBA5DD9u64.to_be(),
            0x7481C88DF0BE4127u64.to_be(),
            0x85E40988B3158553u64.to_be(),
            0x7948B80F5A9C49E0u64.to_be(),
            0x8DD684A7DCA871C3u64.to_be(),
            0x80DFDC4C4DFBE61Fu64.to_be(),
            0x50D2D0FBD24D8B9Du64.to_be(),
            0x32974A347247D001u64.to_be(),
            0xBAD5B16844002569u64.to_be(),
            0x3967E77394DC088Bu64.to_be(),
            0x0ECCFA8D291BA13Du64.to_be(),
            0x44F60B06E2EDB351u64.to_be(),
        ];

        let mut x: [u8; 192] = [0; 192];
        let mut y: [u8; 192] = [0; 192];
        let mut t: [u8; 32] = [0; 32];

        let s: [u8; 192] = unsafe { *(S.as_ptr() as *const [u8; 192]) };

        println!("A {:X?}", &s[0..16]);
        println!("K {:X?}", &s[32..64]);
        println!("I {:X?}", &s[64..113]);


        let mut aead = BashPrgAEAD5121::new(&s[0..16], &s[32..64]);
        aead.encrypt(x, &s[64..109],&mut y, &mut t);
        println!("I {:X?}", y);
        println!("I {:X?}", t);

        assert_eq!(y, unsafe { *(y_.as_ptr() as *const [u8; 192]) });
    }
}

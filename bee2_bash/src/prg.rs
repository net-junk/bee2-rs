pub use bee2_core::error::{Error, IncorrectTag, InvalidCommand, InvalidLength};
pub use bee2_traits::*;

use crate::consts::bash_f;

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
}

#[derive(Clone, Copy)]
pub struct BashPrg {
    /// State.
    state: BashPrgState,
}

impl BashPrgState {
    fn is_key_mode(&self) -> bool {
        // (192 - buf_len) ==? (l + d * l / 2) / 8
        16 * (192 - self.buff_len) == self.l * (2 + self.d)
    }
}

impl BashPrg {
    /// `Commit` command.
    /// Finish previous command and start new one using `code`.
    fn prg_commit(&mut self, code: u8) {
        self.state.s[self.state.pos] ^= code;
        self.state.s[self.state.buff_len] ^= 0x80;
        bash_f(&mut self.state.s);
        self.state.pos = 0;
    }
}

impl PrgStart for BashPrg {
    fn start(
        l: usize,
        d: usize,
        ann_: impl AsRef<[u8]>,
        key_: impl AsRef<[u8]>,
    ) -> Result<Self, InvalidLength> {
        if l != 128 && l != 192 && l != 256 {
            // panic!(format!("Incorrect param of security {:}", l).to_owned());
            return Err(InvalidLength);
        }

        if d != 1 && d != 2 {
            // panic!(format!("Incorrect param of capacity {:}", d).to_owned());
            return Err(InvalidLength);
        }

        let ann = ann_.as_ref();
        let key = key_.as_ref();

        if ann.len() % 4 != 0 || ann.len() > 60 || (!key.is_empty() && key.len() < l / 8) {
            // panic!(format!("Incorrect len of annotation {:}", ann.len()).to_owned());
            return Err(InvalidLength);
        }

        if key.len() % 4 != 0 || key.len() > 60 {
            // panic!(format!("Incorrect len of key {:}", ann.len()).to_owned());
            return Err(InvalidLength);
        }

        // pos <- 8 + |ann| + |key|
        let pos = 1 + ann.len() + key.len();
        let mut s = [0; 192];
        // s[0..pos) <- <|ann|/2 + |key|/32>_8 || ann || key
        s[0] = (ann.len() * 4 + key.len() / 4) as u8;
        s[1..1 + ann.len()].copy_from_slice(ann);
        s[1 + ann.len()..1 + ann.len() + key.len()].copy_from_slice(key);
        // s[1472..) <- <l / 4 + d>_{64}
        s[192 - 8] = (l / 4 + d) as u8;
        // s[pos..) <- 0
        // s[pos..].iter_mut().for_each(|x| *x = 0);
        Ok(BashPrg {
            state: BashPrgState {
                l,
                d,
                pos,
                s,
                t: [0; 192],
                buff_len: match key.len() {
                    0 => (192 - d * l / 4),
                    _ => (192 - l * (2 + d) / 16),
                },
            },
        })
    }
}

impl BashPrg {
    pub fn new(
        l: usize,
        d: usize,
        ann_: impl AsRef<[u8]>,
        key_: impl AsRef<[u8]>,
    ) -> Result<Self, InvalidLength> {
        BashPrg::start(l, d, ann_, key_)
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
    fn restart(
        &mut self,
        ann_: impl AsRef<[u8]>,
        key_: impl AsRef<[u8]>,
    ) -> Result<(), InvalidLength> {
        let ann = ann_.as_ref();
        let key = key_.as_ref();

        if ann.len() % 4 != 0 || ann.len() > 60 || (!key.is_empty() && key.len() < self.state.l / 8)
        {
            // panic!(format!("Incorrect len of annotation {:}", ann.len()).to_owned());
            return Err(InvalidLength);
        }

        if key.len() % 4 != 0 || key.len() > 60 {
            // panic!(format!("Incorrect len of key {:}", ann.len()).to_owned());
            return Err(InvalidLength);
        }

        if !key.is_empty() {
            self.prg_commit(PrgCommands::BASH_PRG_KEY as u8);
        } else {
            self.prg_commit(PrgCommands::BASH_PRG_NULL as u8);
        }

        // pos <- 8 + |ann| + |key|
        self.state.pos = 1 + ann.len() + key.len();
        // s[0..pos) <- s[0..pos) ^ <|ann|/2 + |key|/32>_8  || ann || key
        self.state.s[0] ^= (ann.len() * 4 + key.len() / 4) as u8;
        mem_xor(&mut self.state.s[1..1 + ann.len()], ann);
        mem_xor(&mut self.state.s[1 + ann.len()..self.state.pos], key);

        Ok(())
    }
}

impl PrgAbsorb for BashPrg {
    fn absorb_start(&mut self) {
        self.prg_commit(PrgCommands::BASH_PRG_DATA as u8);
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
        bash_f(&mut self.state.s);

        let mut copy_size = self.state.buff_len - self.state.pos;
        count -= copy_size;

        while count >= self.state.buff_len {
            mem_xor(
                &mut self.state.s,
                &buf[copy_size..copy_size + self.state.buff_len],
            );
            copy_size += self.state.buff_len;
            count -= self.state.buff_len;
            bash_f(&mut self.state.s);
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
        self.prg_commit(PrgCommands::BASH_PRG_OUT as u8);
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
        bash_f(&mut self.state.s);

        let mut copy_size = self.state.buff_len - self.state.pos;
        count -= copy_size;

        while count >= self.state.buff_len {
            mem_cpy(
                &mut buf[copy_size..copy_size + self.state.buff_len],
                &self.state.s[0..self.state.buff_len],
            );
            copy_size += self.state.buff_len;
            count -= self.state.buff_len;
            bash_f(&mut self.state.s);
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
    fn encr_start(&mut self) -> Result<(), InvalidCommand> {
        if !self.state.is_key_mode() {
            // panic!("State not in key mode");
            return Err(InvalidCommand);
        }
        self.prg_commit(PrgCommands::BASH_PRG_TEXT as u8);

        Ok(())
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
        bash_f(&mut self.state.s);

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
            bash_f(&mut self.state.s);
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

    fn encr(&mut self, buf: &mut [u8]) -> Result<(), InvalidCommand> {
        self.encr_start()?;
        self.encr_step(buf);

        Ok(())
    }
}

impl PrgDecr for BashPrg {
    fn decr_start(&mut self) -> Result<(), InvalidCommand> {
        if !self.state.is_key_mode() {
            // panic!("State not in key mode");
            return Err(InvalidCommand);
        }
        self.prg_commit(PrgCommands::BASH_PRG_TEXT as u8);

        Ok(())
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
        bash_f(&mut self.state.s);

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
            bash_f(&mut self.state.s);
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

    fn decr(&mut self, buf: &mut [u8]) -> Result<(), InvalidCommand> {
        self.decr_start()?;
        self.decr_step(buf);

        Ok(())
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
        0_usize
    }
}

/// Block 8.11 Programming
pub fn programming(
    k: impl AsRef<[u8]>,
    i: impl AsRef<[u8]>,
    a1: impl AsRef<[u8]>,
    a2: impl AsRef<[u8]>,
    y1: &mut [u8],
    y2: &mut [u8],
    k1: &mut [u8],
) -> Result<(), Error> {
    // Step 1.
    let mut alpha = BashPrg::start(256, 2, [], k)?;
    // Step 2.
    alpha.absorb(i);
    // Step 3.
    alpha.ratchet();
    // Step 4.
    alpha.squeeze(k1);
    // Step 5.
    let mut beta = BashPrg::start(128, 1, a1, k1)?;
    // Step 6.
    let mut gamma = beta;
    // Step 7.
    gamma.restart(a2, [])?;
    // Step 8.
    beta.encr(y1)?;
    // Step 9.
    gamma.encr(y2)?;

    Ok(())
}

macro_rules! bash_prg_hash {
    ($full_name:ident, $security_level:expr, $capacity:expr) => {
        #[derive(Clone)]
        pub struct $full_name {
            prg: BashPrg,
        }

        impl PrgHasher for $full_name {
            fn new(ann: impl AsRef<[u8]>) -> Result<Self, InvalidLength> {
                Ok(Self {
                    prg: BashPrg::start($security_level, $capacity, ann, [])?,
                })
            }

            fn update(&mut self, data: impl AsRef<[u8]>) {
                self.prg.absorb(data);
            }

            fn hash(&mut self, data: impl AsRef<[u8]>, hash: &mut [u8]) {
                self.prg.absorb(data);
                self.prg.squeeze(hash);
                self.prg.ratchet();
            }
        }
    };
}

bash_prg_hash!(BashPrgHash2561, 128, 1);
bash_prg_hash!(BashPrgHash2562, 128, 2);
bash_prg_hash!(BashPrgHash3841, 192, 1);
bash_prg_hash!(BashPrgHash3842, 192, 2);
bash_prg_hash!(BashPrgHash5121, 256, 1);
bash_prg_hash!(BashPrgHash5122, 256, 2);

macro_rules! bash_prg_aead {
    ($full_name:ident, $security_level:expr, $capacity:expr) => {
        #[derive(Clone)]
        pub struct $full_name {
            prg: BashPrg,
        }

        impl PrgAEAD for $full_name {
            fn new(ann: impl AsRef<[u8]>, key: impl AsRef<[u8]>) -> Result<Self, InvalidLength> {
                Ok(Self {
                    prg: BashPrg::start($security_level, $capacity, ann, key)?,
                })
            }

            fn encrypt(
                &mut self,
                plaintext: impl AsRef<[u8]>,
                header: impl AsRef<[u8]>,
                ciphertext: &mut [u8],
                tag: &mut [u8],
            ) -> Result<(), Error> {
                self.prg.absorb(header);
                ciphertext.clone_from_slice(&plaintext.as_ref());
                self.prg.encr(ciphertext)?;
                self.prg.squeeze(tag);

                Ok(())
            }

            fn decrypt(
                &mut self,
                ciphertext: impl AsRef<[u8]>,
                header: impl AsRef<[u8]>,
                tag_: impl AsRef<[u8]>,
                plaintext: &mut [u8],
            ) -> Result<(), Error> {
                self.prg.absorb(header);
                plaintext.clone_from_slice(&ciphertext.as_ref());
                self.prg.decr(plaintext)?;
                let tag = tag_.as_ref();
                let mut tag_get: Box<[u8]> = vec![0; tag.len()].into_boxed_slice();
                self.prg.squeeze(tag_get.as_mut());
                if tag_get.as_ref().eq(tag) == false {
                    plaintext.iter_mut().for_each(|x| *x = 0);

                    // panic!(format!("Incorrect tag").to_owned());
                    return Err(Error::from(IncorrectTag));
                }

                Ok(())
            }
        }
    };
}

bash_prg_aead!(BashPrgAEAD2561, 128, 1);
bash_prg_aead!(BashPrgAEAD2562, 128, 2);
bash_prg_aead!(BashPrgAEAD3841, 192, 1);
bash_prg_aead!(BashPrgAEAD3842, 192, 2);
bash_prg_aead!(BashPrgAEAD5121, 256, 1);
bash_prg_aead!(BashPrgAEAD5122, 256, 2);

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
        let mut y1: [u8; 23] = [0; 23];
        let mut y2: [u8; 23] = [0; 23];
        let mut k1: [u8; 16] = [0; 16];
        y1[..].copy_from_slice(&s[160..183]);
        y2[..].copy_from_slice(&s[160..183]);

        programming(
            &s[0..32],
            &s[32..127],
            &s[128..144],
            &s[144..148],
            &mut y1,
            &mut y2,
            &mut k1,
        )
        .unwrap();

        assert_eq!(k1, unsafe { *(k1_.as_ptr() as *const [u8; 16]) });
        assert_eq!(y1, unsafe { *(y1_.as_ptr() as *const [u8; 23]) });
        assert_eq!(y2, unsafe { *(y2_.as_ptr() as *const [u8; 23]) });
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

        let mut hasher = BashPrgHash2562::new([]).unwrap();
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

        let mut hasher = BashPrgHash2562::new([]).unwrap();
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

        let mut hasher = BashPrgHash2562::new([]).unwrap();
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

        let mut hasher = BashPrgHash2562::new([]).unwrap();
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

        let mut hasher = BashPrgHash3841::new([]).unwrap();
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

        let mut hasher = BashPrgHash3841::new([]).unwrap();
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

        let mut hasher = BashPrgHash3841::new([]).unwrap();
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

        let x: [u8; 192] = [0; 192];
        let mut y: [u8; 192] = [0; 192];
        let mut t: [u8; 32] = [0; 32];

        let s: [u8; 192] = unsafe { *(S.as_ptr() as *const [u8; 192]) };

        let mut aead = BashPrgAEAD5121::new(&s[0..16], &s[32..64]).unwrap();
        aead.encrypt(x, &s[64..113], &mut y, &mut t).unwrap();

        assert_eq!(y, unsafe { *(y_.as_ptr() as *const [u8; 192]) });
    }
}

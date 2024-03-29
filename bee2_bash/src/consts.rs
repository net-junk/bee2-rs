static C: [u64; 24] = [
    0x3BF5080AC8BA94B1,
    0xC1D1659C1BBD92F6,
    0x60E8B2CE0DDEC97B,
    0xEC5FB8FE790FBC13,
    0xAA043DE6436706A7,
    0x8929FF6A5E535BFD,
    0x98BF1E2C50C97550,
    0x4C5F8F162864BAA8,
    0x262FC78B14325D54,
    0x1317E3C58A192EAA,
    0x098BF1E2C50C9755,
    0xD8EE19681D669304,
    0x6C770CB40EB34982,
    0x363B865A0759A4C1,
    0xC73622B47C4C0ACE,
    0x639B115A3E260567,
    0xEDE6693460F3DA1D,
    0xAAD8D5034F9935A0,
    0x556C6A81A7CC9AD0,
    0x2AB63540D3E64D68,
    0x155B1AA069F326B4,
    0x0AAD8D5034F9935A,
    0x0556C6A81A7CC9AD,
    0xDE8082CD72DEBC78,
];

#[inline]
fn p0(x: u64) -> u64 {
    x
}

#[inline]
fn p1(x: u64) -> u64 {
    if x < 8 {
        8 + (x + 2 * (x & 1) + 7) % 8
    } else if x < 16 {
        8 + (x ^ 1)
    } else {
        (5 * x + 6) % 8
    }
}

#[inline]
fn p2(x: u64) -> u64 {
    p1(p1(x))
}

#[inline]
fn p3(x: u64) -> u64 {
    8 * (x / 8) + (x % 8 + 4) % 8
}

#[inline]
fn p4(x: u64) -> u64 {
    p1(p3(x))
}

#[inline]
fn p5(x: u64) -> u64 {
    p2(p3(x))
}

#[inline]
fn bash_s(s: &mut [u64; 24], w0_: usize, w1_: usize, w2_: usize, m1: u8, n1: u8, m2: u8, n2: u8) {
    let mut t0: u64;
    let mut t1: u64;
    let mut t2: u64;

    t2 = s[w0_].rotate_left(m1 as u32);
    s[w0_] ^= s[w1_] ^ s[w2_];
    t1 = s[w1_] ^ s[w0_].rotate_left(n1 as u32);
    s[w1_] = t1 ^ t2;
    s[w2_] ^= s[w2_].rotate_left(m2 as u32) ^ t1.rotate_left(n2 as u32);
    t1 = s[w0_] | s[w2_];
    t2 = s[w0_] & s[w1_];
    t0 = !s[w2_];
    t0 |= s[w1_];
    s[w0_] ^= t0;
    s[w1_] ^= t1;
    s[w2_] ^= t2;
}

/// A1. Test
#[test]
fn bash_s_test() {
    let w0 = 0xB194BAC80A08F53Bu64.to_be();
    let w1 = 0xE12BDC1AE28257ECu64.to_be();
    let w2 = 0xE9DEE72C8F0C0FA6u64.to_be();

    let w1_ = 0x0F2B2C93ED128EDDu64.to_be();
    let w2_ = 0x41009B1B112DFEF3u64.to_be();
    let w0_ = 0x479E76129979DC5Fu64.to_be();

    let m1 = 8;
    let n1 = 53;
    let m2 = 14;
    let n2 = 1;

    let mut s: [u64; 24] = [0; 24];
    s[0] = w0;
    s[1] = w1;
    s[2] = w2;
    bash_s(&mut s, 0, 1, 2, m1, n1, m2, n2);

    assert_eq!(w0_, s[0]);
    assert_eq!(w1_, s[1]);
    assert_eq!(w2_, s[2]);
}

#[inline]
#[allow(dead_code)]
fn bash_r(s: &mut [u64; 24], p: &dyn Fn(u64) -> u64, p_next: &dyn Fn(u64) -> u64, i: u8) {
    bash_s(
        s,
        p(0) as usize,
        p(8) as usize,
        p(16) as usize,
        8,
        53,
        14,
        1,
    );
    bash_s(
        s,
        p(1) as usize,
        p(9) as usize,
        p(17) as usize,
        56,
        51,
        34,
        7,
    );
    bash_s(
        s,
        p(2) as usize,
        p(10) as usize,
        p(18) as usize,
        8,
        37,
        46,
        49,
    );
    bash_s(
        s,
        p(3) as usize,
        p(11) as usize,
        p(19) as usize,
        56,
        3,
        2,
        23,
    );
    bash_s(
        s,
        p(4) as usize,
        p(12) as usize,
        p(20) as usize,
        8,
        21,
        14,
        33,
    );
    bash_s(
        s,
        p(5) as usize,
        p(13) as usize,
        p(21) as usize,
        56,
        19,
        34,
        39,
    );
    bash_s(
        s,
        p(6) as usize,
        p(14) as usize,
        p(22) as usize,
        8,
        5,
        46,
        17,
    );
    bash_s(
        s,
        p(7) as usize,
        p(15) as usize,
        p(23) as usize,
        56,
        35,
        2,
        55,
    );
    s[p_next(23) as usize] ^= C[(i - 1) as usize];
}

macro_rules! bash_r {
    ($s: expr, $p:expr,$p_next:expr, $i:expr) => {{
        bash_s(
            $s,
            $p(0) as usize,
            $p(8) as usize,
            $p(16) as usize,
            8,
            53,
            14,
            1,
        );
        bash_s(
            $s,
            $p(1) as usize,
            $p(9) as usize,
            $p(17) as usize,
            56,
            51,
            34,
            7,
        );
        bash_s(
            $s,
            $p(2) as usize,
            $p(10) as usize,
            $p(18) as usize,
            8,
            37,
            46,
            49,
        );
        bash_s(
            $s,
            $p(3) as usize,
            $p(11) as usize,
            $p(19) as usize,
            56,
            3,
            2,
            23,
        );
        bash_s(
            $s,
            $p(4) as usize,
            $p(12) as usize,
            $p(20) as usize,
            8,
            21,
            14,
            33,
        );
        bash_s(
            $s,
            $p(5) as usize,
            $p(13) as usize,
            $p(21) as usize,
            56,
            19,
            34,
            39,
        );
        bash_s(
            $s,
            $p(6) as usize,
            $p(14) as usize,
            $p(22) as usize,
            8,
            5,
            46,
            17,
        );
        bash_s(
            $s,
            $p(7) as usize,
            $p(15) as usize,
            $p(23) as usize,
            56,
            35,
            2,
            55,
        );
        $s[$p_next(23) as usize] ^= C[($i - 1) as usize];
    }};
}

#[inline]
pub fn bash_f0(s: &mut [u64; 24]) {
    bash_r!(s, p0, p1, 1);
    bash_r!(s, p1, p2, 2);
    bash_r!(s, p2, p3, 3);
    bash_r!(s, p3, p4, 4);
    bash_r!(s, p4, p5, 5);
    bash_r!(s, p5, p0, 6);
    bash_r!(s, p0, p1, 7);
    bash_r!(s, p1, p2, 8);
    bash_r!(s, p2, p3, 9);
    bash_r!(s, p3, p4, 10);
    bash_r!(s, p4, p5, 11);
    bash_r!(s, p5, p0, 12);
    bash_r!(s, p0, p1, 13);
    bash_r!(s, p1, p2, 14);
    bash_r!(s, p2, p3, 15);
    bash_r!(s, p3, p4, 16);
    bash_r!(s, p4, p5, 17);
    bash_r!(s, p5, p0, 18);
    bash_r!(s, p0, p1, 19);
    bash_r!(s, p1, p2, 20);
    bash_r!(s, p2, p3, 21);
    bash_r!(s, p3, p4, 22);
    bash_r!(s, p4, p5, 23);
    bash_r!(s, p5, p0, 24);
}

#[inline]
pub fn bash_f(s: &mut [u8; 192]) {
    if cfg!(feature = "go-faster") {
        let x: *mut [u64; 24] = s.as_mut_ptr() as *mut [u64; 24];
        bash_f0(unsafe { x.as_mut().unwrap() });
    } else {
        let mut s1: [u64; 24] = [0; 24];
        for (dst, src) in s1.iter_mut().zip(s.chunks_exact(8)) {
            *dst = u64::from_le_bytes([
                src[0], src[1], src[2], src[3], src[4], src[5], src[6], src[7],
            ]);
        }
        bash_f0(&mut s1);
        for (src, dst) in s1.iter().zip(s.chunks_exact_mut(8)) {
            dst.clone_from_slice(&src.to_le_bytes());
        }
    }
}

/// A2. Test
#[test]
fn bash_f0_test() {
    let mut s = [
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

    let s_ = [
        0x8FE727775EA7F140u64.to_be(),
        0xB95BB6A200CBB28Cu64.to_be(),
        0x7F0809C0C0BC68B7u64.to_be(),
        0xDC5AEDC841BD94E4u64.to_be(),
        0x03630C301FC255DFu64.to_be(),
        0x5B67DB53EF65E376u64.to_be(),
        0xE8A4D797A6172F22u64.to_be(),
        0x71BA48093173D329u64.to_be(),
        0xC3502AC946767326u64.to_be(),
        0xA2891971392D3F70u64.to_be(),
        0x89959F5D61621238u64.to_be(),
        0x655975E00E2132A0u64.to_be(),
        0xD5018CEEDB17731Cu64.to_be(),
        0xCD88FC50151D37C0u64.to_be(),
        0xD4A3359506AEDC2Eu64.to_be(),
        0x6109511E7703AFBBu64.to_be(),
        0x014642348D8568AAu64.to_be(),
        0x1A5D9868C4C7E6DFu64.to_be(),
        0xA756B1690C7C2608u64.to_be(),
        0xA2DC136F5997AB8Fu64.to_be(),
        0xBB3F4D9F033C87CAu64.to_be(),
        0x6070E117F099C409u64.to_be(),
        0x4972ACD9D976214Bu64.to_be(),
        0x7CED8E3F8B6E058Eu64.to_be(),
    ];
    bash_f0(&mut s);
    assert_eq!(s, s_);
}

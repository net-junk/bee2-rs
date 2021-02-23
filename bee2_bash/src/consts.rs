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
fn P0(x: u64) -> u64 {
    x
}

#[inline]
fn P1(x: u64) -> u64 {
    if x < 8 {
        8 + (x + 2 * (x & 1) + 7) % 8
    } else if x < 16 {
        8 + (x ^ 1)
    } else {
        (5 * x + 6) % 8
    }
}

#[inline]
fn P2(x: u64) -> u64 {
    P1(P1(x))
}

#[inline]
fn P3(x: u64) -> u64 {
    8 * (x / 8) + (x % 8 + 4) % 8
}

#[inline]
fn P4(x: u64) -> u64 {
    P1(P3(x))
}

#[inline]
fn P5(x: u64) -> u64 {
    P2(P3(x))
}

#[inline]
fn bashS(s: &mut [u64; 24], w0_: usize, w1_: usize, w2_: usize, m1: u8, n1: u8, m2: u8, n2: u8) {
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
fn bashS_test() {
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
    bashS(&mut s, 0, 1, 2, m1, n1, m2, n2);

    assert_eq!(w0_, s[0]);
    assert_eq!(w1_, s[1]);
    assert_eq!(w2_, s[2]);
}

#[inline]
fn bashR(s: &mut [u64; 24], p: &dyn Fn(u64) -> u64, p_next: &dyn Fn(u64) -> u64, i: u8) {
    bashS(
        s,
        p(0) as usize,
        p(8) as usize,
        p(16) as usize,
        8,
        53,
        14,
        1,
    );
    bashS(
        s,
        p(1) as usize,
        p(9) as usize,
        p(17) as usize,
        56,
        51,
        34,
        7,
    );
    bashS(
        s,
        p(2) as usize,
        p(10) as usize,
        p(18) as usize,
        8,
        37,
        46,
        49,
    );
    bashS(
        s,
        p(3) as usize,
        p(11) as usize,
        p(19) as usize,
        56,
        3,
        2,
        23,
    );
    bashS(
        s,
        p(4) as usize,
        p(12) as usize,
        p(20) as usize,
        8,
        21,
        14,
        33,
    );
    bashS(
        s,
        p(5) as usize,
        p(13) as usize,
        p(21) as usize,
        56,
        19,
        34,
        39,
    );
    bashS(
        s,
        p(6) as usize,
        p(14) as usize,
        p(22) as usize,
        8,
        5,
        46,
        17,
    );
    bashS(
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

#[inline]
pub fn bashF0(s: &mut [u64; 24]) {
    bashR(s, &P0, &P1, 1);
    bashR(s, &P1, &P2, 2);
    bashR(s, &P2, &P3, 3);
    bashR(s, &P3, &P4, 4);
    bashR(s, &P4, &P5, 5);
    bashR(s, &P5, &P0, 6);
    bashR(s, &P0, &P1, 7);
    bashR(s, &P1, &P2, 8);
    bashR(s, &P2, &P3, 9);
    bashR(s, &P3, &P4, 10);
    bashR(s, &P4, &P5, 11);
    bashR(s, &P5, &P0, 12);
    bashR(s, &P0, &P1, 13);
    bashR(s, &P1, &P2, 14);
    bashR(s, &P2, &P3, 15);
    bashR(s, &P3, &P4, 16);
    bashR(s, &P4, &P5, 17);
    bashR(s, &P5, &P0, 18);
    bashR(s, &P0, &P1, 19);
    bashR(s, &P1, &P2, 20);
    bashR(s, &P2, &P3, 21);
    bashR(s, &P3, &P4, 22);
    bashR(s, &P4, &P5, 23);
    bashR(s, &P5, &P0, 24);
}

/// A2. Test
#[test]
fn bashF0_test() {
    let mut S = [
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

    let S_ = [
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
    bashF0(&mut S);
    assert_eq!(S, S_);
}

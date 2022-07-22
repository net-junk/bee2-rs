extern crate bee2_traits;

use crate::consts::belt_block_encr;
use crate::consts::belt_block_decr;

#[derive(Clone)]
struct BeltState {
    /// Mode (encryption or decryption)
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

fn belt_ecb_encr(mut buf: impl AsMut<[u8]>, key: & [u32; 8]) {   
    let buffer = buf.as_mut();
    let mut count = buffer.len();
    let mut pos = 0;    

    while count >= 16 {        
        let tmp: *mut [u32; 4] = buffer[pos..pos+16].as_mut_ptr() as *mut [u32; 4];          
        belt_block_encr(unsafe{tmp.as_mut().unwrap()}, & key);            
        pos += 16;    
        count -= 16;               
    }
    
    if count != 0 {
        let (left, right) = buffer.split_at_mut(pos);
        left[pos-16..pos-16+count].swap_with_slice(&mut right[..count]);
        let tmp: *mut [u32; 4] = buffer[pos-16..pos].as_mut_ptr() as *mut [u32; 4];  
        belt_block_encr(unsafe{tmp.as_mut().unwrap()}, & key);
    }  
}

fn belt_ecb_decr(mut buf: impl AsMut<[u8]>, key: & [u32; 8]) {   
    let buffer = buf.as_mut();
    let mut count = buffer.len();
    let mut pos = 0;    

    while count >= 16 {        
        let tmp: *mut [u32; 4] = buffer[pos..pos+16].as_mut_ptr() as *mut [u32; 4];          
        belt_block_decr(unsafe{tmp.as_mut().unwrap()}, & key);            
        pos += 16;    
        count -= 16;               
    }
    
    if count != 0 {
        let (left, right) = buffer.split_at_mut(pos);
        left[pos-16..pos-16+count].swap_with_slice(&mut right[..count]);
        let tmp: *mut [u32; 4] = buffer[pos-16..pos].as_mut_ptr() as *mut [u32; 4];  
        belt_block_decr(unsafe{tmp.as_mut().unwrap()}, & key);
    }  
}

#[test]
fn belt_ecb_encr_tests(){
    let x_then_y: [u32; 12] =  [
        0xB194BAC8u32.to_be(),
        0x0A08F53Bu32.to_be(),
        0x366D008Eu32.to_be(),
        0x584A5DE4u32.to_be(),
        0x8504FA9Du32.to_be(),
        0x1BB6C7ACu32.to_be(),
        0x252E72C2u32.to_be(),
        0x02FDCE0Du32.to_be(),
        0x5BE3D612u32.to_be(),
        0x17B96181u32.to_be(),
        0xFE6786ADu32.to_be(),
        0x716B890Bu32.to_be(),
    ];

    const Y0: [u32; 12] =  [
        0x69CCA1C9u32.to_be(),
        0x3557C9E3u32.to_be(),
        0xD66BC3E0u32.to_be(),
        0xFA88FA6Eu32.to_be(),
        0x5F23102Eu32.to_be(),
        0xF1097107u32.to_be(),
        0x75017F73u32.to_be(),
        0x806DA9DCu32.to_be(),
        0x46FB2ED2u32.to_be(),        
        0xCE771F26u32.to_be(),
        0xDCB5E5D1u32.to_be(),
        0x569F9AB0u32.to_be(),        
    ];   

    const Y1: [u32; 12] =  [
        0x69CCA1C9u32.to_be(),
        0x3557C9E3u32.to_be(),
        0xD66BC3E0u32.to_be(),
        0xFA88FA6Eu32.to_be(),
        0x36F00CFEu32.to_be(),
        0xD6D1CA14u32.to_be(),
        0x98C12798u32.to_be(),
        0xF4BEB207u32.to_be(),
        0x5F23102Eu32.to_be(),        
        0xF1097107u32.to_be(),
        0x75017F73u32.to_be(),
        0x806DA900u32.to_be(),        
    ];   

    const KEY: [u32; 8] = [
        0xE9DEE72Cu32.to_be(),
        0x8F0C0FA6u32.to_be(),
        0x2DDB49F4u32.to_be(),
        0x6F739647u32.to_be(),
        0x06075316u32.to_be(),
        0xED247A37u32.to_be(),
        0x39CBA383u32.to_be(),
        0x03A98BF6u32.to_be(),       
    ];  

    let mut x0_then_y0: [u8; 48] = unsafe { *(x_then_y.as_ptr() as *const [u8; 48]) };
    belt_ecb_encr(&mut x0_then_y0, & KEY);
    assert_eq!(unsafe { *(Y0.as_ptr() as *const [u8; 48]) }, x0_then_y0);  

    let mut x1_then_y0: [u8; 47] = unsafe { *(x_then_y.as_ptr() as *const [u8; 47]) };
    belt_ecb_encr(&mut x1_then_y0, & KEY);
    assert_eq!(unsafe { *(Y1.as_ptr() as *const [u8; 47]) }, x1_then_y0);  
}

#[test]
fn belt_ecb_decr_tests(){
    let y_then_x: [u32; 12] =  [
        0xE12BDC1Au32.to_be(),
        0xE28257ECu32.to_be(),
        0x703FCCF0u32.to_be(),
        0x95EE8DF1u32.to_be(),
        0xC1AB7638u32.to_be(),
        0x9FE678CAu32.to_be(),
        0xF7C6F860u32.to_be(),
        0xD5BB9C4Fu32.to_be(),
        0xF33C657Bu32.to_be(),
        0x637C306Au32.to_be(),
        0xDD4EA779u32.to_be(),
        0x9EB23D31u32.to_be(),
    ];

    const X0: [u32; 12] =  [
        0x0DC53006u32.to_be(),
        0x00CAB840u32.to_be(),
        0xB38448E5u32.to_be(),
        0xE993F421u32.to_be(),        
        0xE55A239Fu32.to_be(),
        0x2AB5C5D5u32.to_be(),
        0xFDB6E81Bu32.to_be(),
        0x40938E2Au32.to_be(),
        0x54120CA3u32.to_be(),
        0xE6E19C7Au32.to_be(),         
        0xD750FC35u32.to_be(),
        0x31DAEAB7u32.to_be(),        
    ];   

    const X1: [u32; 9] =  [
        0x0DC53006u32.to_be(),
        0x00CAB840u32.to_be(),
        0xB38448E5u32.to_be(),
        0xE993F421u32.to_be(),
        0x5780A6E2u32.to_be(),
        0xB69EAFBBu32.to_be(),
        0x258726D7u32.to_be(),
        0xB6718523u32.to_be(),
        0xE55A239Fu32.to_be(),          
    ];   

    const KEY: [u32; 8] = [
        0x92BD9B1Cu32.to_be(),
        0xE5D14101u32.to_be(),
        0x5445FBC9u32.to_be(),
        0x5E4D0EF2u32.to_be(),
        0x682080AAu32.to_be(),
        0x227D642Fu32.to_be(),
        0x2687F934u32.to_be(),
        0x90405511u32.to_be(),       
    ];

    let mut y0_then_x0: [u8; 48] = unsafe { *(y_then_x.as_ptr() as *const [u8; 48]) };
    belt_ecb_decr(&mut y0_then_x0, & KEY);
    assert_eq!(unsafe { *(X0.as_ptr() as *const [u8; 48]) }, y0_then_x0);  

    let mut y1_then_x1: [u8; 36] = unsafe { *(y_then_x.as_ptr() as *const [u8; 36]) };
    belt_ecb_decr(&mut y1_then_x1, & KEY);
    assert_eq!(unsafe { *(X1.as_ptr() as *const [u8; 36]) }, y1_then_x1);
}
// Copyright 2022 John Sahhar

// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:
// 1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.
// 2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#![no_std]

extern crate alloc;
use alloc::{vec::Vec};

const CHARSET: [u8; 32] = [113, 112, 122, 114, 121, 57, 120, 56, 103, 102, 50, 116, 118, 100, 119, 48, 115, 51, 106, 110, 53, 52, 107, 104, 99, 101, 54, 109, 117, 97, 55, 108];
const GENERATOR: [u32; 5] = [0x3b6a57b2, 0x26508e6d, 0x1ea119fa, 0x3d4233dd, 0x2a1462b3];

fn polymod(values: Vec<u8>) -> u32 {
    let mut chk: u32 = 1;
    for v in values.iter() {
        let top: u32 = chk >> 25;
        chk = (chk & 0x1ffffff) << 5;
        chk = chk ^ (*v as u32);
        for i in 0..5 {
            let bit: u32 = top >> i & 1;
            if bit == 1 {
                chk ^= GENERATOR[i];
            }
        }
    }
    return chk;
}

fn hrp_expand(hrp: &Vec<u8>) -> Vec<u8> {
    let mut ret: Vec<u8> = Vec::new();
    let h = hrp.to_ascii_lowercase();
    for c in &h {
        ret.push(*c >> 5);
    }
    ret.push(0);
    for c in &h {
        ret.push(*c & 31);
    }
    return ret;
}

fn verify_checksum(hrp: &Vec<u8>, data: &Vec<u8>) -> bool {
    let mut ret: Vec<u8> = hrp_expand(&hrp);
    for v in data {
        ret.push(*v);
    }
    return polymod(ret) == 1;
}

fn create_checksum(hrp: &Vec<u8>, mut data: Vec<u8>) -> Vec<u8> {
    let mut values: Vec<u8> = hrp_expand(hrp);
    let mut _add: [u8; 6] = [0, 0, 0, 0, 0, 0];
    values.append(&mut data);
    values.append(&mut _add.to_vec());
    let modulo: u32 = polymod(values) ^ 1;
    let mut ret: [u8; 6] = [0,1,2,3,4,5];
    for p in 0..6 {
        let shift: u32 = 5 * (5-p as u32);
        ret[p] = (modulo>>shift) as u8 & 31;
    }
    return ret.to_vec();
}

fn convert_bits(data: &Vec<u8>, frombits: u8, tobits: u8, pad: bool) -> Vec<u8> {
    let mut ret: Vec<u8> = Vec::new();
    let mut acc: u32 = 0;
    let mut bits: u8 = 0;
    let maxv: u8 = if tobits == 5 { 31 } else { 255 };
    for value in data.iter() {
        // Return an empty array if datarange is invalid.
        if (*value as u32) >> (frombits as u32) != 0 {
            return Vec::new();
        }
        acc = (acc<<(frombits as u32)) | (*value as u32);
        bits += frombits;
        while bits >= tobits {
            bits -= tobits;
            ret.push(((acc >> bits) & maxv as u32) as u8);
        }
    }
    if pad {
        if bits > 0 {
            ret.push((acc<<(tobits-bits)) as u8 & maxv)
        }
    } else if bits >= frombits {
        // Improper zero padding, return empty Vec
        return Vec::new();
    } else if (acc<<(tobits-bits)) as u8 & maxv != 0 {
        // Non zero padding, return empty Vec
        return Vec::new();
    }
    return ret;
}

pub fn encode(hrp: &Vec<u8>, data: &Vec<u8>) -> Vec<u8> {
    let values: &Vec<u8> = &convert_bits(data, 8, 5, true); 

    if values.len() == 0 {
        return Vec::new();
    }

    // Too many values
    if hrp.len()+values.len()+7>90 {
        return Vec::new();
    }

    // Invalid HRP
    if hrp.len() < 1 {
        return Vec::new();
    }

    // HRP byte out of valid range
    for c in hrp {
        if *c < 33 || *c > 126 {
            return Vec::new();
        }
    }

    // Mixed case HRP
    if hrp.to_ascii_uppercase() != *hrp && hrp.to_ascii_lowercase() != *hrp {
        return Vec::new();
    }
    
    let is_lower: bool = hrp.to_ascii_lowercase() == *hrp;
    let mut lower = hrp.to_ascii_lowercase().to_vec();
    lower.push(49);

    for p in values.iter() {
        lower.push(CHARSET[*p as usize])
    }

    for p in create_checksum(hrp, values.clone()) {
        lower.push(CHARSET[p as usize])
    }

    if is_lower {
        return lower;
    }
    return lower.to_ascii_uppercase();
}

// Decodes and returns the hrp and data portions of 
// a bech32 encoded string, respectively.
pub fn decode(s: Vec<u8>) -> (Vec<u8>, Vec<u8>) {

    // Invalid length
    if s.len() > 90 {
        return (Vec::new(), Vec::new());
    }
    // Mixed case HRP
    if s.to_ascii_uppercase() != s && s.to_ascii_lowercase() != s {
        return (Vec::new(), Vec::new());
    }

    // Find the last occuring index of 1
    // let pos = s.reverse() .position(|&r| r == 49).unwrap_or(0);
    let mut pos = 0;
    for idx in (0..s.len()-1).rev() {
        if s[idx] == 49 {
            pos = idx;
            break;
        }
    }

    // byte separator at invalid position
    if pos < 1 || pos+7 > s.len() {
        return (Vec::new(), Vec::new());
    }

    let hrp: Vec<u8> = s[..pos].to_vec();
    for c in hrp.iter() {
        // Invalid human-readable data part
        if *c < 33 || *c > 126 {
            return (Vec::new(), Vec::new());
        }
    }

    let lower = s.to_ascii_lowercase();
    let mut data: Vec<u8> = Vec::new();
    for c in lower[pos+1..].iter() {
        let d = CHARSET.iter().position(|&r| r == *c).unwrap_or(usize::MAX);

        // if d == usize::MAX {
        //     return (Vec::new(), Vec::new());
        // }
        data.push(d as u8);
    }
    // Invalid checksum
    if !verify_checksum(&hrp, &data) {
            return (Vec::new(), Vec::new());
    }
    
    return (hrp, convert_bits(&data[..data.len()-6].to_vec(), 5, 8, false));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_encode_decode(input_hrp: &str, input_data: &str, want_encoded: &str) {
        let got_encoded = encode(&input_hrp.as_bytes().to_vec(), &input_data.as_bytes().to_vec());
        assert_eq!(want_encoded.as_bytes().to_vec(), got_encoded);
        let (got_hrp, got_decoded) = decode(got_encoded);
        assert_eq!(got_hrp, input_hrp.as_bytes().to_vec());
        assert_eq!(got_decoded, input_data.as_bytes().to_vec());
    }

    #[test]
    fn test_all() {
        let testcases: [[&str; 3]; 2] = [
            [ "A12", "UEL5L", "A12124Z5CD2V4GNT7N" ],
            [ "aaa", "you look beautiful today", "aaa109hh2grvdahkkgrzv4sh2arfve6kcgr5dajxz7g2q3d2y" ]
        ];

        for testcase in testcases {
            test_encode_decode(testcase[0], testcase[1], testcase[2])
        }

    }
}
#![allow(dead_code)]

use std::{ops::{BitAndAssign, BitOrAssign}, usize};

#[derive(Clone)]
pub struct Vec3vl {
    pub bits: u32,
    pub avec: Vec::<u32>,
    pub bvec: Vec::<u32>
} 

fn wordnum(n: u32) -> u32 { n / 32 }
fn bitnum(n: u32) -> u32 { n & 0x1f }

fn to_hex_internal(start: u32, bits: u32, avec: &Vec<u32>, bvec: &Vec<u32>) -> String {
    let mut out: Vec<char> = vec![];
    let mut bit = 0;
    let mut k = start as usize; 
    
    while bit < bits {
        let a = String::from("00000000") + &format!("{:x}", avec[k]);
        let x = avec[k] ^ bvec[k];
        k += 1;
        let mut b= 0;
        while b < 8 && bit < bits {
            if x & (0xf << (4 * b)) != 0 {
                out.push('x');
            } else {
                out.push(a.chars().nth(a.len() - 1 - b).unwrap());
            }

            b += 1;
            bit += 4;
        }
    }
    
    out.iter().rev().collect()
}

impl Vec3vl {
    pub fn new(bits: u32, avec: Vec<u32>, bvec: Vec<u32>) -> Vec3vl {
        Vec3vl { bits, avec, bvec }
    }

    pub fn make_bool(bits: u32, init: bool) -> Vec3vl {
        let (iva, ivb) = if init {
            (!0u32, !0u32)
        } else {
            (0, 0)
        };
        let words = ((bits + 31) / 32) as usize;
        Vec3vl::new(bits, vec![iva; words], vec![ivb; words])
    }

    pub fn make_int(bits: u32, init: i32) -> Vec3vl {
        let (iva, ivb) = match init {
            1  => (!0u32, !0u32),
            0  => ( 0u32, !0u32),
            -1 => ( 0u32,  0u32),
            _ => panic!()
        };
        let words = ((bits + 31) / 32) as usize;
        Vec3vl::new(
            bits,
            vec![iva; words],
            vec![ivb; words]
        )
    }

    pub fn make_str(bits: u32, init: &str) -> Vec3vl {
        let (iva, ivb) = match init {
            "1" => (!0u32, !0u32),
            "0" => ( 0u32,  0u32),
            "x" => ( 0u32, !0u32),
            _ => panic!()
        };
        let words = ((bits + 31) / 32) as usize;
        Vec3vl::new(
            bits,
            vec![iva; words],
            vec![ivb; words]
        )
    }

    /*pub fn from_binary(number: String, bits: u32) -> Vec3vl {
        
    }*/

    pub fn zeros(bits: u32) -> Vec3vl {
        Vec3vl::make_int(bits, -1)
    }

    pub fn ones(bits: u32) -> Vec3vl {
        Vec3vl::make_int(bits, 1)
    }

    pub fn xes(bits: u32) -> Vec3vl {
        Vec3vl::make_int(bits, 0)
    }

    pub fn concat(vs: &mut Vec<Vec3vl>) -> Vec3vl {
        let submits = vs.iter().fold(0, |acc, v| { acc + v.bits });
        let words = (submits + 31) >> 5;
        let mut bits = 0;
        let mut idx: i32 = -1;
        let mut avec = vec![0; words as usize];
        let mut bvec = vec![0; words as usize];

        for v in vs {
            v.normalize();
            if bitnum(bits) == 0 {
                for k in 0..v.avec.len() {
                    avec[(idx + 1) as usize + k] = v.avec[k];
                    bvec[(idx + 1) as usize + k] = v.bvec[k];
                }
                bits += v.bits;
                idx += ((v.bits + 31) >> 5) as i32;
            } else {
                for k in 0..v.avec.len() {
                    avec[idx as usize] |= v.avec[k] << bitnum(bits);
                    bvec[idx as usize] |= v.bvec[k] << bitnum(bits);
                    idx += 1;
                    if idx == words as i32 { break; }
                    avec[idx as usize] = v.avec[k] >> (32 - bitnum(bits));
                    bvec[idx as usize] = v.bvec[k] >> (32 - bitnum(bits));
                }
                bits += v.bits;
                if (idx + 1) as u32 > (bits + 31) >> 5 {
                    idx -= 1;
                }
            }
        }
        Vec3vl::new(bits, avec, bvec)
    }

    pub fn slice(&self, start: u32, end: u32) -> Vec3vl {
        let mut e = if end > self.bits { self.bits as usize } else { end as usize };
        let s = start as usize;
        if s > end as usize { e = start as usize; }

        if bitnum(s as u32) == 0 {
            let st = s >> 5;
            let et = (e + 31) >> 5;
            let avec = self.avec[st..et].iter().map(|v| *v).collect::<Vec<u32>>();
            let bvec = self.bvec[st..et].iter().map(|v| *v).collect::<Vec<u32>>();
            Vec3vl::new((e - s) as u32, avec, bvec)
        } else {
            let words = (e - s + 31) >> 5;
            let mut avec = vec![0; words];
            let mut bvec = vec![0; words];
            let mut k = 0usize;
            avec[k] = self.avec[(s >> 5) as usize] >> s;
            bvec[k] = self.bvec[(s >> 5) as usize] >> s;

            let mut i = ((s >> 5) + 1) as usize;
            while i <= (e >> 5) as usize {
                avec[k] |= self.avec[i] << -((s % 32) as i32);
                bvec[k] |= self.bvec[i] << -((s % 32) as i32);
                k += 1;
                if k == words { break; }
                avec[k] = self.avec[i] >> (32 - (s % 32));
                bvec[k] = self.bvec[i] >> (32 - (s % 32));
                i += 1;
            }
            Vec3vl::new((e - s) as u32, avec, bvec)
        }
    }

    fn msb(&self) -> u32 {
        self.get(self.bits - 1)
    }

    fn lsb(&self) -> u32 {
        self.get(0)
    }

    pub fn get(&self, n: u32) -> u32 {
        let bn = bitnum(n) as usize;
        let wn = wordnum(n) as usize;
        let a = (self.avec[wn] >> bn) & 1;
        let b = (self.bvec[wn] >> bn) & 1;
        return a + b - 1;
    }

    pub fn is_high(&self) -> bool {
        if self.bits == 0 { return true }
        let lastmask = self.lastmask();
        let vechigh = |v: &Vec<u32>| -> bool {
            v.iter().all(|x| !x == 0) && 
            (v.last().unwrap() & lastmask) == lastmask
        };
        vechigh(&self.avec) && vechigh(&self.bvec)
    }

    pub fn is_low(&self) -> bool {
        if self.bits == 0 { return true }
        let lastmask = self.lastmask();
        let veclow = |v: &Vec<u32>| -> bool {
            v.iter().all(|x| *x == 0) && 
            (v.last().unwrap() & lastmask) == 0
        };
        veclow(&self.avec) && veclow(&self.bvec)
    }

    pub fn is_defined(&self) -> bool {
        if self.bits == 0 { return true }
        let mut dvec = self.avec.iter()
            .zip(self.bvec.iter())
            .map(|(a, b)| *a ^ *b)
            .collect::<Vec<u32>>();
        dvec.last_mut().unwrap().bitor_assign(!self.lastmask());
        !dvec.iter().any(|x| !x == 0)
    }

    pub fn is_fully_defined(&self) -> bool {
        if self.bits == 0 { return true }
        let mut dvec = self.avec.iter()
            .zip(self.bvec.iter())
            .map(|(a, b)| *a ^ *b)
            .collect::<Vec<u32>>();
        dvec.last_mut().unwrap().bitand_assign(self.lastmask());
        !dvec.iter().any(|x| *x != 0)
    }

    pub fn and(&self, v: Vec3vl) -> Vec3vl {
        if self.bits != v.bits {
            panic!()
        }
        Vec3vl::new(
            self.bits, 
            self.avec.iter().zip(v.avec).map(|(a, b)| a & b ).collect(),
            self.bvec.iter().zip(v.bvec).map(|(a, b)| a & b ).collect()
        )
    }

    pub fn or(&self, v: Vec3vl) -> Vec3vl {
        if self.bits != v.bits {
            panic!()
        }
        Vec3vl::new(
            self.bits, 
            self.avec.iter().zip(v.avec).map(|(a, b)| a | b ).collect(),
            self.bvec.iter().zip(v.bvec).map(|(a, b)| a | b ).collect()
        )
    }

    pub fn xor(&self, v: Vec3vl) -> Vec3vl {
        if self.bits != v.bits {
            panic!()
        }
        Vec3vl::new(
            self.bits,
            self.avec.iter().zip(self.bvec.clone()).zip(
                v.avec.iter().zip(v.bvec.clone()))
                .map(|((a1, a2), (b1, b2)) | {
                (a1 | b1) & (a2 ^ b2)
            }).collect(), 
            self.avec.iter().zip(self.bvec.clone()).zip(
                v.avec.iter().zip(v.bvec))
                .map(|((a1, a2), (b1, b2))| {
                (a1 & b1) ^ (a2 | b2)
            }).collect()
        )
    }

    pub fn nand(&self, v: Vec3vl) -> Vec3vl {
        if self.bits != v.bits {
            panic!()
        }
        Vec3vl::new(
            self.bits,
            self.bvec.iter().zip(v.bvec).map(|(a, b)| !(a & b)).collect(),
            self.avec.iter().zip(v.avec).map(|(a, b)| !(a & b)).collect()
        )
    }

    pub fn nor(&self, v: Vec3vl) -> Vec3vl {
        if self.bits != v.bits {
            panic!()
        }
        Vec3vl::new(
            self.bits, 
            self.bvec.iter().zip(v.bvec).map(|(a, b)| !(a | b)).collect(), 
            self.avec.iter().zip(v.avec).map(|(a, b)| !(a | b)).collect()
        )
    }

    pub fn xnor(&self, v: Vec3vl) -> Vec3vl {
        if self.bits != v.bits {
            panic!()
        }
        Vec3vl::new(
            self.bits, 
            self.avec.iter().zip(self.bvec.iter()).zip(
                v.avec.iter().zip(v.bvec.iter())).map(|((a1, a2), (b1, b2))| {
                    !((a1 & b1) ^ (a2 | b2))
                }).collect(), 
            self.avec.iter().zip(self.bvec.iter()).zip(
                v.avec.iter().zip(v.bvec.iter())).map(|((a1, a2), (b1, b2))| {
                    !((a1 | b1) & (a2 ^ b2))
                }).collect()
        )
    }

    pub fn not(&self) -> Vec3vl {
        Vec3vl::new(
            self.bits,
            self.avec.iter().map(|v| !v).collect(),
            self.bvec.iter().map(|v| !v).collect()
        )
    }

    pub fn to_hex(&mut self) -> String {
        self.normalize();
        to_hex_internal(0, self.bits, &self.avec, &self.bvec)
    }

    pub fn from_hex(data: String, len: Option<usize>) -> Vec3vl {
        let nbits = if let Some(s) = len { s } else { data.len() * 4 };
        let words = (nbits + 31) >> 5;

        let mut r = data.chars()
            .rev().collect::<String>()
            .as_bytes()
            .chunks(8)
            .map(|b: &[u8]| {
                let mut v = b.to_vec();
                v.reverse();
                u32::from_str_radix(&String::from_utf8(v).unwrap(), 16).unwrap()
            }).collect::<Vec<u32>>();


        let len = r.len();
        if len < words {
            r = vec![r, vec![0, (words - len) as u32]].concat();
        } else if words < len {
            let _ = r.split_off(words);
        }

        Vec3vl { bits: nbits as u32, avec: r.clone(), bvec: r }
    }

    pub fn normalize(&mut self) {
        let lastmask = self.lastmask();
        let len_a = self.avec.len() - 1;
        let len_b = self.bvec.len() - 1;

        self.avec[len_a] &= lastmask;
        self.bvec[len_b] &= lastmask;
    }

    fn lastmask(&self) -> u32 {
        !0u32 >> (32 - bitnum(self.bits))
    }
}

impl PartialEq for Vec3vl {
    fn eq(&self, other: &Self) -> bool {
        self.bits == other.bits && self.avec == other.avec && self.bvec == other.bvec
    }
}

#[cfg(test)]
mod vector3vl_tests {
    use super::Vec3vl;

    #[test]
    fn test_and_tt() {
        assert!(Vec3vl::ones(1).and(Vec3vl::ones(1)) == Vec3vl::ones(1));
    }

    #[test]
    fn test_and_tf() {
        assert!(Vec3vl::ones(1).and(Vec3vl::zeros(1)) == Vec3vl::zeros(1));
    }

    #[test]
    fn test_and_ft() {
        assert!(Vec3vl::zeros(1).and(Vec3vl::ones(1)) == Vec3vl::zeros(1));
    }

    #[test]
    fn test_and_ff() {
        assert!(Vec3vl::zeros(1).and(Vec3vl::zeros(1)) == Vec3vl::zeros(1));
    }

    #[test]
    fn test_or_tt() {
        assert!(Vec3vl::ones(1).or(Vec3vl::ones(1)) == Vec3vl::ones(1));
    }
     
    #[test]
    fn test_or_tf() {
        assert!(Vec3vl::ones(1).or(Vec3vl::zeros(1)) == Vec3vl::ones(1));
    }

    #[test]
    fn test_or_ft() {
        assert!(Vec3vl::zeros(1).or(Vec3vl::ones(1)) == Vec3vl::ones(1));
    }

    #[test]
    fn test_or_ff() {
        assert!(Vec3vl::zeros(1).or(Vec3vl::zeros(1)) == Vec3vl::zeros(1));
    }

    #[test]
    fn test_xor_tt() {
        assert!(Vec3vl::ones(1).xor(Vec3vl::ones(1)) == Vec3vl::zeros(1));
    }

    #[test]
    fn test_xor_tf() {
        assert!(Vec3vl::ones(1).xor(Vec3vl::zeros(1)) == Vec3vl::ones(1));
    }

    #[test]
    fn test_xor_ft() {
        assert!(Vec3vl::zeros(1).xor(Vec3vl::ones(1)) == Vec3vl::ones(1));
    }

    #[test]
    fn test_xor_ff() {
        assert!(Vec3vl::zeros(1).xor(Vec3vl::zeros(1)) == Vec3vl::zeros(1));
    }

    #[test]
    fn test_nand_tt() {
        assert!(Vec3vl::ones(1).nand(Vec3vl::ones(1)) == Vec3vl::zeros(1));
    }

    #[test]
    fn test_nand_tf() {
        assert!(Vec3vl::ones(1).nand(Vec3vl::zeros(1)) == Vec3vl::ones(1));
    }

    #[test]
    fn test_nand_ft() {
        assert!(Vec3vl::zeros(1).nand(Vec3vl::ones(1)) == Vec3vl::ones(1));
    }

    #[test]
    fn test_nand_ff() {
        assert!(Vec3vl::zeros(1).nand(Vec3vl::zeros(1)) == Vec3vl::ones(1));
    }

    #[test]
    fn test_nor_tt() {
        assert!(Vec3vl::ones(1).nor(Vec3vl::ones(1)) == Vec3vl::zeros(1));
    }

    #[test]
    fn test_nor_tf() {
        assert!(Vec3vl::ones(1).nor(Vec3vl::zeros(1)) == Vec3vl::zeros(1));
    }

    #[test]
    fn test_nor_ft() {
        assert!(Vec3vl::zeros(1).nor(Vec3vl::ones(1)) == Vec3vl::zeros(1));
    }

    #[test]
    fn test_nor_ff() {
        assert!(Vec3vl::zeros(1).nor(Vec3vl::zeros(1)) == Vec3vl::ones(1));
    }

    #[test]
    fn test_xnor_tt() {
        assert!(Vec3vl::ones(1).xnor(Vec3vl::ones(1)) == Vec3vl::ones(1));
    }

    #[test]
    fn test_xnor_tf() {
        assert!(Vec3vl::ones(1).xnor(Vec3vl::zeros(1)) == Vec3vl::zeros(1));
    }

    #[test]
    fn test_xnor_ft() {
        assert!(Vec3vl::zeros(1).xnor(Vec3vl::ones(1)) == Vec3vl::zeros(1));
    }

    #[test]
    fn test_xnor_ff() {
        assert!(Vec3vl::zeros(1).xnor(Vec3vl::zeros(1)) == Vec3vl::ones(1));
    }
}

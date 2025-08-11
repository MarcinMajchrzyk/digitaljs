use std::cmp::Ordering;
use std::ops::{BitAndAssign, BitOrAssign};

use crate::cell_arith::BigInt;
use crate::js_types::JsVec3vl;

#[derive(Clone)]
pub struct Vec3vl {
    pub bits: u32,
    pub avec: Vec::<u32>,
    pub bvec: Vec::<u32>
} 

fn wordnum(n: u32) -> u32 { n / 32 }
fn bitnum(n: u32) -> u32 { n & 0x1f }

type RefPair<'a> = (&'a u32, &'a u32);

impl Vec3vl {
    fn to_hex_internal(start: u32, bits: u32, avec: &[u32], bvec: &[u32]) -> String {
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

    fn zip(f: fn(RefPair) -> u32, a: &[u32], b: &[u32]) -> Vec<u32> {
        a.iter().zip(b.iter()).map(f).collect()
    }

    fn zip4(f: fn((RefPair, RefPair)) -> u32, a1: &[u32], b1: &[u32], a2: &[u32], b2: &[u32]) -> Vec<u32> {
        a1.iter().zip(b1.iter()).zip(a2.iter().zip(b2.iter())).map(f).collect()
    }

    fn bitfold(f: fn(u32, &u32) -> u32, a: &[u32], lastmask: u32, neutral: u32) -> u32 {
        if a.is_empty() {
            return if neutral == 1 { 1 } else { 0 }
        }

        let mut acc = *a.last().unwrap();
        if neutral == 1 { acc |= !lastmask; }
        else { acc &= lastmask; }

        a.iter().fold(acc, f);
        acc = f(acc, &(acc >> 16));
        acc = f(acc, &(acc >> 8));
        acc = f(acc, &(acc >> 4));
        acc = f(acc, &(acc >> 2));
        acc = f(acc, &(acc >> 1));

        acc & 1
    }

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

    pub fn make_int(bits: u32, init: i32) -> Result<Vec3vl, String> {
        let (iva, ivb) = match init {
            1  => (!0u32, !0u32),
            0  => ( 0u32, !0u32),
            -1 => ( 0u32,  0u32),
            _ => return Err(format!("Expected -1,0,1 got {}", init))
        };
        let words = ((bits + 31) / 32) as usize;
        Ok(Vec3vl::new(
            bits,
            vec![iva; words],
            vec![ivb; words]
        ))
    }

    pub fn make_str(bits: u32, init: &str) -> Result<Vec3vl, String> {
        let (iva, ivb) = match init {
            "1" => (!0u32, !0u32),
            "0" => ( 0u32,  0u32),
            "x" => ( 0u32, !0u32),
            _ => return Err(format!("Expected 1,0,x got {}", init))
        };
        let words = ((bits + 31) / 32) as usize;
        Ok(Vec3vl::new(
            bits,
            vec![iva; words],
            vec![ivb; words]
        ))
    }

    pub fn zeros(bits: u32) -> Vec3vl {
        Vec3vl::make_int(bits, -1).unwrap()
    }

    pub fn ones(bits: u32) -> Vec3vl {
        Vec3vl::make_int(bits, 1).unwrap()
    }

    pub fn xes(bits: u32) -> Vec3vl {
        Vec3vl::make_int(bits, 0).unwrap()
    }

    pub fn concat(vs: &mut Vec<Vec3vl>) -> Result<Vec3vl, String> {
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
        Ok(Vec3vl::new(bits, avec, bvec))
    }

    pub fn slice(&self, s: u32, e: u32) -> Result<Vec3vl, String> {
        let mut end = if e > self.bits { self.bits as usize } else { e as usize };
        let start = s as usize;
        if start > end { end = start; }

        if bitnum(start as u32) == 0 {
            let st = start >> 5;
            let et = (end + 31) >> 5;
            let avec = self.avec[st..et].to_vec();
            let bvec = self.bvec[st..et].to_vec();
            Ok(Vec3vl::new((end - start) as u32, avec, bvec))
        } else {
            let words = (end - start + 31) >> 5;
            let mut avec = vec![0; words];
            let mut bvec = vec![0; words];
            let mut k = 0usize;
            avec[k] = self.avec[start >> 5] >> start;
            bvec[k] = self.bvec[start >> 5] >> start;

            let mut i = (start >> 5) + 1;
            while i <= (end >> 5) {
                avec[k] |= self.avec[i] << -((start % 32) as i32);
                bvec[k] |= self.bvec[i] << -((start % 32) as i32);
                k += 1;
                if k == words { break; }
                avec[k] = self.avec[i] >> (32 - (start % 32));
                bvec[k] = self.bvec[i] >> (32 - (start % 32));
                i += 1;
            }
            Ok(Vec3vl::new((end - start) as u32, avec, bvec))
        }
    }

    pub fn msb(&self) -> i32 {
        self.get(self.bits - 1)
    }

    pub fn lsb(&self) -> i32 {
        self.get(0)
    }

    fn get(&self, n: u32) -> i32 {
        let bn = bitnum(n) as usize;
        let wn = wordnum(n) as usize;
        let a = (self.avec[wn] >> bn) & 1;
        let b = (self.bvec[wn] >> bn) & 1;
        a as i32 + b as i32 - 1
    }

    pub fn get_number(&mut self) -> Result<u32, String> {
        if !self.is_fully_defined() {
            Err("Attempting to get number from undefined signal".to_string())
        } else if self.bits > 32 {
            Err("Attempting to get number from larger signal".to_string())
        } else {
            self.normalize();
            Ok(self.avec[0])
        }
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
        let mut dvec = Vec3vl::zip(|(a, b)| { a & b }, &self.avec, &self.bvec);
        dvec.last_mut().unwrap().bitor_assign(!self.lastmask());
        !dvec.iter().any(|x| !x == 0)
    }

    pub fn is_fully_defined(&self) -> bool {
        if self.bits == 0 { return true }
        let mut dvec = Vec3vl::zip(|(a, b)| { a ^ b }, &self.avec, &self.bvec);
        dvec.last_mut().unwrap().bitand_assign(self.lastmask());
        !dvec.iter().any(|x| *x != 0)
    }

    pub fn and(&self, v: &Vec3vl) -> Result<Vec3vl, String> {
        if self.bits != v.bits {
            return Err(format!("And operation on different length bit vectors ({} and {})", self.bits, v.bits));  
        }
        Ok(Vec3vl::new(
            self.bits, 
            Vec3vl::zip(|(a, b)| a & b, &self.avec, &v.avec),
            Vec3vl::zip(|(a, b)| a & b, &self.bvec, &v.bvec)
        ))
    }

    pub fn or(&self, v: &Vec3vl) -> Result<Vec3vl, String> {
        if self.bits != v.bits {
            return Err(format!("Or operation on different length bit vectors ({} and {})", self.bits, v.bits));  
        }
        Ok(Vec3vl::new(
            self.bits, 
            Vec3vl::zip(|(a, b)| { a | b }, &self.avec, &v.avec),
            Vec3vl::zip(|(a, b)| { a | b }, &self.bvec, &v.bvec)
        ))
    }

    pub fn xor(&self, v: &Vec3vl) -> Result<Vec3vl, String> {
        if self.bits != v.bits {
            return Err(format!("Xor operation on different length bit vectors ({} and {})", self.bits, v.bits));  
        }
        Ok(Vec3vl::new(
            self.bits,
            Vec3vl::zip4(|((a1, b1), (a2, b2))| { (a1 | a2) & (b1 ^ b2) }, 
                &v.avec, &v.bvec, &self.avec, &self.bvec),
            Vec3vl::zip4(|((a1, a2), (b1, b2))| { (a1 & b1) ^ (a2 | b2) }, 
                &v.avec, &v.bvec, &self.avec, &self.bvec)
        ))
    }

    pub fn nand(&self, v: &Vec3vl) -> Result<Vec3vl, String> {
        if self.bits != v.bits {
            return Err(format!("Nand operation on different length bit vectors ({} and {})", self.bits, v.bits));  
        }
        Ok(Vec3vl::new(
            self.bits,
            Vec3vl::zip(|(a, b)| { !(a & b) }, &self.bvec, &v.bvec),
            Vec3vl::zip(|(a, b)| { !(a & b) }, &self.avec, &v.avec)
        ))
    }

    pub fn nor(&self, v: &Vec3vl) -> Result<Vec3vl, String> {
        if self.bits != v.bits {
            return Err(format!("Nor operation on different length bit vectors ({} and {})", self.bits, v.bits));  
        }
        Ok(Vec3vl::new(
            self.bits, 
            Vec3vl::zip(|(a, b)| { !(a | b) }, &self.bvec, &v.bvec),
            Vec3vl::zip(|(a, b)| { !(a | b) }, &self.avec, &v.avec)
        ))
    }

    pub fn xnor(&self, v: &Vec3vl) -> Result<Vec3vl, String> {
        if self.bits != v.bits {
            return Err(format!("Nxor operation on different length bit vectors ({} and {})", self.bits, v.bits));  
        }
        Ok(Vec3vl::new(
            self.bits, 
            Vec3vl::zip4(|((a1, a2), (b1, b2))| { !((a1 & b1) ^ (a2 | b2)) }, 
                &self.avec, &self.bvec, &v.avec, &v.bvec),
            Vec3vl::zip4(|((a1, a2), (b1, b2))| { !((a1 | b1) & (a2 ^ b2)) }, 
                &self.avec, &self.bvec, &v.avec, &v.bvec)
        ))
    }

    pub fn not(&self) -> Vec3vl {
        Vec3vl::new(
            self.bits,
            self.avec.iter().map(|v| !v).collect(),
            self.bvec.iter().map(|v| !v).collect()
        )
    }

    pub fn xmask(&self) -> Vec3vl {
        let v = Vec3vl::zip(|(a, b)| { a ^ b }, &self.avec, &self.bvec);
        Vec3vl { bits: self.bits, avec: v.clone(), bvec: v }
    }

    pub fn reduce_and(&self) -> Vec3vl {
        Vec3vl { 
            bits: 1, 
            avec: vec![Vec3vl::bitfold(|a, b| { a & b }, &self.avec, self.lastmask(), 1)], 
            bvec: vec![Vec3vl::bitfold(|a, b| { a & b }, &self.bvec, self.lastmask(), 1)]
        }
    }

    pub fn reduce_or(&self) -> Vec3vl {
        Vec3vl { 
            bits: 1, 
            avec: vec![Vec3vl::bitfold(|a, b| { a | b }, &self.avec, self.lastmask(), 0)], 
            bvec: vec![Vec3vl::bitfold(|a, b| { a | b }, &self.bvec, self.lastmask(), 0)]
        }
    }

    pub fn reduce_nand(&self) -> Vec3vl {
        Vec3vl { 
            bits: 1, 
            avec: vec![!Vec3vl::bitfold(|a, b| { a & b }, &self.bvec, self.lastmask(), 1)], 
            bvec: vec![!Vec3vl::bitfold(|a, b| { a & b }, &self.avec, self.lastmask(), 1)]
        }
    }

    pub fn reduce_nor(&self) -> Vec3vl {
        Vec3vl { 
            bits: 1, 
            avec: vec![!Vec3vl::bitfold(|a, b| { a | b }, &self.bvec, self.lastmask(), 0)], 
            bvec: vec![!Vec3vl::bitfold(|a, b| { a | b }, &self.avec, self.lastmask(), 0)]
        }
    }

    pub fn reduce_xor(&self) -> Vec3vl {
        let xes = Vec3vl::zip(|(a, b)| { !a & b }, &self.avec, &self.bvec);
        let has_x = Vec3vl::bitfold(|a, b| { a | b }, &xes, self.lastmask(), 0);
        let v = Vec3vl::bitfold(|a, b| { a ^ b }, &self.avec, self.lastmask(), 0);
        Vec3vl { 
            bits: 1, 
            avec: vec![v & !has_x], 
            bvec: vec![v | has_x] 
        }
    }

    pub fn reduce_xnor(&self) -> Vec3vl {
        self.reduce_xor().not()
    }

    pub fn to_hex(&mut self) -> String {
        self.normalize();
        Vec3vl::to_hex_internal(0, self.bits, &self.avec, &self.bvec)
    }

    pub fn to_array(&mut self) -> Vec<i32> {
        self.normalize();
        let skip = 1;
        let sm = (1 << skip) - 1;
        let mut bit = 0;
        let mut k = 0;
        let mut m = sm;
        let mut out = vec![];

        while bit < self.bits {
            let a = (self.avec[k] & m) >> bit;
            let b = (self.bvec[k] & m) >> bit;
            let v = (a << skip) | b;
            let mut r = v as i32 - 1;
            if v > 0 { r -= 1; }
            out.push(r);

            m <<= skip;
            bit += skip;
            
            if m == 0 {
                k += 1;
                m = sm;
            }
        }

        out
    }

    pub fn to_bigint(&self) -> Result<BigInt, String> {
        if !self.is_fully_defined() {
            return Err("Atterpting to create BigInt from not fully defined signal".to_string());
        }

        if self.avec.len() > 64 {
            return Err("Attempting to create BigInt from too big signal".to_string());
        }

        let mut digits = self.avec.clone();
        digits.resize(64, 0);
        let d: [u32; 64] = match digits[..].try_into() {
            Ok(d) => d,
            Err(_) => return Err("".to_string())
        };

        Ok(BigInt::from_digits(d))
    }

    pub fn from_bigint(number: &BigInt, bits: u32) -> Vec3vl {
        let mut v = number.digits().to_vec();
        let new_len = wordnum(bits) + if bitnum(bits) > 0 { 1 } else { 0 };
        v.resize(new_len as usize, 0);

        Vec3vl { 
            bits, 
            avec: v.clone(), 
            bvec: v 
        }
    }

    pub fn from_number(number: u32, bits: u32) -> Vec3vl {
        let mut vec = Vec3vl { bits, avec: vec![number], bvec: vec![number] };
        vec.normalize();
        vec
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

        match len.cmp(&words) {
            Ordering::Less => r = [r, vec![0, (words - len) as u32]].concat(),
            Ordering::Equal => { },
            Ordering::Greater => { let _ = r.split_off(words); },
        };

        Vec3vl { bits: nbits as u32, avec: r.clone(), bvec: r }
    }

    pub fn from_binary(data: String, len: Option<usize>) -> Vec3vl {
        let nbits = if let Some(s) = len { s } else { data.len() };
        let words = (nbits + 31) >> 5;
        
        if data.chars().any(|c| c == 'x') {
            return Vec3vl::xes(nbits as u32);
        }

        let mut r = data.chars()
            .rev().collect::<String>()
            .as_bytes()
            .chunks(32)
            .map(|b: &[u8]| {
                let mut v = b.to_vec();
                v.reverse();
                u32::from_str_radix(&String::from_utf8(v).unwrap(), 2).unwrap()
            }).collect::<Vec<u32>>();
        
        let len = r.len();
        
        match len.cmp(&words) {
            Ordering::Less => r = [r, vec![0, (words - len) as u32]].concat(),
            Ordering::Equal => { },
            Ordering::Greater => { let _ = r.split_off(words); }
        }

        Vec3vl { bits: nbits as u32, avec: r.clone(), bvec: r }
    }

    pub fn from_clonable(data: JsVec3vl) -> Vec3vl {
        let mut v = Vec3vl { 
            bits: data.get_bits(), 
            avec: data.get_avec(), 
            bvec: data.get_bvec() 
        };
        v.normalize();
        v
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
        assert!(Vec3vl::ones(1).and(&Vec3vl::ones(1)).unwrap() == Vec3vl::ones(1));
    }

    #[test]
    fn test_and_tf() {
        assert!(Vec3vl::ones(1).and(&Vec3vl::zeros(1)).unwrap() == Vec3vl::zeros(1));
    }

    #[test]
    fn test_and_ft() {
        assert!(Vec3vl::zeros(1).and(&Vec3vl::ones(1)).unwrap() == Vec3vl::zeros(1));
    }

    #[test]
    fn test_and_ff() {
        assert!(Vec3vl::zeros(1).and(&Vec3vl::zeros(1)).unwrap() == Vec3vl::zeros(1));
    }

    #[test]
    fn test_or_tt() {
        assert!(Vec3vl::ones(1).or(&Vec3vl::ones(1)).unwrap() == Vec3vl::ones(1));
    }
     
    #[test]
    fn test_or_tf() {
        assert!(Vec3vl::ones(1).or(&Vec3vl::zeros(1)).unwrap() == Vec3vl::ones(1));
    }

    #[test]
    fn test_or_ft() {
        assert!(Vec3vl::zeros(1).or(&Vec3vl::ones(1)).unwrap() == Vec3vl::ones(1));
    }

    #[test]
    fn test_or_ff() {
        assert!(Vec3vl::zeros(1).or(&Vec3vl::zeros(1)).unwrap() == Vec3vl::zeros(1));
    }

    #[test]
    fn test_xor_tt() {
        assert!(Vec3vl::ones(1).xor(&Vec3vl::ones(1)).unwrap() == Vec3vl::zeros(1));
    }

    #[test]
    fn test_xor_tf() {
        assert!(Vec3vl::ones(1).xor(&Vec3vl::zeros(1)).unwrap() == Vec3vl::ones(1));
    }

    #[test]
    fn test_xor_ft() {
        assert!(Vec3vl::zeros(1).xor(&Vec3vl::ones(1)).unwrap() == Vec3vl::ones(1));
    }

    #[test]
    fn test_xor_ff() {
        assert!(Vec3vl::zeros(1).xor(&Vec3vl::zeros(1)).unwrap() == Vec3vl::zeros(1));
    }

    #[test]
    fn test_nand_tt() {
        assert!(Vec3vl::ones(1).nand(&Vec3vl::ones(1)).unwrap() == Vec3vl::zeros(1));
    }

    #[test]
    fn test_nand_tf() {
        assert!(Vec3vl::ones(1).nand(&Vec3vl::zeros(1)).unwrap() == Vec3vl::ones(1));
    }

    #[test]
    fn test_nand_ft() {
        assert!(Vec3vl::zeros(1).nand(&Vec3vl::ones(1)).unwrap() == Vec3vl::ones(1));
    }

    #[test]
    fn test_nand_ff() {
        assert!(Vec3vl::zeros(1).nand(&Vec3vl::zeros(1)).unwrap() == Vec3vl::ones(1));
    }

    #[test]
    fn test_nor_tt() {
        assert!(Vec3vl::ones(1).nor(&Vec3vl::ones(1)).unwrap() == Vec3vl::zeros(1));
    }

    #[test]
    fn test_nor_tf() {
        assert!(Vec3vl::ones(1).nor(&Vec3vl::zeros(1)).unwrap() == Vec3vl::zeros(1));
    }

    #[test]
    fn test_nor_ft() {
        assert!(Vec3vl::zeros(1).nor(&Vec3vl::ones(1)).unwrap() == Vec3vl::zeros(1));
    }

    #[test]
    fn test_nor_ff() {
        assert!(Vec3vl::zeros(1).nor(&Vec3vl::zeros(1)).unwrap() == Vec3vl::ones(1));
    }

    #[test]
    fn test_xnor_tt() {
        assert!(Vec3vl::ones(1).xnor(&Vec3vl::ones(1)).unwrap() == Vec3vl::ones(1));
    }

    #[test]
    fn test_xnor_tf() {
        assert!(Vec3vl::ones(1).xnor(&Vec3vl::zeros(1)).unwrap() == Vec3vl::zeros(1));
    }

    #[test]
    fn test_xnor_ft() {
        assert!(Vec3vl::zeros(1).xnor(&Vec3vl::ones(1)).unwrap() == Vec3vl::zeros(1));
    }

    #[test]
    fn test_xnor_ff() {
        assert!(Vec3vl::zeros(1).xnor(&Vec3vl::zeros(1)).unwrap() == Vec3vl::ones(1));
    }
}

#[derive(Clone)]
pub struct Vec3vl {
    pub bits: u32,
    pub avec: Vec::<u32>,
    pub bvec: Vec::<u32>
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
        Vec3vl::new(
            bits,
            vec![iva; words],
            vec![ivb; words]
        )
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

    pub fn zeros(bits: u32) -> Vec3vl {
        Vec3vl::make_int(bits, -1)
    }

    pub fn ones(bits: u32) -> Vec3vl {
        Vec3vl::make_int(bits, 1)
    }

    pub fn xes(bits: u32) -> Vec3vl {
        Vec3vl::make_int(bits, 0)
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

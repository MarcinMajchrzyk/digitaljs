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
        if !self.bits == v.bits {
            panic!()
        }
        Vec3vl::new(
            self.bits, 
            self.avec.iter().zip(v.avec).map(|(a, b)| a & b ).collect(),
            self.bvec.iter().zip(v.bvec).map(|(a, b)| a & b ).collect()
        )
    }

    pub fn or(&self, v: Vec3vl) -> Vec3vl {
        if !self.bits == v.bits {
            panic!()
        }
        Vec3vl::new(
            self.bits, 
            self.avec.iter().zip(v.avec).map(|(a, b)| a & b ).collect(),
            self.bvec.iter().zip(v.bvec).map(|(a, b)| a & b ).collect()
        )
    }
}

impl PartialEq for Vec3vl {
    fn eq(&self, other: &Self) -> bool {
        self.bits == other.bits && self.avec == other.avec && self.bvec == other.bvec
    }
}
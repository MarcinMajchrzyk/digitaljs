use std::collections::HashMap;

use bnum::BUintD32;

use crate::operations::ClockHack;
use crate::vector3vl::Vec3vl;

pub type BigInt = BUintD32::<1024>;

pub type ArithBinop = fn(BigInt, BigInt) -> BigInt;

pub fn arith_op(args: HashMap<String, Vec3vl>, op: &ArithBinop) -> ClockHack {
    let mut vecl = args.get("in1").unwrap().clone();
    let mut vecr = args.get("in2").unwrap().clone();

    if !vecl.is_fully_defined() || !vecr.is_fully_defined() {
        return ClockHack::Normal(vec![("out".to_string(), Vec3vl::xes(vecl.bits))]);
    }

    let l = BigInt::from_str_radix(&vecl.to_hex(), 16).unwrap();
    let r = BigInt::from_str_radix(&vecr.to_hex(), 16).unwrap();

    let result = op(l, r);

    let vec = Vec3vl::from_hex(result.to_str_radix(16), Some(vecl.bits as usize));
    
    ClockHack::Normal(vec![("out".to_string(), vec)])
}

pub fn add(l: BigInt, r: BigInt) -> BigInt { l.overflowing_add(r).0 }
pub fn sub(l: BigInt, r: BigInt) -> BigInt { l.overflowing_sub(r).0 }
pub fn mul(l: BigInt, r: BigInt) -> BigInt { l.overflowing_mul(r).0 }
pub fn div(l: BigInt, r: BigInt) -> BigInt { if r == BigInt::ZERO { BigInt::ZERO } else { l.overflowing_div(r).0 } }
pub fn modulo(l: BigInt, r: BigInt) -> BigInt { if r == BigInt::ZERO { BigInt::ZERO } else { l.overflowing_rem(r).0 } }

/*pub fn power(l: Int, r: Int) -> ClockHack {
    todo!()
}

pub fn shift_left(l: Int, r: Int) -> ClockHack {
    todo!()
}

pub fn shift_right(l: Int, r: Int) -> ClockHack {
    todo!()
}

pub fn less(l: Int, r: Int) -> ClockHack {
    todo!()
}

pub fn less_equal(l: Int, r: Int) -> ClockHack {
    todo!()
}

pub fn greater(l: Int, r: Int) -> ClockHack {
    todo!()
}

pub fn greater_equal(l: Int, r: Int) -> ClockHack {
    todo!()
}

pub fn equal(l: Int, r: Int) -> ClockHack {
    todo!()
}

pub fn not_equal(l: Int, r: Int) -> ClockHack {
    todo!()
}*/
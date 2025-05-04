use std::collections::HashMap;

use bnum::BUintD32;

use crate::operations::ClockHack;
use crate::vector3vl::Vec3vl;

pub type BigInt = BUintD32::<64>;

pub type ArithBinop = fn(BigInt, BigInt) -> BigInt;
pub type ArithComp = fn(BigInt, BigInt) -> bool;

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

pub fn arith_const_op(args: HashMap<String, Vec3vl>, op: &ArithBinop, constant: u32, left_op: bool) -> ClockHack {
    let mut vec = args.get("in").unwrap().clone();

    if !vec.is_fully_defined() {
        return ClockHack::Normal(vec![("out".to_string(), Vec3vl::xes(vec.bits))]);
    }

    let a = BigInt::from_str_radix(&vec.to_hex(), 16).unwrap();
    let b = BigInt::from_digit(constant);

    let result = if left_op { op(b, a) } else { op(a, b) };

    let vec = Vec3vl::from_hex(result.to_str_radix(16), Some(vec.bits as usize));
    ClockHack::Normal(vec![("out".to_string(), vec)])
}

pub fn arith_comp_op(args: HashMap<String, Vec3vl>, op: &ArithComp) -> ClockHack {
    let mut vecl = args.get("in1").unwrap().clone();
    let mut vecr = args.get("in2").unwrap().clone();

    if !vecl.is_fully_defined() || !vecr.is_fully_defined() {
        return ClockHack::Normal(vec![("out".to_string(), Vec3vl::xes(1))]);
    }

    let l = BigInt::from_str_radix(&vecl.to_hex(), 16).unwrap();
    let r = BigInt::from_str_radix(&vecr.to_hex(), 16).unwrap();

    let result = op(l, r);

    let vec = Vec3vl::make_bool(1, result);
    
    ClockHack::Normal(vec![("out".to_string(), vec)])
}

pub fn arith_comp_const_op(args: HashMap<String, Vec3vl>, op: &ArithComp, constant: u32, left_op: bool) -> ClockHack {
    let mut vec = args.get("in").unwrap().clone();

    if !vec.is_fully_defined() {
        return ClockHack::Normal(vec![("out".to_string(), Vec3vl::xes(1))]);
    }

    let a = BigInt::from_str_radix(&vec.to_hex(), 16).unwrap();
    let b = BigInt::from_digit(constant);

    let result = if left_op { op(b, a) } else { op(a, b) };
    let vec = Vec3vl::make_bool(1, result);
    ClockHack::Normal(vec![("out".to_string(), vec)])
}

pub fn add(l: BigInt, r: BigInt) -> BigInt { l.overflowing_add(r).0 }
pub fn sub(l: BigInt, r: BigInt) -> BigInt { l.overflowing_sub(r).0 }
pub fn mul(l: BigInt, r: BigInt) -> BigInt { l.overflowing_mul(r).0 }
pub fn div(l: BigInt, r: BigInt) -> BigInt { if r == BigInt::ZERO { BigInt::ZERO } else { l.overflowing_div(r).0 } }
pub fn modulo(l: BigInt, r: BigInt) -> BigInt { if r == BigInt::ZERO { BigInt::ZERO } else { l.overflowing_rem(r).0 } }

pub fn power(l: BigInt, r: BigInt) -> BigInt { l.overflowing_pow(r.digits()[0]).0 }
pub fn shift_left(l: BigInt, r: BigInt) -> BigInt { l.overflowing_shl(r.digits()[0]).0 }
pub fn shift_right(l: BigInt, r: BigInt) -> BigInt { l.overflowing_shr(r.digits()[0]).0 }

pub fn less(l: BigInt, r: BigInt) -> bool { l.lt(&r) }
pub fn less_equal(l: BigInt, r: BigInt) -> bool { l.le(&r) }
pub fn greater(l: BigInt, r: BigInt) -> bool { l.gt(&r) }
pub fn greater_equal(l: BigInt, r: BigInt) -> bool { l.ge(&r) }
pub fn equal(l: BigInt, r: BigInt) -> bool { l.eq(&r) }
pub fn not_equal(l: BigInt, r: BigInt) -> bool { l.ne(&r) }
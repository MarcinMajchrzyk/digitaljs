use std::collections::HashMap;

use bnum::BUintD32;

use crate::operations::ClockHack;
use crate::vector3vl::Vec3vl;

pub type BigInt = BUintD32::<64>;

pub type ArithBinop = fn(BigInt, BigInt) -> BigInt;
pub type ArithComp = fn(BigInt, BigInt) -> bool;

pub fn arith_op(args: HashMap<String, Vec3vl>, op: &ArithBinop) -> Result<ClockHack, String> {
    let mut vecl = match args.get("in1") {
        Some(i) => i.clone(),
        None => return Err("No input in1".to_string())
    };
    let mut vecr = match args.get("in2") {
        Some(i) => i.clone(),
        None => return Err("No input in2".to_string())
    };

    if !vecl.is_fully_defined() || !vecr.is_fully_defined() {
        return Ok(ClockHack::Normal(vec![("out".to_string(), Vec3vl::xes(vecl.bits))]));
    }

    let l = match BigInt::from_str_radix(&vecl.to_hex(), 16) {
        Ok(b) => b,
        Err(_) => return Err("Error creating big int".to_string())
    };
    let r = match BigInt::from_str_radix(&vecr.to_hex(), 16) {
        Ok(b) => b,
        Err(_) => return Err("Error creating big int".to_string())
    };

    let result = op(l, r);

    let vec = Vec3vl::from_hex(result.to_str_radix(16), Some(vecl.bits as usize));
    
    Ok(ClockHack::Normal(vec![("out".to_string(), vec)]))
}

pub fn arith_const_op(args: HashMap<String, Vec3vl>, op: &ArithBinop, constant: u32, left_op: bool) -> Result<ClockHack, String> {
    let mut vec = match args.get("in") {
        Some(v) => v.clone(),
        None => return Err("No input in".to_string())
    };

    if !vec.is_fully_defined() {
        return Ok(ClockHack::Normal(vec![("out".to_string(), Vec3vl::xes(vec.bits))]));
    }

    let a = match BigInt::from_str_radix(&vec.to_hex(), 16) {
        Ok(b) => b,
        Err(_) => return Err("Error creating big int".to_string())
    };

    let b = BigInt::from_digit(constant);

    let result = if left_op { op(b, a) } else { op(a, b) };

    let vec = Vec3vl::from_hex(result.to_str_radix(16), Some(vec.bits as usize));
    Ok(ClockHack::Normal(vec![("out".to_string(), vec)]))
}

pub fn arith_comp_op(args: HashMap<String, Vec3vl>, op: &ArithComp) -> Result<ClockHack, String> {
    let mut vecl = match args.get("in1") {
        Some(v) => v.clone(),
        None => return Err("No input in1".to_string())
    };
    let mut vecr = match args.get("in2") {
        Some(v) => v.clone(),
        None => return Err("No input in2".to_string())
    };

    if !vecl.is_fully_defined() || !vecr.is_fully_defined() {
        return Ok(ClockHack::Normal(vec![("out".to_string(), Vec3vl::xes(1))]));
    }

    let l = match BigInt::from_str_radix(&vecl.to_hex(), 16) { 
        Ok(b) => b,
        Err(_) => return Err("Error creating big int".to_string())
    };
    let r = match BigInt::from_str_radix(&vecr.to_hex(), 16) { 
        Ok(b) => b,
        Err(_) => return Err("Error creating big int".to_string())
    };

    let result = op(l, r);

    let vec = Vec3vl::make_bool(1, result);
    Ok(ClockHack::Normal(vec![("out".to_string(), vec)]))
}

pub fn arith_comp_const_op(args: HashMap<String, Vec3vl>, op: &ArithComp, constant: u32, left_op: bool) -> Result<ClockHack, String> {
    let mut vec = match args.get("in") {
        Some(v) => v.clone(),
        None => return Err("No input in".to_string())
    };

    if !vec.is_fully_defined() {
        return Ok(ClockHack::Normal(vec![("out".to_string(), Vec3vl::xes(1))]));
    }

    let a = match BigInt::from_str_radix(&vec.to_hex(), 16) {
        Ok(b) => b,
        Err(_) =>return Err("Error creating big int".to_string())
    };
    let b = BigInt::from_digit(constant);

    let result = if left_op { op(b, a) } else { op(a, b) };
    let vec = Vec3vl::make_bool(1, result);
    Ok(ClockHack::Normal(vec![("out".to_string(), vec)]))
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
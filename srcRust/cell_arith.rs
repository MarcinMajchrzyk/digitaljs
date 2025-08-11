use std::collections::HashMap;

use bnum::BUintD32;

use crate::operations::ReturnValue;
use crate::vector3vl::Vec3vl;

pub type BigInt = BUintD32::<64>;

pub type ArithBinop = fn(BigInt, BigInt) -> BigInt;
pub type ArithConstBinop = fn(u32, u32) -> u32;

pub type ArithComp = fn(BigInt, BigInt) -> bool;
pub type ArithConstComp = fn(u32, u32) -> bool;

pub fn arith_op(args: &HashMap<String, Vec3vl>, op: &ArithBinop, op_const: &ArithConstBinop) -> Result<ReturnValue, String> {
    let mut vecl = match args.get("in1") {
        Some(i) => i.clone(),
        None => return Err("No input in1".to_string())
    };
    let mut vecr = match args.get("in2") {
        Some(i) => i.clone(),
        None => return Err("No input in2".to_string())
    };

    if !vecl.is_fully_defined() || !vecr.is_fully_defined() {
        return ReturnValue::out(Vec3vl::xes(vecl.bits));
    }

    let vec = if vecl.bits <= 32 && vecr.bits <= 32 {
        let l = vecl.get_number()?;
        let r = vecr.get_number()?;

        let result = op_const(l, r);
        Vec3vl::from_number(result, vecl.bits)
    } else {
        let l = vecl.to_bigint()?;
        let r = vecr.to_bigint()?;
        let result = op(l, r);
        Vec3vl::from_bigint(&result, vecl.bits)
    };
    
    ReturnValue::out(vec)
}

pub fn arith_const_op(args: &HashMap<String, Vec3vl>, op: &ArithConstBinop, constant: Option<u32>, left_op: Option<bool>) -> Result<ReturnValue, String> {
    let lo = match left_op {
        Some(b) => b,
        None => return Err("No left_op argument provided".to_string())
    };
    
    let mut vecl = match args.get("in") {
        Some(v) => v.clone(),
        None => return Err("No input in".to_string())
    };

    if vecl.bits > 32 {
        return Err("Constant operations only support 32 bit values".to_string());
    }

    if !vecl.is_fully_defined() {
        return ReturnValue::out(Vec3vl::xes(vecl.bits));
    }

    let a = vecl.get_number()?;
    let b = match constant {
        Some(c) => c,
        None => return Err("No constant provided".to_string())
    };

    let result = if lo { op(b, a) } else { op(a, b) };

    ReturnValue::out(Vec3vl::from_number(result, vecl.bits))
}

pub fn arith_comp_op(args: &HashMap<String, Vec3vl>, op: &ArithComp, op_const: &ArithConstComp) -> Result<ReturnValue, String> {
    let mut vecl = match args.get("in1") {
        Some(v) => v.clone(),
        None => return Err("No input in1".to_string())
    };
    let mut vecr = match args.get("in2") {
        Some(v) => v.clone(),
        None => return Err("No input in2".to_string())
    };

    if !vecl.is_fully_defined() || !vecr.is_fully_defined() {
        return ReturnValue::out(Vec3vl::xes(1));
    }

    let result = if vecl.bits <= 32 && vecr.bits <= 32 {
        let l = vecl.get_number()?;
        let r = vecr.get_number()?;
        op_const(l, r)
    } else {
        let l = vecl.to_bigint()?;
        let r = vecr.to_bigint()?;
        op(l, r)
    };
    let vec = Vec3vl::make_bool(1, result);
    ReturnValue::out(vec)
}

pub fn arith_comp_const_op(args: &HashMap<String, Vec3vl>, op: &ArithConstComp, constant: Option<u32>, left_op: Option<bool>) -> Result<ReturnValue, String> {
    let lo = match left_op {
        Some(b) => b,
        None => return Err("No left_op argument provided".to_string())
    };
    
    let mut vecl = match args.get("in") {
        Some(v) => v.clone(),
        None => return Err("No input in".to_string())
    };

    if !vecl.is_fully_defined() {
        return ReturnValue::out(Vec3vl::xes(1));
    }

    let a = vecl.get_number()?;
    let b = match constant {
        Some(c) => c,
        None => return Err("No constant provided".to_string())
    };

    let result = if lo { op(b, a) } else { op(a, b) };
    let vec = Vec3vl::make_bool(1, result);
    ReturnValue::out(vec)
}

pub fn add(l: BigInt, r: BigInt)            -> BigInt { l.overflowing_add(r).0 }
pub fn sub(l: BigInt, r: BigInt)            -> BigInt { l.overflowing_sub(r).0 }
pub fn mul(l: BigInt, r: BigInt)            -> BigInt { l.overflowing_mul(r).0 }
pub fn div(l: BigInt, r: BigInt)            -> BigInt { if r == BigInt::ZERO { BigInt::ZERO } else { l.overflowing_div(r).0 } }
pub fn modulo(l: BigInt, r: BigInt)         -> BigInt { if r == BigInt::ZERO { BigInt::ZERO } else { l.overflowing_rem(r).0 } }
pub fn power(l: BigInt, r: BigInt)          -> BigInt { l.overflowing_pow(r.digits()[0]).0 }
pub fn shift_left(l: BigInt, r: BigInt)     -> BigInt { l.overflowing_shl(r.digits()[0]).0 }
pub fn shift_right(l: BigInt, r: BigInt)    -> BigInt { l.overflowing_shr(r.digits()[0]).0 }

pub fn add_c(l: u32, r: u32)            -> u32 { l.overflowing_add(r).0 }
pub fn sub_c(l: u32, r: u32)            -> u32 { l.overflowing_sub(r).0 }
pub fn mul_c(l: u32, r: u32)            -> u32 { l.overflowing_mul(r).0 }
pub fn div_c(l: u32, r: u32)            -> u32 { if r == 0 { 0 } else { l.overflowing_div(r).0 } }
pub fn modulo_c(l: u32, r: u32)         -> u32 { if r == 0 { 0 } else { l.overflowing_rem(r).0 } }
pub fn power_c(l: u32, r: u32)          -> u32 { l.overflowing_pow(r).0 }
pub fn shift_left_c(l: u32, r: u32)     -> u32 { l.overflowing_shl(r).0 }
pub fn shift_right_c(l: u32, r: u32)    -> u32 { l.overflowing_shr(r).0 }

pub fn less(l: BigInt, r: BigInt)           -> bool { l.lt(&r) }
pub fn less_equal(l: BigInt, r: BigInt)     -> bool { l.le(&r) }
pub fn greater(l: BigInt, r: BigInt)        -> bool { l.gt(&r) }
pub fn greater_equal(l: BigInt, r: BigInt)  -> bool { l.ge(&r) }
pub fn equal(l: BigInt, r: BigInt)          -> bool { l.eq(&r) }
pub fn not_equal(l: BigInt, r: BigInt)      -> bool { l.ne(&r) }

pub fn less_c(l: u32, r: u32)           -> bool { l.lt(&r) }
pub fn less_equal_c(l: u32, r: u32)     -> bool { l.le(&r) }
pub fn greater_c(l: u32, r: u32)        -> bool { l.gt(&r) }
pub fn greater_equal_c(l: u32, r: u32)  -> bool { l.ge(&r) }
pub fn equal_c(l: u32, r: u32)          -> bool { l.eq(&r) }
pub fn not_equal_c(l: u32, r: u32)      -> bool { l.ne(&r) }
use std::collections::HashMap;

use bnum::{BIntD32, BUintD32};

use crate::operations::ReturnValue;
use crate::vector3vl::Vec3vl;

pub type BigInt = BIntD32::<64>;
pub type BigUInt = BUintD32::<64>;

pub type ArithMonop = fn(&HashMap<String, Vec3vl>, bits: u32) -> Result<ReturnValue, String>;

pub type ArithBinop = fn(&HashMap<String, Vec3vl>, u32, (bool, bool)) -> Result<ReturnValue, String>;
pub type ArithConstBinop = fn(&HashMap<String, Vec3vl>, &i32, &bool, u32, bool) -> Result<ReturnValue, String>;

pub type ArithComp = fn(&HashMap<String, Vec3vl>, (bool, bool)) -> Result<ReturnValue, String>;
pub type ArithConstComp = fn(&HashMap<String, Vec3vl>, &i32, &bool, bool) -> Result<ReturnValue, String>;

pub fn arith_monop<T: ArithMonopStruct>(args: &HashMap<String, Vec3vl>, bits: u32) -> Result<ReturnValue, String> {
    let mut input = match args.get("in") {
        Some(i) => i.clone(),
        None => return Err("No input".to_string())
    };

    if !input.is_fully_defined() {
        return ReturnValue::out(Vec3vl::xes(bits))
    }

    let vec = if input.bits <= 32 {
        let i = input.get_number()?;
        let result = T::op_const(i);
        Vec3vl::from_number(result, bits)
    } else {
        let i = input.to_biguint()?;
        let result = T::op(i);
        Vec3vl::from_biguint(&result, bits)
    };
    ReturnValue::out(vec)
}

pub fn extend_number(value: u32, bits: u32) -> i32 {
    let b = 32 - bits;
    (value as i32).overflowing_shl(b).0.overflowing_shr(b).0
}

pub fn arith_binop<T: ArithBinopStruct>(args: &HashMap<String, Vec3vl>, bits: u32, sgn: (bool, bool)) -> Result<ReturnValue, String> {
    let mut vecl = match args.get("in1") {
        Some(i) => i.clone(),
        None => return Err("No input in1".to_string())
    };
    let mut vecr = match args.get("in2") {
        Some(i) => i.clone(),
        None => return Err("No input in2".to_string())
    };

    if !vecl.is_fully_defined() || !vecr.is_fully_defined() {
        return ReturnValue::out(Vec3vl::xes(bits));
    }

    let vec = if vecl.bits <= 32 && vecr.bits <= 32 {
        let l = vecl.get_number()?;
        let r = vecr.get_number()?;

        let result = if sgn.0 && sgn.1 {
            let n_l = extend_number(l, vecl.bits);
            let n_r = extend_number(r, vecr.bits);
            T::op_const_sig(n_l, n_r) as u32
        } else {
            T::op_const_uns(l, r)
        };

        Vec3vl::from_number(result, bits)
    } else {
        let result = if sgn.0 && sgn.1 {
            let l = vecl.to_bigint()?;
            let r = vecr.to_bigint()?;
            T::op_sig(l, r).to_bits()
        } else { 
            let l = vecl.to_biguint()?;
            let r = vecr.to_biguint()?;
            T::op_uns(l, r)
        };

        Vec3vl::from_biguint(&result, bits)
    };
    
    ReturnValue::out(vec)
}

pub fn arith_const_binop<T: ArithBinopStruct>(args: &HashMap<String, Vec3vl>, constant: &i32, left_op: &bool, bits: u32, sgn: bool) -> Result<ReturnValue, String> {
    let mut vecl = match args.get("in") {
        Some(v) => v.clone(),
        None => return Err("No input in".to_string())
    };

    if vecl.bits > 32 {
        return Err("Constant operations only support 32 bit values".to_string());
    }

    if !vecl.is_fully_defined() {
        return ReturnValue::out(Vec3vl::xes(bits));
    }

    let a = vecl.get_number()?;
    let b = *constant;

    let result = if sgn {
        let s_a = extend_number(a, vecl.bits);
        let s_b = b;

        if *left_op { T::op_const_sig(s_b, s_a) as u32 } else { T::op_const_sig(s_a, s_b) as u32 }
    } else {
        let u_a = a;
        let u_b = b as u32;

        if *left_op { T::op_const_uns(u_b, u_a) } else { T::op_const_uns(u_a, u_b) }
    };

    ReturnValue::out(Vec3vl::from_number(result, bits))
}

pub fn arith_shift<L: ArithShiftStruct, R: ArithShiftStruct>(args: &HashMap<String, Vec3vl>, bits: u32, sgn: (bool, bool)) -> Result<ReturnValue, String> {
    let mut vecl = match args.get("in1") {
        Some(i) => i.clone(),
        None => return Err("No input in1".to_string())
    };
    let mut vecr = match args.get("in2") {
        Some(i) => i.clone(),
        None => return Err("No input in2".to_string())
    };

    if !vecl.is_fully_defined() || !vecr.is_fully_defined() {
        return ReturnValue::out(Vec3vl::xes(bits));
    }

    let vec = if vecl.bits <= 32 && vecr.bits <= 32 {
        let u_l = vecl.get_number()?;
        let u_r = vecr.get_number()?;
        let s_l = extend_number(u_l, vecl.bits);
        let s_r = extend_number(u_r, vecr.bits);

        let result = match sgn {
            (false, false) => {
                L::op_const_uns(u_l, u_r)
            },
            (true, false) => {
                L::op_const_sig(s_l, u_r) as u32
            },
            (false, true) => {
                if s_r < 0 { R::op_const_uns(u_l, s_r.unsigned_abs()) } else { L::op_const_uns(u_l, u_r) }
            },
            (true, true) => {
                if s_r < 0 { R::op_const_sig(s_l, s_r.unsigned_abs()) as u32 } else { L::op_const_sig(s_l, u_r) as u32 }
            }
        };

        Vec3vl::from_number(result, bits)
    } else {
        if vecr.bits > 32 { return Err("Can't support shifts with signal larger than 32 bits".to_string()); }
        let u_l = vecl.to_biguint()?;
        let u_r = vecr.get_number()?;
        let s_l = vecl.to_bigint()?;
        let s_r = extend_number(u_r, vecr.bits);

        let result = match sgn {
            (false, false) => {
                L::op_uns(u_l, u_r)
            },
            (true, false) => {
                L::op_sig(s_l, u_r).to_bits()
            },
            (false, true) => {
                if s_r < 0 { R::op_uns(u_l, s_r.unsigned_abs()) } else { L::op_uns(u_l, u_r) }
            },
            (true, true) => {
                if s_r < 0 { R::op_sig(s_l, s_r.unsigned_abs()).to_bits() } else { L::op_sig(s_l, u_r).to_bits() }
            }
        };

        Vec3vl::from_biguint(&result, bits)
    };
    
    ReturnValue::out(vec)
}

pub fn arith_shift_const<L: ArithShiftStruct, R: ArithShiftStruct>(args: &HashMap<String, Vec3vl>, constant: &i32, left_op: &bool, bits: u32, sgn: bool) -> Result<ReturnValue, String> {
    let mut vecl = match args.get("in") {
        Some(v) => v.clone(),
        None => return Err("No input in".to_string())
    };

    if vecl.bits > 32 {
        return Err("Constant operations only support 32 bit values".to_string());
    }

    if !vecl.is_fully_defined() {
        return ReturnValue::out(Vec3vl::xes(bits));
    }

    let calc_func = |u_l: u32, u_r: u32, s_l: i32, s_r: i32, sgn: (bool, bool)| -> u32 {
        match sgn {
            (false, false) => {
                L::op_const_uns(u_l, u_r)
            },
            (true, false) => {
                L::op_const_sig(s_l, u_r) as u32
            },
            (false, true) => {
                if s_r < 0 { R::op_const_uns(u_l, s_r.unsigned_abs()) } else { L::op_const_uns(u_l, u_r) }
            },
            (true, true) => {
                if s_r < 0 { R::op_const_sig(s_l, s_r.unsigned_abs()) as u32 } else { L::op_const_sig(s_l, u_r) as u32 }
            }
        }
    };
    let u_l = vecl.get_number()?;
    let u_r = *constant as u32;
    let s_l = extend_number(u_l, vecl.bits);
    let s_r = *constant;

    let result = if *left_op {
        calc_func(u_r, u_l, s_r, s_l, (s_r < 0, sgn))
    } else {
        calc_func(u_l, u_r, s_l, s_r, (sgn, s_r < 0))
    };

    ReturnValue::out(Vec3vl::from_number(result, bits))
}

pub fn arith_comp<T: ArithCompStruct>(args: &HashMap<String, Vec3vl>, sgn: (bool, bool)) -> Result<ReturnValue, String> {
    let mut vecl = match args.get("in1") {
        Some(i) => i.clone(),
        None => return Err("No input in1".to_string())
    };
    let mut vecr = match args.get("in2") {
        Some(i) => i.clone(),
        None => return Err("No input in2".to_string())
    };

    if !vecl.is_fully_defined() || !vecr.is_fully_defined() {
        return ReturnValue::out(Vec3vl::xes(1));
    }

    let result = if vecl.bits <= 32 && vecr.bits <= 32 {
        let l = vecl.get_number()?;
        let r = vecr.get_number()?;

        if sgn.0 && sgn.1 {
            let n_l = extend_number(l, vecl.bits);
            let n_r = extend_number(r, vecr.bits);
            T::op_const_sig(n_l, n_r)
        } else {
            T::op_const_uns(l, r)
        }
    } else if sgn.0 && sgn.1 {
        let l = vecl.to_bigint()?;
        let r = vecr.to_bigint()?;
        T::op_sig(l, r)
    } else { 
        let l = vecl.to_biguint()?;
        let r = vecr.to_biguint()?;
        T::op_uns(l, r)
    };
    let vec = Vec3vl::make_bool(1, result);
    ReturnValue::out(vec)
}

pub fn arith_const_comp<T: ArithCompStruct>(args: &HashMap<String, Vec3vl>, constant: &i32, left_op: &bool, sgn: bool) -> Result<ReturnValue, String> {
    let mut vecl = match args.get("in") {
        Some(v) => v.clone(),
        None => return Err("No input in".to_string())
    };

    if vecl.bits > 32 {
        return Err("Constant operations only support 32 bit values".to_string());
    }

    if !vecl.is_fully_defined() {
        return ReturnValue::out(Vec3vl::xes(1));
    }

    let a = vecl.get_number()?;
    let b = *constant;

    let result = if sgn {
        let s_a = extend_number(a, vecl.bits);
        let s_b = b;

        if *left_op { T::op_const_sig(s_b, s_a) } else { T::op_const_sig(s_a, s_b) }
    } else {
        let u_a = a;
        let u_b = b as u32;

        if *left_op { T::op_const_uns(u_b, u_a) } else { T::op_const_uns(u_a, u_b) }
    };

    ReturnValue::out(Vec3vl::make_bool(1, result))
}

pub trait ArithMonopStruct {
    fn op(i: BigUInt) -> BigUInt;
    fn op_const(i: u32) -> u32;
}

pub struct Negation { }
impl ArithMonopStruct for Negation {
    fn op(i: BigUInt) -> BigUInt { i.overflowing_neg().0 }
    fn op_const(i: u32) -> u32 { i.overflowing_neg().0 }
}

pub struct UnaryPlus { }
impl ArithMonopStruct for UnaryPlus {
    fn op(i: BigUInt) -> BigUInt { i }
    fn op_const(i: u32) -> u32 { i }
}

pub trait ArithBinopStruct {
    fn op_uns(l: BigUInt, r: BigUInt) -> BigUInt;
    fn op_sig(l: BigInt, r: BigInt)   -> BigInt;
    fn op_const_uns(l: u32, r: u32) -> u32;
    fn op_const_sig(l: i32, r: i32) -> i32;
}

pub struct Add { }
impl ArithBinopStruct for Add {
    fn op_uns(l: BigUInt, r: BigUInt) -> BigUInt { l.overflowing_add(r).0 }
    fn op_sig(l: BigInt,  r: BigInt)  -> BigInt  { l.overflowing_add(r).0 }
    fn op_const_uns(l: u32, r: u32) -> u32 { l.overflowing_add(r).0 }
    fn op_const_sig(l: i32, r: i32) -> i32 { l.overflowing_add(r).0 }
}

pub struct Sub { }
impl ArithBinopStruct for Sub {
    fn op_uns(l: BigUInt, r: BigUInt) -> BigUInt { l.overflowing_sub(r).0 }
    fn op_sig(l: BigInt,  r: BigInt)  -> BigInt  { l.overflowing_sub(r).0 }
    fn op_const_uns(l: u32, r: u32) -> u32 { l.overflowing_sub(r).0 }
    fn op_const_sig(l: i32, r: i32) -> i32 { l.overflowing_sub(r).0 }
}

pub struct Mul { }
impl ArithBinopStruct for Mul {
    fn op_uns(l: BigUInt, r: BigUInt) -> BigUInt { l.overflowing_mul(r).0 }
    fn op_sig(l: BigInt,  r: BigInt)  -> BigInt  { l.overflowing_mul(r).0 }
    fn op_const_uns(l: u32, r: u32) -> u32 { l.overflowing_mul(r).0 }
    fn op_const_sig(l: i32, r: i32) -> i32 { l.overflowing_mul(r).0 }
}

pub struct Div { }
impl ArithBinopStruct for Div {
    fn op_uns(l: BigUInt, r: BigUInt) -> BigUInt { if r == BigUInt::ZERO { l } else { l.overflowing_div(r).0 }}
    fn op_sig(l: BigInt, r: BigInt)   -> BigInt  { if r == BigInt::ZERO  { l } else { l.overflowing_div(r).0 }}
    fn op_const_uns(l: u32, r: u32) -> u32 { if r == 0 { l } else { l.overflowing_div(r).0 }}
    fn op_const_sig(l: i32, r: i32) -> i32 { if r == 0 { l } else { l.overflowing_div(r).0 }}
}

pub struct Mod { }
impl ArithBinopStruct for Mod {
    fn op_uns(l: BigUInt, r: BigUInt) -> BigUInt { if r == BigUInt::ZERO { l } else { l.overflowing_rem(r).0 }}
    fn op_sig(l: BigInt, r: BigInt)   -> BigInt  { if r == BigInt::ZERO  { l } else { l.overflowing_rem(r).0 }}
    fn op_const_uns(l: u32, r: u32) -> u32 { if r == 0 { l } else { l.overflowing_rem(r).0 }}
    fn op_const_sig(l: i32, r: i32) -> i32 { if r == 0 { l } else { l.overflowing_rem(r).0 }}
}

pub struct Pow { }
impl ArithBinopStruct for Pow {
    fn op_uns(l: BigUInt, r: BigUInt) -> BigUInt { 
        if r >= BigUInt::ZERO { l.overflowing_pow(r.digits()[0]).0 } 
        else if l == BigUInt::ONE { BUintD32::ONE } 
        else { BigUInt::ZERO }
    }
    fn op_sig(l: BigInt,  r: BigInt)  -> BigInt  { 
        if r >= BigInt::ZERO { l.overflowing_pow(r.to_bits().digits()[0]).0 }
        else if l == BigInt::ONE { BigInt::ONE }
        else if l == BigInt::NEG_ONE { 
            if r.rem(BigInt::TWO) == BigInt::ZERO { BigInt::ONE } else { BigInt::NEG_ONE }
        } else { BigInt::ZERO }
    }
    fn op_const_uns(l: u32, r: u32) -> u32 { l.overflowing_pow(r).0 }
    fn op_const_sig(l: i32, r: i32) -> i32 { 
        if r >= 0 { l.overflowing_pow(r as u32).0 }
        else if l == 1 { 1 }
        else if l == -1 { 
            if r.rem_euclid(2) == 0 { 1 } else { -1 }
        } else { 0 }
    }
}

pub trait ArithShiftStruct {
    fn op_uns(l: BigUInt,   r: u32) -> BigUInt;
    fn op_sig(l: BigInt,    r: u32) -> BigInt;
    fn op_const_uns(l: u32, r: u32) -> u32;
    fn op_const_sig(l: i32, r: u32) -> i32;
}

pub struct Shl { }
impl ArithShiftStruct for Shl {
    fn op_uns(l: BigUInt,   r: u32) -> BigUInt { l.unbounded_shl(r) }
    fn op_sig(l: BigInt,    r: u32) -> BigInt  { l.unbounded_shl(r) }
    fn op_const_uns(l: u32, r: u32) -> u32     { l.unbounded_shl(r) }
    fn op_const_sig(l: i32, r: u32) -> i32     { l.unbounded_shl(r) }
}

pub struct Shr { }
impl ArithShiftStruct for Shr {
    fn op_uns(l: BigUInt,   r: u32) -> BigUInt { l.unbounded_shr(r) }
    fn op_sig(l: BigInt,    r: u32) -> BigInt  { l.unbounded_shr(r) }
    fn op_const_uns(l: u32, r: u32) -> u32     { l.unbounded_shr(r) }
    fn op_const_sig(l: i32, r: u32) -> i32     { l.unbounded_shr(r) }
}

pub trait ArithCompStruct {
    fn op_uns(l: BigUInt,   r: BigUInt) -> bool;
    fn op_sig(l: BigInt,    r: BigInt)  -> bool;
    fn op_const_uns(l: u32, r: u32)     -> bool;
    fn op_const_sig(l: i32, r: i32)     -> bool;
}

pub struct Lt { }
impl ArithCompStruct for Lt {
    fn op_uns(l: BigUInt,   r: BigUInt) -> bool { l.lt(&r) }
    fn op_sig(l: BigInt,    r: BigInt)  -> bool { l.lt(&r) }
    fn op_const_uns(l: u32, r: u32)     -> bool { l.lt(&r) }
    fn op_const_sig(l: i32, r: i32)     -> bool { l.lt(&r) }
}

pub struct Le { }
impl ArithCompStruct for Le {
    fn op_uns(l: BigUInt,   r: BigUInt) -> bool { l.le(&r) }
    fn op_sig(l: BigInt,    r: BigInt)  -> bool { l.le(&r) }
    fn op_const_uns(l: u32, r: u32)     -> bool { l.le(&r) }
    fn op_const_sig(l: i32, r: i32)     -> bool { l.le(&r) }
}

pub struct Gt { }
impl ArithCompStruct for Gt {
    fn op_uns(l: BigUInt,   r: BigUInt) -> bool { l.gt(&r) }
    fn op_sig(l: BigInt,    r: BigInt)  -> bool { l.gt(&r) }
    fn op_const_uns(l: u32, r: u32)     -> bool { l.gt(&r) }
    fn op_const_sig(l: i32, r: i32)     -> bool { l.gt(&r) }
}

pub struct Ge { }
impl ArithCompStruct for Ge {
    fn op_uns(l: BigUInt,   r: BigUInt) -> bool { l.ge(&r) }
    fn op_sig(l: BigInt,    r: BigInt)  -> bool { l.ge(&r) }
    fn op_const_uns(l: u32, r: u32)     -> bool { l.ge(&r) }
    fn op_const_sig(l: i32, r: i32)     -> bool { l.ge(&r) }
}

pub struct Eq { }
impl ArithCompStruct for Eq {
    fn op_uns(l: BigUInt,   r: BigUInt) -> bool { l.eq(&r) }
    fn op_sig(l: BigInt,    r: BigInt)  -> bool { l.eq(&r) }
    fn op_const_uns(l: u32, r: u32)     -> bool { l.eq(&r) }
    fn op_const_sig(l: i32, r: i32)     -> bool { l.eq(&r) }
}

pub struct Ne { }
impl ArithCompStruct for Ne {
    fn op_uns(l: BigUInt,   r: BigUInt) -> bool { l.ne(&r) }
    fn op_sig(l: BigInt,    r: BigInt)  -> bool { l.ne(&r) }
    fn op_const_uns(l: u32, r: u32)     -> bool { l.ne(&r) }
    fn op_const_sig(l: i32, r: i32)     -> bool { l.ne(&r) }
}

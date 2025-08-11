use std::ops::BitXorAssign;

use crate::operations::ReturnValue;
use crate::vector3vl::Vec3vl;

pub fn clock(clock_val: &mut bool) -> Result<ReturnValue, String> {
    clock_val.bitxor_assign(true);
    ReturnValue::clock(Vec3vl::make_bool(1, *clock_val))
}

pub fn constant(value: Option<String>) -> Result<ReturnValue, String> {
    match value {
        Some(s) => ReturnValue::out(Vec3vl::from_binary(s, None)),
        None => Err("Constant not provided".to_string())
    }     
}
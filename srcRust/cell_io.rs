use std::ops::BitXorAssign;

use crate::operations::ClockHack;
use crate::vector3vl::Vec3vl;

pub fn clock(clock_val: &mut bool) -> Result<ClockHack, String> {
    clock_val.bitxor_assign(true);
    Ok(ClockHack::Clock(vec![("out".to_string(), Vec3vl::make_bool(1, *clock_val))]))
}

pub fn constant(value: String) -> Result<ClockHack, String> {
    Ok(ClockHack::Normal(vec![("out".to_string(), Vec3vl::from_binary(value, None))]))
}
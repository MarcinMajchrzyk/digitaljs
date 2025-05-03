use std::ops::BitXorAssign;

use crate::operations::ClockHack;
use crate::vector3vl::Vec3vl;

pub fn clock(clock_val: &mut bool) -> ClockHack {
  clock_val.bitxor_assign(true);
  ClockHack::Clock(vec![("out".to_string(), Vec3vl::make_bool(1, *clock_val))])
}

pub fn constant() -> ClockHack {
  ClockHack::Normal(vec![("out".to_string(), Vec3vl::make_bool(1, false))])
}
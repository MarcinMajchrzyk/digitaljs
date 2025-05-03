use std::collections::HashMap;

use crate::gate::PolarityOptions;
use crate::vector3vl::Vec3vl;
use crate::operations::ClockHack;

pub fn dff(_args: HashMap<String, Vec3vl>, _polarity: &mut PolarityOptions, _last_clk: &mut u32) -> ClockHack {
  /*let pol = |what: bool| -> i32 {
    if what { 1 } else { -1 }
  };
  
  let lclk: u32;
  let mut srbits: Option<Vec3vl> = None;
  let mut srbitmask: Option<Vec3vl> = None;

  let apply_sr = |v: Vec3vl| -> ClockHack {
    if let Some(srb) = srbits.clone() {
      ClockHack::Normal(vec![(
        "out".to_string(),
        v.and(srbitmask.clone().unwrap()).or(srb)
      )])
    } else {
      ClockHack::Normal(vec![("out".to_string(), v)])
    }
  };
                
  if polarity.clock.is_some() {
    lclk = *last_clk;
    *last_clk = args.get("clk").unwrap().get(0);
  }

    if let Some(arst) = polarity.arst {
      if args.get("arst").unwrap().get(0) as i32 == pol(arst) {
        // TODO
      }
    }

    if let Some(aload) = polarity.aload {
      if args.get("aload").unwrap().get(0) as i32 == pol(aload) {
        return ClockHack::Normal(vec![(
          "out".to_string(), 
          args.get("ain").unwrap().clone()
        )])
      }
    }

    if let Some(set) = polarity.set {
      let data_set = args.get("set").unwrap().clone();
      srbits = if set { Some(data_set.clone()) } else { Some(data_set.not()) };
      srbitmask = if set { Some(data_set.not()) } else { Some(data_set) };
    }

    if let Some(clr) = polarity.clr {
      srbits = if let Some(srb) = srbits {
        Some(srb)
    } else {
      Some(Vec3vl::zeros(1)) // TODO
    };
      let clrbitmask = if clr { 
        args.get("clr").unwrap().not() 
      } else { 
        args.get("clr").unwrap().clone() 
      };
      srbitmask = if let Some(srbm) = srbitmask {
        Some(clrbitmask.and(srbm))
      } else {
        Some(clrbitmask)
      };
    }

    if let Some(enable) = polarity.enable {
      if args.get("en").unwrap().get(0) as i32 == pol(enable) {
      //return apply_sr();
      todo!()
    }
  }

  if polarity.clock.is_none() || args.get("clk").unwrap().get(0) as i32 == pol(polarity.clock.unwrap()) && *last_clk as i32 == -pol(polarity.clock.unwrap()) {
                    
  } else {
                    
  }*/

  todo!()
}
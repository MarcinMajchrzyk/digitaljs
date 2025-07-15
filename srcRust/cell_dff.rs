use std::collections::HashMap;

use crate::gate::PolarityOptions;
use crate::vector3vl::Vec3vl;
use crate::operations::ClockHack;

pub fn dff(args: HashMap<String, Vec3vl>, polarity: &PolarityOptions, out: &mut Vec3vl, last_clk: &mut u32) -> Result<ClockHack, String> {  
  let pol = |what: bool| -> i32 {
    if what { 1 } else { -1 }
  };
  
  let mut lclk: i32 = 1;
  let mut srbits: Option<Vec3vl> = None;
  let mut srbitmask: Option<Vec3vl> = None;

  let apply_sr = |v: Vec3vl, srbits: Option<Vec3vl>, srbitmask: Option<Vec3vl>| -> ClockHack {
    if let Some(srb) = srbits.clone() {
      ClockHack::Normal(vec![(
        "out".to_string(),
        v.and(srbitmask.clone().unwrap()).unwrap().or(srb.clone()).unwrap()
      )])
    } else {
      ClockHack::Normal(vec![("out".to_string(), v)])
    }
  };
                
  if polarity.clock.is_some() {
    lclk = *last_clk as i32;
    *last_clk = args.get("clk").unwrap().get(0);
  }

  /*if let Some(arst) = polarity.arst {
    if args.get("arst").unwrap().get(0) as i32 == pol(arst) {
      // TODO
    }
  }*/

  if let Some(aload) = polarity.aload {
    if args.get("aload").unwrap().get(0) as i32 == pol(aload) {
      return Ok(ClockHack::Normal(vec![(
        "out".to_string(), 
        args.get("ain").unwrap().clone()
      )]))
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
      Some(clrbitmask.and(srbm).unwrap())
    } else {
      Some(clrbitmask)
    };
  }

  /*if let Some(enable) = polarity.enable {
    if args.get("en").unwrap().get(0) as i32 == pol(enable) {
      //return apply_sr();
      todo!()
    }
  }*/


  if polarity.clock.is_none() || args.get("clk").unwrap().get(0) as i32 == pol(polarity.clock.unwrap()) && lclk as i32 == -pol(polarity.clock.unwrap()) {
    if polarity.enable.is_some() && args.get("en").unwrap().get(0) as i32 != pol(polarity.enable.unwrap()) {
      
    } else {
      *out = args.get("in").unwrap().clone();
    }
  } 

  Ok(apply_sr(out.clone(), srbits, srbitmask))
}
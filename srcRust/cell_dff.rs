use std::collections::HashMap;

use crate::gate::{GateParams, PolarityOptions};
use crate::vector3vl::Vec3vl;
use crate::operations::ReturnValue;

pub struct DffState {
  arst_value: Option<String>,
  bits: u32,
  last_clk: i32,
  out: Vec3vl,
  polarity: PolarityOptions
}

pub fn dff(args: &HashMap<String, Vec3vl>, state: &mut DffState) -> Result<ReturnValue, String> {  
  let pol = |what: bool| -> i32 {
    if what { 1 } else { -1 }
  };
  
  let mut lclk: i32 = 1;
  let mut srbits: Option<Vec3vl> = None;
  let mut srbitmask: Option<Vec3vl> = None;

  let apply_sr = |v: &Vec3vl, srbits: Option<Vec3vl>, srbitmask: Option<Vec3vl>| -> Result<ReturnValue, String> {
    if let Some(srb) = srbits {
      ReturnValue::out(v.and(&srbitmask.unwrap())?.or(&srb)?)
    } else {
      ReturnValue::out(v.clone())
    }
  };
                
  if state.polarity.clock.is_some() {
    lclk = state.last_clk;
    state.last_clk = args.get("clk").unwrap().lsb();
  }

  if let Some(arst) = state.polarity.arst {
    if args.get("arst").unwrap().lsb() == pol(arst) {
      state.out = Vec3vl::from_binary(state.arst_value.clone().unwrap(), Some(state.bits as usize));
      return apply_sr(&state.out, srbits, srbitmask);
    }
  }

  if let Some(aload) = state.polarity.aload {
    if args.get("aload").unwrap().lsb() == pol(aload) {
      return ReturnValue::out(args.get("ain").unwrap().clone());
    }
  }

  if let Some(set) = state.polarity.set {
    let data_set = args.get("set").unwrap().clone();
    srbits = if set { Some(data_set.clone()) } else { Some(data_set.not()) };
    srbitmask = if set { Some(data_set.not()) } else { Some(data_set) };
  }

  if let Some(clr) = state.polarity.clr {
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
      Some(clrbitmask.and(&srbm)?)
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


  if state.polarity.clock.is_none() || args.get("clk").unwrap().lsb() == pol(state.polarity.clock.unwrap()) && lclk == -pol(state.polarity.clock.unwrap()) {
    if state.polarity.enable.is_some() && args.get("en").unwrap().lsb() != pol(state.polarity.enable.unwrap()) {
      
    } else {
      state.out = args.get("in").unwrap().clone();
    }
  } 

  apply_sr(&state.out, srbits, srbitmask)
}

impl DffState {
  pub fn new(params: &GateParams) -> DffState {
    DffState { 
      arst_value: params.arst_value.clone(), 
      bits: params.bits_in, 
      last_clk: 0, 
      out: Vec3vl::xes(params.bits_in), 
      polarity: params.polarity
    }
  }
}
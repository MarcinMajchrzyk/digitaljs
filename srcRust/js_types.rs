use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
  pub type GateParams;

  #[wasm_bindgen(method, structural, getter = bits)]
  pub fn get_bits(this: &GateParams) -> u32;

  #[wasm_bindgen(method, structural, getter = iputs)]
  pub fn get_inputs(this: &GateParams) -> Option<u32>;

  #[wasm_bindgen(method, structural, getter = label)]
  pub fn get_label(this: &GateParams) -> Option<String>;

  #[wasm_bindgen(method, structural, getter = net)]
  pub fn get_net(this: &GateParams) -> Option<String>;

  #[wasm_bindgen(method, structural, getter = numbase)]
  pub fn get_numbase(this: &GateParams) -> Option<String>;

  #[wasm_bindgen(method, structural, getter = propagation)]
  pub fn get_propagation(this: &GateParams) -> Option<u32>;

  #[wasm_bindgen(method, structural, getter = type)]
  pub fn get_type(this: &GateParams) -> String;
}

#[wasm_bindgen]
extern "C" {
  pub type PortParams;

  #[wasm_bindgen(method, structural, getter = id)]
  pub fn get_id(this: &PortParams) -> String;

  #[wasm_bindgen(method, structural, getter = dir)]
  pub fn get_dir(this: &PortParams) -> String;

  #[wasm_bindgen(method, structural, getter = bits)]
  pub fn get_bits(this: &PortParams) -> u32;

  #[wasm_bindgen(method, structural, getter = avec)]
  pub fn get_avec(this: &PortParams) -> Vec<u32>;

  #[wasm_bindgen(method, structural, getter = bvec)]
  pub fn get_bvec(this: &PortParams) -> Vec<u32>;
}

#[wasm_bindgen]
extern "C" {
  pub type TargetParams;

  #[wasm_bindgen(method, structural, getter = id)]
  pub fn get_id(this: &TargetParams) -> String;

  #[wasm_bindgen(method, structural, getter = port)]
  pub fn get_port(this: &TargetParams) -> String;

  #[wasm_bindgen(method, structural, getter = magnet)]
  pub fn get_magnet(this: &TargetParams) -> String;
}

#[wasm_bindgen]
extern "C" {
  pub type SigParams;

  #[wasm_bindgen(method, structural, getter = _bits)]
  pub fn get_bits(this: &SigParams) -> u32;

  #[wasm_bindgen(method, structural, getter = _avec)]
  pub fn get_avec(this: &SigParams) -> Vec<u32>;

  #[wasm_bindgen(method, structural, getter = _bvec)]
  pub fn get_bvec(this: &SigParams) -> Vec<u32>;
}

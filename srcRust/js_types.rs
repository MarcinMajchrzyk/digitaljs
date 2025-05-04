use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
  pub type JsGateParams;

  #[wasm_bindgen(method, structural, getter = bits)]
  pub fn get_bits(this: &JsGateParams) -> u32;

  #[wasm_bindgen(method, structural, getter = iputs)]
  pub fn get_inputs(this: &JsGateParams) -> Option<u32>;

  #[wasm_bindgen(method, structural, getter = label)]
  pub fn get_label(this: &JsGateParams) -> Option<String>;

  #[wasm_bindgen(method, structural, getter = net)]
  pub fn get_net(this: &JsGateParams) -> Option<String>;

  #[wasm_bindgen(method, structural, getter = numbase)]
  pub fn get_numbase(this: &JsGateParams) -> Option<String>;

  #[wasm_bindgen(method, structural, getter = propagation)]
  pub fn get_propagation(this: &JsGateParams) -> u32;

  #[wasm_bindgen(method, structural, getter = type)]
  pub fn get_type(this: &JsGateParams) -> String;

  #[wasm_bindgen(method, structural, getter = slice)]
  pub fn get_slice(this: &JsGateParams) -> Option<SliceType>;

  #[wasm_bindgen(method, structural, getter = polarity)]
  pub fn get_polarity(this: &JsGateParams) -> Option<PolarityStruct>;

  #[wasm_bindgen(method, structural, getter = leftOp)]
  pub fn get_left_op(this: &JsGateParams) -> Option<bool>;

  #[wasm_bindgen(method, structural, getter = constant)]
  pub fn get_constant(this: &JsGateParams) -> Option<u32>;
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

#[wasm_bindgen]
extern "C" {
  pub type IOmap;

  #[wasm_bindgen(method, structural, getter = port)]
  pub fn get_port(this: &IOmap) -> String;

  #[wasm_bindgen(method, structural, getter = io_id)]
  pub fn get_io_id(this: &IOmap) -> String;
}

#[wasm_bindgen]
extern "C" {
  pub type SliceType;

  #[wasm_bindgen(method, structural, getter = count)]
  pub fn get_count(this: &SliceType) -> u32;

  #[wasm_bindgen(method, structural, getter = first)]
  pub fn get_first(this: &SliceType) -> u32;

  #[wasm_bindgen(method, structural, getter = total)]
  pub fn get_total(this: &SliceType) -> u32;
}

#[wasm_bindgen]
extern "C" {
  pub type PolarityStruct;

  #[wasm_bindgen(method, structural, getter = aload)]
  pub fn get_aload(this: &PolarityStruct) -> Option<bool>;

  #[wasm_bindgen(method, structural, getter = arst)]
  pub fn get_arst(this: &PolarityStruct) -> Option<bool>;

  #[wasm_bindgen(method, structural, getter = clock)]
  pub fn get_clock(this: &PolarityStruct) -> Option<bool>;

  #[wasm_bindgen(method, structural, getter = clr)]
  pub fn get_clr(this: &PolarityStruct) -> Option<bool>;

  #[wasm_bindgen(method, structural, getter = enable)]
  pub fn get_enable(this: &PolarityStruct) -> Option<bool>;

  #[wasm_bindgen(method, structural, getter = set)]
  pub fn get_set(this: &PolarityStruct) -> Option<bool>;
}
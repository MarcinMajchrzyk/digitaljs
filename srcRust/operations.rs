use std::collections::HashMap;

use crate::cell_arith::{add, add_c, arith_comp_const_op, arith_comp_op, arith_const_op, arith_op, div, div_c, equal, equal_c, greater, greater_c, greater_equal, greater_equal_c, less, less_c, less_equal, less_equal_c, modulo, modulo_c, mul, mul_c, not_equal, not_equal_c, power, power_c, shift_left, shift_left_c, shift_right, shift_right_c, sub, sub_c, ArithBinop, ArithComp, ArithConstBinop, ArithConstComp};
use crate::cell_bus::{bus_group, bus_slice};
use crate::cell_dff::{dff, DffState};
use crate::cell_io::{clock, constant};
use crate::cell_mux::{mux1hot_idx, mux_idx, mux_op, MuxIdx};
use crate::gate::{GateParams, SliceOptions};
use crate::vector3vl::Vec3vl;

use crate::cell_gates::{gate_11, gate_x1, Binop, Monop, not, and, or, xor, nand, nor, xnor};

pub enum ClockHack {
    Clock(Vec<(String, Vec3vl)>),
    Normal(Vec<(String, Vec3vl)>)
}

pub enum Operation {
    //Arith11(Monop),
    Arith21(ArithBinop, ArithConstBinop),
    ArithConst(ArithConstBinop, Option<u32>, Option<bool>),
    Comp(ArithComp, ArithConstComp),
    CompConst(ArithConstComp, Option<u32>, Option<bool>),
    BusGroup,
    BusSlice(Option<SliceOptions>),
    Clock(bool),
    Constant(Option<String>),
    Dff(DffState),
    Gate11(Monop),
    GateX1(Binop),
    Mux(MuxIdx),
    None
}

impl Operation {
    pub fn from_name(name: String, gate_params: &GateParams) -> Operation {
        match name.as_str() {
            "Not"       => Operation::Gate11(not),
            "And"       => Operation::GateX1(and),
            "Or"        => Operation::GateX1(or),
            "Xor"       => Operation::GateX1(xor),
            "Nand"      => Operation::GateX1(nand),
            "Nor"       => Operation::GateX1(nor),
            "Xnor"      => Operation::GateX1(xnor),

            "BusSlice"  => Operation::BusSlice(gate_params.slice),

            "BusGroup"  => Operation::BusGroup,
            "Constant"  => Operation::Constant(gate_params.constant_str.clone()),
            "Clock"     => Operation::Clock(false),
            "Dff"       => Operation::Dff(DffState::new(gate_params)),

            "Lt"        => Operation::Comp(less, less_c),
            "Le"        => Operation::Comp(less_equal, less_equal_c),
            "Gt"        => Operation::Comp(greater, greater_c),
            "Ge"        => Operation::Comp(greater_equal, greater_equal_c),
            "Eq"        => Operation::Comp(equal, equal_c),
            "Ne"        => Operation::Comp(not_equal, not_equal_c),

            "LtConst"   => Operation::CompConst(less_c,          gate_params.constant_num, gate_params.left_op),
            "LeConst"   => Operation::CompConst(less_equal_c,    gate_params.constant_num, gate_params.left_op),
            "GtConst"   => Operation::CompConst(greater_c,       gate_params.constant_num, gate_params.left_op),
            "GeConst"   => Operation::CompConst(greater_equal_c, gate_params.constant_num, gate_params.left_op),
            "EqConst"   => Operation::CompConst(equal_c,         gate_params.constant_num, gate_params.left_op),
            "NeConst"   => Operation::CompConst(not_equal_c,     gate_params.constant_num, gate_params.left_op),

            "Addition"       => Operation::Arith21(add, add_c),
            "Subtraction"    => Operation::Arith21(sub, sub_c),
            "Multiplication" => Operation::Arith21(mul, mul_c),
            "Division"       => Operation::Arith21(div, div_c),
            "Modulo"         => Operation::Arith21(modulo, modulo_c),
            "Power"          => Operation::Arith21(power, power_c),
            "ShiftLeft"      => Operation::Arith21(shift_left, shift_left_c),
            "ShiftRight"     => Operation::Arith21(shift_right, shift_right_c),

            "AdditionConst"       => Operation::ArithConst(add_c,         gate_params.constant_num, gate_params.left_op),
            "SubtractionConst"    => Operation::ArithConst(sub_c,         gate_params.constant_num, gate_params.left_op),
            "MultiplicationConst" => Operation::ArithConst(mul_c,         gate_params.constant_num, gate_params.left_op),
            "DivisionConst"       => Operation::ArithConst(div_c,         gate_params.constant_num, gate_params.left_op),
            "ModuloConst"         => Operation::ArithConst(modulo_c,      gate_params.constant_num, gate_params.left_op),
            "PowerConst"          => Operation::ArithConst(power_c,       gate_params.constant_num, gate_params.left_op),
            "ShiftLeftConst"      => Operation::ArithConst(shift_left_c,  gate_params.constant_num, gate_params.left_op),
            "ShiftRightConst"     => Operation::ArithConst(shift_right_c, gate_params.constant_num, gate_params.left_op),

            "Mux"       => Operation::Mux(mux_idx),
            "Mux1Hot"   => Operation::Mux(mux1hot_idx),
            _           => Operation::None
        }
    }

    pub fn op(&mut self, args: HashMap<String, Vec3vl>) -> Result<ClockHack, String> {
        match self {
            Operation::Arith21(op, op_c) => arith_op(args, op, op_c),
            Operation::ArithConst(op, constant, left_op) => arith_const_op(args, op, *constant, *left_op),
            Operation::Comp(op, op_c) => arith_comp_op(args, op, op_c),
            Operation::CompConst(op, constant, left_op) => arith_comp_const_op(args, op, *constant, *left_op),
            Operation::Gate11(op) => gate_11(op, &args),
            Operation::GateX1(op) => gate_x1(op, &args),
            Operation::BusSlice(options) => bus_slice(&args, options),
            Operation::BusGroup => bus_group(&args),
            Operation::Constant(value) => constant(value.clone()),
            Operation::Clock(clock_val) => clock(clock_val),
            Operation::Dff(state) => dff(args, state),
            Operation::Mux(op) => mux_op(args, op),
            Operation::None => Ok(ClockHack::Normal(vec![]))
        }
    }
}

fn neg (i: u32) -> u32 { !i }
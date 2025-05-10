use std::collections::HashMap;

use crate::cell_arith::{add, arith_comp_const_op, arith_comp_op, arith_const_op, arith_op, div, equal, greater, greater_equal, less, less_equal, modulo, mul, not_equal, power, shift_left, shift_right, sub, ArithBinop, ArithComp};
use crate::cell_bus::{bus_group, bus_slice};
use crate::cell_dff::dff;
use crate::cell_io::{clock, constant};
use crate::cell_mux::{mux1hot_idx, mux_idx, mux_op, MuxIdx};
use crate::gate::{GateParams, PolarityOptions};
use crate::vector3vl::Vec3vl;

use crate::cell_gates::{gate_11, gate_x1, Binop, Monop, not, and, or, xor, nand, nor, xnor};

pub enum ClockHack {
    Clock(Vec<(String, Vec3vl)>),
    Normal(Vec<(String, Vec3vl)>)
}

pub enum Operation {
    //Arith11(Monop),
    Arith21(ArithBinop),
    ArithConst(ArithBinop, u32, bool),
    Comp(ArithComp),
    CompConst(ArithComp, u32, bool),
    BusGroup,
    BusSlice(u32, u32),    // first, count
    Clock(bool),
    Constant,
    Gate11(Monop),
    GateX1(Binop),
    Dff(PolarityOptions, u32),
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

            "BusSlice"  => {
                let (f, c) = match &gate_params.slice {
                    Some(s) => {
                        (s.get_first(), s.get_count())
                    },
                    None => panic!()
                };
                Operation::BusSlice(f, c)
            },
            "BusGroup"  => Operation::BusGroup,
            "Constant"  => Operation::Constant,
            "Clock"     => Operation::Clock(false),
            //"Dff"       => Operation::Dff(gate_params.polarity, 0),

            "Lt"        => Operation::Comp(less),
            "Le"        => Operation::Comp(less_equal),
            "Gt"        => Operation::Comp(greater),
            "Ge"        => Operation::Comp(greater_equal),
            "Eq"        => Operation::Comp(equal),
            "Ne"        => Operation::Comp(not_equal),

            "LtConst"   => Operation::CompConst(less, gate_params.constant.unwrap(), gate_params.left_op.unwrap()),
            "LeConst"   => Operation::CompConst(less_equal, gate_params.constant.unwrap(), gate_params.left_op.unwrap()),
            "GtConst"   => Operation::CompConst(greater, gate_params.constant.unwrap(), gate_params.left_op.unwrap()),
            "GeConst"   => Operation::CompConst(greater_equal, gate_params.constant.unwrap(), gate_params.left_op.unwrap()),
            "EqConst"   => Operation::CompConst(equal, gate_params.constant.unwrap(), gate_params.left_op.unwrap()),
            "NeConst"   => Operation::CompConst(not_equal, gate_params.constant.unwrap(), gate_params.left_op.unwrap()),

            "Addition"  => Operation::Arith21(add),
            "Subtraction" => Operation::Arith21(sub),
            "Multiplication" => Operation::Arith21(mul),
            "Division"  => Operation::Arith21(div),
            "Modulo"    => Operation::Arith21(modulo),
            "Power"     => Operation::Arith21(power),
            "ShiftLeft" => Operation::Arith21(shift_left),
            "ShiftRight" => Operation::Arith21(shift_right),

            "AdditionConst" => Operation::ArithConst(add, gate_params.constant.unwrap(), gate_params.left_op.unwrap()),
            "SubtractionConst" => Operation::ArithConst(sub, gate_params.constant.unwrap(), gate_params.left_op.unwrap()),
            "MultiplicationConst" => Operation::ArithConst(mul, gate_params.constant.unwrap(), gate_params.left_op.unwrap()),
            "DivisionConst"  => Operation::ArithConst(div, gate_params.constant.unwrap(), gate_params.left_op.unwrap()),
            "ModuloConst"    => Operation::ArithConst(modulo, gate_params.constant.unwrap(), gate_params.left_op.unwrap()),
            "PowerConst"     => Operation::ArithConst(power, gate_params.constant.unwrap(), gate_params.left_op.unwrap()),
            "ShiftLeftConst" => Operation::ArithConst(shift_left, gate_params.constant.unwrap(), gate_params.left_op.unwrap()),
            "ShiftRightConst" => Operation::ArithConst(shift_right, gate_params.constant.unwrap(), gate_params.left_op.unwrap()),

            "Mux"       => Operation::Mux(mux_idx),
            "Mux1Hot"   => Operation::Mux(mux1hot_idx),
            _           => Operation::None
        }
    }

    pub fn op(&mut self, args: HashMap<String, Vec3vl>) -> ClockHack {
        match self {
            Operation::Arith21(op) => arith_op(args, op),
            Operation::ArithConst(op, constant, left_op) => arith_const_op(args, op, *constant, *left_op),
            Operation::Comp(op) => arith_comp_op(args, op),
            Operation::CompConst(op, constant, left_op) => arith_comp_const_op(args, op, *constant, *left_op),
            Operation::Gate11(op) => gate_11(op, &args),
            Operation::GateX1(op) => gate_x1(op, &args),
            Operation::BusSlice(f, c) => bus_slice(&args, *f, *c),
            Operation::BusGroup => bus_group(&args),
            Operation::Constant => constant(),      // TODO read arguments
            Operation::Clock(clock_val) => clock(clock_val),
            Operation::Dff(polarity, last_clk) => dff(args, polarity, last_clk),
            Operation::Mux(op) => mux_op(args, op),
            Operation::None => ClockHack::Normal(vec![])
        }
    }
}

fn neg (i: u32) -> u32 { !i }
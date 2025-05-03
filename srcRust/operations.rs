use std::collections::HashMap;

use crate::cell_arith::{add, arith_op, mul, sub, div, modulo, ArithBinop};
use crate::cell_bus::{bus_group, bus_slice};
use crate::cell_dff::dff;
use crate::cell_io::{clock, constant};
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
    BusGroup,
    BusSlice(u32, u32),    // first, count
    Clock(bool),
    Constant,
    Gate11(Monop),
    GateX1(Binop),
    Dff(PolarityOptions, u32),
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
            "Dff"       => Operation::Dff(gate_params.polarity, 0),
            "Addition"  => Operation::Arith21(add),
            "Subtraction" => Operation::Arith21(sub),
            "Multiplication" => Operation::Arith21(mul),
            "Division"  => Operation::Arith21(div),
            "Modulo"    => Operation::Arith21(modulo),
            _           => Operation::None
        }
    }

    pub fn op(&mut self, args: HashMap<String, Vec3vl>) -> ClockHack {
        match self {
            Operation::Arith21(op) => arith_op(args, op),
            Operation::Gate11(op) => gate_11(op, &args),
            Operation::GateX1(op) => gate_x1(op, &args),
            Operation::BusSlice(f, c) => bus_slice(&args, *f, *c),
            Operation::BusGroup => bus_group(&args),
            Operation::Constant => constant(),      // TODO read arguments
            Operation::Clock(clock_val) => clock(clock_val),
            Operation::Dff(polarity, last_clk) => dff(args, polarity, last_clk),
            Operation::None => ClockHack::Normal(vec![])
        }
    }
}

fn neg (i: u32) -> u32 { !i }
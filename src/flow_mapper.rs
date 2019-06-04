//! # Path
//! Data structures that represent a possible execution path through a WASM program.

extern crate bit_vec;
extern crate termcolor;

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::str;
use std::io::Write;
use self::termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use self::bit_vec::BitVec;
use crate::Operator;
use crate::{WasmDecoder, ParserState, ValidatingParser};

pub struct Path {
    instrs: Vec<usize>,
    branches: Vec<usize>,
    calls: Vec<usize>
}

pub struct Frame {
    instrs: Vec<usize>
}

pub struct Mapper {
    paths: Vec<Path>,
    instr_list: Vec<usize>,
    frames: Vec<Frame>,
    processed_instrs: BitVec,
    instrs_to_process: Vec<usize>,
    num_instrs_to_process: usize
}

impl Path {
    fn default () -> Path {
        let mut instrs:Vec<usize> = Vec::new();
        let mut branches:Vec<usize> = Vec::new();
        let mut calls:Vec<usize> = Vec::new();
        Path {instrs: instrs, branches: branches, calls: calls}
    }

    pub fn add_branch(&mut self, branch_index:usize) {
        self.branches.push(branch_index);
    }

    pub fn has_branch(&mut self, branch_index:usize) -> bool {
        self.branches.iter().any(|x| *x == branch_index)
    }

    pub fn add_call(&mut self, call_index:usize) {
        self.branches.push(call_index);
    }

    pub fn has_call(&mut self, call_index:usize) -> bool {
        self.branches.iter().any(|x| *x == call_index)
    }
}

impl Mapper {
    fn default () -> Mapper {
        let paths:Vec<Path> = Vec::new();
        let instr_list:Vec<usize> = Vec::new();
        let frames:Vec<Frame> = Vec::new();
        let processed_instrs:BitVec = BitVec::new();
        let instrs_to_process:Vec<usize> = Vec::new();
        let num_instrs_to_process:usize = 0;

        Mapper{
            paths: paths,
            instr_list: instr_list,
            frames: frames,
            processed_instrs: processed_instrs,
            instrs_to_process: instrs_to_process,
            num_instrs_to_process: num_instrs_to_process
        }
    }

    fn read_wasm(&mut self, file: &str) -> io::Result<Vec<u8>> {
        let mut data = Vec::new();
        let mut f = File::open(file)?;
        f.read_to_end(&mut data)?;
        Ok(data)
    }

    pub fn map(&mut self, input_file:&str) {

        println!("Analyzing {}...", input_file);

        self.num_instrs_to_process = 0;

        let mut main:Path = Path::default();
        let mut path:&mut Path = &mut main;
        let buf: Vec<u8> = self.read_wasm(&input_file).unwrap();

        let mut parser = ValidatingParser::new(&buf, None);
        let mut i = 0;

        let mut stdout = StandardStream::stdout(ColorChoice::Always);

        loop {

            i += 1;
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)));

            match *parser.read() {
                ParserState::Error(err) => panic!("Error: {:?}", err),
                ParserState::EndWasm => break,
                ParserState::BeginFunctionBody { .. } => (),
                _ => {
                    println!("{}. {:?}", i, *parser.last_state());
                    continue;
                }
            }
            
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
            println!("{}. {:?}", i, *parser.last_state());

            let mut reader = parser.create_validating_operator_parser();

            loop {
                
                i += 1;

                let read = reader.next(parser.get_resources());
                if let Ok(ref op) = read {
                    println!("{}. {:?}", i, op);
                    match op {
                        Operator::Unreachable => {}
                        Operator::Nop => {
                        }
                        Operator::Block { ty } => {
                        }
                        Operator::Loop { ty } => {
                        }
                        Operator::If { ty } => {
                        }
                        Operator::Else => {
                        }
                        Operator::End => {
                            break;
                        }
                        Operator::Br { relative_depth } => {
                            if !path.has_branch(i) {
                                path.add_branch(i);
                            }
                            self.instrs_to_process.push(i);
                        }
                        Operator::BrIf { relative_depth } => {
                            if !path.has_branch(i) {
                                path.add_branch(i);
                            }
                            self.instrs_to_process.push(i);
                        }
                        Operator::BrTable { ref table } => {
                            for relative_depth in table {
                                self.instrs_to_process.push(i + relative_depth as usize);
                            }
                        }
                        Operator::Return => {
                            break;
                        }
                        Operator::Call { function_index } => {
                            if !path.has_call(i) {
                                path.add_call(i);
                            }
                        }
                        Operator::CallIndirect { index, table_index } => {
                            if !path.has_call(i) {
                                path.add_call(i);
                            }
                        }
                        Operator::Drop => {
                        }
                        Operator::Select => {
                        }
                        Operator::GetLocal { local_index } => {
                        }
                        Operator::SetLocal { local_index } => {
                        }
                        Operator::TeeLocal { local_index } => {
                        }
                        Operator::GetGlobal { global_index } => {
                        }
                        Operator::SetGlobal { global_index } => {
                        }
                        Operator::I32Load { ref memarg } => {
                        }
                        Operator::I64Load { ref memarg } => {
                        }
                        Operator::F32Load { ref memarg } => {
                        }
                        Operator::F64Load { ref memarg } => {
                        }
                        Operator::I32Load8S { ref memarg } => {
                        }
                        Operator::I32Load8U { ref memarg } => {
                        }
                        Operator::I32Load16S { ref memarg } => {
                        }
                        Operator::I32Load16U { ref memarg } => {
                        }
                        Operator::I64Load8S { ref memarg } => {
                        }
                        Operator::I64Load8U { ref memarg } => {
                        }
                        Operator::I64Load16S { ref memarg } => {
                        }
                        Operator::I64Load16U { ref memarg } => {
                        }
                        Operator::I64Load32S { ref memarg } => {
                        }
                        Operator::I64Load32U { ref memarg } => {
                        }
                        Operator::I32Store { ref memarg } => {
                        }
                        Operator::I64Store { ref memarg } => {
                        }
                        Operator::F32Store { ref memarg } => {
                        }
                        Operator::F64Store { ref memarg } => {
                        }
                        Operator::I32Store8 { ref memarg } => {
                        }
                        Operator::I32Store16 { ref memarg } => {
                        }
                        Operator::I64Store8 { ref memarg } => {
                        }
                        Operator::I64Store16 { ref memarg } => {
                        }
                        Operator::I64Store32 { ref memarg } => {
                        }
                        Operator::MemorySize {
                            reserved: memory_index,
                        } => {
                            
                        }
                        Operator::MemoryGrow {
                            reserved: memory_index,
                        } => {
                        }
                        Operator::I32Const { .. } => {

                        }
                        Operator::I64Const { .. } => {

                        }
                        Operator::F32Const { .. } => {

                        }
                        Operator::F64Const { .. } => {

                        }
                        Operator::I32Eqz => {
                        }
                        Operator::I32Eq
                        | Operator::I32Ne
                        | Operator::I32LtS
                        | Operator::I32LtU
                        | Operator::I32GtS
                        | Operator::I32GtU
                        | Operator::I32LeS
                        | Operator::I32LeU
                        | Operator::I32GeS
                        | Operator::I32GeU => {
                        }
                        Operator::I64Eqz => {
                        }
                        Operator::I64Eq
                        | Operator::I64Ne
                        | Operator::I64LtS
                        | Operator::I64LtU
                        | Operator::I64GtS
                        | Operator::I64GtU
                        | Operator::I64LeS
                        | Operator::I64LeU
                        | Operator::I64GeS
                        | Operator::I64GeU => {
                        }
                        Operator::F32Eq
                        | Operator::F32Ne
                        | Operator::F32Lt
                        | Operator::F32Gt
                        | Operator::F32Le
                        | Operator::F32Ge => {
                        }
                        Operator::F64Eq
                        | Operator::F64Ne
                        | Operator::F64Lt
                        | Operator::F64Gt
                        | Operator::F64Le
                        | Operator::F64Ge => {
                        }
                        Operator::I32Clz | Operator::I32Ctz | Operator::I32Popcnt => {
                        }
                        Operator::I32Add
                        | Operator::I32Sub
                        | Operator::I32Mul
                        | Operator::I32DivS
                        | Operator::I32DivU
                        | Operator::I32RemS
                        | Operator::I32RemU
                        | Operator::I32And
                        | Operator::I32Or
                        | Operator::I32Xor
                        | Operator::I32Shl
                        | Operator::I32ShrS
                        | Operator::I32ShrU
                        | Operator::I32Rotl
                        | Operator::I32Rotr => {
                        }
                        Operator::I64Clz | Operator::I64Ctz | Operator::I64Popcnt => {
                        }
                        Operator::I64Add
                        | Operator::I64Sub
                        | Operator::I64Mul
                        | Operator::I64DivS
                        | Operator::I64DivU
                        | Operator::I64RemS
                        | Operator::I64RemU
                        | Operator::I64And
                        | Operator::I64Or
                        | Operator::I64Xor
                        | Operator::I64Shl
                        | Operator::I64ShrS
                        | Operator::I64ShrU
                        | Operator::I64Rotl
                        | Operator::I64Rotr => {
                        }
                        Operator::F32Abs
                        | Operator::F32Neg
                        | Operator::F32Ceil
                        | Operator::F32Floor
                        | Operator::F32Trunc
                        | Operator::F32Nearest
                        | Operator::F32Sqrt => {
                        }
                        Operator::F32Add
                        | Operator::F32Sub
                        | Operator::F32Mul
                        | Operator::F32Div
                        | Operator::F32Min
                        | Operator::F32Max
                        | Operator::F32Copysign => {
                        }
                        Operator::F64Abs
                        | Operator::F64Neg
                        | Operator::F64Ceil
                        | Operator::F64Floor
                        | Operator::F64Trunc
                        | Operator::F64Nearest
                        | Operator::F64Sqrt => {
                        }
                        Operator::F64Add
                        | Operator::F64Sub
                        | Operator::F64Mul
                        | Operator::F64Div
                        | Operator::F64Min
                        | Operator::F64Max
                        | Operator::F64Copysign => {
                        }
                        Operator::I32WrapI64 => {
                        }
                        Operator::I32TruncSF32 | Operator::I32TruncUF32 => {
                        }
                        Operator::I32TruncSF64 | Operator::I32TruncUF64 => {
                        }
                        Operator::I64ExtendSI32 | Operator::I64ExtendUI32 => {
                        }
                        Operator::I64TruncSF32 | Operator::I64TruncUF32 => {
                        }
                        Operator::I64TruncSF64 | Operator::I64TruncUF64 => {
                        }
                        Operator::F32ConvertSI32 | Operator::F32ConvertUI32 => {
                        }
                        Operator::F32ConvertSI64 | Operator::F32ConvertUI64 => {
                        }
                        Operator::F32DemoteF64 => {
                        }
                        Operator::F64ConvertSI32 | Operator::F64ConvertUI32 => {
                        }
                        Operator::F64ConvertSI64 | Operator::F64ConvertUI64 => {
                        }
                        Operator::F64PromoteF32 => {
                        }
                        Operator::I32ReinterpretF32 => {
                        }
                        Operator::I64ReinterpretF64 => {
                        }
                        Operator::F32ReinterpretI32 => {
                        }
                        Operator::F64ReinterpretI64 => {
                        }
                        Operator::I32TruncSSatF32 | Operator::I32TruncUSatF32 => {
                        }
                        Operator::I32TruncSSatF64 | Operator::I32TruncUSatF64 => {
                        }
                        Operator::I64TruncSSatF32 | Operator::I64TruncUSatF32 => {
                        }
                        Operator::I64TruncSSatF64 | Operator::I64TruncUSatF64 => {
                        }
                        Operator::I32Extend16S | Operator::I32Extend8S => {
                        }

                        Operator::I64Extend32S | Operator::I64Extend16S | Operator::I64Extend8S => {
                        }

                        Operator::I32AtomicLoad { ref memarg }
                        | Operator::I32AtomicLoad16U { ref memarg }
                        | Operator::I32AtomicLoad8U { ref memarg } => {
                        }
                        Operator::I64AtomicLoad { ref memarg }
                        | Operator::I64AtomicLoad32U { ref memarg }
                        | Operator::I64AtomicLoad16U { ref memarg }
                        | Operator::I64AtomicLoad8U { ref memarg } => {
                        }
                        Operator::I32AtomicStore { ref memarg }
                        | Operator::I32AtomicStore16 { ref memarg }
                        | Operator::I32AtomicStore8 { ref memarg } => {
                        }
                        Operator::I64AtomicStore { ref memarg }
                        | Operator::I64AtomicStore32 { ref memarg }
                        | Operator::I64AtomicStore16 { ref memarg }
                        | Operator::I64AtomicStore8 { ref memarg } => {
                        }
                        Operator::I32AtomicRmwAdd { ref memarg }
                        | Operator::I32AtomicRmwSub { ref memarg }
                        | Operator::I32AtomicRmwAnd { ref memarg }
                        | Operator::I32AtomicRmwOr { ref memarg }
                        | Operator::I32AtomicRmwXor { ref memarg }
                        | Operator::I32AtomicRmw16UAdd { ref memarg }
                        | Operator::I32AtomicRmw16USub { ref memarg }
                        | Operator::I32AtomicRmw16UAnd { ref memarg }
                        | Operator::I32AtomicRmw16UOr { ref memarg }
                        | Operator::I32AtomicRmw16UXor { ref memarg }
                        | Operator::I32AtomicRmw8UAdd { ref memarg }
                        | Operator::I32AtomicRmw8USub { ref memarg }
                        | Operator::I32AtomicRmw8UAnd { ref memarg }
                        | Operator::I32AtomicRmw8UOr { ref memarg }
                        | Operator::I32AtomicRmw8UXor { ref memarg } => {
                        }
                        Operator::I64AtomicRmwAdd { ref memarg }
                        | Operator::I64AtomicRmwSub { ref memarg }
                        | Operator::I64AtomicRmwAnd { ref memarg }
                        | Operator::I64AtomicRmwOr { ref memarg }
                        | Operator::I64AtomicRmwXor { ref memarg }
                        | Operator::I64AtomicRmw32UAdd { ref memarg }
                        | Operator::I64AtomicRmw32USub { ref memarg }
                        | Operator::I64AtomicRmw32UAnd { ref memarg }
                        | Operator::I64AtomicRmw32UOr { ref memarg }
                        | Operator::I64AtomicRmw32UXor { ref memarg }
                        | Operator::I64AtomicRmw16UAdd { ref memarg }
                        | Operator::I64AtomicRmw16USub { ref memarg }
                        | Operator::I64AtomicRmw16UAnd { ref memarg }
                        | Operator::I64AtomicRmw16UOr { ref memarg }
                        | Operator::I64AtomicRmw16UXor { ref memarg }
                        | Operator::I64AtomicRmw8UAdd { ref memarg }
                        | Operator::I64AtomicRmw8USub { ref memarg }
                        | Operator::I64AtomicRmw8UAnd { ref memarg }
                        | Operator::I64AtomicRmw8UOr { ref memarg }
                        | Operator::I64AtomicRmw8UXor { ref memarg } => {
                        }
                        Operator::I32AtomicRmwXchg { ref memarg }
                        | Operator::I32AtomicRmw16UXchg { ref memarg }
                        | Operator::I32AtomicRmw8UXchg { ref memarg } => {
                        }
                        Operator::I32AtomicRmwCmpxchg { ref memarg }
                        | Operator::I32AtomicRmw16UCmpxchg { ref memarg }
                        | Operator::I32AtomicRmw8UCmpxchg { ref memarg } => {
                        }
                        Operator::I64AtomicRmwXchg { ref memarg }
                        | Operator::I64AtomicRmw32UXchg { ref memarg }
                        | Operator::I64AtomicRmw16UXchg { ref memarg }
                        | Operator::I64AtomicRmw8UXchg { ref memarg } => {
                        }
                        Operator::I64AtomicRmwCmpxchg { ref memarg }
                        | Operator::I64AtomicRmw32UCmpxchg { ref memarg }
                        | Operator::I64AtomicRmw16UCmpxchg { ref memarg }
                        | Operator::I64AtomicRmw8UCmpxchg { ref memarg } => {
                        }
                        Operator::Wake { ref memarg } => {
                        }
                        Operator::I32Wait { ref memarg } => {
                        }
                        Operator::I64Wait { ref memarg } => {
                        }
                        Operator::RefNull => {
                        }
                        Operator::RefIsNull => {
                        }
                        Operator::V128Load { ref memarg } => {
                        }
                        Operator::V128Store { ref memarg } => {
                        }
                        Operator::V128Const { .. } => {
                        }
                        Operator::V8x16Shuffle { ref lines } => {
                        }
                        Operator::I8x16Splat | Operator::I16x8Splat | Operator::I32x4Splat => {
                        }
                        Operator::I64x2Splat => {
                        }
                        Operator::F32x4Splat => {
                        }
                        Operator::F64x2Splat => {
                        }
                        Operator::I8x16ExtractLaneS { line } | Operator::I8x16ExtractLaneU { line } => {
                        }
                        Operator::I16x8ExtractLaneS { line } | Operator::I16x8ExtractLaneU { line } => {
                        }
                        Operator::I32x4ExtractLane { line } => {
                        }
                        Operator::I8x16ReplaceLane { line } => {
                        }
                        Operator::I16x8ReplaceLane { line } => {
                        }
                        Operator::I32x4ReplaceLane { line } => {
                        }
                        Operator::I64x2ExtractLane { line } => {
                        }
                        Operator::I64x2ReplaceLane { line } => {
                        }
                        Operator::F32x4ExtractLane { line } => {
                        }
                        Operator::F32x4ReplaceLane { line } => {
                        }
                        Operator::F64x2ExtractLane { line } => {
                        }
                        Operator::F64x2ReplaceLane { line } => {
                        }
                        Operator::I8x16Eq
                        | Operator::I8x16Ne
                        | Operator::I8x16LtS
                        | Operator::I8x16LtU
                        | Operator::I8x16GtS
                        | Operator::I8x16GtU
                        | Operator::I8x16LeS
                        | Operator::I8x16LeU
                        | Operator::I8x16GeS
                        | Operator::I8x16GeU
                        | Operator::I16x8Eq
                        | Operator::I16x8Ne
                        | Operator::I16x8LtS
                        | Operator::I16x8LtU
                        | Operator::I16x8GtS
                        | Operator::I16x8GtU
                        | Operator::I16x8LeS
                        | Operator::I16x8LeU
                        | Operator::I16x8GeS
                        | Operator::I16x8GeU
                        | Operator::I32x4Eq
                        | Operator::I32x4Ne
                        | Operator::I32x4LtS
                        | Operator::I32x4LtU
                        | Operator::I32x4GtS
                        | Operator::I32x4GtU
                        | Operator::I32x4LeS
                        | Operator::I32x4LeU
                        | Operator::I32x4GeS
                        | Operator::I32x4GeU
                        | Operator::F32x4Eq
                        | Operator::F32x4Ne
                        | Operator::F32x4Lt
                        | Operator::F32x4Gt
                        | Operator::F32x4Le
                        | Operator::F32x4Ge
                        | Operator::F64x2Eq
                        | Operator::F64x2Ne
                        | Operator::F64x2Lt
                        | Operator::F64x2Gt
                        | Operator::F64x2Le
                        | Operator::F64x2Ge
                        | Operator::V128And
                        | Operator::V128Or
                        | Operator::V128Xor
                        | Operator::I8x16Add
                        | Operator::I8x16AddSaturateS
                        | Operator::I8x16AddSaturateU
                        | Operator::I8x16Sub
                        | Operator::I8x16SubSaturateS
                        | Operator::I8x16SubSaturateU
                        | Operator::I8x16Mul
                        | Operator::I16x8Add
                        | Operator::I16x8AddSaturateS
                        | Operator::I16x8AddSaturateU
                        | Operator::I16x8Sub
                        | Operator::I16x8SubSaturateS
                        | Operator::I16x8SubSaturateU
                        | Operator::I16x8Mul
                        | Operator::I32x4Add
                        | Operator::I32x4Sub
                        | Operator::I32x4Mul
                        | Operator::I64x2Add
                        | Operator::I64x2Sub
                        | Operator::F32x4Add
                        | Operator::F32x4Sub
                        | Operator::F32x4Mul
                        | Operator::F32x4Div
                        | Operator::F32x4Min
                        | Operator::F32x4Max
                        | Operator::F64x2Add
                        | Operator::F64x2Sub
                        | Operator::F64x2Mul
                        | Operator::F64x2Div
                        | Operator::F64x2Min
                        | Operator::F64x2Max => {
                        }
                        Operator::V128Not
                        | Operator::I8x16Neg
                        | Operator::I16x8Neg
                        | Operator::I32x4Neg
                        | Operator::I64x2Neg
                        | Operator::F32x4Abs
                        | Operator::F32x4Neg
                        | Operator::F32x4Sqrt
                        | Operator::F64x2Abs
                        | Operator::F64x2Neg
                        | Operator::F64x2Sqrt
                        | Operator::I32x4TruncSF32x4Sat
                        | Operator::I32x4TruncUF32x4Sat
                        | Operator::I64x2TruncSF64x2Sat
                        | Operator::I64x2TruncUF64x2Sat
                        | Operator::F32x4ConvertSI32x4
                        | Operator::F32x4ConvertUI32x4
                        | Operator::F64x2ConvertSI64x2
                        | Operator::F64x2ConvertUI64x2 => {
                        }
                        Operator::V128Bitselect => {
                        }
                        Operator::I8x16AnyTrue
                        | Operator::I8x16AllTrue
                        | Operator::I16x8AnyTrue
                        | Operator::I16x8AllTrue
                        | Operator::I32x4AnyTrue
                        | Operator::I32x4AllTrue
                        | Operator::I64x2AnyTrue
                        | Operator::I64x2AllTrue => {
                        }
                        Operator::I8x16Shl
                        | Operator::I8x16ShrS
                        | Operator::I8x16ShrU
                        | Operator::I16x8Shl
                        | Operator::I16x8ShrS
                        | Operator::I16x8ShrU
                        | Operator::I32x4Shl
                        | Operator::I32x4ShrS
                        | Operator::I32x4ShrU
                        | Operator::I64x2Shl
                        | Operator::I64x2ShrS
                        | Operator::I64x2ShrU => {
                        }

                        Operator::MemoryInit { segment } => {
                        }
                        Operator::DataDrop { segment } => {
                        }
                        Operator::MemoryCopy | Operator::MemoryFill => {
                        }
                        Operator::TableInit { segment } => {
                        }
                        Operator::ElemDrop { segment } => {
                        }
                        Operator::TableCopy => {
                        }
                        Operator::TableGet { table } => {
                        }
                        Operator::TableSet { table } => {
                        }
                        Operator::TableGrow { table } => {
                        }
                        Operator::TableSize { table } => {
                        }
                    }
                } else {
                    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)));
                    println!("Bad wasm code {:?}", read.err());
                }
            }
        }
    }
}

/// Initializes a path mapper.
pub fn new_mapper() -> Mapper {
    Mapper::default()
}

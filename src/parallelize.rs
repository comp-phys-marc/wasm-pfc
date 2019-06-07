//! # Parallelize
//! Data structures that represent the possible parallel executions of portions of WASM programs.

extern crate termcolor;
extern crate print_flat_tree;

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::process::Command;
use std::str;
use std::io::Write;
use std::collections::HashMap;
use primitives::{Type, MemoryImmediate};
use self::print_flat_tree::fmt;
use self::termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use crate::Operator;
use crate::{WasmDecoder, ParserState, ParserInput, ValidatingParser, ValidatingOperatorParser};
use crate::operators_validator::WasmModuleResources;

#[derive(Clone, Debug)]
pub struct Node {
    id: usize,
    instrs: Vec<u8>,
    branches: HashMap<usize, usize>,
    calls: HashMap<usize, usize>,
    start: usize,
    end: usize,
    children: HashMap<usize, Node>,
    constants: HashMap<usize, Type>,
    input_variables: HashMap<usize, Type>,
    output_variables: HashMap<usize, Type>,
    data_couplings: HashMap<usize, usize>,
    blocks: HashMap<usize, usize>
}

impl Node {
    fn default () -> Node {
        let instrs:Vec<u8> = Vec::new();
        let branches:HashMap<usize, usize> = HashMap::new();
        let calls:HashMap<usize, usize> = HashMap::new();
        let children:HashMap<usize, Node> = HashMap::new();
        let blocks:HashMap<usize, usize> = HashMap::new();
        let start = 0;
        let end = 0;
        let id = 0;
        let input_variables = HashMap::new();
        let output_variables = HashMap::new();
        let constants = HashMap::new();
        let data_couplings = HashMap::new();

        Node {
            id: id,
            instrs: instrs,
            branches: branches,
            calls: calls,
            start: start,
            end: end,
            children: children,
            blocks: blocks,
            input_variables: input_variables,
            output_variables: output_variables,
            constants: constants,
            data_couplings: data_couplings
        }
    }

    pub fn set_id(&mut self, id:usize) {
        self.id = id;
    }

    pub fn add_input_variable(&mut self, ty:Type) -> usize {
        let var_id = self.id + 0 + self.input_variables.len();
        self.input_variables.insert(var_id, ty);
        var_id
    }

    pub fn add_constant(&mut self, ty:Type) -> usize {
        let var_id = self.id + 1 + self.constants.len();
        self.constants.insert(var_id, ty);
        var_id
    }

    pub fn add_data_coupling(&mut self, memarg:&MemoryImmediate, var_id:usize) {
        // self.data_couplings.insert(memarg, var_id);
    }

    pub fn add_branch(&mut self, branch_index:usize, relative_depth:usize) {
        self.branches.insert(branch_index, relative_depth);
    }

    pub fn has_branch(&self, branch_index:usize) -> bool {
        self.branches.contains_key(&branch_index)
    }

    pub fn add_block(&mut self, start_index:usize, block_index:usize) {
        self.blocks.insert(start_index, block_index);
    }

    pub fn get_blocks(&self) -> HashMap<usize, usize> {
        self.blocks.clone()
    }

    pub fn add_call(&mut self, call_index:usize, function_index:usize) {
        self.calls.insert(call_index, function_index);
    }

    pub fn has_call(&self, call_index:usize) -> bool {
        self.calls.contains_key(&call_index)
    }

    pub fn get_calls(&self) -> HashMap<usize, usize> {
        self.calls.clone()
    }

    fn remove_calls(&mut self, calls:Vec<usize>) {
        for index in calls {
            self.calls.remove(&index);
        }
    }

    pub fn set_start(&mut self, start:usize) {
        self.start = start;
    }

    pub fn set_end(&mut self, end:usize) {
        self.end = end;
    }

    pub fn get_start(&self) -> usize {
        self.start
    }

    pub fn get_end(&self) -> usize {
        self.end
    }

    pub fn set_children(&mut self, children:HashMap<usize, Node>) {
        self.children = children;
    }

    pub fn add_children(&mut self, children:HashMap<usize, Node>) {
        self.children.extend(children);
    }

    pub fn add_child(&mut self, index:usize, child:Node) {
        self.children.insert(index, child);
    }

    pub fn has_child(&self, key:usize) -> bool {
        self.children.contains_key(&key)
    }

    pub fn get_child(&self, key:usize) -> Option<Node> {
        if self.children.contains_key(&key) {
            Some(self.children[&key].clone())
        } else {
            None
        }
    }

    fn remove_children(&mut self, children:Vec<usize>) {
        for index in children {
            self.children.remove(&index);
        }
    }

    pub fn set_instrs(&mut self, instrs:Vec<u8>) {
        self.instrs = instrs;
    }

    pub fn get_instrs(&mut self) -> Vec<u8> {
        self.instrs.clone()
    }

    pub fn remove_instrs(&mut self, start:usize, end:usize) {
        let mut new_instrs:Vec<u8> = Vec::new();
        let old_instrs = self.get_instrs();
        let mut i = 0;
        while i < start {
            new_instrs.push(old_instrs[i]);
            i += 1;
        }
        i = end;
        while i < old_instrs.len() {
            new_instrs.push(old_instrs[i]);
            i += 1;
        }
        self.set_instrs(new_instrs);
    }

    pub fn collapse(&mut self) -> Node {
        let mut removed_children:Vec<usize> = Vec::new();
        let mut removed_calls:Vec<usize> = Vec::new();
        for (call, index) in self.get_calls() {
            let old_child = self.get_child(index);
            match old_child {
                Some(mut old_child) => {
                    let child = old_child.collapse();
                    let child_instrs = child.clone().get_instrs();
                    let old_instrs = self.get_instrs();
                    let mut new_instrs:Vec<u8> = Vec::new();
                    let mut i = 0;
                    while i < call {
                        new_instrs.push(old_instrs[i]);
                        i += 1;
                    }
                    new_instrs.append(&mut child_instrs.clone());
                    i += child_instrs.len();
                    while i < old_instrs.len() {
                        new_instrs.push(old_instrs[i]);
                        i += 1;
                    }
                    self.set_instrs(new_instrs);
                    removed_children.push(index);
                    removed_calls.push(call);
                }
                None => {
                    println!("Can't collapse call {} to function {}", call, index);
                }
            }
        }
        self.remove_children(removed_children);
        self.remove_calls(removed_calls);
        self.clone()
    }
}

pub struct Mapper {
    blocks:HashMap<usize, Node>,
    functions:HashMap<usize, Node>
}

impl Mapper {
    fn default () -> Mapper {
        let blocks:HashMap<usize, Node> = HashMap::new();
        let functions:HashMap<usize, Node> = HashMap::new();

        Mapper{
            blocks: blocks,
            functions: functions
        }
    }

    pub fn unique_block_id(&self) -> usize {
        let functions = self.get_functions();
        let max = functions.keys().max();
        let mut true_max = 0;
        match max {
            Some(max) => {
                true_max = *max;
           }
           _ => ()
        }
        true_max + 1
    }

    fn add_block(&mut self, block:Node) -> usize {
        let blocks = self.get_blocks();
        let index = blocks.keys().max();
        let mut insert_index = 0;
        match index {
            Some(index) => {
                insert_index = *index + 1;
           }
           _ => ()
        }
        self.blocks.insert(insert_index, block);
        insert_index
    }

    fn get_functions(&self) -> HashMap<usize, Node> {
        self.functions.clone()
    }

    fn get_blocks(&self) -> HashMap<usize, Node> {
        self.blocks.clone()
    }

    fn get_block(&self, index:usize) -> Node {
        self.blocks[&index].clone()
    }

    fn remove_block(&mut self, index:usize) {
        self.blocks.remove(&index);
    }

    pub fn read_wasm(&mut self, file: &str) -> io::Result<Vec<u8>> {
        let mut data = Vec::new();
        let mut f = File::open(file)?;
        f.read_to_end(&mut data)?;
        Ok(data)
    }

    fn get_indices(&self, tree:HashMap<usize, Node>) -> Vec<usize> {
        let mut indices:Vec<usize> = Vec::new();
        for key in tree.keys() {
            indices.push(*key);
        }
        indices
    }

    pub fn print_tree(&self, nodes:HashMap<usize, Node>) {
        let indices = self.get_indices(nodes);
        print!("{}", fmt(&indices));
    }

    pub fn map(&mut self, buf:Vec<u8>) -> HashMap<usize, Node> {
        let mut parser = ValidatingParser::new(&buf, None);
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        let mut parser_input = None;
        
        let mut nodes:HashMap<usize, Node> = HashMap::new();
        let mut node:Node = Node::default();

        let mut func_start = 0;
        let mut func_index = 0;

        loop {

            stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)));

            let next_input = parser_input.take().unwrap_or(ParserInput::Default);
            parser_input = None;

            match *parser.read_with_input(next_input) {
                ParserState::Error(err) => println!("Error: {:?}", err),
                ParserState::EndWasm => break,
                ParserState::BeginFunctionBody { range } => {
                    parser_input = Some(ParserInput::SkipFunctionBody);
                    func_start = range.start;
                },
                _ => {
                    println!("{:?}", *parser.last_state());
                    continue;
                }
            }

            stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)));
            println!("{:?}", *parser.last_state());

            func_index = parser.current_func_index;

            let mut reader = parser.create_validating_operator_parser();
            let resources = parser.get_resources();

            node = self.map_helper(&mut reader, &buf, resources, func_start, func_index as usize);
            self.functions.insert(func_index as usize, node.clone());
            nodes.insert(func_index as usize, node.clone());
        }
        let indices = self.get_indices(nodes.clone());
        println!("First pass found {} functions:", indices.len());
        println!("{:?}", indices);
        nodes = self.expand_tree(nodes);
        nodes.clone()
    }

    fn expand_tree(&mut self, nodes:HashMap<usize, Node>) -> HashMap<usize, Node> {
        let mut tree = nodes.clone();
        
        for (index, mut func) in nodes {

            let mut stdin = io::stdin();
            let mut input = String::new();
            println!("Parallelize function {} (yes/no)?", index);
            stdin.read_line(&mut input);
            if input == "no\n" || input == "n\n" {
                continue;
            }
            
            println!("Analyzing function {}...", index);
            tree.remove(&index);
            let mut path_nodes = HashMap::new();
            let node = self.expand_func_tree_helper(func, index, tree.clone(), path_nodes);
            tree.insert(index, node);
        }
        tree
    }

    fn expand_block_tree_helper(&mut self, mut block:Node, node_id:usize, nodes:HashMap<usize, Node>, mut path_nodes:HashMap<usize, Node>) -> Node {
        let mut tree = nodes;

        let inner_blocks = block.get_blocks();
        println!("Found {} blocks in block {}", inner_blocks.keys().len(), node_id);
        for (start, index) in inner_blocks {
            let mut inner_block = self.get_block(index);
            println!("Breaking block {} out from block {}", index, node_id);
            let block_id = self.unique_block_id();
            let inner_block_end = inner_block.get_end();
            block.remove_instrs(start, inner_block_end);
            block.add_call(start, block_id);
            block.add_child(block_id, self.expand_block_tree_helper(inner_block.clone(), index, tree.clone(), path_nodes.clone()));
            self.functions.insert(block_id, inner_block.clone());
        }
        tree.remove(&node_id);
        tree.insert(node_id, block.clone());

        let calls = block.get_calls();
        println!("Found {} calls to other functions from block {}", calls.keys().len(), node_id);
        for (call, index) in calls {
            if path_nodes.contains_key(&index) {
                println!("Skipping reference loop in block {}", node_id);
                continue;
            }
            if block.has_child(index){
                println!("Skipping already registered call to function {} from block {}", index, node_id);
                continue;
            }
            path_nodes.insert(node_id, block.clone());
            println!("Registering call to function {} from block {}", index, node_id);
            block.add_child(index, self.expand_func_tree_helper(tree[&index].clone(), index, tree.clone(), path_nodes.clone()));
        }
        tree.remove(&node_id);
        tree.insert(node_id, block.clone());
        block
    }

    fn expand_func_tree_helper(&mut self, mut func:Node, node_id:usize, nodes:HashMap<usize, Node>, mut path_nodes:HashMap<usize, Node>) -> Node {
        let mut tree = nodes;

        let blocks = func.get_blocks();
        println!("Found {} blocks in function {}", blocks.keys().len(), node_id);
        for (start, index) in blocks {
            let mut block = self.get_block(index);
            println!("Breaking block {} out from function {}", index, node_id);
            let block_id = self.unique_block_id();
            func.add_call(start, block_id);
            path_nodes.insert(node_id, func.clone());
            func.add_child(block_id, self.expand_block_tree_helper(block.clone(), block_id, tree.clone(), path_nodes.clone()));
            self.functions.insert(block_id, block.clone());
        }
        tree.remove(&node_id);
        tree.insert(node_id, func.clone());

        let calls = func.get_calls();
        println!("Found {} calls to other functions from function {}", calls.keys().len(), node_id);
        for (call, index) in calls {
            if index == node_id {
                println!("Skipping self referencing call in function {}", node_id);
                continue;
            }
            if path_nodes.contains_key(&index) {
                println!("Skipping reference loop in function {}", node_id);
                continue;
            }
            if func.has_child(index) {
                println!("Skipping already registered call to function {} from function {}", index, node_id);
                continue;
            }
            path_nodes.insert(node_id, func.clone());
            println!("Registering call to function {} from function {}", index, node_id);
            func.add_child(index, self.expand_func_tree_helper(tree[&index].clone(), index, tree.clone(), path_nodes.clone()));
        }
        tree.remove(&node_id);
        tree.insert(node_id, func.clone());
        func
    }
    
    fn map_helper(&mut self, reader:&mut ValidatingOperatorParser, buf:&Vec<u8>, resources:&WasmModuleResources, start:usize, index:usize) -> Node {
        let mut process_next_line = true;
        let mut i = 0;

        let mut stdout = StandardStream::stdout(ColorChoice::Always);

        let mut node:Node = Node::default();

        node.set_start(start);
        node.set_id(index);

        loop {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)));
            let read = reader.next(resources);
            let position = reader.current_position();
            i += 1;
            
            if let Ok(ref op) = read {
                match op {
                    Operator::Unreachable => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)));
                    }
                    Operator::Nop => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)));
                    }
                    Operator::Block { ty } => {
                        let block_node = self.map_helper(reader, buf, resources, position, i);
                        let block_id = self.add_block(block_node);
                        node.add_block(position, block_id);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
                    }
                    Operator::Loop { ty } => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
                    }
                    Operator::If { ty } => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
                    }
                    Operator::Else => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
                    }
                    Operator::Return => {}
                    Operator::End => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)));
                        node.set_end(position + start);
                        println!("{}. {:?}", i, op);
                        break;
                    }
                    Operator::Br { relative_depth } => {
                        if !node.has_branch(i) {
                            node.add_branch(i, *relative_depth as usize);
                        }
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
                    }
                    Operator::BrIf { relative_depth } => {
                        if !node.has_branch(i) {
                            node.add_branch(i, *relative_depth as usize);
                        }
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
                    }
                    Operator::BrTable { ref table } => {
                        for relative_depth in table {
                            node.add_branch(i, table.buffer[relative_depth as usize] as usize);
                        }
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
                    }
                    Operator::Call { function_index } => {
                        if !node.has_call(position) {
                            node.add_call(position, *function_index as usize);
                        }
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)));
                    }
                    Operator::CallIndirect { index, table_index } => {
                        if !node.has_call(position) {
                            node.add_call(position, *table_index as usize);
                        }
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)));
                    }
                    Operator::Drop => {
                    }
                    Operator::Select => {
                    }
                    Operator::GetLocal { local_index } => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::SetLocal { local_index } => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::TeeLocal { local_index } => {
                    }
                    Operator::GetGlobal { global_index } => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::SetGlobal { global_index } => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::I32Load { ref memarg } => {
                        let var_id = node.add_input_variable(Type::I32);
                        node.add_data_coupling(memarg, var_id);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::I64Load { ref memarg } => {
                        let var_id = node.add_input_variable(Type::I64);
                        node.add_data_coupling(memarg, var_id);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::F32Load { ref memarg } => {
                        let var_id = node.add_input_variable(Type::F32);
                        node.add_data_coupling(memarg, var_id);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::F64Load { ref memarg } => {
                        let var_id = node.add_input_variable(Type::F64);
                        node.add_data_coupling(memarg, var_id);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::I32Load8S { ref memarg }
                    | Operator::I32Load8U { ref memarg }
                    | Operator::I32Load16S { ref memarg }
                    | Operator::I32Load16U { ref memarg } => {
                        let var_id = node.add_input_variable(Type::I32);
                        node.add_data_coupling(memarg, var_id);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::I64Load8S { ref memarg } 
                    | Operator::I64Load8U { ref memarg } 
                    | Operator::I64Load16U { ref memarg }
                    | Operator::I64Load32S { ref memarg }
                    | Operator::I64Load32U { ref memarg }
                    | Operator::I64Load16S { ref memarg } => {
                        let var_id = node.add_input_variable(Type::I64);
                        node.add_data_coupling(memarg, var_id);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::I32Store { ref memarg } => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::I64Store { ref memarg } => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::F32Store { ref memarg } => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::F64Store { ref memarg } => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::I32Store8 { ref memarg } => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::I32Store16 { ref memarg } => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::I64Store8 { ref memarg } => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::I64Store16 { ref memarg } => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::I64Store32 { ref memarg } => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
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
                        node.add_constant(Type::I32);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::I64Const { .. } => {
                        node.add_constant(Type::I64);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::F32Const { .. } => {
                        node.add_constant(Type::F32);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::F64Const { .. } => {
                        node.add_constant(Type::F64);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
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
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::I64AtomicLoad { ref memarg }
                    | Operator::I64AtomicLoad32U { ref memarg }
                    | Operator::I64AtomicLoad16U { ref memarg }
                    | Operator::I64AtomicLoad8U { ref memarg } => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::I32AtomicStore { ref memarg }
                    | Operator::I32AtomicStore16 { ref memarg }
                    | Operator::I32AtomicStore8 { ref memarg } => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::I64AtomicStore { ref memarg }
                    | Operator::I64AtomicStore32 { ref memarg }
                    | Operator::I64AtomicStore16 { ref memarg }
                    | Operator::I64AtomicStore8 { ref memarg } => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
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
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::V128Store { ref memarg } => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
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
                println!("{}. {:?}", i, op);
            } else {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)));
                panic!("Bad wasm code {:?}", read.err());
            }
        }
        let end = node.get_end();
        node.set_instrs(buf[start..end].to_vec());
        node
    }
}

/// Initializes a Node mapper.
pub fn new_mapper() -> Mapper {
    Mapper::default()
}

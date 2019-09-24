//! # Parallelize
//! Data structures that represent the various transformations of WASM programs throughout parallelization, 
//! dependency tree collapse and compilation to simulatable transfer functions for D-Wave

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
use primitives::Type;
use self::print_flat_tree::fmt;
use self::termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use crate::Operator;
use crate::{WasmDecoder, ParserState, ParserInput, ValidatingParser, ValidatingOperatorParser};
use crate::operators_validator::WasmModuleResources;
use crate::readers::FunctionBody;


/// The physical expression enum represents the valid
/// operations and data types that can be understood by PyQUBO.
#[derive(Clone, Debug)]
pub enum PhysicalExpression {
    Add{ operand_one: PhysicalExpression, operand_two: PhysicalExpression },
    Mul{ operand_one: PhysicalExpression, operand_two: PhysicalExpression },
    Spin{ val: bool }, // 0 represents -1
    Num{ val: usize },
    Binary{ val: bool }
}


/// The abstract operation enum represents logical operations
/// that can be compiled to simulatable transfer functions
/// for quantum annealers.
#[derive(Clone, Debug)]
pub enum AbstractExpression {
    Spin { id: usize },
    Num { val: usize },
    Add { ty: Type },
    Mul { ty: Type }
}


/// A QUBO represents a nestable quantum unconstrained
/// boolean optimization problem expression.
#[derive(Clone, Debug)]
pub struct QUBO {
    id: usize, // maps each QUBO to its node
    expression: PhysicalExpression // low level boolean expressions
}


impl QUBO {
    fn default (node_id:usize) -> QUBO {

        QUBO {
            id: node_id,
            expression: None
        }
    }
}


/// A node represents a segment of WASM code
/// These include functions and blocks at first,
/// then are transformed to combinational segments 
/// of code after parallelization.
#[derive(Clone, Debug)]
pub struct Node {
    id: usize, // each function and block has an id
    instrs: Vec<u8>, // hex instructions of the node
    branches: HashMap<usize, usize>, // internal locations and targets of branches
    calls: HashMap<usize, usize>, // calls to other functions
    start: usize, // where the node's insturctions start in the WASM source file
    end: usize, // where the node's insturctions end in the WASM source file
    children: HashMap<usize, Node>, // calls to other functions, or internal blocks of code
    constants: HashMap<usize, Type>, // constants instantiated within the scope of the node
    internal_variables: HashMap<usize, Type>, // internal variables that will be used to simulate flow control
    input_variables: HashMap<usize, Type>, // all input variables including parameters, memory references, global references are given ids
    output_variables: HashMap<usize, Type>, // all output varibles including writes to memory and returns
    global_input_data_couplings: HashMap<usize, usize>, // map of global variable locations to the Spind node's input variable ids
    global_output_data_couplings: HashMap<usize, usize>, // map of global variable locations to the Spind node's output variable ids
    flow_control_couplings: HashMap<usize, usize>, // map of instruction locations to Spind flow control variable ids
    input_data_couplings: HashMap<usize, usize>, // map of memory locations to the Spind node's input variable ids
    output_data_couplings: HashMap<usize, usize>, // map of memory locations to the Spind node's output variable ids
    blocks: HashMap<usize, usize>, // internal blocks' locations mapped to their ids as maintained by the mapper
    operations: HashMap<usize, AbstractExpression> // simulatable operations
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
        let internal_variables = HashMap::new();
        let input_variables = HashMap::new();
        let output_variables = HashMap::new();
        let constants = HashMap::new();
        let flow_control_couplings = HashMap::new();
        let input_data_couplings = HashMap::new();
        let output_data_couplings = HashMap::new();
        let global_input_data_couplings = HashMap::new();
        let global_output_data_couplings = HashMap::new();
        let operations = HashMap::new();

        Node {
            id: id,
            instrs: instrs,
            branches: branches,
            calls: calls,
            start: start,
            end: end,
            children: children,
            blocks: blocks,
            internal_variables: internal_variables,
            input_variables: input_variables,
            output_variables: output_variables,
            constants: constants,
            flow_control_couplings: flow_control_couplings,
            input_data_couplings: input_data_couplings,
            output_data_couplings: output_data_couplings,
            global_input_data_couplings: global_input_data_couplings,
            global_output_data_couplings: global_output_data_couplings,
            operations: operations
        }
    }

    // lowers the node's code to a representation compatible with PyQUBO
    pub fn lower(&mut self) -> QUBO {

        // couplings can be made between all the types of variables
        let input_variables = self.get_input_variables(); 
        let internal_variables = self.get_internal_variables();
        let constants = self.get_constants();

        // describe the node to the user
        println!("Node {} has {} input variabes, {} internal variables coupled with other nodes, and {} constants.", self.id, input_variables.len(), internal_variables.len(), constants.len());

        // ask the user if they would still like to lower the node
        let mut stdin = io::stdin();
        let mut input = String::new();
        println!("Do you want to lower node {} (yes/no)?", self.id);
        stdin.read_line(&mut input);
        if !(input == "no\n" || input == "n\n") {

            for (i, operation) in self.operations {

                match operation {
                    AbstractExpression::Add{ ty: Type::I32 } => {

                        let mut operand_one:AbstractExpression;
                        let mut operand_two:AbstractExpression;
                        let mut var_id:usize = 0;

                        match self.operations[i - 1] {
                            AbstractExpression::Spin { id }=> {
                                if !(ty == Type::I32) {
                                    panic!("Invalid operand for I32 addition near line {}!", i - 1);
                                } else {
                                    var_id = id;
                                }
                            }
                        }

                        match self.operations[i - 2] {
                            AbstractExpression::Spin { id }=> {
                                if !(ty == Type::I32) {
                                    panic!("Invalid operand for I32 addition near line {}!", i - 2);
                                } else {
                                    var_id = id;
                                }
                            }
                        }

                        match internal_variables.get(&i) {
                            Some(internal) => {
                                if *internal == var_id && self.has_child(i) {
                                    let child = self.get_child(i);
                                    let child_variables = child.get_input_variables();
                                    let coupled_var = self.get_flow_control_couplings()[var_id];
                                    let child_var = child_variables[coupled_var];

                                    // ask the user if they would like to lower the nested node
                                    let mut stdin = io::stdin();
                                    let mut input = String::new();
                                    println!("Do you want to lower the nested node {} (yes/no)?", child.id);
                                    stdin.read_line(&mut input);
                                    if !(input == "no\n" || input == "n\n") {
                                        let sub_expression = child.lower();
                                    } else {
                                        let sub_expression = QUBO::default(child.id);
                                    }
                                }
                            }
                        }
                    }
                    AbstractExpression::Add{ ty: Type::I64 } => {
                        
                    }
                    AbstractExpression::Add{ ty: Type::F32 } => {
                        
                    }
                    AbstractExpression::Add{ ty: Type::F64 } => {
                        
                    }
                    AbstractExpression::Mul{ ty: Type::I32 } => {
                        
                    }
                    AbstractExpression::Mul{ ty: Type::I64 } => {
                        
                    }
                    AbstractExpression::Mul{ ty: Type::F32 } => {
                        
                    }
                    AbstractExpression::Mul{ ty: Type::F64 } => {
                        
                    }
                }
            }

            self.clone()
        }
    }

    // sets the node id
    pub fn set_id(&mut self, id:usize) {
        self.id = id;
    }

    // registers an internal variable of any kind
    pub fn add_internal_variable(&mut self, i:usize, ty:Type) -> usize {
        self.internal_variables.insert(i, ty);
        i - 1
    }

    // registers an input variable of any kind
    pub fn add_input_variable(&mut self, ty:Type) -> usize {
        let var_id = self.id + self.input_variables.len();
        self.input_variables.insert(var_id, ty);
        var_id
    }

    // registers an output variable of any kind
    pub fn add_output_variable(&mut self, ty:Type) -> usize {
        let var_id = self.id + self.output_variables.len();
        self.output_variables.insert(var_id, ty);
        var_id
    }

    // registers a locally scoped constant
    pub fn add_constant(&mut self, ty:Type) -> usize {
        let var_id = self.id + 1 + self.constants.len();
        self.constants.insert(var_id, ty);
        var_id
    }

    // registers a simulatable operation
     pub fn add_operation(&mut self, i:usize, op:AbstractExpression) {
        self.operations.insert(i, op);
    }

    // registers an internal data coupling for flow control simulation
    pub fn add_flow_control_coupling(&mut self, i:usize, var_id:usize) {
        self.flow_control_couplings.insert(i, var_id);
    }

    // registers a memory input data dependency
    pub fn add_input_data_coupling(&mut self, memarg:usize, var_id:usize) {
        self.input_data_couplings.insert(memarg as usize, var_id);
    }

    // registers a memory output data dependency
    pub fn add_output_data_coupling(&mut self, memarg:usize, var_id:usize) {
        self.output_data_couplings.insert(memarg as usize, var_id);
    }

    // registers a global input data dependency
    pub fn add_global_input_data_coupling(&mut self, memarg:usize, var_id:usize) {
        self.global_input_data_couplings.insert(memarg as usize, var_id);
    }

    // registers a global output data dependency
    pub fn add_global_output_data_coupling(&mut self, memarg:usize, var_id:usize) {
        self.global_output_data_couplings.insert(memarg as usize, var_id);
    }

    // registers a branch at a particular location with target depth
    pub fn add_branch(&mut self, branch_index:usize, relative_depth:usize) {
        self.branches.insert(branch_index, relative_depth);
    }

    // checks if a branch has been registered at the given index
    pub fn has_branch(&self, branch_index:usize) -> bool {
        self.branches.contains_key(&branch_index)
    }

    // registers the location of a block with the given id
    pub fn add_block(&mut self, start_index:usize, block_index:usize) {
        self.blocks.insert(start_index, block_index);
    }

    // returns the set of registered blocks
    pub fn get_blocks(&self) -> HashMap<usize, usize> {
        self.blocks.clone()
    }

    // registers the call to other functions found in this node
    pub fn add_call(&mut self, call_index:usize, function_index:usize) {
        self.calls.insert(call_index, function_index);
    }

    // checks if a call has been registered at the given index
    pub fn has_call(&self, call_index:usize) -> bool {
        self.calls.contains_key(&call_index)
    }

    // returns the set of registered calls
    pub fn get_calls(&self) -> HashMap<usize, usize> {
        self.calls.clone()
    }

    // returns the set of registered constants
    pub fn get_constants(&self) -> HashMap<usize, Type> {
        self.constants.clone()
    }

    // returns the set of registered internal variables
    pub fn get_internal_variables(&self) -> HashMap<usize, Type> {
        self.internal_variables.clone()
    }

    // returns the set of registered input variables
    pub fn get_input_variables(&self) -> HashMap<usize, Type> {
        self.input_variables.clone()
    }

    // returns the node's least recently registered input variable
    pub fn get_first_input_variable(&self) -> Type {
        let mut ty = Type::AnyRef;
        let index = self.input_variables.keys().min();

        match index {
            Some(index) => {
                ty = self.input_variables[index]
            }
            _ => {
                println!("Error: No input variables have been registered.")
            }
        }
        ty
    }

    // returns the set of registered flow control couplings
    pub fn get_flow_control_couplings(&self) -> HashMap<usize, usize> {
        self.flow_control_couplings.clone()
    }

    // returns the node's least recently registered flow control coupling
    pub fn get_first_flow_control_coupling(&self) -> usize {
        let mut coupling = 0;
        let index = self.flow_control_couplings.keys().min();

        match index {
            Some(index) => {
                coupling = self.flow_control_couplings[index];
            }
            _ => {
                println!("Error: No control flow couplings have been registered.");
            }
        }
        coupling
    }

    // checks if the variables with the given id is Spind to any global or memory dependency
    pub fn input_variable_is_param(&self, var_id:usize) -> bool {
        let mut param = true;

        for (loc, var) in self.global_input_data_couplings.clone() {
            if (var == var_id) {
                param = false
            }
        }
        for (loc, var) in self.input_data_couplings.clone() {
            if (var == var_id) {
                param = false
            }
        }
        param
    }

    // removes all calls
    fn remove_calls(&mut self, calls:Vec<usize>) {
        for index in calls {
            self.calls.remove(&index);
        }
    }

    // registers the location of the node in the source WASM file
    pub fn set_start(&mut self, start:usize) {
        self.start = start;
    }

    // registers the end of the node in the source WASM file
    pub fn set_end(&mut self, end:usize) {
        self.end = end;
    }

    // returns the location of the node in the source WASM file
    pub fn get_start(&self) -> usize {
        self.start
    }

    // returns the end of the node in the source WASM file
    pub fn get_end(&self) -> usize {
        self.end
    }

    // sets this node's list of child nodes
    pub fn set_children(&mut self, children:HashMap<usize, Node>) {
        self.children = children;
    }

    // add multiple new children to this node's list of child nodes
    pub fn add_children(&mut self, children:HashMap<usize, Node>) {
        self.children.extend(children);
    }

    // inserts a child at a given index in this node's list of child nodes
    pub fn add_child(&mut self, index:usize, child:Node) {
        self.children.insert(index, child);
    }

    // checks if this node's list of children contains a particular node
    pub fn has_child(&self, key:usize) -> bool {
        self.children.contains_key(&key)
    }

    // returns a particular node if it is registered a child of this node
    pub fn get_child(&self, key:usize) -> Option<Node> {
        if self.children.contains_key(&key) {
            Some(self.children[&key].clone())
        } else {
            None
        }
    }

    // clears this node's list of child nodes
    fn remove_children(&mut self, children:Vec<usize>) {
        for index in children {
            self.children.remove(&index);
        }
    }

    // sets this node's list of hex instructions
    pub fn set_instrs(&mut self, instrs:Vec<u8>) {
        self.instrs = instrs;
    }

    // returns this node's list of hex instructions
    pub fn get_instrs(&mut self) -> Vec<u8> {
        self.instrs.clone()
    }

    // clears a segment of this node's list of hex instructions
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
}


/// The mapper is responsible for performing the mapping of arbitrary 
/// input WASM to its parallel and simulatable form
pub struct Mapper {
    blocks:HashMap<usize, Node>, // registered code segments originally include ambiguous blocks,
    nodes:HashMap<usize, Node>, // and eventually only uniquely adressed nodes
}


impl Mapper {
    fn default () -> Mapper {
        let blocks:HashMap<usize, Node> = HashMap::new();
        let nodes:HashMap<usize, Node> = HashMap::new();

        Mapper{
            blocks: blocks,
            nodes: nodes,
        }
    }

    // returns a unique id so that a block can be normalized and introduced uniquely into the list of functions
    pub fn unique_block_id(&self) -> usize {
        let nodes = self.get_nodes();
        let max = nodes.keys().max();
        let mut true_max = 0;
        match max {
            Some(max) => {
                true_max = *max;
           }
           _ => ()
        }
        true_max + 1
    }

    // registers a block
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

    // returns the set of registered nodes
    fn get_nodes(&self) -> HashMap<usize, Node> {
        self.nodes.clone()
    }

    // returns the set of registered nodes
    fn get_blocks(&self) -> HashMap<usize, Node> {
        self.blocks.clone()
    }

    // returns a specific registered block
    fn get_block(&self, index:usize) -> Node {
        self.blocks[&index].clone()
    }

    // removes a registered block
    fn remove_block(&mut self, index:usize) {
        self.blocks.remove(&index);
    }

    // reads a WASM file
    pub fn read_wasm(&mut self, file: &str) -> io::Result<Vec<u8>> {
        let mut data = Vec::new();
        let mut f = File::open(file)?;
        f.read_to_end(&mut data)?;
        Ok(data)
    }

    // extracts the node indeces from a flat tree of nodes
    fn get_indices(&self, tree:HashMap<usize, Node>) -> Vec<usize> {
        let mut indices:Vec<usize> = Vec::new();
        for key in tree.keys() {
            indices.push(*key);
        }
        indices
    }

    // prints a flat tree of nodes
    pub fn print_tree(&self, nodes:HashMap<usize, Node>) {
        let indices = self.get_indices(nodes);
        print!("{}", fmt(&indices));
    }
    

    // Associates a function's type signature with its corresponding node
    fn attach_signature(&mut self, resources:&WasmModuleResources, mut node:Node, func_count:usize, func_types:Vec<u32>) -> Node {

        // the function's type signature can be assigned after the node has been created
        let func_signature = resources.types()[func_types[func_count - 1] as usize].clone();
        let params = func_signature.params;
        let rets = func_signature.returns;
        let mut param = 0;
        let mut ret = 0;

        // the parser's resources object contains info about each function's params
        while param < params.len() {
            match params[param] {
                Type => {
                    let var_id = node.add_input_variable(params[param]);
                }
                _ => {
                    println!("Encountered unknown function parameter type.");
                    break;
                }
            }
            param += 1;
        }

        // the parser's resources object contains info about each function's outputs
        while ret < rets.len() {
            match rets[ret] {
                Type => {
                    let var_id = node.add_output_variable(rets[ret]);
                }
                _ => {
                    println!("Encountered unknown function ret type.");
                    break;
                }
            }
            ret += 1;
        }
        node.clone()
    }


    // entry point to the mapping functionality of the mapper
    pub fn map(&mut self, buf:Vec<u8>) -> HashMap<usize, Node> {

        // creates a new parser and colorful output stream
        let mut parser = ValidatingParser::new(&buf, None);
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        let mut parser_input = None;
        
        // one top-level node at a time is processed recursively 
        let mut nodes:HashMap<usize, Node> = HashMap::new();
        let mut node:Node = Node::default();

        // function parameters that can be determined before entering the function bodies themselves
        let mut func_start = 0;
        let mut func_end = 0;
        let mut func_index = 0;
        let mut func_types = Vec::new();

        // number of encountered functions
        let mut func_count = 0;

        // loop until we reach the end of the input WASM code
        loop {

            // white is for non-significant printout that does not represent a simulatable 
            // operation or control flow instruction
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)));

            // prepare the parser input
            let next_input = parser_input.take().unwrap_or(ParserInput::Default);
            parser_input = None;

            // parse the input
            match *parser.read_with_input(next_input) {
                // print encountered errors
                ParserState::Error(err) => println!("Error: {:?}", err),
                // break out of the loop when the file has been processed
                ParserState::EndWasm => break,
                // extract the function section entry's reference to the function's type signature
                ParserState::FunctionSectionEntry { 0: value } => { 
                    func_types.push(value);
                    continue;
                },
                // when we encounter the start of a function body extract what info we can and have the 
                // parser skip the body itself
                ParserState::BeginFunctionBody { range } => {
                    parser_input = Some(ParserInput::SkipFunctionBody);
                    func_start = range.start;
                    func_end = range.end;
                    node.set_end(func_end);
                },
                // print the parser's interpretation of everything else that is encountered
                _ => {
                    println!("{:?}", *parser.last_state());
                    continue;
                }
            }

            stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)));
            println!("{:?}", *parser.last_state());

            // the parser will have a reference to the most recent function its encountered
            func_index = parser.current_func_index;
            func_count += 1;

            // a new parser will handle the block
            let mut reader = parser.create_validating_operator_parser();

            // the parser has information about globals and keeps track of each function's type signature
            let resources = parser.get_resources();

            // the map helper will use the validating operator parser to recursively process the function
            // body and create a corresponding node
            node = self.map_helper(&mut reader, &buf, resources, func_start, func_index as usize, node.clone());

            node = self.attach_signature(resources, node.clone(), func_count, func_types.clone());

            // register the encountered function and corresponding processed node
            self.nodes.insert(func_index as usize, node.clone());
            nodes.insert(func_index as usize, node.clone());
        }

        // print out some basic metrics
        let indices = self.get_indices(nodes.clone());
        println!("First pass found {} functions:", indices.len());
        println!("{:?}", indices);

        // call the parallelizing function
        nodes = self.expand_tree(nodes);
        nodes.clone()
    }

    // provides optional parallelization of each processed node in the provided node tree
    fn expand_tree(&mut self, nodes:HashMap<usize, Node>) -> HashMap<usize, Node> {
        let mut tree = nodes.clone();
        
        for (index, mut func) in nodes {

            // ask the user if they would like to parallelize each top-level node
            let mut stdin = io::stdin();
            let mut input = String::new();
            println!("Parallelize function {} (yes/no)?", index);
            stdin.read_line(&mut input);
            if input == "no\n" || input == "n\n" {
                continue;
            }
            
            println!("Analyzing function {}...", index);
            
            // this node will be replaced with an expanded version
            tree.remove(&index);

            // this node will represent a possible execution path through the code
            let mut path_nodes = HashMap::new();

            // a helper function recursively expands the node
            let node = self.expand_func_tree_helper(func, index, tree.clone(), path_nodes);
            tree.insert(index, node);
        }
        tree
    }

    // recursively discovers and normalizes structure in the given block
    fn expand_block_tree_helper(&mut self, mut block:Node, node_id:usize, nodes:HashMap<usize, Node>, mut path_nodes:HashMap<usize, Node>) -> Node {
        let mut tree = nodes;

        // normalizes block references to the node format for simplicity
        let inner_blocks = block.get_blocks();
        println!("Found {} blocks in block {}", inner_blocks.keys().len(), node_id);
        for (start, index) in inner_blocks {

            // get the inner block by index
            let mut inner_block = self.get_block(index);
            println!("Breaking block {} out from block {}", index, node_id);

            // generate an id that won't collide with any other block or function's id
            let block_id = self.unique_block_id();

            // split the inner block's code out from the outer node's
            let inner_block_end = inner_block.get_end();
            block.remove_instrs(start, inner_block_end);

            // register a call to the separated block
            block.add_call(start, block_id);
            
            // recursively process the separated block 
            block.add_child(block_id, self.expand_block_tree_helper(inner_block.clone(), index, tree.clone(), path_nodes.clone()));

            // register the separated block as a node
            self.nodes.insert(block_id, inner_block.clone());
        }

        // updates the node in the node tree with any transformations made so far
        tree.remove(&node_id);
        tree.insert(node_id, block.clone());

        // traverses calls searching for feed-forward execution paths
        let calls = block.get_calls();
        println!("Found {} calls to other functions from block {}", calls.keys().len(), node_id);
        for (call, index) in calls {

            // reference loops will expand infinitely and can't be unrolled at compile time,
            // so these loops are not generally simulatable
            if path_nodes.contains_key(&index) {
                println!("Skipping reference loop in block {}", node_id);
                continue;
            }

            // skips functions already encountered; they don't need to be expanded again, just referenced again by location
            if block.has_child(index){
                println!("Skipping already registered call to function {} from block {}", index, node_id);
                continue;
            }

            // updates the node in the execution path with any transformations made in this frame
            path_nodes.insert(node_id, block.clone());

            println!("Registering call to function {} from block {}", index, node_id);

            // Any call that was not skipped is recursively analyzed
            block.add_child(index, self.expand_func_tree_helper(tree[&index].clone(), index, tree.clone(), path_nodes.clone()));
        }

        // updates the node in the node tree
        tree.remove(&node_id);
        tree.insert(node_id, block.clone());
        block
    }

    // recursively discovers and normalizes structure in the given function
    fn expand_func_tree_helper(&mut self, mut func:Node, node_id:usize, nodes:HashMap<usize, Node>, mut path_nodes:HashMap<usize, Node>) -> Node {
        let mut tree = nodes;

        // normalizes block references to the node format for simplicity
        let blocks = func.get_blocks();
        println!("Found {} blocks in function {}", blocks.keys().len(), node_id);
        for (start, index) in blocks {

            // get the block by index
            let mut block = self.get_block(index);
            println!("Breaking block {} out from function {}", index, node_id);

            // generate an id that won't collide with any other block or function's id
            let block_id = self.unique_block_id();

            // register a call to the block
            func.add_call(start, block_id);

            // updates the node in the execution path with any transformations made so far
            path_nodes.insert(node_id, func.clone());

            // recursively process the block 
            func.add_child(block_id, self.expand_block_tree_helper(block.clone(), block_id, tree.clone(), path_nodes.clone()));

            // register the block as a node
            self.nodes.insert(block_id, block.clone());
        }

        // updates the node in the node tree with any transformations made so far
        tree.remove(&node_id);
        tree.insert(node_id, func.clone());

        // traverses calls searching for feed-forward execution paths
        let calls = func.get_calls();
        println!("Found {} calls to other functions from function {}", calls.keys().len(), node_id);
        for (call, index) in calls {

            // skips self references since these can't be unrolled at compile time,
            // and aren't generally simulatable
            if index == node_id {
                println!("Skipping self referencing call in function {}", node_id);
                continue;
            }

            // reference loops will expand infinitely and can't be unrolled at compile time,
            // so these loops are not generally simulatable
            if path_nodes.contains_key(&index) {
                println!("Skipping reference loop in function {}", node_id);
                continue;
            }

            // skips functions already encountered; they don't need to be expanded again, just referenced again by location
            if func.has_child(index) {
                println!("Skipping already registered call to function {} from function {}", index, node_id);
                continue;
            }

            // updates the node in the execution path with any transformations made in this frame
            path_nodes.insert(node_id, func.clone());

            println!("Registering call to function {} from function {}", index, node_id);

            // Any call that was not skipped is recursively analyzed
            func.add_child(index, self.expand_func_tree_helper(tree[&index].clone(), index, tree.clone(), path_nodes.clone()));
        }

        // updates the node in the node tree
        tree.remove(&node_id);
        tree.insert(node_id, func.clone());
        func
    }

    // processes a function body using a validating operator parser
    fn map_helper(&mut self, reader:&mut ValidatingOperatorParser, buf:&Vec<u8>, resources:&WasmModuleResources, start:usize, index:usize, mut node:Node) -> Node {

        // the number of reads made by the operator parser
        let mut i = 0;

        // initiates a colorful output stream
        let mut stdout = StandardStream::stdout(ColorChoice::Always);

        // sets initial pre-determined node properties
        node.set_start(start);
        node.set_id(index);

        loop {

            // green is for simulatable instructions
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)));

            // read the next operator
            let read = reader.next(resources);

            // update the cursor position
            let position = reader.current_position();

            // update the read counter
            i += 1;

            if let Ok(ref op) = read {

                // mapping of WASM instructions to node properties including data couplings and abstract 
                // simulatable operations; a number of instructions are not yet supported

                // white is for non-critical code
                // yellow is for control dependencies
                // blue is for data dependencies
                // purple is for function calls
                // green is for simulatable operations

                match op {
                    Operator::Unreachable => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)));
                    }
                    Operator::Nop => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)));
                    }
                    Operator::Block { ty } => {

                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
                        print!("==== New Block: ");
                        println!("{}. {:?}", i, op);

                        // blocks can simply be registered... they don't have parameters
                        let block_node = self.map_helper(reader, buf, resources, position, i, Node::default());
                        let block_id = self.add_block(block_node);
                        node.add_block(i, block_id);

                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
                        print!("==== End of: ")
                    }
                    Operator::Loop { ty } => {

                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
                        print!("==== New Loop: ");
                        println!("{}. {:?}", i, op);

                        // loops don't have parameters so they can be registered as blocks
                        let loop_node = self.map_helper(reader, buf, resources, position, i, Node::default());
                        let loop_id = self.add_block(loop_node);
                        node.add_block(i, loop_id);

                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
                        print!("==== End of: ")
                    }
                    Operator::If { ty } => {

                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
                        print!("==== New If Condition: ");
                        println!("{}. {:?}", i, op);

                        // if conditions imply a single data dependency
                        let mut conditional_node = Node::default();
                        
                        // create variable to represent the condition
                        let outer_var_id = node.add_internal_variable(i, *ty);

                        // create data coupling to simulate flow control
                        let inner_var_id = conditional_node.add_input_variable(*ty);
                        conditional_node.add_flow_control_coupling(outer_var_id, inner_var_id);
                        
                        conditional_node = self.map_helper(reader, buf, resources, position, i, conditional_node);

                        // register the conditional block
                        let conditional_id = self.add_block(conditional_node.clone());
                        node.add_block(i, conditional_id);

                        // add a spin to each node
                        node.add_operation(i, AbstractExpression::Spin{ id: outer_var_id });
                        conditional_node.add_operation(i, AbstractExpression::Spin{ id: inner_var_id });

                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
                        print!("==== End of: ")
                    }
                    Operator::Else => {

                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));

                        // else implies a single data anti-dependency
                        // it needs to be constructed from within the if so we can have easy access to its coupling parameters
                        // however, it will be lifted out during the collapse of its top-level parent function

                        // we should have most recently registered a conditional node with only one flow control coupling
                        let couplings = node.get_flow_control_couplings();
                        let coupling_count = couplings.keys().len();

                        // we should have most recently registered a conditional node with only one input variable
                        let input_variables = node.get_input_variables();
                        let input_variable_count = input_variables.keys().len();

                        // if we aren't in a conditional already, don't process the else
                        if (coupling_count == 1 && input_variable_count == 1) {

                            print!("==== New Else Clause: ");
                            println!("{}. {:?}", i, op);

                            // get coupling details from the if condition details
                            let Spind_var_id = node.get_first_flow_control_coupling();
                            let input_type = node.get_first_input_variable();

                            let mut else_node = Node::default();

                            // create data anti-chain coupling to simulate flow control
                            let inner_var_id = else_node.add_input_variable(input_type);
                            else_node.add_flow_control_coupling(Spind_var_id, inner_var_id);

                            else_node = self.map_helper(reader, buf, resources, position, i, else_node);

                            // the else's end also terminates the if clause
                            let if_end = else_node.get_end();
                            node.set_end(if_end);

                            // register the else block
                            let else_id = self.add_block(else_node);
                            node.add_block(i, else_id);
                        
                            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
                            print!("==== End of: ");
                            println!("{}. {:?}", i, op);
                            
                            // finish processing the if node
                            break;
                        }
                    }
                    Operator::Return
                    | Operator::End => {

                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)));

                        // if the node represetns a function, the function end was already extracted from the function metadata
                        if (node.get_end() == 0) {
                            // otherwise, deduce the end from the number of loops performed within this frame
                            node.set_end(position + start);
                        }
                        println!("{}. {:?}", i, op);

                        // finish processing the node
                        break;
                    }
                    Operator::Br { relative_depth } => {
                        node.add_branch(i, *relative_depth as usize);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
                    }
                    Operator::BrIf { relative_depth } => {
                        node.add_branch(i, *relative_depth as usize);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
                    }
                    Operator::BrTable { ref table } => {
                        for relative_depth in table {
                            node.add_branch(i, table.buffer[relative_depth as usize] as usize);
                        }
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
                    }
                    Operator::Call { function_index } => {
                        node.add_call(i, *function_index as usize);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)));
                    }
                    Operator::CallIndirect { index, table_index } => {
                        node.add_call(i, *table_index as usize);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)));
                    }
                    Operator::Drop => { 
                        // TODO 
                    }
                    Operator::Select => { 
                        // TODO 
                    }
                    Operator::GetLocal { local_index } => {
                        let local_vars = self.get_input_variables();
                        let var_id = self.id + local_index;
                        let var_type = local_vars[var_id];
                        node.add_operation(i, AbstractExpression::Spin{ id: var_id });
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::SetLocal { local_index } => {
                        // TODO
                    }
                    Operator::TeeLocal { local_index } => { 
                        // TODO 
                    }
                    Operator::GetGlobal { global_index } => {
                        let var_id = node.add_input_variable(resources.globals()[*global_index as usize].content_type);
                        node.add_global_input_data_coupling(*global_index as usize, var_id);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::SetGlobal { global_index } => {
                        let var_id = node.add_output_variable(resources.globals()[*global_index as usize].content_type);
                        node.add_global_output_data_coupling(*global_index as usize, var_id);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::F32Load { ref memarg } => {
                        let var_id = node.add_input_variable(Type::F32);
                        node.add_input_data_coupling(memarg.offset as usize, var_id);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::F64Load { ref memarg } => {
                        let var_id = node.add_input_variable(Type::F64);
                        node.add_input_data_coupling(memarg.offset as usize, var_id);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::I32Load8S { ref memarg }
                    | Operator::I32Load { ref memarg }
                    | Operator::I32Load8U { ref memarg }
                    | Operator::I32Load16S { ref memarg }
                    | Operator::I32Load16U { ref memarg }
                    | Operator::I32AtomicLoad { ref memarg }
                    | Operator::I32AtomicLoad16U { ref memarg }
                    | Operator::I32AtomicLoad8U { ref memarg } => {
                        let var_id = node.add_input_variable(Type::I32);
                        node.add_input_data_coupling(memarg.offset as usize, var_id);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::I64Load8S { ref memarg } 
                    | Operator::I64Load { ref memarg }
                    | Operator::I64Load8U { ref memarg } 
                    | Operator::I64Load16U { ref memarg }
                    | Operator::I64Load32S { ref memarg }
                    | Operator::I64Load32U { ref memarg }
                    | Operator::I64Load16S { ref memarg }
                    | Operator::I64AtomicLoad { ref memarg }
                    | Operator::I64AtomicLoad32U { ref memarg }
                    | Operator::I64AtomicLoad16U { ref memarg }
                    | Operator::I64AtomicLoad8U { ref memarg } => {
                        let var_id = node.add_input_variable(Type::I64);
                        node.add_input_data_coupling(memarg.offset as usize, var_id);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::I32Store { ref memarg } 
                    | Operator::I32Store8 { ref memarg }
                    | Operator::I32Store16 { ref memarg }
                    | Operator::I32AtomicStore { ref memarg }
                    | Operator::I32AtomicStore8 { ref memarg }
                    | Operator::I32AtomicStore16 { ref memarg } => {
                        let var_id = node.add_output_variable(Type::I32);
                        node.add_output_data_coupling(memarg.offset as usize, var_id);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::I64Store { ref memarg }
                    | Operator::I64Store8 { ref memarg }
                    | Operator::I64Store16 { ref memarg }
                    | Operator::I64Store32 { ref memarg }
                    | Operator::I64AtomicStore { ref memarg }
                    | Operator::I64AtomicStore32 { ref memarg }
                    | Operator::I64AtomicStore16 { ref memarg }
                    | Operator::I64AtomicStore8 { ref memarg } => {
                        let var_id = node.add_output_variable(Type::I64);
                        node.add_output_data_coupling(memarg.offset as usize, var_id);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::F32Store { ref memarg } => {
                        let var_id = node.add_output_variable(Type::F32);
                        node.add_output_data_coupling(memarg.offset as usize, var_id);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::F64Store { ref memarg } => {
                        let var_id = node.add_output_variable(Type::F64);
                        node.add_output_data_coupling(memarg.offset as usize, var_id);
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::MemorySize {
                        reserved: memory_index,
                    } => { 
                        // TODO 
                    }
                    Operator::MemoryGrow {
                        reserved: memory_index,
                    } => { 
                        // TODO 
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
                        // TODO
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
                        // TODO
                    }
                    Operator::I64Eqz => {
                        // TODO
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
                        // TODO
                    }
                    Operator::F32Eq
                    | Operator::F32Ne
                    | Operator::F32Lt
                    | Operator::F32Gt
                    | Operator::F32Le
                    | Operator::F32Ge => {
                        // TODO
                    }
                    Operator::F64Eq
                    | Operator::F64Ne
                    | Operator::F64Lt
                    | Operator::F64Gt
                    | Operator::F64Le
                    | Operator::F64Ge => {
                        // TODO
                    }
                    Operator::I32Clz | Operator::I32Ctz | Operator::I32Popcnt => { 
                        // TODO 
                    }
                        // TODO
                    Operator::I32Add => {
                        node.add_operation(i, AbstractExpression::Add{ty: Type::I32});
                    }
                    Operator::I32Sub => {
                        // TODO
                    }
                    Operator::I32Mul => {
                        node.add_operation(i, AbstractExpression::Mul{ty: Type::I32});
                    }
                    Operator::I32DivS
                    | Operator::I32DivU => {
                        // TODO
                    }
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
                        // TODO
                    }
                    Operator::I64Clz | Operator::I64Ctz | Operator::I64Popcnt => {
                        // TODO
                    }
                    Operator::I64Add => {
                        node.add_operation(i, AbstractExpression::Add{ty: Type::I64});
                    }
                    Operator::I64Sub
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
                        // TODO
                    }
                    Operator::F32Abs
                    | Operator::F32Neg
                    | Operator::F32Ceil
                    | Operator::F32Floor
                    | Operator::F32Trunc
                    | Operator::F32Nearest
                    | Operator::F32Sqrt => {
                        // TODO
                    }
                    Operator::F32Add => {
                        node.add_operation(i, AbstractExpression::Add{ty: Type::F32});
                    }
                    Operator::F32Sub => {
                        // TODO
                    }
                    Operator::F32Mul => {
                        node.add_operation(i, AbstractExpression::Mul{ty: Type::F32});
                    }
                    | Operator::F32Div
                    | Operator::F32Min
                    | Operator::F32Max
                    | Operator::F32Copysign => {
                        // TODO
                    }
                    Operator::F64Abs
                    | Operator::F64Neg
                    | Operator::F64Ceil
                    | Operator::F64Floor
                    | Operator::F64Trunc
                    | Operator::F64Nearest
                    | Operator::F64Sqrt => {
                        // TODO
                    }
                    Operator::F64Add => {
                        node.add_operation(i, AbstractExpression::Add{ty: Type::F64});
                    }
                    | Operator::F64Sub
                    | Operator::F64Mul
                    | Operator::F64Div
                    | Operator::F64Min
                    | Operator::F64Max
                    | Operator::F64Copysign => {
                        // TODO
                    }
                    Operator::I32WrapI64 => {
                        // TODO
                    }
                    Operator::I32TruncSF32 | Operator::I32TruncUF32 => {
                        // TODO
                    }
                    Operator::I32TruncSF64 | Operator::I32TruncUF64 => {
                        // TODO
                    }
                    Operator::I64ExtendSI32 | Operator::I64ExtendUI32 => {
                        // TODO
                    }
                    Operator::I64TruncSF32 | Operator::I64TruncUF32 => {
                        // TODO
                    }
                    Operator::I64TruncSF64 | Operator::I64TruncUF64 => {
                        // TODO
                    }
                    Operator::F32ConvertSI32 | Operator::F32ConvertUI32 => {
                        // TODO
                    }
                    Operator::F32ConvertSI64 | Operator::F32ConvertUI64 => {
                        // TODO
                    }
                    Operator::F32DemoteF64 => {
                        // TODO
                    }
                    Operator::F64ConvertSI32 | Operator::F64ConvertUI32 => {
                        // TODO
                    }
                    Operator::F64ConvertSI64 | Operator::F64ConvertUI64 => {
                        // TODO
                    }
                    Operator::F64PromoteF32 => {
                        // TODO
                    }
                    Operator::I32ReinterpretF32 => {
                        // TODO
                    }
                    Operator::I64ReinterpretF64 => {
                        // TODO
                    }
                    Operator::F32ReinterpretI32 => {
                        // TODO
                    }
                    Operator::F64ReinterpretI64 => {
                        // TODO
                    }
                    Operator::I32TruncSSatF32 | Operator::I32TruncUSatF32 => {
                        // TODO
                    }
                    Operator::I32TruncSSatF64 | Operator::I32TruncUSatF64 => {
                        // TODO
                    }
                    Operator::I64TruncSSatF32 | Operator::I64TruncUSatF32 => {
                        // TODO
                    }
                    Operator::I64TruncSSatF64 | Operator::I64TruncUSatF64 => {
                        // TODO
                    }
                    Operator::I32Extend16S | Operator::I32Extend8S => {
                        // TODO
                    }

                    Operator::I64Extend32S | Operator::I64Extend16S | Operator::I64Extend8S => {
                        // TODO
                    }
                    Operator::I32AtomicRmwAdd { ref memarg }
                    | Operator::I32AtomicRmw16UAdd { ref memarg } 
                    | Operator::I32AtomicRmw8UAdd { ref memarg } => {
                        node.add_operation(i, AbstractExpression::Add{ty: Type::I32});
                    }
                    Operator::I64AtomicRmwAdd { ref memarg } 
                    | Operator::I64AtomicRmw32UAdd { ref memarg } 
                    | Operator::I64AtomicRmw8UAdd { ref memarg } => {
                        node.add_operation(i, AbstractExpression::Add{ty: Type::I64});
                    }
                    | Operator::I32AtomicRmwSub { ref memarg }
                    | Operator::I32AtomicRmwAnd { ref memarg }
                    | Operator::I32AtomicRmwOr { ref memarg }
                    | Operator::I32AtomicRmwXor { ref memarg }
                    | Operator::I32AtomicRmw16USub { ref memarg }
                    | Operator::I32AtomicRmw16UAnd { ref memarg }
                    | Operator::I32AtomicRmw16UOr { ref memarg }
                    | Operator::I32AtomicRmw16UXor { ref memarg }
                    | Operator::I32AtomicRmw8USub { ref memarg }
                    | Operator::I32AtomicRmw8UAnd { ref memarg }
                    | Operator::I32AtomicRmw8UOr { ref memarg }
                    | Operator::I32AtomicRmw8UXor { ref memarg } => {
                        // TODO
                    }
                    Operator::I64AtomicRmw32UAdd { ref memarg }
                    | Operator::I64AtomicRmw16UAdd { ref memarg }
                    | Operator::I64AtomicRmw8UAdd { ref memarg }  => {
                        node.add_operation(i, AbstractExpression::Add{ty: Type::I64});
                    }
                    Operator::I64AtomicRmwSub { ref memarg }
                    | Operator::I64AtomicRmwAnd { ref memarg }
                    | Operator::I64AtomicRmwOr { ref memarg }
                    | Operator::I64AtomicRmwXor { ref memarg }
                    | Operator::I64AtomicRmw32USub { ref memarg }
                    | Operator::I64AtomicRmw32UAnd { ref memarg }
                    | Operator::I64AtomicRmw32UOr { ref memarg }
                    | Operator::I64AtomicRmw32UXor { ref memarg }
                    | Operator::I64AtomicRmw16USub { ref memarg }
                    | Operator::I64AtomicRmw16UAnd { ref memarg }
                    | Operator::I64AtomicRmw16UOr { ref memarg }
                    | Operator::I64AtomicRmw16UXor { ref memarg }
                    | Operator::I64AtomicRmw8USub { ref memarg }
                    | Operator::I64AtomicRmw8UAnd { ref memarg }
                    | Operator::I64AtomicRmw8UOr { ref memarg }
                    | Operator::I64AtomicRmw8UXor { ref memarg } => {
                        // TODO
                    }
                    Operator::I32AtomicRmwXchg { ref memarg }
                    | Operator::I32AtomicRmw16UXchg { ref memarg }
                    | Operator::I32AtomicRmw8UXchg { ref memarg } => {
                        // TODO
                    }
                    Operator::I32AtomicRmwCmpxchg { ref memarg }
                    | Operator::I32AtomicRmw16UCmpxchg { ref memarg }
                    | Operator::I32AtomicRmw8UCmpxchg { ref memarg } => {
                        // TODO
                    }
                    Operator::I64AtomicRmwXchg { ref memarg }
                    | Operator::I64AtomicRmw32UXchg { ref memarg }
                    | Operator::I64AtomicRmw16UXchg { ref memarg }
                    | Operator::I64AtomicRmw8UXchg { ref memarg } => {
                         // TODO
                    }
                    Operator::I64AtomicRmwCmpxchg { ref memarg }
                    | Operator::I64AtomicRmw32UCmpxchg { ref memarg }
                    | Operator::I64AtomicRmw16UCmpxchg { ref memarg }
                    | Operator::I64AtomicRmw8UCmpxchg { ref memarg } => {
                         // TODO
                    }
                    Operator::Wake { ref memarg } => {
                         // TODO
                    }
                    Operator::I32Wait { ref memarg } => {
                         // TODO
                    }
                    Operator::I64Wait { ref memarg } => {
                         // TODO
                    }
                    Operator::RefNull => {
                         // TODO
                    }
                    Operator::RefIsNull => {
                         // TODO
                    }
                    Operator::V128Load { ref memarg } => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::V128Store { ref memarg } => {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                    }
                    Operator::V128Const { .. } => {
                        node.add_constant(Type::V128);
                    }
                    Operator::V8x16Shuffle { ref lines } => {
                         // TODO
                    }
                    Operator::I8x16Splat | Operator::I16x8Splat | Operator::I32x4Splat => {
                         // TODO
                    }
                    Operator::I64x2Splat => {
                         // TODO
                    }
                    Operator::F32x4Splat => {
                         // TODO
                    }
                    Operator::F64x2Splat => {
                         // TODO
                    }
                    Operator::I8x16ExtractLaneS { line } | Operator::I8x16ExtractLaneU { line } => { 
                        // TODO 
                    }
                    Operator::I16x8ExtractLaneS { line } | Operator::I16x8ExtractLaneU { line } => { 
                        // TODO 
                    }
                    Operator::I32x4ExtractLane { line } => { 
                        // TODO 
                    }
                    Operator::I8x16ReplaceLane { line } => { 
                        // TODO 
                    }
                    Operator::I16x8ReplaceLane { line } => { 
                        // TODO 
                    }
                    Operator::I32x4ReplaceLane { line } => { 
                        // TODO 
                    }
                    Operator::I64x2ExtractLane { line } => { 
                        // TODO 
                    }
                    Operator::I64x2ReplaceLane { line } => { 
                        // TODO 
                    }
                    Operator::F32x4ExtractLane { line } => { 
                        // TODO 
                    }
                    Operator::F32x4ReplaceLane { line } => { 
                        // TODO 
                    }
                    Operator::F64x2ExtractLane { line } => { 
                        // TODO 
                    }
                    Operator::F64x2ReplaceLane { line } => { 
                        // TODO 
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
                        // TODO 
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
                        // TODO 
                    }
                    Operator::V128Bitselect => { 
                        // TODO 
                    }
                    Operator::I8x16AnyTrue
                    | Operator::I8x16AllTrue
                    | Operator::I16x8AnyTrue
                    | Operator::I16x8AllTrue
                    | Operator::I32x4AnyTrue
                    | Operator::I32x4AllTrue
                    | Operator::I64x2AnyTrue
                    | Operator::I64x2AllTrue => { 
                        // TODO 
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
                        // TODO 
                    }

                    Operator::MemoryInit { segment } => { 
                        // TODO 
                    }
                    Operator::DataDrop { segment } => { 
                        // TODO 
                    }
                    Operator::MemoryCopy | Operator::MemoryFill => { 
                        // TODO 
                    }
                    Operator::TableInit { segment } => { 
                        // TODO 
                    }
                    Operator::ElemDrop { segment } => { 
                        // TODO 
                    }
                    Operator::TableCopy => { 
                        // TODO 
                    }
                    Operator::TableGet { table } => { 
                        // TODO 
                    }
                    Operator::TableSet { table } => { 
                        // TODO 
                    }
                    Operator::TableGrow { table } => { 
                        // TODO 
                    }
                    Operator::TableSize { table } => { 
                        // TODO 
                    }
                }
                // print out each encountered operator
                println!("{}. {:?}", i, op);
            } else {

                // red is for bad WASM
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)));
                println!("Bad wasm code {:?}", read.err());
            }
        }

        // set the node's instruction list
        let end = node.get_end();
        node.set_instrs(buf[start..end].to_vec());

        node
    }
}


// Initializes a Node mapper
pub fn new_mapper() -> Mapper {
    Mapper::default()
}

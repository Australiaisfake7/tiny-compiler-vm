use crate::{STACK_SIZE, compiler::{CompiledData, OpCode}};
use std::fmt;

enum Value {
    Int(i32),
    Float(f32),
    String(String),
    Bool(bool),
    Null,
}
#[derive(Debug)]
enum VmError {
    StackMissingValues { opcode: OpCode, stack_size: usize },
}
impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VmError::StackMissingValues { opcode, stack_size } => write!(f, "Stack of size {stack_size} missing values for opcode {opcode:?}"),
        }
    }
}
impl std::error::Error for VmError {}
struct Stack {
    data: [Value; STACK_SIZE],
    address: usize,
}

pub fn excecute(data: CompiledData) {
    let stack: Stack = Stack { data: [Null; STACK_SIZE], address: 0 };
    for opcode in data.opcodes {
        excecute_opcode(&opcode, &mut stack);
    }
}
fn excecute_opcode(opcode: &OpCode, stack: &mut Stack) -> Result<(), Box<dyn std::error::Error>> {
    match opcode {
        OpCode::Add => {
            if stack.address < 2 {
                return Err(VmError::StackMissingValues { opcode: OpCode::Add, stack_size: stack.address });
            } 
            *stack.data.get_mut(stack.address - 2).unwrap() = stack.data.get(stack.address - 2) + stack.data.get(stack.address - 1);
            stack.address -= 1;
        },

    }
}
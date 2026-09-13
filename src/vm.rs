use crate::{STACK_SIZE, compiler::{CompiledData, OpCode}};
use std::fmt;

#[derive(Default)]
enum Value {
    Int(i32),
    Float(f32),
    String(String),
    Bool(bool),
    #[default]
    Null,
}
#[derive(Debug)]
enum VmError {
    StackUnderflow, StackOverflow,
}
impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VmError::StackUnderflow => write!(f, "stack underflow"),
            VmError::StackOverflow => write!(f, "stack overflow"),
        }
    }
}
impl std::error::Error for VmError {}
struct Stack {
    data: [Value; STACK_SIZE],
    address: usize,
}

impl Stack {
    fn new() -> Self {
        Stack { data: std::array::from_fn(|_| Value::Null), address: 0 }
    }
    fn pop(&mut self) -> Result<Value, VmError> {
        if self.address == 0 {
            return Err(VmError::StackUnderflow);
        }

        self.address -= 1;
        Ok(std::mem::take(self.data.get_mut(self.address).unwrap()))
    }
    fn push(&mut self, value: Value) -> Result<(), VmError> {
        if self.address >= STACK_SIZE {
            return Err(VmError::StackOverflow);
        }

        *self.data.get_mut(self.address).unwrap() = value;
        Ok(())
    }
}

pub fn excecute(data: CompiledData) {
    let mut stack: Stack = Stack::new();
    for opcode in data.opcodes {
        excecute_opcode(&opcode, &mut stack);
    }
}
fn excecute_opcode(opcode: &OpCode, stack: &mut Stack) -> Result<(), VmError> {
    
}
use crate::{STACK_SIZE, compiler::{CompiledData, OpCode}};
use std::fmt;

#[derive(Debug, Default, PartialEq)]
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
    StackUnderflow, 
    StackOverflow,
    TypeMismatchUnary { opcode: String, operand: Value },
    TypeMismatchBinary { opcode: String, left: Value, right: Value },
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VmError::StackUnderflow => write!(f, "stack underflow"),
            VmError::StackOverflow => write!(f, "stack overflow"),
            VmError::TypeMismatchUnary { opcode, operand } => write!(f, "type mismatch of value {operand:?} with opcode {opcode}"),
            VmError::TypeMismatchBinary { opcode, left, right } => write!(f, "type mismatch of values {left:?} and {right:?} with opcode {opcode}"),
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
        let _ = excecute_opcode(&opcode, &mut stack);
    }
}

fn excecute_opcode(opcode: &OpCode, stack: &mut Stack) -> Result<(), VmError> {
    match opcode {
        OpCode::LNot | OpCode::Negate => {
            let a = stack.pop()?;
            let result = apply_unary(opcode, a)
                .map_err(|operand| VmError::TypeMismatchUnary { 
                    opcode: format!("{opcode:?}"), 
                    operand 
                })?;
            stack.push(result)?;
        }

        OpCode::Add | OpCode::Subtract | OpCode::Multiply | OpCode::Divide
        | OpCode::Equal | OpCode::NotEqual
        | OpCode::Less | OpCode::LessEqual | OpCode::Greater | OpCode::GreaterEqual => {
            let b = stack.pop()?;
            let a = stack.pop()?;
            let result = apply_binary(opcode, a, b)
                .map_err(|(left, right)| VmError::TypeMismatchBinary { 
                    opcode: format!("{opcode:?}"), 
                    left, 
                    right 
                })?;
            stack.push(result)?;
        }
    };

    Ok(())
}

fn apply_unary(opcode: &OpCode, a: Value) -> Result<Value, Value> {
    match (opcode, a) {
        (OpCode::LNot, Value::Bool(x)) => Ok(Value::Bool(!x)),
        (OpCode::Negate, Value::Int(x)) => Ok(Value::Int(-x)),
        (OpCode::Negate, Value::Float(x)) => Ok(Value::Float(-x)),
        (_, a) => Err(a),
    }
}

fn apply_binary(opcode: &OpCode, a: Value, b: Value) -> Result<Value, (Value, Value)> {
    match (opcode, a, b) {
        (OpCode::Add, Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
        (OpCode::Add, Value::Float(x), Value::Float(y)) => Ok(Value::Float(x + y)),
        (OpCode::Add, Value::String(x), Value::String(y)) => Ok(Value::String(x + &y)),

        (OpCode::Subtract, Value::Int(x), Value::Int(y)) => Ok(Value::Int(x - y)),
        (OpCode::Subtract, Value::Float(x), Value::Float(y)) => Ok(Value::Float(x - y)),

        (OpCode::Multiply, Value::Int(x), Value::Int(y)) => Ok(Value::Int(x * y)),
        (OpCode::Multiply, Value::Float(x), Value::Float(y)) => Ok(Value::Float(x * y)),

        (OpCode::Divide, Value::Int(x), Value::Int(y)) if y != 0 => Ok(Value::Int(x / y)),
        (OpCode::Divide, Value::Float(x), Value::Float(y)) => Ok(Value::Float(x / y)),

        (OpCode::Less, Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x < y)),
        (OpCode::Less, Value::Float(x), Value::Float(y)) => Ok(Value::Bool(x < y)),
        (OpCode::Less, Value::String(x), Value::String(y)) => Ok(Value::Bool(x < y)),

        (OpCode::LessEqual, Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x <= y)),
        (OpCode::LessEqual, Value::Float(x), Value::Float(y)) => Ok(Value::Bool(x <= y)),
        (OpCode::LessEqual, Value::String(x), Value::String(y)) => Ok(Value::Bool(x <= y)),

        (OpCode::Greater, Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x > y)),
        (OpCode::Greater, Value::Float(x), Value::Float(y)) => Ok(Value::Bool(x > y)),
        (OpCode::Greater, Value::String(x), Value::String(y)) => Ok(Value::Bool(x > y)),

        (OpCode::GreaterEqual, Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x >= y)),
        (OpCode::GreaterEqual, Value::Float(x), Value::Float(y)) => Ok(Value::Bool(x >= y)),
        (OpCode::GreaterEqual, Value::String(x), Value::String(y)) => Ok(Value::Bool(x >= y)),

        (OpCode::Equal, a, b) => Ok(Value::Bool(a == b)),
        (OpCode::NotEqual, a, b) => Ok(Value::Bool(a != b)),

        (_, a, b) => Err((a, b)),
    }
}
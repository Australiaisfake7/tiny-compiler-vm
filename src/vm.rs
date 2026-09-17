use crate::{STACK_SIZE, compiler::{CompiledData, OpCode, LiteralType}};
use std::{cell::RefCell, rc::Rc, fmt};

#[derive(Debug)]
struct Instance {
    id: usize,
    fields: Vec<Value>,
}

type InstanceRef = Rc<RefCell<Instance>>;

#[derive(Debug, Default, Clone)]
enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    #[default]
    Null,
    Instance(InstanceRef),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Instance(a), Value::Instance(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

#[derive(Debug)]
enum VmError {
    StackOverflow,
    NoValueOnStack,
    TypeMismatchUnary { opcode: String, operand: Value },
    TypeMismatchBinary { opcode: String, left: Value, right: Value },
    TypeMismatchJump(Value),
    JumpOutOfBounds(usize),
    StackIndexOutOfBounds(usize),
    GlobalIndexOutOfBounds(usize),
    RebaseOutsideStack(usize),
    NoAddressOnCallStack,
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VmError::StackOverflow => write!(f, "stack overflow"),
            VmError::NoValueOnStack => write!(f, "no value on stack"),
            VmError::TypeMismatchUnary { opcode, operand } => write!(f, "type mismatch of value {operand:?} with opcode {opcode}"),
            VmError::TypeMismatchBinary { opcode, left, right } => write!(f, "type mismatch of values {left:?} and {right:?} with opcode {opcode}"),
            VmError::TypeMismatchJump(operand) => write!(f, "type mismatch for conditional jump {operand:?}"),
            VmError::JumpOutOfBounds(index) => write!(f, "jump index {index} out of bounds"),
            VmError::StackIndexOutOfBounds(index) => write!(f, "stack index {index} out of bounds"),
            VmError::GlobalIndexOutOfBounds(index) => write!(f, "global index {index} out of bounds"),
            VmError::RebaseOutsideStack(index) => write!(f, "attempted to rebase stack to index {index} which is outside stack"),
            VmError::NoAddressOnCallStack => write!(f, "no return address on call stack"),
        }
    }
}

impl std::error::Error for VmError {}


impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{b}"),
            Value::Null => write!(f, "null"),
            Value::Int(i) => write!(f, "{i}"),
            Value::Float(fl) => write!(f, "{fl}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Instance(i) => write!(f, "instance of class {}", i.borrow().id),
        }
    }
}
struct Stack {
    data: [Value; STACK_SIZE],
    size: usize,
    base: usize,
}

impl Stack {
    fn new() -> Self {
        Stack { data: std::array::from_fn(|_| Value::Null), size: 0, base: 0 }
    }
    
    fn pop(&mut self) -> Result<Value, VmError> {
        if self.size <= self.base {
            return Err(VmError::NoValueOnStack);
        }

        self.size -= 1;
        Ok(std::mem::take(self.data.get_mut(self.size).unwrap()))
    }
    fn pop_n(&mut self, n: usize) -> Result<(), VmError> {
        if self.size - self.base < n {
            return Err(VmError::NoValueOnStack);
        }

        self.size -= n;
        Ok(())
    }
    fn push(&mut self, value: Value) -> Result<(), VmError> {
        *self.data.get_mut(self.size).ok_or(VmError::StackOverflow)? = value;
        self.size += 1;
        Ok(())
    }
    fn peek(&self) -> Result<&Value, VmError> {
        if self.size <= self.base {
            return Err(VmError::NoValueOnStack)
        }

        Ok(self.data.get(self.size - 1).unwrap())
    }
    fn read(&self, index: usize) -> Result<&Value, VmError> {
        if let Some(v) = self.data.get(index) {
            return Ok(v);
        }

        Err(VmError::StackIndexOutOfBounds(index))
    }
    fn write(&mut self, index: usize, value: Value) -> Result<(), VmError> {
        if let Some(v) = self.data.get_mut(index) {
            *v = value;
            return Ok(());
        }

        Err(VmError::StackIndexOutOfBounds(index))
    }
    fn rebase(&mut self, index: usize) -> Result<(), VmError> {
        if self.size < index {
            return Err(VmError::RebaseOutsideStack(index));
        }

        self.base = index;
        Ok(())
    }
    fn rebased_size(&self) -> usize {
        self.size - self.base
    }
}

pub fn excecute(data: CompiledData) -> Result<(), VmError> {
    let mut stack: Stack = Stack::new();
    let mut ptr: usize = 0;
    let mut global_vars: Vec<Value> = Vec::new();
    let mut call_stack: Vec<(usize, usize)> = Vec::new();
    loop {
        let opcode: &OpCode = data.opcodes.get(ptr).ok_or(VmError::JumpOutOfBounds(ptr))?;
        ptr += 1;
        excecute_opcode(opcode, &mut stack, &mut ptr, &mut global_vars, &mut call_stack)?;
    }

    Ok(())
}

fn excecute_opcode(opcode: &OpCode, stack: &mut Stack, ptr: &mut usize, global_vars: &mut Vec<Value>, call_stack: &mut Vec<(usize, usize)>) -> Result<(), VmError> {
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
        },
        OpCode::Jump(index) => { *ptr = *index; },
        OpCode::JumpIfFalse { index, pop } => jump_conditonal(stack, *index, ptr, false, *pop)?,
        OpCode::JumpIfTrue { index, pop } => jump_conditonal(stack, *index, ptr, true, *pop)?,
        OpCode::PushConst(lt) => {
            let value: Value = match lt {
                LiteralType::Null => Value::Null,
                LiteralType::Bool(b) => Value::Bool(*b),
                LiteralType::Int(i) => Value::Int(*i),
                LiteralType::Float(f) => Value::Float(*f),
                LiteralType::String(s) => Value::String(s.clone()),
            };
            stack.push(value)?;
        },
        OpCode::Pop(n) => stack.pop_n(*n)?,
        OpCode::Print => println!("{}", stack.pop()?),
        OpCode::GetVar(i) => {
            stack.push(stack.read(*i)?.clone())?;
        },
        OpCode::SetVar(i) => {
            stack.write(*i, stack.pop()?)?;
        },
        OpCode::DefineGlobal => global_vars.push(stack.pop()?),
        OpCode::GetGlobal(i) => stack.push(global_vars.get(*i).map(|v| v.clone()).ok_or(VmError::GlobalIndexOutOfBounds(*i))?)?,
        OpCode::SetGlobal(i) => *global_vars.get_mut(*i).ok_or(VmError::GlobalIndexOutOfBounds(*i))? = stack.pop()?,
        OpCode::Call { index, parameters } => {
            if stack.rebased_size() < *parameters {
                return Err(VmError::NoValueOnStack);
            }
            call_stack.push((*ptr, stack.base));
            *ptr = *index;
            stack.base = stack.size - *parameters;
        },
        OpCode::Return => {
            if let Some((index, base)) = call_stack.pop() {
                let value: Value = stack.pop()?;
                stack.pop_n(stack.rebased_size())?;

                stack.rebase(base)?;
                *ptr = index;

                stack.push(value)?;
            }
            else {
                return Err(VmError::NoAddressOnCallStack);
            }
        },
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

fn jump_conditonal(stack: &mut Stack, index: usize, ptr: &mut usize, jump_on: bool, pop: bool) -> Result<(), VmError> {
    let b: bool = match stack.peek()? {
        Value::Bool(b) => *b,
        v => return Err(VmError::TypeMismatchJump(v.clone())),
    };

    if pop { 
        stack.pop()?;
    }
    if b == jump_on {
        *ptr = index;
    }

    Ok(())
}
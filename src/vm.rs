use crate::{STACK_SIZE, compiler::{CompiledData, OpCode, LiteralType}};
use std::{cell::RefCell, fmt, io::{self, BufRead}, rc::Rc};

#[derive(PartialEq)]
enum ReturnCode {
    None, Exit,
}

#[derive(Debug)]
struct Instance {
    id: usize,
    fields: Vec<Value>,
}

type RcMut<T> = Rc<RefCell<T>>;
type ArrayData = RcMut<Box<[Value]>>;

#[derive(Debug, Default, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(Rc<str>),
    Bool(bool),
    #[default]
    Null,
    Instance(RcMut<Instance>),
    Array(ArrayData),
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
            (Value::Array(a), Value::Array(b)) => Rc::ptr_eq(a, b) || *a.borrow() == *b.borrow(),
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum VmError {
    StackOverflow,
    CallStackOverflow,
    DivisionByZero,
    NegationOverflow(i64),
    ArithmeticOverflow { left: i64, op: &'static str, right: i64 },
    ArrayIndexOutOfBounds { index: i64, len: usize },
    AssertOnNull,
    InvalidUnaryOperand { op: &'static str, operand: &'static str },
    InvalidBinaryOperands { op: &'static str, left: &'static str, right: &'static str },
    NonBoolCondition(&'static str),
    NotAnInstance(&'static str),
    NotAnArray(&'static str),
    InvalidIndexType(&'static str),
    InputFailed(io::Error),
    InvalidBytecode(InvalidBytecode),
}

#[derive(Debug)]
pub enum InvalidBytecode {
    JumpOutOfBounds(usize),
    StackUnderflow,
    StackIndexOutOfBounds(usize),
    GlobalIndexOutOfBounds(usize),
    RebaseOutsideStack(usize),
    EmptyCallStack,
    ClassOutOfBounds(usize),
    VtableSlotOutOfBounds { class: usize, slot: usize },
    MemberOutOfBounds { class: usize, member: usize },
    UnexpectedOperand { opcode: &'static str, expected: &'static str, found: &'static str },
}

impl fmt::Display for InvalidBytecode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidBytecode::JumpOutOfBounds(index) => write!(f, "jump target {index} is outside the program"),
            InvalidBytecode::StackUnderflow => write!(f, "a value was popped from an empty stack frame"),
            InvalidBytecode::StackIndexOutOfBounds(index) => write!(f, "stack index {index} is out of bounds"),
            InvalidBytecode::GlobalIndexOutOfBounds(index) => write!(f, "global index {index} is out of bounds"),
            InvalidBytecode::RebaseOutsideStack(index) => write!(f, "cannot rebase the stack to index {index}, which is outside the stack"),
            InvalidBytecode::EmptyCallStack => write!(f, "returned with no return address on the call stack"),
            InvalidBytecode::ClassOutOfBounds(class) => write!(f, "class {class} does not exist"),
            InvalidBytecode::VtableSlotOutOfBounds { class, slot } => write!(f, "class {class} has no vtable slot {slot}"),
            InvalidBytecode::MemberOutOfBounds { class, member } => write!(f, "an instance of class {class} has no member {member}"),
            InvalidBytecode::UnexpectedOperand { opcode, expected, found } => write!(f, "'{opcode}' expected an operand of type '{expected}', found '{found}'"),
        }
    }
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "runtime error: ")?;

        match self {
            VmError::StackOverflow => write!(f, "stack overflow"),
            VmError::CallStackOverflow => write!(f, "call stack overflow"),
            VmError::DivisionByZero => write!(f, "division by zero"),
            VmError::NegationOverflow(operand) => write!(f, "integer overflow in '-({operand})'"),
            VmError::ArithmeticOverflow { left, op, right } => write!(f, "integer overflow in '{left} {op} {right}'"),
            VmError::ArrayIndexOutOfBounds { index, len } => write!(f, "index {index} is out of bounds for an array of length {len}"),
            VmError::AssertOnNull => write!(f, "'!!' applied to a null value"),
            VmError::InvalidUnaryOperand { op, operand } => write!(f, "cannot apply '{op}' to an operand of type '{operand}'"),
            VmError::InvalidBinaryOperands { op, left, right } => write!(f, "cannot apply '{op}' to operands of type '{left}' and '{right}'"),
            VmError::NonBoolCondition(found) => write!(f, "a condition must be of type 'bool', found '{found}'"),
            VmError::NotAnInstance(found) => write!(f, "cannot access a member of a value of type '{found}'"),
            VmError::NotAnArray(found) => write!(f, "cannot use a value of type '{found}' as an array"),
            VmError::InvalidIndexType(found) => write!(f, "array index must be of type 'int', found '{found}'"),
            VmError::InputFailed(error) => write!(f, "cannot read from standard input: {error}"),
            VmError::InvalidBytecode(invalid) => write!(f, "invalid bytecode: {invalid}"),
        }
    }
}

impl std::error::Error for VmError {}

impl From<InvalidBytecode> for VmError {
    fn from(invalid: InvalidBytecode) -> Self {
        VmError::InvalidBytecode(invalid)
    }
}

impl Value {
    fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Bool(_) => "bool",
            Value::Null => "null",
            Value::Instance(_) => "instance",
            Value::Array(_) => "array",
        }
    }
}

fn operator_symbol(opcode: &OpCode) -> &'static str {
    match opcode {
        OpCode::LNot => "!",
        OpCode::Negate | OpCode::Subtract => "-",
        OpCode::Add => "+",
        OpCode::Multiply => "*",
        OpCode::Divide => "/",
        OpCode::Equal => "==",
        OpCode::NotEqual => "!=",
        OpCode::Less => "<",
        OpCode::LessEqual => "<=",
        OpCode::Greater => ">",
        OpCode::GreaterEqual => ">=",
        _ => "?",
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{b}"),
            Value::Null => write!(f, "null"),
            Value::Int(i) => write!(f, "{i}"),
            Value::Float(fl) => write!(f, "{fl:?}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Instance(i) => write!(f, "instance of class {}", i.borrow().id),
            Value::Array(a) => {
                let items: Vec<String> = a.borrow().iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", items.join(", "))
            },
        }
    }
}

struct Stack {
    data: Box<[Value]>,
    size: usize,
    base: usize,
}

impl Stack {
    fn new() -> Self {
        Stack { data: vec![Value::default(); STACK_SIZE].into_boxed_slice(), size: 0, base: 0 }
    }
    
    fn pop(&mut self) -> Result<Value, VmError> {
        if self.size <= self.base {
            return Err(InvalidBytecode::StackUnderflow.into());
        }

        self.size -= 1;
        Ok(std::mem::take(self.data.get_mut(self.size).unwrap()))
    }
    fn pop_n(&mut self, n: usize) -> Result<(), VmError> {
        if self.size - self.base < n {
            return Err(InvalidBytecode::StackUnderflow.into());
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
        if self.rebased_size() == 0 {
            return Err(InvalidBytecode::StackUnderflow.into());
        }

        self.get(self.size - 1)
    }
    fn get(&self, index: usize) -> Result<&Value, VmError> {
        if index >= self.size {
            return Err(InvalidBytecode::StackIndexOutOfBounds(index).into());
        }

        Ok(self.data.get(index).unwrap())
    }
    fn get_mut(&mut self, index: usize) -> Result<&mut Value, VmError> {
        if index >= self.size {
            return Err(InvalidBytecode::StackIndexOutOfBounds(index).into());
        }

        Ok(self.data.get_mut(index).unwrap())
    }
    fn rebase(&mut self, index: usize) -> Result<(), VmError> {
        if self.size < index {
            return Err(InvalidBytecode::RebaseOutsideStack(index).into());
        }

        self.base = index;
        Ok(())
    }
    fn rebased_size(&self) -> usize {
        self.size - self.base
    }
}

struct CallStack {
    data: Box<[(usize, usize)]>,
    size: usize,
}

impl CallStack {
    fn new() -> Self {
        CallStack { data: vec![(0, 0); 512].into_boxed_slice(), size: 0 }
    }
    fn push(&mut self, value: (usize, usize)) -> Result<(), VmError> {
        if self.size >= 512 {
            return Err(VmError::CallStackOverflow);
        }

        *self.data.get_mut(self.size).unwrap() = value;
        self.size += 1;

        Ok(())
    }
    fn pop(&mut self) -> Result<(usize, usize), VmError> {
        if self.size == 0 {
            return Err(InvalidBytecode::EmptyCallStack.into());
        }

        self.size -= 1;
        Ok(std::mem::take(self.data.get_mut(self.size).unwrap()))
    }
}

pub fn execute(mut data: CompiledData) -> Result<(), VmError> {
    let mut stack: Stack = Stack::new();
    let mut ptr: usize = 0;
    let mut global_vars: Vec<Value> = Vec::new();
    let mut call_stack: CallStack = CallStack::new();
    loop {
        let opcode: &OpCode = data.opcodes.get(ptr).ok_or(InvalidBytecode::JumpOutOfBounds(ptr))?;
        ptr += 1;
        if execute_opcode(opcode, &mut stack, &mut ptr, &mut global_vars, &mut call_stack, &mut data.vtables, &mut data.field_counts)? == ReturnCode::Exit {
            break;
        }
    }

    Ok(())
}

fn execute_opcode(opcode: &OpCode, stack: &mut Stack, ptr: &mut usize, global_vars: &mut Vec<Value>, call_stack: &mut CallStack, vtables: &mut Vec<Vec<usize>>, field_counts: &Vec<usize>) -> Result<ReturnCode, VmError> {
    match opcode {
        OpCode::LNot | OpCode::Negate => {
            let a: Value = stack.pop()?;
            let result: Value = apply_unary(opcode, a)?;
            stack.push(result)?;
        }
        OpCode::Add | OpCode::Subtract | OpCode::Multiply | OpCode::Divide
        | OpCode::Equal | OpCode::NotEqual
        | OpCode::Less | OpCode::LessEqual | OpCode::Greater | OpCode::GreaterEqual => {
            let b: Value = stack.pop()?;
            let a: Value = stack.pop()?;
            let result: Value = apply_binary(opcode, a, b)?;
            stack.push(result)?;
        },
        OpCode::Jump(index) => { *ptr = *index; },
        OpCode::JumpIfFalse { index, pop } => jump_conditional(stack, *index, ptr, false, *pop)?,
        OpCode::JumpIfTrue { index, pop } => jump_conditional(stack, *index, ptr, true, *pop)?,
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
        OpCode::GetVar(index) => {
            stack.push(stack.get(stack.base + *index)?.clone())?;
        },
        OpCode::SetVar(index) => {
            *stack.get_mut(stack.base + *index)? = stack.peek()?.clone();
        },
        OpCode::DefineGlobal => global_vars.push(stack.pop()?),
        OpCode::GetGlobal(i) => stack.push(global_vars.get(*i).map(|v| v.clone()).ok_or(InvalidBytecode::GlobalIndexOutOfBounds(*i))?)?,
        OpCode::SetGlobal(i) => *global_vars.get_mut(*i).ok_or(InvalidBytecode::GlobalIndexOutOfBounds(*i))? = stack.peek()?.clone(),
        OpCode::Call { index, parameters } => {
            if stack.rebased_size() < *parameters {
                return Err(InvalidBytecode::StackUnderflow.into());
            }
            call_stack.push((*ptr, stack.base))?;
            *ptr = *index;
            stack.rebase(stack.size - *parameters)?;
        },
        OpCode::Return => {
            let (index, base) = call_stack.pop()?;
            let value: Value = stack.pop()?;
            stack.pop_n(stack.rebased_size())?;

            stack.rebase(base)?;
            *ptr = index;

            stack.push(value)?;
        },
        OpCode::CallVirtual { slot, parameters } => {
            if stack.rebased_size() < *parameters {
                return Err(InvalidBytecode::StackUnderflow.into());
            }
            let instance: &Value = stack.get(stack.size - *parameters)?;
            if let Value::Instance(i) = instance {
                call_stack.push((*ptr, stack.base))?;
                let class_id: usize = i.borrow().id;
                *ptr = *vtables.get(class_id).ok_or(InvalidBytecode::ClassOutOfBounds(class_id))?.get(*slot).ok_or(InvalidBytecode::VtableSlotOutOfBounds { class: class_id, slot: *slot })?;
                stack.rebase(stack.size - *parameters)?;
            }
            else {
                return Err(VmError::NotAnInstance(instance.type_name()));
            }
        },
        OpCode::GetMember(index) => {
            let value: Value = stack.pop()?;
            if let Value::Instance(instance) = value {
                stack.push(instance.borrow().fields.get(*index).ok_or_else(|| InvalidBytecode::MemberOutOfBounds { class: instance.borrow().id, member: *index })?.clone())?;
            }
            else {
                return Err(VmError::NotAnInstance(value.type_name()));
            }
        },
        OpCode::SetMember(index) => {
            let value: Value = stack.pop()?;
            let i_value: Value = stack.pop()?;

            if let Value::Instance(instance) = i_value {
                let mut inst = instance.borrow_mut();
                match inst.fields.get_mut(*index) {
                    Some(field) => *field = value.clone(),
                    None => {
                        let instance_id = inst.id;
                        return Err(InvalidBytecode::MemberOutOfBounds { class: instance_id, member: *index }.into());
                    }
                }
                stack.push(value)?;
            } else {
                return Err(VmError::NotAnInstance(i_value.type_name()));
            }
        },
        OpCode::NewInstance(id) => {
            let count: usize = *field_counts.get(*id).ok_or(InvalidBytecode::ClassOutOfBounds(*id))?;

            stack.push(Value::Instance(Rc::new(RefCell::new(Instance { id: *id, fields: vec![Value::default(); count] }))))?;
        },
        OpCode::Exit => return Ok(ReturnCode::Exit),
        OpCode::NewArray(len) => {
            let mut values: Vec<Value> = Vec::with_capacity(*len);

            for _ in 0_usize..*len {
                values.push(stack.pop()?);
            }
            values.reverse();

            stack.push(Value::Array(Rc::new(RefCell::new(values.into_boxed_slice()))))?;
        },
        OpCode::GetArrayMember => {
            let index: i64 = match stack.pop()? {
                Value::Int(i) => i,
                v => return Err(VmError::InvalidIndexType(v.type_name())),
            };
            let array: ArrayData = match stack.pop()? {
                Value::Array(a) => a,
                v => return Err(VmError::NotAnArray(v.type_name())),
            };

            let len: usize = array.borrow().len();
            if index < 0 || index as usize >= len {
                return Err(VmError::ArrayIndexOutOfBounds { index, len });
            }

            stack.push(array.borrow().get(index as usize).unwrap().clone())?;
        },
        OpCode::SetArrayMember => {
            let value: Value = stack.pop()?;
            let index: i64 = match stack.pop()? {
                Value::Int(i) => i,
                v => return Err(VmError::InvalidIndexType(v.type_name())),
            };
            let array: ArrayData = match stack.pop()? {
                Value::Array(a) => a,
                v => return Err(VmError::NotAnArray(v.type_name())),
            };

            let len: usize = array.borrow().len();
            if index < 0 || index as usize >= len {
                return Err(VmError::ArrayIndexOutOfBounds { index, len });
            }

            *array.borrow_mut().get_mut(index as usize).unwrap() = value.clone();
            stack.push(value)?;
        },
        OpCode::CloneArray => {
            let array: ArrayData = match stack.pop()? {
                Value::Array(a) => a,
                v => return Err(VmError::NotAnArray(v.type_name())),
            };

            stack.push(Value::Array(clone_array(array)))?;
        },
        OpCode::Assert => {
            if stack.peek()? == &Value::Null {
                return Err(VmError::AssertOnNull);
            }
        },
        OpCode::JumpIfNull(index) => {
            if matches!(stack.peek()?, Value::Null) {
                *ptr = *index;
            }
        },
        OpCode::JumpIfNotNull(index) => {
            if !matches!(stack.peek()?, Value::Null) {
                *ptr = *index;
            }
        },
        OpCode::FillArray(len) => {
            let value: Value = stack.pop()?;

            let values: Vec<Value> = if let Value::Array(a) = value {
                std::iter::repeat_with(|| Value::Array(clone_array(a.clone()))).take(*len).collect()
            }
            else {
                vec![value; *len]
            };

            stack.push(Value::Array(Rc::new(RefCell::new(values.into_boxed_slice()))))?;
        },
        OpCode::Input => {
            let line: Value = read_input_line(&mut io::stdin().lock())?;
            stack.push(line)?;
        },
        OpCode::ParseInt => {
            let result: Value = match pop_parse_operand(stack, "ParseInt")? {
                Some(text) => parse_int(&text),
                None => Value::Null,
            };
            stack.push(result)?;
        },
        OpCode::ParseFloat => {
            let result: Value = match pop_parse_operand(stack, "ParseFloat")? {
                Some(text) => parse_float(&text),
                None => Value::Null,
            };
            stack.push(result)?;
        },
    };

    Ok(ReturnCode::None)
}

fn read_input_line(reader: &mut impl BufRead) -> Result<Value, VmError> {
    let mut bytes: Vec<u8> = Vec::new();
    let bytes_read: usize = reader.read_until(b'\n', &mut bytes).map_err(VmError::InputFailed)?;

    if bytes_read == 0 {
        return Ok(Value::Null);
    }
    if bytes.last() == Some(&b'\n') {
        bytes.pop();
        if bytes.last() == Some(&b'\r') {
            bytes.pop();
        }
    }

    Ok(Value::String(String::from_utf8_lossy(&bytes).into()))
}

fn pop_parse_operand(stack: &mut Stack, opcode: &'static str) -> Result<Option<Rc<str>>, VmError> {
    match stack.pop()? {
        Value::String(text) => Ok(Some(text)),
        Value::Null => Ok(None),
        other => Err(InvalidBytecode::UnexpectedOperand { opcode, expected: "string?", found: other.type_name() }.into()),
    }
}

fn parse_int(text: &str) -> Value {
    text.parse::<i64>().map_or(Value::Null, Value::Int)
}

fn parse_float(text: &str) -> Value {
    match text.parse::<f64>() {
        Ok(float) if float.is_finite() => Value::Float(float),
        _ => Value::Null,
    }
}

fn apply_unary(opcode: &OpCode, a: Value) -> Result<Value, VmError> {
    match (opcode, a) {
        (OpCode::LNot, Value::Bool(x)) => Ok(Value::Bool(!x)),
        (OpCode::Negate, Value::Int(x)) => x.checked_neg()
        .map(|v| Value::Int(v))
        .ok_or_else(|| VmError::NegationOverflow(x)),
        (OpCode::Negate, Value::Float(x)) => Ok(Value::Float(-x)),
        (_, operand) => Err(VmError::InvalidUnaryOperand { op: operator_symbol(opcode), operand: operand.type_name() }),
    }
}

fn apply_binary(opcode: &OpCode, a: Value, b: Value) -> Result<Value, VmError> {
    match (opcode, a, b) {
        (OpCode::Add, Value::Int(x), Value::Int(y)) => x.checked_add(y)
        .map(|v| Value::Int(v))
        .ok_or_else(|| VmError::ArithmeticOverflow { left: x, op: operator_symbol(opcode), right: y }),
        (OpCode::Add, Value::Float(x), Value::Float(y)) => Ok(Value::Float(x + y)),
        (OpCode::Add, Value::String(x), Value::String(y)) => Ok(Value::String(Rc::from([&*x, &*y].concat().as_str()))),

        (OpCode::Subtract, Value::Int(x), Value::Int(y)) => x.checked_sub(y)
        .map(|v| Value::Int(v))
        .ok_or_else(|| VmError::ArithmeticOverflow { left: x, op: operator_symbol(opcode), right: y }),
        (OpCode::Subtract, Value::Float(x), Value::Float(y)) => Ok(Value::Float(x - y)),

        (OpCode::Multiply, Value::Int(x), Value::Int(y)) => x.checked_mul(y)
        .map(|v| Value::Int(v))
        .ok_or_else(|| VmError::ArithmeticOverflow { left: x, op: operator_symbol(opcode), right: y }),
        (OpCode::Multiply, Value::Float(x), Value::Float(y)) => Ok(Value::Float(x * y)),

        (OpCode::Divide, Value::Int(_), Value::Int(0)) => Err(VmError::DivisionByZero),
        (OpCode::Divide, Value::Int(x), Value::Int(y)) if y != 0 => x.checked_div(y)
        .map(|v| Value::Int(v))
        .ok_or_else(|| VmError::ArithmeticOverflow { left: x, op: operator_symbol(opcode), right: y }),
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

        (_, left, right) => Err(VmError::InvalidBinaryOperands { op: operator_symbol(opcode), left: left.type_name(), right: right.type_name() })
    }
}

fn jump_conditional(stack: &mut Stack, index: usize, ptr: &mut usize, jump_on: bool, pop: bool) -> Result<(), VmError> {
    let b: bool = match stack.peek()? {
        Value::Bool(b) => *b,
        v => return Err(VmError::NonBoolCondition(v.type_name())),
    };

    if pop { 
        stack.pop()?;
    }
    if b == jump_on {
        *ptr = index;
    }

    Ok(())
}

fn clone_array(array: ArrayData) -> ArrayData {
    let new_array: ArrayData = Rc::new(RefCell::new(array.borrow().clone()));

    for v in new_array.borrow_mut().iter_mut() {
        match v {
            Value::Array(a) => *v = Value::Array(clone_array(a.clone())),
            _ => (),
        }
    }

    new_array
}
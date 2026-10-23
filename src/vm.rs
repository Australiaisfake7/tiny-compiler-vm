use crate::{STACK_SIZE, compiler::{CompiledData, OpCode, LiteralType}};
use std::{cell::RefCell, rc::Rc, fmt};

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
    String(String),
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
    NoValueOnStack,
    TypeMismatchUnary { opcode: OpCode, operand: Value },
    TypeMismatchBinary { opcode: OpCode, left: Value, right: Value },
    TypeMismatchJump(Value),
    TypeMismatchMemberAccess(Value),
    TypeMismatchIndex(Value),
    TypeMismatchArrayAccess(Value),
    TypeMismatchCloneArray(Value),
    JumpOutOfBounds(usize),
    StackIndexOutOfBounds(usize),
    GlobalIndexOutOfBounds(usize),
    RebaseOutsideStack(usize),
    NoAddressOnCallStack,
    InstanceMissingMember { instance_id: usize, member: usize },
    ClassIndexOutOfBounds(usize),
    VtableIndexOutOfBounds { slot: usize, class: usize },
    CallStackOverflow,
    DivisionByZero,
    IntegerOutOfBoundsUnary { opcode: OpCode, operand: i64 },
    IntegerOutOfBoundsBinary { opcode: OpCode, left: i64, right: i64 },
    ArrayIndexOutOfBounds { array: ArrayData, index: i64 },
    AssertOnNull,
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VmError::StackOverflow => write!(f, "stack overflow"),
            VmError::NoValueOnStack => write!(f, "no value on stack"),
            VmError::TypeMismatchUnary { opcode, operand } => write!(f, "type mismatch of value {operand:?} with opcode {opcode:?}"),
            VmError::TypeMismatchBinary { opcode, left, right } => write!(f, "type mismatch of values {left:?} and {right:?} with opcode {opcode:?}"),
            VmError::TypeMismatchJump(operand) => write!(f, "type mismatch for conditional jump {operand:?}"),
            VmError::TypeMismatchMemberAccess(value) => write!(f, "type mismatch for member access instance {value}"),
            VmError::JumpOutOfBounds(index) => write!(f, "jump index {index} out of bounds"),
            VmError::StackIndexOutOfBounds(index) => write!(f, "stack index {index} out of bounds"),
            VmError::GlobalIndexOutOfBounds(index) => write!(f, "global index {index} out of bounds"),
            VmError::RebaseOutsideStack(index) => write!(f, "attempted to rebase stack to index {index} which is outside stack"),
            VmError::NoAddressOnCallStack => write!(f, "no return address on call stack"),
            VmError::InstanceMissingMember { instance_id, member } => write!(f, "attemted to access member {member} on instance {instance_id} which does not exist"),
            VmError::ClassIndexOutOfBounds(index) => write!(f, "attempted to access class {index} which does not exist"),
            VmError::VtableIndexOutOfBounds { slot, class } => write!(f, "attempted to access vtable slot {slot} on class {class} which does not exist"),
            VmError::CallStackOverflow => write!(f, "call stack overflow"),
            VmError::DivisionByZero => write!(f, "division by zero"),
            VmError::IntegerOutOfBoundsUnary { opcode, operand } => write!(f, "integer overflow on operation {opcode:?} with operand {operand}"),
            VmError::IntegerOutOfBoundsBinary { opcode, left, right } => write!(f, "integer overflow on operation {opcode:?} with operands {left} and {right}"),
            VmError::ArrayIndexOutOfBounds { array, index } => write!(f, "index {index} out of bounds for array {}", Value::Array(array.clone())),
            VmError::TypeMismatchArrayAccess(value) => write!(f, "type mismatch attepmting to index value {value} as array"),
            VmError::TypeMismatchCloneArray(value) => write!(f, "type mismatch attempting to clone value {value} as array"),
            VmError::TypeMismatchIndex(value) => write!(f, "type mismatch using {value} as index"),
            VmError::AssertOnNull => write!(f, "assert called on null value"),
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
            Value::Float(fl) => write!(f, "{fl:?}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Instance(i) => write!(f, "instance of class {}", i.borrow().id),
            Value::Array(a) => {
                let values: &[Value] = &a.borrow();
                let mut strs: Vec<String> = Vec::with_capacity(values.len());
                let mut char_len: usize = 0;

                for v in values {
                    let s: String = v.to_string();
                    char_len += s.len() + 2;
                    strs.push(s);
                }

                let mut str: String = String::with_capacity(char_len);

                for s in strs {
                    str.push_str(&s);
                    str.push_str(", ");
                }
                str.truncate(str.len() - 2);

                write!(f, "[{str}]")
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
        if self.rebased_size() == 0 {
            return Err(VmError::NoValueOnStack);
        }

        self.get(self.size - 1)
    }
    fn get(&self, index: usize) -> Result<&Value, VmError> {
        if index >= self.size {
            return Err(VmError::StackIndexOutOfBounds(index));
        }

        Ok(self.data.get(index).unwrap())
    }
    fn get_mut(&mut self, index: usize) -> Result<&mut Value, VmError> {
        if index >= self.size {
            return Err(VmError::StackIndexOutOfBounds(index));
        }

        Ok(self.data.get_mut(index).unwrap())
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
            return Err(VmError::NoAddressOnCallStack);
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
        let opcode: &OpCode = data.opcodes.get(ptr).ok_or(VmError::JumpOutOfBounds(ptr))?;
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
        OpCode::GetGlobal(i) => stack.push(global_vars.get(*i).map(|v| v.clone()).ok_or(VmError::GlobalIndexOutOfBounds(*i))?)?,
        OpCode::SetGlobal(i) => *global_vars.get_mut(*i).ok_or(VmError::GlobalIndexOutOfBounds(*i))? = stack.peek()?.clone(),
        OpCode::Call { index, parameters } => {
            if stack.rebased_size() < *parameters {
                return Err(VmError::NoValueOnStack);
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
                return Err(VmError::NoValueOnStack);
            }
            let instance: &Value = stack.get(stack.size - *parameters)?;
            if let Value::Instance(i) = instance {
                call_stack.push((*ptr, stack.base))?;
                let class_id: usize = i.borrow().id;
                *ptr = *vtables.get(class_id).ok_or(VmError::ClassIndexOutOfBounds(class_id))?.get(*slot).ok_or(VmError::VtableIndexOutOfBounds { slot: *slot, class: class_id })?;
                stack.rebase(stack.size - *parameters)?;
            }
            else {
                return Err(VmError::TypeMismatchMemberAccess(instance.clone()));
            }
        },
        OpCode::GetMember(index) => {
            let value: Value = stack.pop()?;
            if let Value::Instance(instance) = value {
                stack.push(instance.borrow().fields.get(*index).ok_or_else(|| VmError::InstanceMissingMember { instance_id: instance.borrow().id, member: *index })?.clone())?;
            }
            else {
                return Err(VmError::TypeMismatchMemberAccess(value));
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
                        return Err(VmError::InstanceMissingMember { instance_id, member: *index });
                    }
                }
                stack.push(value)?;
            } else {
                return Err(VmError::TypeMismatchMemberAccess(i_value));
            }
        },
        OpCode::NewInstance(id) => {
            let count: usize = *field_counts.get(*id).ok_or(VmError::ClassIndexOutOfBounds(*id))?;

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
                v => return Err(VmError::TypeMismatchIndex(v)),
            };
            let array: ArrayData = match stack.pop()? {
                Value::Array(a) => a,
                v => return Err(VmError::TypeMismatchArrayAccess(v)),
            };

            if index < 0 || index as usize >= array.borrow().len() {
                return Err(VmError::ArrayIndexOutOfBounds { array, index });
            }

            stack.push(array.borrow().get(index as usize).unwrap().clone())?;
        },
        OpCode::SetArrayMember => {
            let value: Value = stack.pop()?;
            let index: i64 = match stack.pop()? {
                Value::Int(i) => i,
                v => return Err(VmError::TypeMismatchIndex(v)),
            };
            let array: ArrayData = match stack.pop()? {
                Value::Array(a) => a,
                v => return Err(VmError::TypeMismatchArrayAccess(v)),
            };

            if index < 0 || index as usize >= array.borrow().len() {
                return Err(VmError::ArrayIndexOutOfBounds { array, index });
            }

            *array.borrow_mut().get_mut(index as usize).unwrap() = value.clone();
            stack.push(value)?;
        },
        OpCode::CloneArray => {
            let array: ArrayData = match stack.pop()? {
                Value::Array(a) => a,
                v => return Err(VmError::TypeMismatchCloneArray(v)),
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
    };

    Ok(ReturnCode::None)
}

fn apply_unary(opcode: &OpCode, a: Value) -> Result<Value, VmError> {
    match (opcode, a) {
        (OpCode::LNot, Value::Bool(x)) => Ok(Value::Bool(!x)),
        (OpCode::Negate, Value::Int(x)) => x.checked_neg()
        .map(|v| Value::Int(v))
        .ok_or_else(|| VmError::IntegerOutOfBoundsUnary { opcode: opcode.clone(), operand: x }),
        (OpCode::Negate, Value::Float(x)) => Ok(Value::Float(-x)),
        (_, operand) => Err(VmError::TypeMismatchUnary { opcode: opcode.clone(), operand }),
    }
}

fn apply_binary(opcode: &OpCode, a: Value, b: Value) -> Result<Value, VmError> {
    match (opcode, a, b) {
        (OpCode::Add, Value::Int(x), Value::Int(y)) => x.checked_add(y)
        .map(|v| Value::Int(v))
        .ok_or_else(|| VmError::IntegerOutOfBoundsBinary { opcode: opcode.clone(), left: x, right: y }),
        (OpCode::Add, Value::Float(x), Value::Float(y)) => Ok(Value::Float(x + y)),
        (OpCode::Add, Value::String(x), Value::String(y)) => Ok(Value::String(x + &y)),

        (OpCode::Subtract, Value::Int(x), Value::Int(y)) => x.checked_sub(y)
        .map(|v| Value::Int(v))
        .ok_or_else(|| VmError::IntegerOutOfBoundsBinary { opcode: opcode.clone(), left: x, right: y }),
        (OpCode::Subtract, Value::Float(x), Value::Float(y)) => Ok(Value::Float(x - y)),

        (OpCode::Multiply, Value::Int(x), Value::Int(y)) => x.checked_mul(y)
        .map(|v| Value::Int(v))
        .ok_or_else(|| VmError::IntegerOutOfBoundsBinary { opcode: opcode.clone(), left: x, right: y }),
        (OpCode::Multiply, Value::Float(x), Value::Float(y)) => Ok(Value::Float(x * y)),

        (OpCode::Divide, Value::Int(_), Value::Int(0)) => Err(VmError::DivisionByZero),
        (OpCode::Divide, Value::Int(x), Value::Int(y)) if y != 0 => x.checked_div(y)
        .map(|v| Value::Int(v))
        .ok_or_else(|| VmError::IntegerOutOfBoundsBinary { opcode: opcode.clone(), left: x, right: y }),
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

        (_, left, right) => Err(VmError::TypeMismatchBinary { opcode: opcode.clone(), left, right })
    }
}

fn jump_conditional(stack: &mut Stack, index: usize, ptr: &mut usize, jump_on: bool, pop: bool) -> Result<(), VmError> {
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
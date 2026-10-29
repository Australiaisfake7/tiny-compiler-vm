use std::collections::HashMap;
use std::fmt;
use serde::{Serialize, Deserialize};

use crate::compiler::{lexer::DataType, parser::{BinaryOp, Expression, FunctionData, LiteralType, Statement, UnaryOp}};
use crate::builtin::Builtin;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpCode {
    PushConst(LiteralType), Pop(usize),
    LNot, Negate, Add, Subtract, Multiply, Divide,
    Equal, Greater, GreaterEqual, Less, LessEqual, NotEqual,
    JumpIfTrue { index: usize, pop: bool }, JumpIfFalse { index: usize, pop: bool }, Jump(usize),
    JumpIfNull(usize), JumpIfNotNull(usize),
    GetVar(usize), SetVar(usize),
    GetGlobal(usize), SetGlobal(usize), DefineGlobal,
    Call { index: usize, parameters: usize }, CallVirtual { slot: usize, parameters: usize },
    Return, Print,
    GetMember(usize), SetMember(usize),
    NewInstance(usize),
    Exit,
    GetArrayMember, SetArrayMember, NewArray(usize), CloneArray,
    Assert,
    FillArray(usize),
    Input, ParseInt, ParseFloat,
}

#[derive(Debug)]
pub enum FlattenError {
    // Name resolution
    UndeclaredVariable(String),
    UndeclaredFunction(String),
    UndeclaredClass(String),
    UndeclaredType(DataType),
    UndeclaredClassVariable { class: String, member: String },
    UndeclaredClassFunction { class: String, member: String },
    Shadowing(String),
    ReservedKeyword(String),

    // Declarations and class structure
    FunctionDeclarationInsideScope(String),
    ClassDeclarationInsideScope(String),
    StaticOutsideClass(String),
    InvalidClassMember(Statement),
    InvalidOverride(String),
    OverrideSignatureMismatch { name: String, expected: (DataType, Vec<DataType>), found: (DataType, Vec<DataType>) },
    ParentIsSelf(String),
    ConstructorUnavailable(String),

    // Control flow
    BreakOutsideLoop,
    ContinueOutsideLoop,
    ReturnOutsideFunction,
    MissingReturn(String),
    UnreachableCode,

    // Types
    DeclarationTypeMismatch { variable: String, expected: DataType, found: DataType },
    AssignmentTypeMismatch { target: Box<Expression>, expected: DataType, found: DataType },
    ParameterTypeMismatch { callee: Box<Expression>, index: usize, expected: DataType, found: DataType },
    ReturnTypeMismatch { function: String, expected: DataType, found: DataType },
    ArrayElementTypeMismatch { array: Expression, index: usize },
    ArrayFillTypeMismatch { array: Expression, expected: DataType },
    InvalidBinaryOperands { left: DataType, operator: BinaryOp, right: DataType },
    InvalidUnaryOperand { operator: UnaryOp, operand: DataType },
    NonBoolCondition,
    VoidValueUsed,
    UnknownArrayLiteralType(Expression),

    // Expressions
    ParameterCountMismatch { callee: Box<Expression>, expected: usize, found: usize },
    InvalidAssignmentTarget(Box<Expression>),
    NotCallable(Box<Expression>),
    NotAnInstance(Box<Expression>),
    NotANullableInstance(Box<Expression>),
    NotAnArray(Box<Expression>),
    InvalidIndexType(Box<Expression>),
    ArrayIndexOutOfBounds { array: Box<Expression>, index: i64, len: usize },
    AssertOnNonNullable { expr: Box<Expression>, found: DataType },

    /// A fault in the compiler itself rather than in the program being compiled.
    Internal(String),
}

impl FlattenError {
    fn message(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn signature((return_type, parameters): &(DataType, Vec<DataType>)) -> String {
            let parameters: Vec<String> = parameters.iter().map(|p| p.to_string()).collect();
            format!("{return_type}({})", parameters.join("; "))
        }

        match self {
            FlattenError::UndeclaredVariable(name) => write!(f, "undeclared variable '{name}'"),
            FlattenError::UndeclaredFunction(name) => write!(f, "undeclared function '{name}'"),
            FlattenError::UndeclaredClass(name) => write!(f, "undeclared class '{name}'"),
            FlattenError::UndeclaredType(data_type) => write!(f, "undeclared type '{data_type}'"),
            FlattenError::UndeclaredClassVariable { class, member } =>
                write!(f, "undeclared class variable '{class}.{member}'"),
            FlattenError::UndeclaredClassFunction { class, member } =>
                write!(f, "undeclared class function '{class}.{member}'"),
            FlattenError::Shadowing(name) => write!(f, "'{name}' shadows an existing declaration"),
            FlattenError::ReservedKeyword(name) =>
                write!(f, "'{name}' is a reserved keyword and cannot be used as an identifier"),

            FlattenError::FunctionDeclarationInsideScope(name) =>
                write!(f, "function '{name}' cannot be declared inside a nested scope"),
            FlattenError::ClassDeclarationInsideScope(name) =>
                write!(f, "class '{name}' cannot be declared inside a nested scope"),
            FlattenError::StaticOutsideClass(name) =>
                write!(f, "'{name}' is marked static but declared outside a class"),
            FlattenError::InvalidClassMember(statement) =>
                write!(f, "{} is not allowed in a class body", statement.description()),
            FlattenError::InvalidOverride(name) =>
                write!(f, "'{name}' is marked override but does not override anything"),
            FlattenError::OverrideSignatureMismatch { name, expected, found } =>
                write!(f, "'{name}' overrides with signature '{}', but the parent declares '{}'", signature(found), signature(expected)),
            FlattenError::ParentIsSelf(name) => write!(f, "class '{name}' cannot inherit from itself"),
            FlattenError::ConstructorUnavailable(name) =>
                write!(f, "class '{name}' cannot be instantiated here because its constructor is not defined yet"),

            FlattenError::BreakOutsideLoop => write!(f, "'break' used outside of a loop"),
            FlattenError::ContinueOutsideLoop => write!(f, "'continue' used outside of a loop"),
            FlattenError::ReturnOutsideFunction => write!(f, "'return' used outside of a function"),
            FlattenError::MissingReturn(name) =>
                write!(f, "function '{name}' is missing a return statement on some code path"),
            FlattenError::UnreachableCode => write!(f, "some code path contains unreachable code"),

            FlattenError::DeclarationTypeMismatch { variable, expected, found } =>
                write!(f, "'{variable}' is declared as '{expected}', but assigned a value of type '{found}'"),
            FlattenError::AssignmentTypeMismatch { target, expected, found } =>
                write!(f, "cannot assign a value of type '{found}' to '{target}', which has type '{expected}'"),
            FlattenError::ParameterTypeMismatch { callee, index, expected, found } =>
                write!(f, "parameter {} of '{callee}' expects type '{expected}', found '{found}'", index + 1),
            FlattenError::ReturnTypeMismatch { function, expected, found } =>
                write!(f, "function '{function}' returns '{expected}', but a value of type '{found}' was returned"),
            FlattenError::ArrayElementTypeMismatch { array, index } =>
                write!(f, "the element at index {index} of array '{array}' has an unexpected type"),
            FlattenError::ArrayFillTypeMismatch { array, expected } =>
                write!(f, "array '{array}' expected a value of type {expected}"),
            FlattenError::InvalidBinaryOperands { left, operator, right } =>
                write!(f, "cannot apply '{operator}' to operands of type '{left}' and '{right}'"),
            FlattenError::InvalidUnaryOperand { operator, operand } =>
                write!(f, "cannot apply '{operator}' to an operand of type '{operand}'"),
            FlattenError::NonBoolCondition => write!(f, "a condition must be of type 'bool'"),
            FlattenError::VoidValueUsed => write!(f, "a 'void' value cannot be used"),
            FlattenError::UnknownArrayLiteralType(array) =>
                write!(f, "cannot infer the type of array '{array}'"),

            FlattenError::ParameterCountMismatch { callee, expected, found } =>
                write!(f, "'{callee}' expects {expected} parameter{}, but {found} {} given",
                    if *expected == 1 { "" } else { "s" }, if *found == 1 { "was" } else { "were" }),
            FlattenError::InvalidAssignmentTarget(target) =>
                write!(f, "'{target}' is not a valid assignment target"),
            FlattenError::NotCallable(callee) => write!(f, "'{callee}' cannot be called as a function"),
            FlattenError::NotAnInstance(expr) => write!(f, "'{expr}' is not a class instance"),
            FlattenError::NotANullableInstance(expr) =>
                write!(f, "'{expr}' is not a nullable class instance, so '?.' cannot be used on it"),
            FlattenError::NotAnArray(expr) => write!(f, "'{expr}' is not an array"),
            FlattenError::InvalidIndexType(index) => write!(f, "array index '{index}' must be of type 'int'"),
            FlattenError::ArrayIndexOutOfBounds { array, index, len } =>
                write!(f, "index {index} is out of bounds for array '{array}' of length {len}"),
            FlattenError::AssertOnNonNullable { expr, found } =>
                write!(f, "'!!' can only be applied to a nullable value, but '{expr}' has type '{found}'"),

            FlattenError::Internal(message) => write!(f, "{message}"),
        }
    }
}

impl fmt::Display for FlattenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlattenError::Internal(_) => write!(f, "internal compiler error: ")?,
            _ => write!(f, "semantic error: ")?,
        }
        self.message(f)
    }
}

impl std::error::Error for FlattenError {}

struct ClassData {
    vars: Vec<(String, DataType)>,
    funcs: HashMap<String, (usize, DataType, Vec<DataType>, bool)>,
    vtable: Vec<usize>,
    parent: Option<String>,
    constructor: usize,
    id: usize,
}
#[derive(Serialize, Deserialize)]
pub struct CompiledData {
    pub opcodes: Vec<OpCode>,
    pub vtables: Vec<Vec<usize>>,
    pub field_counts: Vec<usize>,
}
pub fn flatten_ast(statements: &[Statement]) -> Result<CompiledData, FlattenError> {
    let mut opcodes: Vec<OpCode> = Vec::new();
    let mut global_vars: Vec<(String, DataType)> = Vec::new();
    let mut vars: Vec<(String, DataType)> = Vec::new();
    let mut funcs: HashMap<String, (usize, DataType, Vec<DataType>)> = HashMap::new();
    let mut classes: HashMap<String, ClassData> = HashMap::new();
    let mut loop_starts: Vec<(usize, usize, Vec<usize>)> = Vec::new();

    flatten_statements(statements, &mut opcodes, &mut global_vars, &mut vars, &mut funcs, &mut classes, &mut loop_starts, 0, None)?;

    opcodes.push(OpCode::Exit);

    let mut vtables: Vec<Vec<usize>> = vec![Vec::new(); classes.len()];
    let mut field_counts: Vec<usize> = vec![0; classes.len()];

    for (_, data) in classes {
        *vtables.get_mut(data.id).unwrap() = data.vtable;
        *field_counts.get_mut(data.id).unwrap() = data.vars.len();
    }

    Ok(CompiledData { opcodes, vtables, field_counts })
}
fn flatten_statements(statements: &[Statement], opcodes: &mut Vec<OpCode>, global_vars: &mut Vec<(String, DataType)>, vars: &mut Vec<(String, DataType)>, funcs: &mut HashMap<String, (usize, DataType, Vec<DataType>)>, classes: &mut HashMap<String, ClassData>, loop_starts: &mut Vec<(usize, usize, Vec<usize>)>, depth: usize, func_data: Option<(&str, &DataType)>) -> Result<(bool, bool), FlattenError> {
    let mut r: bool = false;
    let mut t: bool = false;

    for (i, statement) in statements.iter().enumerate() {
        r = flatten_statement(statement, opcodes, global_vars, vars, funcs, classes, loop_starts, depth, func_data)?;

        if r || matches!(statement, Statement::Break | Statement::Continue) {
            if i + 1 < statements.len() {
                return Err(FlattenError::UnreachableCode);
            }
            t = true;
            break;
        }
    }

    Ok((r, t))
}
fn flatten_statement(statement: &Statement, opcodes: &mut Vec<OpCode>, global_vars: &mut Vec<(String, DataType)>, vars: &mut Vec<(String, DataType)>, funcs: &mut HashMap<String, (usize, DataType, Vec<DataType>)>, classes: &mut HashMap<String, ClassData>, loop_starts: &mut Vec<(usize, usize, Vec<usize>)>, depth: usize, func_data: Option<(&str, &DataType)>) -> Result<bool, FlattenError> {
    match statement {
        Statement::Block(statements) => {
            return flatten_block(statements, opcodes, global_vars, vars, funcs, classes, loop_starts, depth, func_data);
        }
        Statement::Expression(expression) => { flatten_expression(expression, opcodes, global_vars, vars, funcs, classes)?; emit_pop(opcodes, 1); },
        Statement::If { condition, block } => {
            if flatten_expression(condition, opcodes, global_vars, vars, funcs, classes)? != DataType::Bool {
                return Err(FlattenError::NonBoolCondition);
            }

            let start_index: usize = opcodes.len();
            opcodes.push(OpCode::JumpIfFalse { index: 0, pop: true });

            flatten_block(block, opcodes, global_vars, vars, funcs, classes, loop_starts, depth, func_data)?;
            *opcodes.get_mut(start_index).unwrap() = OpCode::JumpIfFalse { index: opcodes.len(), pop: true };
        },
        Statement::IfElse { condition, block1, block2 } => {
            if flatten_expression(condition, opcodes, global_vars, vars, funcs, classes)? != DataType::Bool {
                return Err(FlattenError::NonBoolCondition);
            }

            let start_index_1: usize = opcodes.len();
            opcodes.push(OpCode::JumpIfFalse { index: 0, pop: true });
            let r_1: bool = flatten_block(block1, opcodes, global_vars, vars, funcs, classes, loop_starts, depth, func_data)?;

            let start_index_2: usize = opcodes.len();
            opcodes.push(OpCode::Jump(0));
            *opcodes.get_mut(start_index_1).unwrap() = OpCode::JumpIfFalse { index: opcodes.len(), pop: true };

            let r_2: bool = flatten_block(block2, opcodes, global_vars, vars, funcs, classes, loop_starts, depth, func_data)?;
            *opcodes.get_mut(start_index_2).unwrap() = OpCode::Jump(opcodes.len());

            return Ok(r_1 && r_2);
        },
        Statement::Return(expression) => {
            if let Some(data) = func_data {
                match expression {
                    Some(e) => {
                        let d: DataType = flatten_expression_expected(e, opcodes, global_vars, vars, funcs, classes, func_data.map(|(_, d)| d))?;
                        if !is_compatible(&data.1.clone(), &d, classes) {
                            return Err(FlattenError::ReturnTypeMismatch { function: data.0.to_owned(), expected: data.1.clone(), found: d });
                        }
                    },
                    None => {
                        if !matches!(data.1, DataType::Void | DataType::Nullable(_)) {
                            return Err(FlattenError::ReturnTypeMismatch { function: data.0.to_owned(), expected: data.1.clone(), found: DataType::Null });
                        }
                        
                        flatten_expression(&Expression::Literal(LiteralType::Null), opcodes, global_vars, vars, funcs, classes)?;
                    },
                };

                opcodes.push(OpCode::Return);
            }
            else {
                return Err(FlattenError::ReturnOutsideFunction);
            }

            return Ok(true)
        },
        Statement::Continue => {
            let (start_index, vars_len) = match loop_starts.last() {
                Some(loop_start) => (loop_start.0, loop_start.1),
                None => return Err(FlattenError::ContinueOutsideLoop),
            };

            emit_pop(opcodes, vars.len() - vars_len);
            opcodes.push(OpCode::Jump(start_index));
        },
        Statement::Break => {
            if loop_starts.len() == 0 {
                return Err(FlattenError::BreakOutsideLoop);
            }

            emit_pop(opcodes, vars.len() - loop_starts.last_mut().unwrap().1);
            loop_starts.last_mut().unwrap().2.push(opcodes.len());
            opcodes.push(OpCode::Jump(0));
        }
        Statement::While { condition, block } => {
            let start_index: usize = opcodes.len();
            if flatten_expression(condition, opcodes, global_vars, vars, funcs, classes)? != DataType::Bool {
                return Err(FlattenError::NonBoolCondition);
            }

            let jump_index: usize = opcodes.len();
            opcodes.push(OpCode::JumpIfFalse { index: 0, pop: true });

            loop_starts.push((start_index, vars.len(), Vec::new()));
            flatten_block(block, opcodes, global_vars, vars, funcs, classes, loop_starts, depth, func_data)?;
            let (_, _, unpatched_breaks) = loop_starts.pop().unwrap();
            let never_exits: bool = matches!(&**condition, Expression::Literal(LiteralType::Bool(true))) && unpatched_breaks.is_empty();

            opcodes.push(OpCode::Jump(start_index));
            *opcodes.get_mut(jump_index).unwrap() = OpCode::JumpIfFalse { index: opcodes.len(), pop: true };

            for index in unpatched_breaks {
                *opcodes.get_mut(index).unwrap() = OpCode::Jump(opcodes.len());
            }

            return Ok(never_exits);
        },
        Statement::For { initializer, condition, update, block } => {
            let outer_vars_len: usize = vars.len();
            flatten_statement(initializer, opcodes, global_vars, vars, funcs, classes, loop_starts, depth + 1, func_data)?;
            let inner_vars_len: usize = vars.len();

            let skip_index: usize = opcodes.len();
            opcodes.push(OpCode::Jump(0));

            flatten_expression(update, opcodes, global_vars, vars, funcs, classes)?;
            emit_pop(opcodes, 1);
            
            *opcodes.get_mut(skip_index).unwrap() = OpCode::Jump(opcodes.len());

            if flatten_expression(condition, opcodes, global_vars, vars, funcs, classes)? != DataType::Bool {
                return Err(FlattenError::NonBoolCondition);
            }
            let jump_index: usize = opcodes.len();
            opcodes.push(OpCode::JumpIfFalse { index: 0, pop: true });

            loop_starts.push((skip_index + 1, inner_vars_len, Vec::new()));
            flatten_block(block, opcodes, global_vars, vars, funcs, classes, loop_starts, depth, func_data)?;
            let (_, _, unpatched_breaks) = loop_starts.pop().unwrap();
            let never_exits: bool = matches!(**condition, Expression::Literal(LiteralType::Bool(true))) && unpatched_breaks.is_empty();

            opcodes.push(OpCode::Jump(skip_index + 1));

            let end_index: usize = opcodes.len();
            *opcodes.get_mut(jump_index).unwrap() = OpCode::JumpIfFalse { index: opcodes.len(), pop: true };
            emit_pop(opcodes, vars.len() - outer_vars_len);
            vars.truncate(outer_vars_len);

            for index in unpatched_breaks {
                *opcodes.get_mut(index).unwrap() = OpCode::Jump(end_index);
            }

            return Ok(never_exits);
        },
        Statement::Print(expression) => {
            let d: DataType = flatten_expression(expression, opcodes, global_vars, vars, funcs, classes)?;
            if d == DataType::Void {
                return Err(FlattenError::VoidValueUsed);
            }
            opcodes.push(OpCode::Print);
        },
        Statement::Declaration { name, value, data_type, is_static } => {
            if !is_valid_type(data_type, classes) {
                return Err(FlattenError::UndeclaredType(data_type.clone()));
            }
            if name == "this" {
                return Err(FlattenError::ReservedKeyword(name.clone()));
            }
            if *is_static {
                return Err(FlattenError::StaticOutsideClass(name.clone()));
            }
            if depth == 0 && global_vars.iter().any(|(s, _)| s == name) || depth != 0 && vars.iter().any(|(s, d)| s == name) || funcs.contains_key(name) || classes.contains_key(name) {
                return Err(FlattenError::Shadowing(name.clone()));
            }

            let d: DataType = flatten_expression_expected(value, opcodes, global_vars, vars, funcs, classes, Some(data_type))?;

            if !is_compatible(data_type, &d, classes) {
                return Err(FlattenError::DeclarationTypeMismatch { variable: name.clone(), expected: data_type.clone(), found: d });
            }

            if depth != 0 {
                vars.push((name.clone(), data_type.clone()));
            }
            else {
                opcodes.push(OpCode::DefineGlobal);
                global_vars.push((name.clone(), data_type.clone()));
            }
        },
        Statement::Function { name, data, should_override, is_static } => {
            if !is_valid_type(&data.data_type, classes) {
                return Err(FlattenError::UndeclaredType(data.data_type.clone()));
            }
            if *is_static {
                return Err(FlattenError::StaticOutsideClass(name.clone()));
            }
            if depth != 0 {
                return Err(FlattenError::FunctionDeclarationInsideScope(name.to_owned()))
            }
            if funcs.contains_key(name) || global_vars.iter().any(|(s, _)| s == name) || classes.contains_key(name) {
                return Err(FlattenError::Shadowing(name.to_owned())); 
            }
            if *should_override {
                return Err(FlattenError::InvalidOverride(name.clone()));
            }

            let index: usize = opcodes.len() + 1;
            funcs.insert(name.clone(), (index, data.data_type.clone(), data.parameters.iter().map(|(d, _)| d.clone()).collect()));
            flatten_function(name, data, opcodes, global_vars, funcs, classes, loop_starts, depth, None)?;
        },
        Statement::Class { name, block, parent } => {
            if depth != 0 {
                return Err(FlattenError::ClassDeclarationInsideScope(name.clone()));
            }
            if classes.contains_key(name) || global_vars.iter().any(|(s, _)| s == name) || funcs.contains_key(name) {
                return Err(FlattenError::Shadowing(name.clone()));
            }
            if let Some(n) = parent {
                if n == name {
                    return Err(FlattenError::ParentIsSelf(name.clone()));
                }
            }

            let id: usize = classes.len();
            classes.insert(name.clone(), ClassData { vars: Vec::new(), funcs: HashMap::new(), vtable: Vec::new(), parent: parent.clone(), constructor: 0, id });

            for member in block {
                match member {
                    Statement::Declaration { name: var_name, value, data_type, is_static: true } => {
                        if !is_valid_type(data_type, classes) {
                            return Err(FlattenError::UndeclaredType(data_type.clone()));
                        }
                        if global_vars.iter().any(|(s, _)| s == &format!("{}.{}", name, var_name)) || funcs.contains_key(&format!("{}.{}", name, var_name)) {
                            return Err(FlattenError::Shadowing(var_name.clone()));
                        }

                        let d: DataType = flatten_expression_expected(value, opcodes, global_vars, vars, funcs, classes, Some(data_type))?;

                        if !is_compatible(data_type, &d, classes) {
                            return Err(FlattenError::DeclarationTypeMismatch { variable: format!("{}.{}", name, var_name), expected: data_type.clone(), found: d });
                        }

                        opcodes.push(OpCode::DefineGlobal);

                        global_vars.push((format!("{}.{}", name, var_name), data_type.clone()));
                    },
                    Statement::Function { name: func_name, data, should_override, is_static: true  } => {
                        if *should_override {
                            return Err(FlattenError::InvalidOverride(format!("{}.{}", name.clone(), func_name.clone())));
                        }
                        if funcs.iter().any(|(s, _)| s == &format!("{}.{}", name, func_name)) || global_vars.iter().any(|(s, _)| s == &format!("{}.{}", name, func_name)) {
                            return Err(FlattenError::Shadowing(func_name.clone()));
                        }

                        let index: usize = opcodes.len() + 1;
                        funcs.insert(format!("{}.{}", name, func_name), (index, data.data_type.clone(), data.parameters.iter().map(|(d, _)| d.clone()).collect()));
                        flatten_function(func_name, data, opcodes, global_vars, funcs, classes, loop_starts, depth, None)?;
                    },
                    _ => continue,
                }
            }

            let jump_index: usize = opcodes.len();
            classes.get_mut(name).unwrap().constructor = opcodes.len() + 1;

            if let Some(n) = parent {
                let (parent_vars, parent_funcs, parent_vtable) = {
                    if let Some(p) = classes.get(n) {
                        (p.vars.clone(), p.funcs.clone(), p.vtable.clone())
                    } else {
                        return Err(FlattenError::UndeclaredClass(n.clone()));
                    }
                };
                
                let class: &mut ClassData = classes.get_mut(name).unwrap();
                class.vars = parent_vars;
                class.funcs = parent_funcs;
                class.vtable = parent_vtable;

                for (_, (_, _, _, overridable)) in &mut class.funcs {
                    *overridable = true;
                }
            } 

            opcodes.push(OpCode::Jump(0));

            if let Some(n) = parent {
                if let Some(p) = classes.get(n) {
                    opcodes.push(OpCode::GetVar(0));
                    opcodes.push(OpCode::Call { index: p.constructor, parameters: 1 });
                    emit_pop(opcodes, 1);
                }
            }

            let empty_vars: Vec<(String, DataType)> = Vec::new();

            for member in block {
                match member {
                    Statement::Declaration { name: var_name, value, data_type, is_static: false } => {
                        if !is_valid_type(data_type, classes) {
                            return Err(FlattenError::UndeclaredType(data_type.clone()));
                        }
                        {
                            let class: &mut ClassData = classes.get_mut(name).unwrap();
                            if class.vars.iter().any(|(s, _d)| s == var_name) || class.funcs.contains_key(var_name) || global_vars.iter().any(|(s, _)| s == &format!("{}.{}", name, var_name)) || funcs.contains_key(&format!("{}.{}", name, var_name)) {
                                return Err(FlattenError::Shadowing(var_name.clone()));
                            }
                        }

                        opcodes.push(OpCode::GetVar(0));
                        let d: DataType = flatten_expression_expected(value, opcodes, global_vars, &empty_vars, funcs, classes, Some(data_type))?;

                        if !is_compatible(data_type, &d, classes) {
                            return Err(FlattenError::DeclarationTypeMismatch { variable: format!("{}.{}", name, var_name), expected: data_type.clone(), found: d });
                        }

                        {
                            let class: &mut ClassData = classes.get_mut(name).unwrap();

                            opcodes.push(OpCode::SetMember(class.vars.len()));
                            emit_pop(opcodes, 1);

                            class.vars.push((var_name.clone(), data_type.clone()));
                        }
                    },
                    Statement::Function { name: func_name, data, should_override, is_static: false  } => {
                        let (override_slot, overridable, class_has_var, base_sig):
                            (Option<usize>, bool, bool, Option<(DataType, Vec<DataType>)>) = {
                            let class: &ClassData = classes.get(name).unwrap();
                            let entry = class.funcs.get(func_name);
                            (
                                entry.map(|(slot, _, _, _)| *slot),
                                entry.map(|(_, _, _, overridable)| *overridable).unwrap_or(false),
                                class.vars.iter().any(|(s, _)| s == func_name),
                                entry.map(|(_, r, p, _)| (r.clone(), p.clone())),
                            )
                        };

                        if (override_slot.is_some() && (!*should_override || !overridable)) || class_has_var
                            || global_vars.iter().any(|(s, _)| s == &format!("{}.{}", name, func_name))
                            || funcs.contains_key(&format!("{}.{}", name, func_name)) {
                            return Err(FlattenError::Shadowing(func_name.clone()));
                        }
                        if *should_override && override_slot.is_none() {
                            return Err(FlattenError::InvalidOverride(func_name.clone()));
                        }

                        if let Some((base_return, base_params)) = &base_sig {
                            let params_ok = base_params.len() == data.parameters.len()
                                && base_params.iter().zip(data.parameters.iter())
                                    .all(|(base_p, (new_p, _))| base_p == new_p);

                            if !is_compatible(base_return, &data.data_type, classes) || !params_ok {
                                return Err(FlattenError::OverrideSignatureMismatch {
                                    name: format!("{}.{}", name, func_name),
                                    expected: (base_return.clone(), base_params.clone()),
                                    found: (data.data_type.clone(), data.parameters.iter().map(|(d, _)| d.clone()).collect()),
                                });
                            }
                        }

                        let slot: usize = {
                            let class: &mut ClassData = classes.get_mut(name).unwrap();
                            if let Some(slot) = override_slot {
                                *class.vtable.get_mut(slot).unwrap() = opcodes.len() + 1;
                                slot
                            } else {
                                class.vtable.push(opcodes.len() + 1);
                                class.vtable.len() - 1
                            }
                        };

                        classes.get_mut(name).unwrap().funcs.insert(func_name.clone(), (slot, data.data_type.clone(), data.parameters.iter().map(|(d, _)| d.clone()).collect(), false));
                        flatten_function(func_name, data, opcodes, global_vars, funcs, classes, loop_starts, depth, Some(name))?;
                    },
                    Statement::Declaration { name: _, value: _, data_type: _, is_static: true } => {
                        continue
                    },
                    Statement::Function { name: _, data: _, should_override: _, is_static: true  } => {
                        continue
                    },
                    statement => return Err(FlattenError::InvalidClassMember(statement.clone())),
                }
            }

            opcodes.push(OpCode::GetVar(0));
            opcodes.push(OpCode::Return);
            *opcodes.get_mut(jump_index).unwrap() = OpCode::Jump(opcodes.len());
        }
    };

    Ok(false)
}

fn flatten_block(statements: &[Statement], opcodes: &mut Vec<OpCode>, global_vars: &mut Vec<(String, DataType)>, vars: &mut Vec<(String, DataType)>, funcs: &mut HashMap<String, (usize, DataType, Vec<DataType>)>, classes: &mut HashMap<String, ClassData>, loop_starts: &mut Vec<(usize, usize, Vec<usize>)>, depth: usize, func_data: Option<(&str, &DataType)>) -> Result<bool, FlattenError> {
    let start_index: usize = vars.len();
    let (r, t): (bool, bool) = flatten_statements(statements, opcodes, global_vars, vars, funcs, classes, loop_starts, depth + 1, func_data)?;

    if !t { 
        emit_pop(opcodes, vars.len() - start_index); 
    }
    vars.truncate(start_index);

    Ok(r)
}

fn flatten_expression(expression: &Expression, opcodes: &mut Vec<OpCode>, global_vars: &Vec<(String, DataType)>, vars: &Vec<(String, DataType)>, funcs: &HashMap<String, (usize, DataType, Vec<DataType>)>, classes: &HashMap<String, ClassData>) -> Result<DataType, FlattenError> {
    match expression {
        Expression::Unary { operator, right } => Ok(flatten_unary(operator, right, opcodes, global_vars, vars, funcs, classes)?),
        Expression::Binary { left, operator, right } => flatten_binary(left, operator, right, opcodes, global_vars, vars, funcs, classes),
        Expression::Assignment { target, value } => {
            match &**target {
                Expression::Variable(i) => {
                    if i == "this" {
                        return Err(FlattenError::InvalidAssignmentTarget(target.clone()));
                    }
                    let (expected_type, index, is_global): (&DataType, usize, bool) = match vars.iter().rposition(|(s, _)| s == i) {
                        Some(index) => {
                            let d: &DataType = &vars.get(index).unwrap().1;

                            (d, index, false)
                        },
                        None => match global_vars.iter().rposition(|(s, _)| s == i) {
                            Some(index) => {
                                let d: &DataType = &global_vars.get(index).unwrap().1;
                                
                                (d, index, true)
                            },
                            None => return Err(FlattenError::UndeclaredVariable(i.clone())),
                        }
                    };
                    
                    let value_type = flatten_expression_expected(value, opcodes, global_vars, vars, funcs, classes, Some(expected_type))?;

                if !is_compatible(expected_type, &value_type, classes) {
                        return Err(FlattenError::AssignmentTypeMismatch { target: target.clone(), expected: expected_type.clone(), found: value_type });
                    }

                    if is_global {
                        opcodes.push(OpCode::SetGlobal(index));
                    }
                    else {
                        opcodes.push(OpCode::SetVar(index));
                    }

                    Ok(value_type)
                },
                Expression::MemberAccess { class, member } => {
                    if let Expression::Variable(name) = &**class {
                        if classes.contains_key(name) {
                            if let Some(index) = get_static_member(name, member, classes, global_vars) {
                                let expected: &DataType = &global_vars.get(index).unwrap().1;
                                let d: DataType = flatten_expression_expected(value, opcodes, global_vars, vars, funcs, classes, Some(expected))?;

                                if !is_compatible(expected, &d, classes) {
                                    return Err(FlattenError::AssignmentTypeMismatch { target: target.clone(), expected: global_vars.get(index).unwrap().1.clone(), found: d });
                                }

                                opcodes.push(OpCode::SetGlobal(index));
                                return Ok(d);
                            }
                            else {
                                return Err(FlattenError::UndeclaredClassVariable { class: name.to_string(), member: member.to_string() });
                            }
                        }
                    }
                    let class_type: DataType = flatten_expression(class, opcodes, global_vars, vars, funcs, classes)?;

                    if let DataType::Instance(name) = class_type {
                        if let Some(class_data) = classes.get(&name) {
                            if let Some(index) = class_data.vars.iter().position(|(s, _)| s == member) {
                                let expected: &DataType = &class_data.vars.get(index).unwrap().1;
                                let d: DataType = flatten_expression_expected(value, opcodes, global_vars, vars, funcs, classes, Some(expected))?;

                                if !is_compatible(expected, &d, classes) {
                                    return Err(FlattenError::AssignmentTypeMismatch { target: target.clone(), expected: class_data.vars.get(index).unwrap().1.clone(), found: d });
                                }

                                opcodes.push(OpCode::SetMember(index));
                                return Ok(d);
                            }
                            return Err(FlattenError::UndeclaredClassVariable { class: name.to_string(), member: member.to_string() });
                        }
                    }

                    Err(FlattenError::NotAnInstance(class.clone()))
                },
                Expression::ArrayAccess { target: target_array, index } => {
                    let (data_type, len): (DataType, usize) = match flatten_expression(target_array, opcodes, global_vars, vars, funcs, classes)? {
                        DataType::Array(d, l) => (*d, l),
                        _ => return Err(FlattenError::NotAnArray(target_array.clone())),
                    };
                    
                    if let Some(i) = is_const_index_invalid(len, index) {
                        return Err(FlattenError::ArrayIndexOutOfBounds { array: target_array.clone(), index: i, len });
                    }
                    if flatten_expression(index, opcodes, global_vars, vars, funcs, classes)? != DataType::Int {
                        return Err(FlattenError::InvalidIndexType(index.clone()));
                    }
                    let d: DataType = flatten_expression_expected(value, opcodes, global_vars, vars, funcs, classes, Some(&data_type))?;
                    if !is_compatible(&data_type, &d, classes) {
                        return Err(FlattenError::AssignmentTypeMismatch { target: target.clone(), expected: data_type, found: d });
                    }

                    opcodes.push(OpCode::SetArrayMember);
                    Ok(d)
                },
                _ => unreachable!(),
            }
        },
        Expression::Literal(l) => { opcodes.push(OpCode::PushConst(l.clone())); Ok(DataType::try_from(l).unwrap()) },
        Expression::Call { callee, parameters } => {
            match &**callee {
                Expression::Variable(i) => {
                    if let Some(f) = funcs.get(i) {
                        if parameters.len() != f.2.len() {
                            return Err(FlattenError::ParameterCountMismatch { callee: callee.clone(), expected: f.2.len(), found: parameters.len() })
                        }
                        for (i, parameter) in parameters.iter().enumerate() {
                            let expected = f.2.get(i).unwrap();
                            let d: DataType = flatten_expression_expected(parameter, opcodes, global_vars, vars, funcs, classes, Some(expected))?;
                            if !is_compatible(expected, &d, classes) {
                                return Err(FlattenError::ParameterTypeMismatch { callee: callee.clone(), expected: expected.clone(), found: d, index: i });
                            }
                        }

                        opcodes.push(OpCode::Call { index: f.0, parameters: parameters.len() });
                        Ok(f.1.clone())
                    }
                    else if let Some(c) = classes.get(i) {
                        if parameters.len() != 0 {
                            return Err(FlattenError::ParameterCountMismatch { callee: callee.clone(), expected: 0, found: parameters.len() });
                        }
                        if c.constructor == 0 {
                            return Err(FlattenError::ConstructorUnavailable(i.clone()));
                        }

                        opcodes.push(OpCode::NewInstance(c.id));
                        opcodes.push(OpCode::Call { index: c.constructor, parameters: parameters.len() + 1 });
                        Ok(DataType::Instance(i.clone()))
                    }
                    else if let Some(builtin) = Builtin::lookup(i) {
                        let (p, r) = builtin.signature();
                        
                        if p.len() != parameters.len() {
                            return Err(FlattenError::ParameterCountMismatch { callee: callee.clone(), expected: p.len(), found: parameters.len() });
                        }
                        for (i, e) in parameters.iter().enumerate() {
                            let d: DataType = flatten_expression_expected(e, opcodes, global_vars, vars, funcs, classes, Some(p.get(i).unwrap()))?;

                            if !is_compatible(p.get(i).unwrap(), &d, classes) {
                                return Err(FlattenError::ParameterTypeMismatch { callee: callee.clone(), index: i, expected: p.get(i).unwrap().clone(), found: d });
                            }
                        }
                        builtin.call(opcodes);
                        Ok(r)
                    }
                    else {
                        return Err(FlattenError::UndeclaredFunction(i.clone()));
                    }
                },
                Expression::MemberAccess { class, member } => {
                    if let Expression::Variable(n) = &**class {
                        if classes.contains_key(n) {
                            if let Some((index, return_type, p)) = get_static_func(n, member, classes, funcs) {
                                if parameters.len() != p.len() {
                                    return Err(FlattenError::ParameterCountMismatch { callee: callee.clone(), expected: p.len(), found: parameters.len() });
                                }
                                for (i, parameter) in parameters.iter().enumerate() {
                                    let expected = p.get(i).unwrap();
                                    let d: DataType = flatten_expression_expected(parameter, opcodes, global_vars, vars, funcs, classes, Some(expected))?;
                                    if !is_compatible(expected, &d, classes) {
                                        return Err(FlattenError::ParameterTypeMismatch { callee: callee.clone(), expected: expected.clone(), found: d, index: i });
                                    }
                                }
                                opcodes.push(OpCode::Call { index: *index, parameters: parameters.len() });
                                return Ok(return_type.clone());
                            }
                            return Err(FlattenError::UndeclaredClassFunction { class: n.to_string(), member: member.to_string() });
                        }
                    }

                    let class_type: DataType = flatten_expression(class, opcodes, global_vars, vars, funcs, classes)?;

                    if let DataType::Instance(class_name) = class_type {
                        if let Some(class_data) = classes.get(&class_name) {
                            if let Some((slot, d, p, _)) = class_data.funcs.get(member) {
                                if parameters.len() != p.len() {
                                    return Err(FlattenError::ParameterCountMismatch { callee: callee.clone(), expected: p.len(), found: parameters.len() });
                                }

                                for (i, parameter) in parameters.iter().enumerate() {
                                    let expected = p.get(i).unwrap();
                                    let d: DataType = flatten_expression_expected(parameter, opcodes, global_vars, vars, funcs, classes, Some(expected))?;
                                    if !is_compatible(expected, &d, classes) {
                                        return Err(FlattenError::ParameterTypeMismatch { callee: callee.clone(), expected: expected.clone(), found: d, index: i });
                                    }
                                }

                                opcodes.push(OpCode::CallVirtual { slot: *slot, parameters: parameters.len() + 1 });
                                return Ok(d.clone());
                            }
                            return Err(FlattenError::UndeclaredClassFunction { class: class_name.to_string(), member: member.to_string() });
                        }
                        return Err(FlattenError::UndeclaredClass(class_name));
                    }
                    if member == "clone" {
                        if let DataType::Array(_, _) = class_type {
                            if parameters.len() != 0 {
                                return Err(FlattenError::ParameterCountMismatch { callee: callee.clone(), expected: 0, found: parameters.len() });
                            }

                            opcodes.push(OpCode::CloneArray);
                            return Ok(class_type);
                        }
                    }
                    Err(FlattenError::NotAnInstance(class.clone()))
                },
                Expression::SafeAccess { class, member } => {
                    safe_member_access(class, member, Some(parameters), opcodes, global_vars, vars, funcs, classes)
                }
                _ => return Err(FlattenError::NotCallable(callee.clone())),
            }
        },
        Expression::MemberAccess { class, member } => {
            if let Expression::Variable(name) = &**class {
                if classes.contains_key(name) {
                    if let Some(index) = get_static_member(name, member, classes, global_vars) {
                        opcodes.push(OpCode::GetGlobal(index));
                        return Ok(global_vars.get(index).unwrap().1.clone());
                    }
                    else {
                        return Err(FlattenError::UndeclaredClassVariable { class: name.to_string(), member: member.to_string() });
                    }
                }
            }
 
            let class_type: DataType = flatten_expression(class, opcodes, global_vars, vars, funcs, classes)?;

            if let DataType::Instance(name) = class_type {
                if let Some(class_data) = classes.get(&name) {
                    if let Some(index) = class_data.vars.iter().position(|(s, _)| s == member) {
                        opcodes.push(OpCode::GetMember(index));
                        return Ok(class_data.vars.get(index).unwrap().1.clone());
                    }
                    return Err(FlattenError::UndeclaredClassVariable { class: name.to_string(), member: member.to_string() });
                }
            }

            Err(FlattenError::NotAnInstance(class.clone()))
        },
        Expression::Variable(i) => {
            match vars.iter().rposition(|(s, d)| s == i) {
                Some(index) => { opcodes.push(OpCode::GetVar(index)); Ok(vars.get(index).unwrap().1.clone()) },
                None => match global_vars.iter().rposition(|(s, d)| s == i) {
                    Some(index) => { opcodes.push(OpCode::GetGlobal(index)); Ok(global_vars.get(index).unwrap().1.clone()) },
                    None => return Err(FlattenError::UndeclaredVariable(i.clone())),
                }
            }
        },
        Expression::ArrayAccess { target, index } => {
            if let DataType::Array(data_type, len) = flatten_expression(target, opcodes, global_vars, vars, funcs, classes)? {
                if let Some(i) = is_const_index_invalid(len, index) {
                    return Err(FlattenError::ArrayIndexOutOfBounds { array: target.clone(), index: i, len });
                }
                if let DataType::Int = flatten_expression(index, opcodes, global_vars, vars, funcs, classes)? {
                    opcodes.push(OpCode::GetArrayMember);
                    Ok(*data_type)
                }
                else {
                    Err(FlattenError::InvalidIndexType(index.clone()))
                }
            }
            else {
                Err(FlattenError::NotAnArray(target.clone()))
            }
        },
        Expression::ArrayLiteral(_) => flatten_expression_expected(expression, opcodes, global_vars, vars, funcs, classes, None),
        Expression::Assert(expr) => {
            let d: DataType = match flatten_expression(expr, opcodes, global_vars, vars, funcs, classes)? {
                DataType::Nullable(d) => *d,
                found => return Err(FlattenError::AssertOnNonNullable { expr: expr.clone(), found }),
            };

            opcodes.push(OpCode::Assert);
            Ok(d)
        },
        Expression::SafeAccess { class, member } => {
            safe_member_access(class, member, None, opcodes, global_vars, vars, funcs, classes)
        },
        Expression::Evaluated(d) => Ok(d.clone()),
        Expression::ArrayFill { .. } =>  flatten_expression_expected(expression, opcodes, global_vars, vars, funcs, classes, None),
    }
}
fn flatten_expression_expected(expression: &Expression, opcodes: &mut Vec<OpCode>, global_vars: &Vec<(String, DataType)>, vars: &Vec<(String, DataType)>, funcs: &HashMap<String, (usize, DataType, Vec<DataType>)>, classes: &HashMap<String, ClassData>, expected_data_type: Option<&DataType>) -> Result<DataType, FlattenError> {
    match expression {
        Expression::ArrayLiteral(values) => {
            if values.len() == 0 {
                panic!("Array literal with len 0");
            }

            let expected: Option<&DataType> = match expected_data_type {
                Some(DataType::Nullable(inner)) if matches!(&**inner, DataType::Array(_, _)) => Some(&**inner),
                e => e,
            };
            let d: DataType = if let Some(DataType::Array(data_type, len)) = expected {
                for (i, v) in values.iter().enumerate() {
                    let d: DataType = flatten_expression_expected(v, opcodes, global_vars, vars, funcs, classes, Some(data_type))?;

                    if d == DataType::Void {
                        return Err(FlattenError::VoidValueUsed);
                    }

                    if !is_compatible(&**data_type, &d, classes) {
                        return Err(FlattenError::ArrayElementTypeMismatch { array: expression.clone(), index: i });
                    }
                }

                DataType::Array(data_type.clone(), values.len())
            }
            else {
                let mut nullable: bool = false;
                let mut data_type: Option<DataType> = None;

                for (i, v) in values.iter().enumerate() {
                    match flatten_expression_expected(v, opcodes, global_vars, vars, funcs, classes, data_type.as_ref())? {
                        DataType::Void => return Err(FlattenError::VoidValueUsed),
                        DataType::Null => nullable = true,
                        DataType::Nullable(inner_type) => {
                            nullable = true;
                            if let Some(ref d) = data_type {
                                let outer_type: DataType = DataType::Nullable(inner_type);
                                if !is_compatible(d, &outer_type, classes) {
                                    if is_compatible(&outer_type, d, classes) {
                                        data_type = Some(match outer_type {
                                            DataType::Nullable(i) => *i,
                                            _ => unreachable!(),
                                        });
                                    }
                                    else {
                                        return Err(FlattenError::ArrayElementTypeMismatch { array: expression.clone(), index: i });
                                    }
                                }
                            }
                            else {
                                data_type = Some(*inner_type);
                            }
                        },
                        expression_type => {
                            if let Some(ref d) = data_type {
                                if !is_compatible(d, &expression_type, classes) {
                                    if is_compatible(&expression_type, d, classes) {
                                        data_type = Some(expression_type);
                                    }
                                    else {
                                        return Err(FlattenError::ArrayElementTypeMismatch { array: expression.clone(), index: i });
                                    }
                                }
                            }
                            else {
                                data_type = Some(expression_type);
                            }
                        }
                    }
                }

                if let Some(mut d) = data_type {
                    if nullable {
                        d = DataType::Nullable(Box::new(d));
                    }
                    DataType::Array(Box::new(d), values.len())
                }
                else {
                    return Err(FlattenError::UnknownArrayLiteralType(expression.clone()));
                }
            };

            opcodes.push(OpCode::NewArray(values.len()));
            Ok(d)
        },
        Expression::ArrayFill { value, length } => {
            let expected_element_type: Option<&DataType> = match expected_data_type {
                Some(DataType::Nullable(innner_type)) => match &**innner_type {
                    DataType::Array(d, _) => Some(&**d),
                    _ => None,
                },
                Some(DataType::Array(d, _)) => Some(&**d),
                _ => None,
            };

            let d: DataType = flatten_expression_expected(value, opcodes, global_vars, vars, funcs, classes, expected_element_type)?;

            if matches!(d, DataType::Void) {
                return Err(FlattenError::VoidValueUsed);
            }
            if matches!(d, DataType::Null) && expected_element_type.is_none() {
                return Err(FlattenError::UnknownArrayLiteralType(expression.clone()));
            }

            let element_type: DataType = match expected_element_type {
                Some(e) => {
                    if !is_compatible(e, &d, classes) {
                        return Err(FlattenError::ArrayFillTypeMismatch { array: expression.clone(), expected: e.clone() });
                    }
                    e.clone()
                },
                None => d,
            };

            opcodes.push(OpCode::FillArray(*length as usize));

            Ok(DataType::Array(Box::new(element_type), usize::try_from(*length).unwrap()))
        },
        e => flatten_expression(e, opcodes, global_vars, vars, funcs, classes),
    }
}

fn flatten_unary(operator: &UnaryOp, right: &Box<Expression>, opcodes: &mut Vec<OpCode>, global_vars: &Vec<(String, DataType)>, vars: &Vec<(String, DataType)>, funcs: &HashMap<String, (usize, DataType, Vec<DataType>)>, classes: &HashMap<String, ClassData>) -> Result<DataType, FlattenError> {
    let operand_type: DataType = flatten_expression(right, opcodes, global_vars, vars, funcs, classes)?;

    let d: DataType = match (operator, &operand_type) {
        (UnaryOp::LNot, DataType::Bool) => DataType::Bool,
        (UnaryOp::Negate, DataType::Int) => DataType::Int,
        (UnaryOp::Negate, DataType::Float) => DataType::Float,
        _ => return Err(FlattenError::InvalidUnaryOperand {
            operator: operator.clone(),
            operand: operand_type,
        }),
    };

    opcodes.push(match operator {
        UnaryOp::LNot => OpCode::LNot,
        UnaryOp::Negate => OpCode::Negate,
    });

    Ok(d)
}

fn flatten_binary(left: &Box<Expression>, operator: &BinaryOp, right: &Box<Expression>, opcodes: &mut Vec<OpCode>, global_vars: &Vec<(String, DataType)>, vars: &Vec<(String, DataType)>, funcs: &HashMap<String, (usize, DataType, Vec<DataType>)>, classes: &HashMap<String, ClassData>) -> Result<DataType, FlattenError> {
    let d: DataType = match operator {
        BinaryOp::Add => normal_binary(left, right, OpCode::Add, opcodes, global_vars, vars, funcs, classes)?,
        BinaryOp::Subtract => normal_binary(left, right, OpCode::Subtract, opcodes, global_vars, vars, funcs, classes)?,
        BinaryOp::Multiply => normal_binary(left, right, OpCode::Multiply, opcodes, global_vars, vars, funcs, classes)?,
        BinaryOp::Divide => normal_binary(left, right, OpCode::Divide, opcodes, global_vars, vars, funcs, classes)?,
        BinaryOp::Equal => normal_binary(left, right, OpCode::Equal, opcodes, global_vars, vars, funcs, classes)?,
        BinaryOp::Greater => normal_binary(left, right, OpCode::Greater, opcodes, global_vars, vars, funcs, classes)?,
        BinaryOp::GreaterEqual => normal_binary(left, right, OpCode::GreaterEqual, opcodes, global_vars, vars, funcs, classes)?,
        BinaryOp::Less => normal_binary(left, right, OpCode::Less, opcodes, global_vars, vars, funcs, classes)?,
        BinaryOp::LessEqual => normal_binary(left, right, OpCode::LessEqual, opcodes, global_vars, vars, funcs, classes)?,
        BinaryOp::NotEqual => normal_binary(left, right, OpCode::NotEqual, opcodes, global_vars, vars, funcs, classes)?,
        BinaryOp::LOr => {
            short_circuit_binary(left, right, true, opcodes, global_vars, vars, funcs, classes)?
        },
        BinaryOp::LAnd => {
            short_circuit_binary(left, right, false, opcodes, global_vars, vars, funcs, classes)?
        },
        BinaryOp::Coalesce => coalesce_binary(left, right, opcodes, global_vars, vars, funcs, classes)?,
    };

    Ok(d)
}

fn normal_binary(left: &Box<Expression>, right: &Box<Expression>, opcode: OpCode, opcodes: &mut Vec<OpCode>, global_vars: &Vec<(String, DataType)>, vars: &Vec<(String, DataType)>, funcs: &HashMap<String, (usize, DataType, Vec<DataType>)>, classes: &HashMap<String, ClassData>) -> Result<DataType, FlattenError> {
    let l: DataType = flatten_expression(left, opcodes, global_vars, vars, funcs, classes)?;
    let r: DataType = flatten_expression(right, opcodes, global_vars, vars, funcs, classes)?;

    let operator: BinaryOp = BinaryOp::try_from(&opcode).map_err(|op| FlattenError::Internal(format!("opcode {op:?} is not a binary operator")))?;
    let d: DataType = get_binary_type(&l, &operator, &r, classes).ok_or_else(|| FlattenError::InvalidBinaryOperands { left: l, operator, right: r })?;

    opcodes.push(opcode.clone());

    Ok(d)
}

fn short_circuit_binary(left: &Box<Expression>, right: &Box<Expression>, jump_on: bool, opcodes: &mut Vec<OpCode>, global_vars: &Vec<(String, DataType)>, vars: &Vec<(String, DataType)>, funcs: &HashMap<String, (usize, DataType, Vec<DataType>)>, classes: &HashMap<String, ClassData>) -> Result<DataType, FlattenError> {
    let l: DataType = flatten_expression(left, opcodes, global_vars, vars, funcs, classes)?;

    if l != DataType::Bool {
        return Err(FlattenError::InvalidBinaryOperands { left: l, operator: if jump_on { BinaryOp::LOr } else { BinaryOp::LAnd }, right: DataType::Bool });
    }

    let start_index: usize = opcodes.len();
    opcodes.push(if jump_on { OpCode::JumpIfTrue { index: 0, pop: false } } else { OpCode::JumpIfFalse { index: 0, pop: false } });
    emit_pop(opcodes, 1);
    
    let r: DataType = flatten_expression(right, opcodes, global_vars, vars, funcs, classes)?;

    if r != DataType::Bool {
        return Err(FlattenError::InvalidBinaryOperands { left: l, operator: if jump_on { BinaryOp::LOr } else { BinaryOp::LAnd }, right: r });
    }

    *opcodes.get_mut(start_index).unwrap() = if jump_on { OpCode::JumpIfTrue { index: opcodes.len(), pop: false } } else { OpCode::JumpIfFalse { index: opcodes.len(), pop: false } };

    Ok(DataType::Bool)
}

fn coalesce_binary(left: &Expression, right: &Expression, opcodes: &mut Vec<OpCode>, global_vars: &Vec<(String, DataType)>, vars: &Vec<(String, DataType)>, funcs: &HashMap<String, (usize, DataType, Vec<DataType>)>, classes: &HashMap<String, ClassData>) -> Result<DataType, FlattenError> {
    let l: DataType = flatten_expression(left, opcodes, global_vars, vars, funcs, classes)?;

    let jump_index: usize = opcodes.len();
    opcodes.push(OpCode::JumpIfNotNull(0));
    opcodes.push(OpCode::Pop(1));

    let r: DataType = flatten_expression(right, opcodes, global_vars, vars, funcs, classes)?;

    *opcodes.get_mut(jump_index).unwrap() = OpCode::JumpIfNotNull(opcodes.len());

    if let Some(d) = unify_types(&l, &r, classes) {
        Ok(d)
    }
    else {
        Err(FlattenError::InvalidBinaryOperands { left: l, operator: BinaryOp::Coalesce, right: r })
    }
}

fn flatten_function(name: &str, data: &FunctionData, opcodes: &mut Vec<OpCode>, global_vars: &mut Vec<(String, DataType)>, funcs: &mut HashMap<String, (usize, DataType, Vec<DataType>)>, classes: &mut HashMap<String, ClassData>, loop_starts: &mut Vec<(usize, usize, Vec<usize>)>, depth: usize, class_name: Option<&String>) -> Result<(), FlattenError> {
    if !is_valid_type(&data.data_type, classes) {
        return Err(FlattenError::UndeclaredType(data.data_type.clone()));
    }

    let jump_index: usize = opcodes.len();
    opcodes.push(OpCode::Jump(0));

    let mut func_vars: Vec<(String, DataType)> = Vec::new();

    if let Some(n) = class_name {
        func_vars.push(("this".to_owned(), DataType::Instance(n.clone())));
    }

    for (data_type, name) in &data.parameters {
        if !is_valid_type(data_type, classes) {
            return Err(FlattenError::UndeclaredType(data_type.clone()));
        }
        if name == "this" {
            return Err(FlattenError::ReservedKeyword(name.clone()));
        }
        if func_vars.iter().any(|(s, _)| s == name) || funcs.contains_key(name) || classes.contains_key(name) {
            return Err(FlattenError::Shadowing(name.clone()));
        }
        func_vars.push((name.clone(), data_type.clone()));
    }


    let r: bool = flatten_block(&data.block, opcodes, global_vars, &mut func_vars, funcs, classes, loop_starts, depth, Some((name, &data.data_type)))?;

    if !r {
        if !matches!(data.data_type, DataType::Void | DataType::Nullable(_)) {
            return Err(FlattenError::MissingReturn(name.to_owned()));
        }

        flatten_expression(&Expression::Literal(LiteralType::Null), opcodes, global_vars, &mut func_vars, funcs, classes)?;
        opcodes.push(OpCode::Return);
    }    
    
    *opcodes.get_mut(jump_index).unwrap() = OpCode::Jump(opcodes.len());

    Ok(())
}

fn emit_pop(opcodes: &mut Vec<OpCode>, count: usize) {
    if count > 0 {
        opcodes.push(OpCode::Pop(count));
    }
}

impl<'a> TryFrom<&'a LiteralType> for DataType {
    type Error = &'a LiteralType;

    fn try_from(literal_type: &LiteralType) -> Result<Self, Self::Error> {
        Ok(match literal_type {
            LiteralType::Bool(_) => DataType::Bool,
            LiteralType::Int(_) => DataType::Int,
            LiteralType::Float(_) => DataType::Float,
            LiteralType::String(_) => DataType::String,
            LiteralType::Null => DataType::Null,
        })
    }
}

impl<'a> TryFrom<&'a OpCode> for BinaryOp {
    type Error = &'a OpCode;

    fn try_from(opcode: &'a OpCode) -> Result<Self, Self::Error> {
        match opcode {
            OpCode::Add => Ok(BinaryOp::Add),
            OpCode::Subtract => Ok(BinaryOp::Subtract),
            OpCode::Multiply => Ok(BinaryOp::Multiply),
            OpCode::Divide => Ok(BinaryOp::Divide),
            OpCode::Equal => Ok(BinaryOp::Equal),
            OpCode::NotEqual => Ok(BinaryOp::NotEqual),
            OpCode::Less => Ok(BinaryOp::Less),
            OpCode::LessEqual => Ok(BinaryOp::LessEqual),
            OpCode::Greater => Ok(BinaryOp::Greater),
            OpCode::GreaterEqual => Ok(BinaryOp::GreaterEqual),
            _ => Err(opcode),
        }
    }
}

fn is_valid_type(data_type: &DataType, classes: &HashMap<String, ClassData>) -> bool {
    match data_type {
        DataType::Nullable(d) => is_valid_type(d, classes),
        DataType::Instance(n) => classes.contains_key(n),
        DataType::Array(d, _) => is_valid_type(d, classes),
        _ => true,
    }
}

fn is_compatible(expected: &DataType, found: &DataType, classes: &HashMap<String, ClassData>) -> bool {
    match (expected, found) {
        (DataType::Nullable(_), DataType::Null) => true,
        (DataType::Nullable(a), DataType::Nullable(b)) => is_compatible(a, b, classes),
        (DataType::Nullable(inner), actual) => is_compatible(inner, actual, classes),
        (DataType::Instance(p), DataType::Instance(c)) if is_subclass(c, p, classes) => true,
        (expected, found) => expected == found,
    }
}

fn is_subclass(child: &str, parent: &str, classes: &HashMap<String, ClassData>) -> bool {
    let mut class: &str = child;

    while class != parent {
        if let Some(c) = classes.get(class) {
            if let Some(ref p) = c.parent {
                class = p;
                continue;
            }
        }
        return false;
    }
    true
}

fn get_static_member<'a>(mut class: &'a str, member: &str, classes: &'a HashMap<String, ClassData>, global_vars: &Vec<(String, DataType)>) -> Option<usize> {
    loop {
        if let Some(index) = global_vars.iter().position(|(s, _)| s == &format!("{}.{}", class, member)) {
            return Some(index);
        }
        if let Some(p) = classes.get(class).and_then(|c| c.parent.as_deref()) {
            class = p;
        }
        else {
            return None;
        }
    }
}

fn get_static_func<'a>(mut class: &'a str, member: &str, classes: &'a HashMap<String, ClassData>, funcs: &'a HashMap<String, (usize, DataType, Vec<DataType>)>) -> Option<&'a (usize, DataType, Vec<DataType>)> {
    loop {
        let func_name: String = format!("{}.{}", class, member);
        if let Some(d) = funcs.get(&func_name) {
            return Some(d);
        }
        if let Some(p) = classes.get(class).and_then(|c| c.parent.as_deref()) {
            class = p;
        }
        else {
            return None;
        }
    }
}

fn get_binary_type(left: &DataType, operator: &BinaryOp, right: &DataType, classes: &HashMap<String, ClassData>) -> Option<DataType> {
    match (left, operator, right) {
        (DataType::Int, BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide, DataType::Int) => Some(DataType::Int),
        (DataType::Float, BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide, DataType::Float) => Some(DataType::Float),
        (DataType::String, BinaryOp::Add, DataType::String) => Some(DataType::String),

        (DataType::Int, BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual, DataType::Int) => Some(DataType::Bool),
        (DataType::Float, BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual, DataType::Float) => Some(DataType::Bool),
        (DataType::String, BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual, DataType::String) => Some(DataType::Bool),

        (l, BinaryOp::Equal | BinaryOp::NotEqual, r) if *l != DataType::Void && (is_compatible(l, r, classes) || is_compatible(r, l, classes)) => Some(DataType::Bool),

        _ => None,
    }
}

fn is_const_index_invalid(len: usize, index: &Box<Expression>) -> Option<i64> {
    if let Expression::Literal(LiteralType::Int(i)) = &**index {
        if *i < 0 || *i >= len.try_into().unwrap() {
            return Some(*i);
        } 
    }
    None
}

fn safe_member_access(class: &Box<Expression>, member: &String, parameters: Option<&Vec<Expression>>, opcodes: &mut Vec<OpCode>, global_vars: &Vec<(String, DataType)>, vars: &Vec<(String, DataType)>, funcs: &HashMap<String, (usize, DataType, Vec<DataType>)>, classes: &HashMap<String, ClassData>) -> Result<DataType, FlattenError> {
    let d: DataType = match flatten_expression(&class, opcodes, global_vars, vars, funcs, classes)? {
        DataType::Nullable(inner) if matches!(*inner, DataType::Instance(_) | DataType::Array(_, _)) => *inner,
        _ => return Err(FlattenError::NotANullableInstance(class.clone()))
    };

    let jump_index: usize = opcodes.len();
    opcodes.push(OpCode::JumpIfNull(0));

    let access: Expression = Expression::MemberAccess { class: Box::new(Expression::Evaluated(d)), member: member.clone() };
    let expr: Expression = if let Some(p) = parameters {
        Expression::Call { callee: Box::new(access), parameters: p.clone() }
    }
    else {
        access
    };
    let result: DataType = flatten_expression(&expr, opcodes, global_vars, vars, funcs, classes)?;

    *opcodes.get_mut(jump_index).unwrap() = OpCode::JumpIfNull(opcodes.len());

    Ok(match result {
        DataType::Void => DataType::Void,
        DataType::Nullable(_) => result,
        d => DataType::Nullable(Box::new(d)),
    })
}

fn unify_types(left: &DataType, right: &DataType, classes: &HashMap<String, ClassData>) -> Option<DataType> {
    match left {
        DataType::Nullable(inner_left) => {
            match right {
                DataType::Nullable(inner_right) => {
                    if is_compatible(inner_left, inner_right, classes) {
                        Some(DataType::Nullable(inner_left.clone()))
                    }
                    else if is_compatible(inner_right, inner_left, classes) {
                        Some(DataType::Nullable(inner_right.clone()))
                    }
                    else {
                        None
                    }
                },
                r => {
                    if is_compatible(inner_left, r, classes) {
                        Some(*inner_left.clone())
                    }
                    else if is_compatible(r, inner_left, classes) {
                        Some(r.clone())
                    }
                    else {
                        None
                    }
                }
            }
        },
        _ => None,
    }
}
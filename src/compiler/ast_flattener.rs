use std::collections::HashMap;

use crate::compiler::{lexer::DataType, parser::{BinaryOp, Expression, FunctionData, LiteralType, Statement, UnaryOp}};

#[derive(Clone)]
enum OpCode {
    PushConst(LiteralType), Pop(usize),
    LNot, Negate, Add, Subtract, Multiply, Divide,
    Equal, Greater, GreaterEqual, Less, LessEqual, NotEqual,
    JumpIfTrue { index: usize, pop: bool }, JumpIfFalse { index: usize, pop: bool }, Jump(usize),
    GetVar(usize), SetVar(usize),
    GetGlobal(usize), SetGlobal(usize), DefineGlobal,
    Call { index: usize, parameters: usize }, CallVirtual { slot: usize, parameters: usize },
    Return, Print,
    GetMember(usize), SetMember(usize),
    NewStack, PopStack,
    NewInstance(String), 
}
enum FlattenError {
    UndeclaredVariable(String), UndeclaredFunction(String), UndeclaredClass(String), InvalidFunctionCallee(Box<Expression>), ContinueOutsideLoop, BreakOutsideLoop,
    Shadowing(String), FunctionDeclarationInsideScope(String), ClassDeclarationInsideScope(String), UnexpectedClassMember(Statement), UnexpectedOverride(String),
    UndeclaredClassVar(String),
    UnexpectedBinaryOpOpCode(OpCode), UnexpectedBinaryOpOperands { left: DataType, operator: BinaryOp, right: DataType }, UnexpectedUnaryOpOperands { operator: UnaryOp, operand: DataType },
    UnexpectedParameterCount { callee: Box<Expression>, expected: usize, received: usize },
    ExpressionIsNotClass(Box<Expression>), StaticOutsideClass(String), UnexpectedParameterType { callee: Box<Expression>, expected: DataType, received: DataType, index: usize },
    UnexpectedDeclarationValueType { variable: String, expected: DataType, received: DataType }, UnexpectedAssignmentValueType { variable: String, expected: DataType, received: DataType, },
    UnexpectedReturnValueType { func: String, expected: DataType, received: DataType }, ReturnOutsideFunction, MissingReturnStatement(String),
}
struct ClassData {
    vars: Vec<(String, DataType)>,
    funcs: HashMap<String, (usize, DataType, Vec<DataType>, bool)>,
    vtable: Vec<usize>,
    parent: Option<String>,
    constructor: usize,
}
fn flatten_statements(statements: &[Statement], opcodes: &mut Vec<OpCode>, global_vars: &mut Vec<(String, DataType)>, vars: &mut Vec<(String, DataType)>, funcs: &mut HashMap<String, (usize, DataType, Vec<DataType>)>, classes: &mut HashMap<String, ClassData>, loop_starts: &mut Vec<(usize, usize, Vec<usize>)>, depth: usize, func_data: Option<(&str, &DataType)>) -> Result<bool, FlattenError> {
    let mut r: bool = false;

    for statement in statements {
        r = flatten_statement(statement, opcodes, global_vars, vars, funcs, classes, loop_starts, depth, func_data)?;

        if r {
            break;
        }

        if matches!(statement, Statement::Break | Statement::Continue) {
            break;
        }
    }

    Ok(r)
}

fn flatten_statement(statement: &Statement, opcodes: &mut Vec<OpCode>, global_vars: &mut Vec<(String, DataType)>, vars: &mut Vec<(String, DataType)>, funcs: &mut HashMap<String, (usize, DataType, Vec<DataType>)>, classes: &mut HashMap<String, ClassData>, loop_starts: &mut Vec<(usize, usize, Vec<usize>)>, depth: usize, func_data: Option<(&str, &DataType)>) -> Result<bool, FlattenError> {
    match statement {
        Statement::Block(statements) => {
            return flatten_block(statements, opcodes, global_vars, vars, funcs, classes, loop_starts, depth, func_data);
        }
        Statement::Expression(expression) => { flatten_expression(expression, opcodes, global_vars, vars, funcs, classes)?; opcodes.push(OpCode::Pop(1)); },
        Statement::If { condition, block } => {
            flatten_expression(condition, opcodes, global_vars, vars, funcs, classes)?;

            let start_index: usize = opcodes.len();
            opcodes.push(OpCode::JumpIfFalse { index: 0, pop: true });

            flatten_block(block, opcodes, global_vars, vars, funcs, classes, loop_starts, depth, func_data)?;
            *opcodes.get_mut(start_index).unwrap() = OpCode::JumpIfFalse { index: opcodes.len(), pop: true };
        },
        Statement::IfElse { condition, block1, block2 } => {
            flatten_expression(condition, opcodes, global_vars, vars, funcs, classes)?;

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
                        let d: DataType = flatten_expression(e, opcodes, global_vars, vars, funcs, classes)?;
                        if !is_compatible(&data.1.clone(), &d, classes) {
                            return Err(FlattenError::UnexpectedReturnValueType { func: data.0.to_owned(), expected: data.1.clone(), received: d });
                        }
                    },
                    None => {
                        if !is_compatible(&data.1.clone(), &DataType::Null, classes) {
                            return Err(FlattenError::UnexpectedReturnValueType { func: data.0.to_owned(), expected: data.1.clone(), received: DataType::Null });
                        }
                        
                        flatten_expression(&Expression::Literal(LiteralType::Null), opcodes, global_vars, vars, funcs, classes)?;
                    },
                };

                opcodes.push(OpCode::PopStack);
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

            opcodes.push(OpCode::Pop(vars.len() - vars_len));
            opcodes.push(OpCode::Jump(start_index));
        },
        Statement::Break => {
            if loop_starts.len() == 0 {
                return Err(FlattenError::BreakOutsideLoop);
            }

            opcodes.push(OpCode::Pop(vars.len() - loop_starts.last_mut().unwrap().1));
            loop_starts.last_mut().unwrap().2.push(opcodes.len());
            opcodes.push(OpCode::Jump(0));
        }
        Statement::While { condition, block } => {
            let start_index: usize = opcodes.len();
            flatten_expression(condition, opcodes, global_vars, vars, funcs, classes)?;

            let jump_index: usize = opcodes.len();
            opcodes.push(OpCode::JumpIfFalse { index: 0, pop: true });

            loop_starts.push((start_index, vars.len(), Vec::new()));
            flatten_block(block, opcodes, global_vars, vars, funcs, classes, loop_starts, depth, func_data)?;
            let (_, _, unpatched_breaks) = loop_starts.pop().unwrap();

            opcodes.push(OpCode::Jump(start_index));
            *opcodes.get_mut(jump_index).unwrap() = OpCode::JumpIfFalse { index: opcodes.len(), pop: true };

            for index in unpatched_breaks {
                *opcodes.get_mut(index).unwrap() = OpCode::Jump(opcodes.len());
            }
        },
        Statement::For { initializer, condition, update, block } => {
            let outer_vars_len: usize = vars.len();
            flatten_statement(initializer, opcodes, global_vars, vars, funcs, classes, loop_starts, depth + 1, func_data)?;
            let inner_vars_len: usize = vars.len();

            let skip_index: usize = opcodes.len();
            opcodes.push(OpCode::Jump(0));

            flatten_expression(update, opcodes, global_vars, vars, funcs, classes)?;
            opcodes.push(OpCode::Pop(1));
            
            *opcodes.get_mut(skip_index).unwrap() = OpCode::Jump(opcodes.len());

            flatten_expression(condition, opcodes, global_vars, vars, funcs, classes)?;
            let jump_index: usize = opcodes.len();
            opcodes.push(OpCode::JumpIfFalse { index: 0, pop: true });

            loop_starts.push((skip_index + 1, inner_vars_len, Vec::new()));
            flatten_block(block, opcodes, global_vars, vars, funcs, classes, loop_starts, depth, func_data)?;
            let (_, _, unpatched_breaks) = loop_starts.pop().unwrap();

            opcodes.push(OpCode::Jump(skip_index + 1));

            *opcodes.get_mut(jump_index).unwrap() = OpCode::JumpIfFalse { index: opcodes.len(), pop: true };
            opcodes.push(OpCode::Pop(vars.len() - outer_vars_len));
            vars.truncate(outer_vars_len);

            for index in unpatched_breaks {
                *opcodes.get_mut(index).unwrap() = OpCode::Jump(opcodes.len() - 1);
            }
        },
        Statement::Print(expression) => {
            flatten_expression(expression, opcodes, global_vars, vars, funcs, classes)?;
            opcodes.push(OpCode::Print);
        },
        Statement::Declaration { name, value, data_type, is_static } => {
            if *is_static {
                return Err(FlattenError::StaticOutsideClass(name.clone()));
            }
            if depth == 0 && global_vars.iter().any(|(s, _)| s == name) || depth != 0 && vars.iter().any(|(s, d)| s == name) || funcs.contains_key(name) || classes.contains_key(name) {
                return Err(FlattenError::Shadowing(name.clone()));
            }

            let d: DataType = flatten_expression(value, opcodes, global_vars, vars, funcs, classes)?;

            if !is_compatible(data_type, &d, classes) {
                return Err(FlattenError::UnexpectedDeclarationValueType { variable: name.clone(), expected: data_type.clone(), received: d });
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
                return Err(FlattenError::UnexpectedOverride(name.clone()));
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

            classes.insert(name.clone(), ClassData { vars: Vec::new(), funcs: HashMap::new(), vtable: Vec::new(), parent: parent.clone(), constructor: 0 });

            for member in block {
                match member {
                    Statement::Declaration { name: var_name, value, data_type, is_static: true } => {
                        if global_vars.iter().any(|(s, _)| s == &format!("{}.{}", name, var_name)) || funcs.contains_key(&format!("{}.{}", name, var_name)) {
                            return Err(FlattenError::Shadowing(var_name.clone()));
                        }

                        let d: DataType = flatten_expression(value, opcodes, global_vars, vars, funcs, classes)?;

                        if !is_compatible(data_type, &d, classes) {
                            return Err(FlattenError::UnexpectedDeclarationValueType { variable: format!("{}.{}", name, var_name), expected: data_type.clone(), received: d });
                        }

                        opcodes.push(OpCode::DefineGlobal);

                        global_vars.push((format!("{}.{}", name, var_name), data_type.clone()));
                    },
                    Statement::Function { name: func_name, data, should_override, is_static: true  } => {
                        if *should_override {
                            return Err(FlattenError::UnexpectedOverride(format!("{}.{}", name.clone(), func_name.clone())));
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
            opcodes.push(OpCode::NewStack);

            if let Some(n) = parent {
                if let Some(p) = classes.get(n) {
                    opcodes.push(OpCode::GetVar(0));
                    opcodes.push(OpCode::Call { index: p.constructor, parameters: 1 });
                    opcodes.push(OpCode::Pop(1));
                }
            }

            let self_reference_vars: Vec<(String, DataType)> = vec![("this".to_owned(), DataType::Instance(name.clone()))];

            for member in block {
                match member {
                    Statement::Declaration { name: var_name, value, data_type, is_static: false } => {
                        {
                            let class: &mut ClassData = classes.get_mut(name).unwrap();
                            if class.vars.iter().any(|(s, _d)| s == var_name) || class.funcs.contains_key(var_name) || global_vars.iter().any(|(s, _)| s == &format!("{}.{}", name, var_name)) || funcs.contains_key(&format!("{}.{}", name, var_name)) {
                                return Err(FlattenError::Shadowing(var_name.clone()));
                            }
                        }

                        opcodes.push(OpCode::GetVar(0));
                        let d: DataType = flatten_expression(value, opcodes, global_vars, &self_reference_vars, funcs, classes)?;

                        if !is_compatible(data_type, &d, classes) {
                            return Err(FlattenError::UnexpectedDeclarationValueType { variable: format!("{}.{}", name, var_name), expected: data_type.clone(), received: d });
                        }

                        {
                            let class: &mut ClassData = classes.get_mut(name).unwrap();

                            opcodes.push(OpCode::SetMember(class.vars.len()));
                            opcodes.push(OpCode::Pop(1));

                            class.vars.push((var_name.clone(), data_type.clone()));
                        }
                    },
                    Statement::Function { name: func_name, data, should_override, is_static: false  } => {
                        {
                            let class: &mut ClassData = classes.get_mut(name).unwrap();

                            let override_slot: Option<usize> = class.funcs.get(func_name).map(|(slot, _, _, _)| *slot);
                            if (override_slot.is_some() && (!*should_override || !class.funcs.get(func_name).unwrap().3)) || class.vars.iter().any(|(s, _)| s == func_name) || global_vars.iter().any(|(s, _)| s == &format!("{}.{}", name, func_name)) || funcs.contains_key(&format!("{}.{}", name, func_name)) {
                                return Err(FlattenError::Shadowing(func_name.clone()));
                            }
                            if *should_override && override_slot.is_none() {
                                return Err(FlattenError::UnexpectedOverride(func_name.clone()));
                            }

                            let slot: usize = if let Some(slot) = override_slot {
                                *class.vtable.get_mut(slot).unwrap() = opcodes.len() + 1;
                                slot
                            }
                            else {
                                class.vtable.push(opcodes.len() + 1);
                                class.vtable.len() - 1
                            };

                            class.funcs.insert(func_name.clone(), (slot, data.data_type.clone(), data.parameters.iter().map(|(d, _)| d.clone()).collect(), false));
                        }
                        flatten_function(func_name, data, opcodes, global_vars, funcs, classes, loop_starts, depth, Some(name))?;
                    },
                    Statement::Declaration { name: _, value: _, data_type: _, is_static: true } => {
                        continue
                    },
                    Statement::Function { name: _, data: _, should_override: _, is_static: true  } => {
                        continue
                    },
                    statement => return Err(FlattenError::UnexpectedClassMember(statement.clone())),
                }
            }

            opcodes.push(OpCode::GetVar(0));
            opcodes.push(OpCode::PopStack);
            opcodes.push(OpCode::Return);
            *opcodes.get_mut(jump_index).unwrap() = OpCode::Jump(opcodes.len());
        }
    };

    Ok(false)
}

fn flatten_block(statements: &[Statement], opcodes: &mut Vec<OpCode>, global_vars: &mut Vec<(String, DataType)>, vars: &mut Vec<(String, DataType)>, funcs: &mut HashMap<String, (usize, DataType, Vec<DataType>)>, classes: &mut HashMap<String, ClassData>, loop_starts: &mut Vec<(usize, usize, Vec<usize>)>, depth: usize, func_data: Option<(&str, &DataType)>) -> Result<bool, FlattenError> {
    let start_index: usize = vars.len();
    let r: bool = flatten_statements(statements, opcodes, global_vars, vars, funcs, classes, loop_starts, depth + 1, func_data)?;

    opcodes.push(OpCode::Pop(vars.len() - start_index));
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
                    let value_type: DataType = flatten_expression(value, opcodes, global_vars, vars, funcs, classes)?;
                    opcodes.push(match vars.iter().rposition(|(s, _)| s == i) {
                        Some(index) => {
                            let d: &DataType = &vars.get(index).unwrap().1;
                            if !is_compatible(d, &value_type, classes) {
                                return Err(FlattenError::UnexpectedAssignmentValueType { variable: i.clone(), expected: d.clone(), received: value_type });
                            }
                            OpCode::SetVar(index)
                        },
                        None => match global_vars.iter().rposition(|(s, _)| s == i) {
                            Some(index) => {
                                let d: &DataType = &global_vars.get(index).unwrap().1;
                                if !is_compatible(d, &value_type, classes) {
                                    return Err(FlattenError::UnexpectedAssignmentValueType { variable: i.clone(), expected: d.clone(), received: value_type });
                                }
                                OpCode::SetGlobal(index)
                            },
                            None => return Err(FlattenError::UndeclaredVariable(i.clone())),
                        }
                    });
                    Ok(value_type)
                },
                Expression::MemberAccess { class, member } => {
                    if let Expression::Variable(name) = &**class {
                        if classes.contains_key(name) {
                            if let Some(index) = global_vars.iter().position(|(s, _)| s == &format!("{}.{}", name, member)) {
                                let d: DataType = flatten_expression(value, opcodes, global_vars, vars, funcs, classes)?;

                                if !is_compatible(&global_vars.get(index).unwrap().1, &d, classes) {
                                    return Err(FlattenError::UnexpectedAssignmentValueType { variable: format!("{}.{}", name, member), expected: global_vars.get(index).unwrap().1.clone(), received: d });
                                }

                                opcodes.push(OpCode::SetGlobal(index));
                                return Ok(d);
                            }
                            else {
                                return Err(FlattenError::UndeclaredClassVar(format!("{}.{}", name, member)));
                            }
                        }
                    }
                    let class_type: DataType = flatten_expression(class, opcodes, global_vars, vars, funcs, classes)?;

                    if let DataType::Instance(name) = class_type {
                        if let Some(class_data) = classes.get(&name) {
                            if let Some(index) = class_data.vars.iter().position(|(s, _)| s == member) {
                                let d: DataType = flatten_expression(value, opcodes, global_vars, vars, funcs, classes)?;

                                if !is_compatible(&class_data.vars.get(index).unwrap().1, &d, classes) {
                                    return Err(FlattenError::UnexpectedAssignmentValueType { variable: format!("{}.{}", name, member), expected: class_data.vars.get(index).unwrap().1.clone(), received: d });
                                }

                                opcodes.push(OpCode::SetMember(index));
                                return Ok(d);
                            }
                            return Err(FlattenError::UndeclaredClassVar(format!("{}.{}", name, member)));
                        }
                    }

                    Err(FlattenError::ExpressionIsNotClass(class.clone()))
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
                            return Err(FlattenError::UnexpectedParameterCount { callee: callee.clone(), expected: f.2.len(), received: parameters.len() })
                        }
                        for (i, parameter) in parameters.iter().enumerate() {
                            let d: DataType = flatten_expression(parameter, opcodes, global_vars, vars, funcs, classes)?;
                            let expected = f.2.get(i).unwrap();
                            if !is_compatible(expected, &d, classes) {
                                return Err(FlattenError::UnexpectedParameterType { callee: callee.clone(), expected: expected.clone(), received: d, index: i });
                            }
                        }

                        opcodes.push(OpCode::Call { index: f.0, parameters: parameters.len() });
                        Ok(f.1.clone())
                    }
                    else if let Some(c) = classes.get(i) {
                        opcodes.push(OpCode::NewInstance(i.clone()));
                        if parameters.len() != 0 {
                            return Err(FlattenError::UnexpectedParameterCount { callee: callee.clone(), expected: 0, received: parameters.len() });
                        }

                        opcodes.push(OpCode::Call { index: c.constructor, parameters: parameters.len() + 1 });
                        Ok(DataType::Instance(i.clone()))
                    }
                    else {
                        return Err(FlattenError::UndeclaredFunction(i.clone()));
                    }
                },
                Expression::MemberAccess { class, member } => {
                    if let Expression::Variable(n) = &**class {
                        if let Some(class_data) = classes.get(n) {
                            if let Some((index, return_type, p)) = funcs.get(&format!("{}.{}", n, member)) {
                                if parameters.len() != p.len() {
                                    return Err(FlattenError::UnexpectedParameterCount { callee: callee.clone(), expected: p.len(), received: parameters.len() });
                                }
                                for (i, parameter) in parameters.iter().enumerate() {
                                    let d: DataType = flatten_expression(parameter, opcodes, global_vars, vars, funcs, classes)?;
                                    let expected = p.get(i).unwrap();
                                    if !is_compatible(expected, &d, classes) {
                                        return Err(FlattenError::UnexpectedParameterType { callee: callee.clone(), expected: expected.clone(), received: d, index: i });
                                    }
                                }
                                opcodes.push(OpCode::Call { index: *index, parameters: parameters.len() });
                                return Ok(return_type.clone());
                            }
                        }
                    }

                    let class_type: DataType = flatten_expression(class, opcodes, global_vars, vars, funcs, classes)?;

                    if let DataType::Instance(class_name) = class_type {
                        if let Some(class_data) = classes.get(&class_name) {
                            if let Some((slot, d, p, _)) = class_data.funcs.get(member) {
                                if parameters.len() != p.len() {
                                    return Err(FlattenError::UnexpectedParameterCount { callee: callee.clone(), expected: p.len(), received: parameters.len() });
                                }

                                for (i, parameter) in parameters.iter().enumerate() {
                                    let d: DataType = flatten_expression(parameter, opcodes, global_vars, vars, funcs, classes)?;
                                    let expected = p.get(i).unwrap();
                                    if !is_compatible(expected, &d, classes) {
                                        return Err(FlattenError::UnexpectedParameterType { callee: callee.clone(), expected: expected.clone(), received: d, index: i });
                                    }
                                }

                                opcodes.push(OpCode::CallVirtual { slot: *slot, parameters: parameters.len() + 1 });
                                return Ok(d.clone());
                            }
                        }
                        return Err(FlattenError::UndeclaredClass(class_name));
                    }
                    Err(FlattenError::ExpressionIsNotClass(class.clone()))
                },
                _ => return Err(FlattenError::InvalidFunctionCallee(callee.clone())),
            }
        },
        Expression::MemberAccess { class, member } => {
            if let Expression::Variable(name) = &**class {
                if classes.contains_key(name) {
                    if let Some(index) = global_vars.iter().position(|(s, _)| s == &format!("{}.{}", name, member)) {
                        opcodes.push(OpCode::GetGlobal(index));
                        return Ok(global_vars.get(index).unwrap().1.clone());
                    }
                    else {
                        return Err(FlattenError::UndeclaredClassVar(format!("{}.{}", name, member)));
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
                    return Err(FlattenError::UndeclaredClassVar(format!("{}.{}", name, member)));
                }
            }

            Err(FlattenError::ExpressionIsNotClass(class.clone()))


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
    }
}

fn flatten_unary(operator: &UnaryOp, right: &Box<Expression>, opcodes: &mut Vec<OpCode>, global_vars: &Vec<(String, DataType)>, vars: &Vec<(String, DataType)>, funcs: &HashMap<String, (usize, DataType, Vec<DataType>)>, classes: &HashMap<String, ClassData>) -> Result<DataType, FlattenError> {
    let operand_type: DataType = flatten_expression(right, opcodes, global_vars, vars, funcs, classes)?;

    let d: DataType = match (operator, &operand_type) {
        (UnaryOp::LNot, DataType::Bool) => DataType::Bool,
        (UnaryOp::Negate, DataType::Int) => DataType::Int,
        (UnaryOp::Negate, DataType::Float) => DataType::Float,
        _ => return Err(FlattenError::UnexpectedUnaryOpOperands {
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
    };

    Ok(d)
}

fn normal_binary(left: &Box<Expression>, right: &Box<Expression>, opcode: OpCode, opcodes: &mut Vec<OpCode>, global_vars: &Vec<(String, DataType)>, vars: &Vec<(String, DataType)>, funcs: &HashMap<String, (usize, DataType, Vec<DataType>)>, classes: &HashMap<String, ClassData>) -> Result<DataType, FlattenError> {
    let l: DataType = flatten_expression(left, opcodes, global_vars, vars, funcs, classes)?;
    let r: DataType = flatten_expression(right, opcodes, global_vars, vars, funcs, classes)?;
    opcodes.push(opcode.clone());

    let operator: BinaryOp = BinaryOp::try_from(&opcode).map_err(|op| FlattenError::UnexpectedBinaryOpOpCode(op.clone()))?;

    get_binary_type(&l, &operator, &r, classes).map_err(|_| FlattenError::UnexpectedBinaryOpOperands { left: l, operator, right: r })
}

fn short_circuit_binary(left: &Box<Expression>, right: &Box<Expression>, jump_on: bool, opcodes: &mut Vec<OpCode>, global_vars: &Vec<(String, DataType)>, vars: &Vec<(String, DataType)>, funcs: &HashMap<String, (usize, DataType, Vec<DataType>)>, classes: &HashMap<String, ClassData>) -> Result<DataType, FlattenError> {
    let l: DataType = flatten_expression(left, opcodes, global_vars, vars, funcs, classes)?;

    if l != DataType::Bool {
        return Err(FlattenError::UnexpectedBinaryOpOperands { left: l, operator: if jump_on { BinaryOp::LOr } else { BinaryOp::LAnd }, right: DataType::Bool });
    }

    let start_index: usize = opcodes.len();
    opcodes.push(if jump_on { OpCode::JumpIfTrue { index: 0, pop: false } } else { OpCode::JumpIfFalse { index: 0, pop: false } });
    opcodes.push(OpCode::Pop(1));
    
    let r: DataType = flatten_expression(right, opcodes, global_vars, vars, funcs, classes)?;

    if r != DataType::Bool {
        return Err(FlattenError::UnexpectedBinaryOpOperands { left: l, operator: if jump_on { BinaryOp::LOr } else { BinaryOp::LAnd }, right: r });
    }

    *opcodes.get_mut(start_index).unwrap() = if jump_on { OpCode::JumpIfTrue { index: opcodes.len(), pop: false } } else { OpCode::JumpIfFalse { index: opcodes.len(), pop: false } };

    Ok(DataType::Bool)
}

fn flatten_function(name: &str, data: &FunctionData, opcodes: &mut Vec<OpCode>, global_vars: &mut Vec<(String, DataType)>, funcs: &mut HashMap<String, (usize, DataType, Vec<DataType>)>, classes: &mut HashMap<String, ClassData>, loop_starts: &mut Vec<(usize, usize, Vec<usize>)>, depth: usize, class_name: Option<&String>) -> Result<(), FlattenError> {
    let jump_index: usize = opcodes.len();
    opcodes.push(OpCode::Jump(0));

    let mut func_vars: Vec<(String, DataType)> = Vec::new();

    if let Some(n) = class_name {
        func_vars.push(("this".to_owned(), DataType::Instance(n.clone())));
    }

    for (data_type, name) in &data.parameters {
        if func_vars.iter().any(|(s, _)| s == name) || funcs.contains_key(name) || classes.contains_key(name) {
            return Err(FlattenError::Shadowing(name.clone()));
        }
        func_vars.push((name.clone(), data_type.clone()));
    }

    opcodes.push(OpCode::NewStack);

    let r: bool = flatten_block(&data.block, opcodes, global_vars, &mut func_vars, funcs, classes, loop_starts, depth, Some((name, &data.data_type)))?;

    if !r && !matches!(data.data_type, DataType::Null | DataType::Nullable(_)) {
        return Err(FlattenError::MissingReturnStatement(name.to_owned()));
    }

    flatten_expression(&Expression::Literal(LiteralType::Null), opcodes, global_vars, &mut func_vars, funcs, classes)?;
    opcodes.push(OpCode::PopStack);
    opcodes.push(OpCode::Return);

    *opcodes.get_mut(jump_index).unwrap() = OpCode::Jump(opcodes.len());

    Ok(())
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

fn is_compatible(expected: &DataType, received: &DataType, classes: &HashMap<String, ClassData>) -> bool {
    match (expected, received) {
        (DataType::Nullable(_), DataType::Null) => true,
        (DataType::Nullable(a), DataType::Nullable(b)) => is_compatible(a, b, classes),
        (DataType::Nullable(inner), actual) => is_compatible(inner, actual, classes),
        (DataType::Instance(p), DataType::Instance(c)) if is_subclass(c, p, classes) => true,
        (expected, received) => expected == received,
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

fn get_binary_type(left: &DataType, operator: &BinaryOp, right: &DataType, classes: &HashMap<String, ClassData>) -> Result<DataType, ()> {
    match (left, operator, right) {
        (DataType::Int, BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide, DataType::Int) => Ok(DataType::Int),
        (DataType::Float, BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide, DataType::Float) => Ok(DataType::Float),
        (DataType::String, BinaryOp::Add, DataType::String) => Ok(DataType::String),

        (DataType::Bool, BinaryOp::LAnd | BinaryOp::LOr, DataType::Bool) => Ok(DataType::Bool),

        (DataType::Int, BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual, DataType::Int) => Ok(DataType::Bool),
        (DataType::Float, BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual, DataType::Float) => Ok(DataType::Bool),
        (DataType::String, BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual, DataType::String) => Ok(DataType::Bool),

        (l, BinaryOp::Equal | BinaryOp::NotEqual, r) if is_compatible(l, r, classes) || is_compatible(r, l, classes) => Ok(DataType::Bool),

        _ => Err(()),
    }
}
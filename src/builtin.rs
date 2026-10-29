use crate::compiler::{DataType, OpCode};

pub enum Builtin {
    Input, ParseInt, ParseFloat,
}

impl Builtin {
    pub fn lookup(name: &str) -> Option<Builtin> {
        match name {
            "input" => Some(Builtin::Input),
            "parseInt" => Some(Builtin::ParseInt),
            "parseFloat" => Some(Builtin::ParseFloat),
            _ => None,
        }
    }
    pub fn signature(&self) -> (Vec<DataType>, DataType) {
        match self {
            Builtin::Input => (Vec::new(), DataType::Nullable(Box::new(DataType::String))),
            Builtin::ParseInt => (vec![DataType::Nullable(Box::new(DataType::String))], DataType::Nullable(Box::new(DataType::Int))),
            Builtin::ParseFloat => (vec![DataType::Nullable(Box::new(DataType::String))], DataType::Nullable(Box::new(DataType::Float))),
        }
    }
    pub fn call(&self, opcodes: &mut Vec<OpCode>) {
        match self {
            Builtin::Input => opcodes.push(OpCode::Input),
            Builtin::ParseInt => opcodes.push(OpCode::ParseInt),
            Builtin::ParseFloat => opcodes.push(OpCode::ParseFloat),
        };
    }
}
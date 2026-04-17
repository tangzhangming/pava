use crate::ast::Type;
use crate::error::CompileResult;

pub fn resolve_type(type_name: &str) -> Option<Type> {
    match type_name {
        "string" | "String" => Some(Type::String),
        "boolean" | "bool" => Some(Type::Boolean),
        "int8" => Some(Type::Int8),
        "int16" => Some(Type::Int16),
        "int32" => Some(Type::Int32),
        "int64" => Some(Type::Int64),
        "float32" => Some(Type::Float32),
        "float64" => Some(Type::Float64),
        "byte" => Some(Type::Int8),
        "int" => Some(Type::Int64),
        "float" => Some(Type::Float64),
        "void" => Some(Type::Void),
        _ => None,
    }
}

pub fn isAssignable(from: &Type, to: &Type) -> bool {
    if from.is_nullable() && !to.is_nullable() {
        return false;
    }
    match (from, to) {
        (Type::Int8, Type::Int8) => true,
        (Type::Int8, Type::Int16) => true,
        (Type::Int8, Type::Int32) => true,
        (Type::Int8, Type::Int64) => true,
        (Type::Int16, Type::Int16) => true,
        (Type::Int16, Type::Int32) => true,
        (Type::Int16, Type::Int64) => true,
        (Type::Int32, Type::Int32) => true,
        (Type::Int32, Type::Int64) => true,
        (Type::Int64, Type::Int64) => true,
        (Type::Float32, Type::Float32) => true,
        (Type::Float32, Type::Float64) => true,
        (Type::Float64, Type::Float64) => true,
        (Type::String, Type::String) => true,
        (Type::Boolean, Type::Boolean) => true,
        _ => false,
    }
}

pub fn get_widest_type(a: &Type, b: &Type) -> Type {
    match (a, b) {
        (Type::Int64, _) | (_, Type::Int64) => Type::Int64,
        (Type::Int32, _) | (_, Type::Int32) => Type::Int32,
        (Type::Int16, _) | (_, Type::Int16) => Type::Int16,
        (Type::Int8, _) | (_, Type::Int8) => Type::Int8,
        (Type::Float64, _) | (_, Type::Float64) => Type::Float64,
        (Type::Float32, _) | (_, Type::Float32) => Type::Float32,
        _ => a.clone(),
    }
}

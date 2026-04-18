use crate::ast::{Expr, Type};
use crate::error::{CompileError, CompileResult};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TypeContext {
    variables: HashMap<String, (Type, bool)>, // (type, is_initialized)
    current_class: String,
}

impl TypeContext {
    pub fn new(class_name: &str) -> Self {
        TypeContext {
            variables: HashMap::new(),
            current_class: class_name.to_string(),
        }
    }

    pub fn declare_var(&mut self, name: &str, ty: Type) {
        self.variables.insert(name.to_string(), (ty, false));
    }

    pub fn initialize_var(&mut self, name: &str) {
        if let Some((ty, _)) = self.variables.get(name) {
            self.variables.insert(name.to_string(), (ty.clone(), true));
        }
    }

    pub fn is_initialized(&self, name: &str) -> bool {
        self.variables
            .get(name)
            .map(|(_, init)| *init)
            .unwrap_or(true)
    }

    pub fn get_var_type(&self, name: &str) -> Option<&Type> {
        self.variables.get(name).map(|(ty, _)| ty)
    }

    pub fn check_null_assignment(&self, target_type: &Type) -> CompileResult<()> {
        if !target_type.is_nullable() {
            return Err(CompileError::TypeError(format!(
                "Cannot assign null to non-nullable type {:?}",
                target_type
            )));
        }
        Ok(())
    }

    pub fn check_non_null_initialization(&self, name: &str, ty: &Type) -> CompileResult<()> {
        if !ty.is_nullable() && !self.is_initialized(name) {
            return Err(CompileError::TypeError(format!(
                "Non-nullable variable '{}' must be initialized before use",
                name
            )));
        }
        Ok(())
    }

    pub fn check_condition_type(&self, expr: &Expr) -> CompileResult<()> {
        let expr_type = infer_expr_type(expr);
        if !expr_type.can_be_condition() {
            return Err(CompileError::TypeError(format!(
                "Condition expression must be boolean, got {:?}",
                expr_type
            )));
        }
        Ok(())
    }
}

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

pub fn is_assignable(from: &Type, to: &Type) -> bool {
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
        (Type::Nullable(inner_from), Type::Nullable(inner_to)) => {
            is_assignable(inner_from, inner_to)
        }
        (Type::Nullable(inner), to) if to.is_nullable() => is_assignable(inner, to),
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

pub fn infer_expr_type(expr: &Expr) -> Type {
    match expr {
        Expr::IntLiteral(_) => Type::Int32,
        Expr::FloatLiteral(_) => Type::Float64,
        Expr::StringLiteral(_) => Type::String,
        Expr::BoolLiteral(_) => Type::Boolean,
        Expr::NullLiteral => Type::Nullable(Box::new(Type::String)),
        Expr::BinaryOp(op, left, right) => {
            let left_type = infer_expr_type(left);
            let right_type = infer_expr_type(right);
            match op {
                crate::ast::BinaryOp::Add
                | crate::ast::BinaryOp::Sub
                | crate::ast::BinaryOp::Mul
                | crate::ast::BinaryOp::Div
                | crate::ast::BinaryOp::Mod => {
                    if left_type == Type::String || right_type == Type::String {
                        Type::String
                    } else {
                        get_widest_type(&left_type, &right_type)
                    }
                }
                crate::ast::BinaryOp::Lt
                | crate::ast::BinaryOp::Le
                | crate::ast::BinaryOp::Gt
                | crate::ast::BinaryOp::Ge
                | crate::ast::BinaryOp::Eq
                | crate::ast::BinaryOp::Ne => Type::Boolean,
                crate::ast::BinaryOp::And | crate::ast::BinaryOp::Or => Type::Boolean,
                crate::ast::BinaryOp::Assign => infer_expr_type(right),
                crate::ast::BinaryOp::AddAssign
                | crate::ast::BinaryOp::SubAssign
                | crate::ast::BinaryOp::MulAssign
                | crate::ast::BinaryOp::DivAssign
                | crate::ast::BinaryOp::ModAssign => infer_expr_type(left),
            }
        }
        Expr::UnaryOp(op, inner) => match op {
            crate::ast::UnaryOp::Neg => infer_expr_type(inner),
            crate::ast::UnaryOp::Not => Type::Boolean,
            crate::ast::UnaryOp::PreIncrement
            | crate::ast::UnaryOp::PostIncrement
            | crate::ast::UnaryOp::PreDecrement
            | crate::ast::UnaryOp::PostDecrement => infer_expr_type(inner),
        },
        Expr::InstanceOf(_, _) => Type::Boolean,
        Expr::Cast(_, target_type) => target_type.clone(),
        Expr::NewObject(class_name, _) => Type::Object(class_name.clone()),
        _ => Type::String,
    }
}

pub fn check_assignability(from: &Type, to: &Type) -> CompileResult<()> {
    if !is_assignable(from, to) {
        return Err(CompileError::TypeError(format!(
            "Cannot assign {:?} to {:?}",
            from, to
        )));
    }
    Ok(())
}

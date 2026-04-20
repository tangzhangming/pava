impl Expr {
    pub fn result_type(&self) -> Result<Type, String> {
        match self {
            Expr::IntLiteral(_) => Ok(Type::Int32),
            Expr::FloatLiteral(_) => Ok(Type::Float64),
            Expr::StringLiteral(_) => Ok(Type::String),
            Expr::BoolLiteral(_) => Ok(Type::Boolean),
            Expr::NullLiteral => Ok(Type::Nullable(Box::new(Type::String))),
            Expr::Variable(_) => Ok(Type::Int32),
            Expr::BinaryOp(op, _, _) => match op {
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                    Ok(Type::Int32)
                }
                BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => Ok(Type::Boolean),
                BinaryOp::Eq | BinaryOp::Ne => Ok(Type::Boolean),
                BinaryOp::And | BinaryOp::Or => Ok(Type::Boolean),
                BinaryOp::Assign
                | BinaryOp::AddAssign
                | BinaryOp::SubAssign
                | BinaryOp::MulAssign
                | BinaryOp::DivAssign
                | BinaryOp::ModAssign => Ok(Type::Int32),
            },
            Expr::UnaryOp(op, _) => match op {
                UnaryOp::Neg => Ok(Type::Int32),
                UnaryOp::Not => Ok(Type::Boolean),
                UnaryOp::PreIncrement
                | UnaryOp::PostIncrement
                | UnaryOp::PreDecrement
                | UnaryOp::PostDecrement => Ok(Type::Int32),
            },
            Expr::InstanceOf(_, _) => Ok(Type::Boolean),
            _ => Ok(Type::String),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Import {
    pub path: String,          // 如 "java/util/HashMap"
    pub is_star: bool,         // 是否是 import java.util.*;
    pub alias: Option<String>, // 别名 (可选)
}

#[derive(Debug, Clone)]
pub struct CompilationUnit {
    pub package: Option<String>, // 如 "com/example"
    pub imports: Vec<Import>,
    pub classes: Vec<Class>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Nothing,
    String,
    Boolean,
    Int8,
    Int16,
    Int32,
    Int64,
    Float32,
    Float64,
    Void,
    Nullable(Box<Type>),
    Array(Box<Type>),
    Object(String),
}

impl Type {
    pub fn to_jvm_descriptor(&self) -> String {
        match self {
            Type::Nothing => "Ljava/lang/Object;".to_string(),
            Type::String => "Ljava/lang/String;".to_string(),
            Type::Boolean => "Z".to_string(),
            Type::Int8 => "B".to_string(),
            Type::Int16 => "S".to_string(),
            Type::Int32 => "I".to_string(),
            Type::Int64 => "J".to_string(),
            Type::Float32 => "F".to_string(),
            Type::Float64 => "D".to_string(),
            Type::Void => "V".to_string(),
            Type::Nullable(inner) => inner.to_jvm_descriptor(),
            Type::Array(inner) => format!("[{}", inner.to_jvm_descriptor()),
            Type::Object(name) => format!("L{};", name),
        }
    }

    pub fn is_nullable(&self) -> bool {
        matches!(self, Type::Nullable(_))
    }

    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            Type::Boolean
                | Type::Int8
                | Type::Int16
                | Type::Int32
                | Type::Int64
                | Type::Float32
                | Type::Float64
        )
    }

    pub fn can_be_condition(&self) -> bool {
        matches!(self, Type::Boolean)
    }
}

#[derive(Debug, Clone)]
pub struct ClassField {
    pub name: String,
    pub field_type: Type,
    pub is_nullable: bool,
    pub is_static: bool,
    pub is_public: bool,
    pub is_private: bool,
    pub is_protected: bool,
    pub is_internal: bool, // 模块内可见
    pub is_final: bool,
    pub initializer: Option<Expr>,
    pub property_hooks: Vec<PropertyHook>,
}

#[derive(Debug, Clone)]
pub struct ClassMethod {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub promoted_params: Vec<PromotedParam>,
    pub return_type: Type,
    pub body: Vec<Stmt>,
    pub is_static: bool,
    pub is_public: bool,
    pub is_private: bool,
    pub is_protected: bool,
    pub is_internal: bool, // 模块内可见
    pub is_abstract: bool,
    pub is_default: bool,
}

#[derive(Debug, Clone)]
pub struct PromotedParam {
    pub name: String,
    pub param_type: Type,
    pub is_public: bool,
    pub is_private: bool,
    pub is_protected: bool,
    pub is_internal: bool, // 模块内可见
}

#[derive(Debug, Clone)]
pub struct ClassConst {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
    pub full_name: String, // 包含包名的完整类名，如 "com/example/MyClass"
    pub extends: Option<String>,
    pub implements: Vec<String>,
    pub is_abstract: bool,
    pub is_final: bool,
    pub is_open: bool,
    pub is_interface: bool,
    pub is_enum: bool,
    pub enum_backed_type: Option<Type>,
    pub fields: Vec<ClassField>,
    pub methods: Vec<ClassMethod>,
    pub constants: Vec<ClassConst>,
    pub constructor: Option<ClassMethod>,
    pub enum_values: Vec<EnumValue>,
}

impl Class {
    pub fn with_package(&self, package: &Option<String>) -> Self {
        let mut new_class = self.clone();
        new_class.full_name = match package {
            Some(pkg) => format!("{}/{}", pkg, self.name),
            None => self.name.clone(),
        };
        new_class
    }
}

impl Default for Class {
    fn default() -> Self {
        Self {
            name: String::new(),
            full_name: String::new(),
            extends: None,
            implements: Vec::new(),
            is_abstract: false,
            is_final: false,
            is_open: false,
            is_interface: false,
            is_enum: false,
            enum_backed_type: None,
            fields: Vec::new(),
            methods: Vec::new(),
            constants: Vec::new(),
            constructor: None,
            enum_values: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnumValue {
    pub name: String,
    pub value: i64,
}

#[derive(Debug, Clone)]
pub enum Expr {
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    InterpolatedString(Vec<Expr>), // "hi {$name}" -> [StringLiteral("hi "), Variable("name")]
    BoolLiteral(bool),
    NullLiteral,
    Variable(String),
    BinaryOp(BinaryOp, Box<Expr>, Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),
    Cast(Box<Expr>, Type),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>), // $a ? $b : $c
    Elvis(Box<Expr>, Box<Expr>),              // $a ?: $c (when $a is truthy, return $a; else $c)
    NullCoalescing(Box<Expr>, Box<Expr>),     // $a ?? $c (when $a is null, return $c; else $a)
    InstanceOf(Box<Expr>, String),            // $obj instanceof ClassName
    Throw(Box<Expr>),                         // throw $expr
    NewObject(String, Vec<Expr>),
    MethodCall(Box<Expr>, String, Vec<Expr>),
    StaticCall(String, String, Vec<Expr>),
    FieldAccess(Box<Expr>, String),
    StaticFieldAccess(String, String),
    Closure(ClosureExpr),
    ClosureCall(Box<Expr>, Vec<Expr>),
}

/// 捕获变量定义
#[derive(Debug, Clone)]
pub struct CaptureVar {
    pub name: String,
    pub is_reference: bool, // true for &$var, false for $var
}

/// 闭包表达式
#[derive(Debug, Clone)]
pub struct ClosureExpr {
    pub params: Vec<(String, Type)>, // 参数名和类型
    pub return_type: Type,           // 返回类型
    pub captures: Vec<CaptureVar>,   // use 子句捕获的变量
    pub body: Vec<Stmt>,             // 闭包体
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Assign,
    AddAssign, // +=
    SubAssign, // -=
    MulAssign, // *=
    DivAssign, // /=
    ModAssign, // %=
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neg,
    Not,
    PreIncrement,  // ++$i
    PostIncrement, // $i++
    PreDecrement,  // --$i
    PostDecrement, // $i--
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    Return(Option<Expr>),
    If(Expr, Vec<Stmt>, Vec<(Expr, Vec<Stmt>)>, Option<Vec<Stmt>>),
    While(Expr, Vec<Stmt>),
    For(Box<Stmt>, Expr, Box<Stmt>, Vec<Stmt>),
    Assign(String, Expr),
    TypedAssign(String, Type, Expr),
    Break,
    Continue,
    Print(Expr),
    Println(Expr),
    Printf(Expr, Vec<Expr>),
    Block(Vec<Stmt>),
    TryCatch {
        try_body: Vec<Stmt>,
        catch_clauses: Vec<CatchClause>,
        finally_body: Option<Vec<Stmt>>,
    },
}

#[derive(Debug, Clone)]
pub struct CatchClause {
    pub exception_types: Vec<String>,
    pub var_name: String,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct PropertyHook {
    pub hook_type: PropertyHookType,
    pub body: Vec<Stmt>,
    pub param_type: Option<Type>, // 用于setter的参数类型
    pub param_name: Option<String>, // 用于setter的参数名
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyHookType {
    Get,
    Set,
}

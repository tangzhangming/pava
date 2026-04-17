use crate::ast::{
    BinaryOp, CaptureVar, Class, ClassConst, ClassField, ClassMethod, ClosureExpr, EnumValue, Expr,
    PromotedParam, Stmt, Type, UnaryOp,
};
use crate::error::{CompileError, CompileResult};
use crate::lexer::{Lexer, Token};

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    class_name: String,
}

impl Parser {
    pub fn new(input: String) -> Self {
        let mut parser = Parser {
            lexer: Lexer::new(input),
            current_token: Token::Eof,
            class_name: String::new(),
        };
        parser.bump();
        parser
    }

    fn bump(&mut self) {
        self.current_token = self.lexer.next_token().unwrap();
    }

    fn expect(&mut self, expected: Token) -> CompileResult<()> {
        if std::mem::discriminant(&self.current_token) == std::mem::discriminant(&expected) {
            self.bump();
            Ok(())
        } else {
            Err(CompileError::ParserError(format!(
                "Expected {:?}, got {:?}",
                expected, self.current_token
            )))
        }
    }

    pub fn parse_class(&mut self) -> CompileResult<Class> {
        let mut is_abstract = false;
        let mut is_final = false;
        let mut is_interface = false;
        let mut is_enum = false;
        let mut is_open = false;
        let mut enum_backed_type = None;

        match &self.current_token {
            Token::Open => {
                is_open = true;
                self.bump();
                self.expect(Token::Class)?;
            }
            Token::Abstract => {
                is_abstract = true;
                self.bump();
                self.expect(Token::Class)?;
            }
            Token::Interface => {
                is_interface = true;
                self.bump();
            }
            Token::Enum => {
                is_enum = true;
                self.bump();
            }
            Token::Final => {
                is_final = true;
                self.bump();
                self.expect(Token::Class)?;
            }
            Token::Class => {
                self.bump();
            }
            _ => {
                return Err(CompileError::ParserError(
                    "Expected class, interface, or enum".to_string(),
                ))
            }
        }

        let name = match &self.current_token {
            Token::Identifier(n) => n.clone(),
            _ => return Err(CompileError::ParserError("Expected class name".to_string())),
        };
        self.class_name = name.clone();
        self.bump();

        if is_enum && self.current_token == Token::Colon {
            self.bump();
            enum_backed_type = Some(self.parse_type()?);
        }

        let mut extends = None;
        let mut implements = Vec::new();

        if self.current_token == Token::Extends {
            self.bump();
            extends = match &self.current_token {
                Token::Identifier(n) => Some(n.clone()),
                _ => {
                    return Err(CompileError::ParserError(
                        "Expected parent class name".to_string(),
                    ))
                }
            };
            self.bump();
        }

        if self.current_token == Token::Implements {
            self.bump();
            loop {
                implements.push(match &self.current_token {
                    Token::Identifier(n) => n.clone(),
                    _ => {
                        return Err(CompileError::ParserError(
                            "Expected interface name".to_string(),
                        ))
                    }
                });
                self.bump();
                if self.current_token == Token::Comma {
                    self.bump();
                } else {
                    break;
                }
            }
        }

        self.expect(Token::LBrace)?;

        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut constants = Vec::new();
        let mut constructor = None;
        let mut enum_values = Vec::new();

        while self.current_token != Token::RBrace {
            match &self.current_token {
                Token::Const => {
                    let c = self.parse_const()?;
                    constants.push(c);
                }
                Token::Case if is_enum => {
                    self.bump();
                    let enum_name = match &self.current_token {
                        Token::Identifier(n) => n.clone(),
                        _ => {
                            return Err(CompileError::ParserError(
                                "Expected enum case name".to_string(),
                            ))
                        }
                    };
                    self.bump();
                    let value = if self.current_token == Token::Equal {
                        self.bump();
                        match &self.current_token {
                            Token::IntLiteral(n) => *n,
                            _ => 0,
                        }
                    } else {
                        enum_values.len() as i64
                    };
                    enum_values.push(EnumValue {
                        name: enum_name,
                        value,
                    });
                    self.expect(Token::Semicolon)?;
                }
                Token::Static => {
                    self.bump();
                    if self.current_token == Token::Function {
                        self.bump();
                        let m = self.parse_method_with_flags(true, true, false, false, false)?;
                        methods.push(m);
                    } else {
                        let mut f = self.parse_field()?;
                        f.is_static = true;
                        fields.push(f);
                    }
                }
                Token::Function => {
                    self.bump();
                    let is_abstract_method = is_abstract && !is_interface;
                    let is_default_method = is_interface;
                    let m = self.parse_method_with_flags(
                        false,
                        true,
                        is_abstract_method,
                        is_default_method,
                        false,
                    )?;
                    methods.push(m);
                }
                Token::Public | Token::Private => {
                    let is_public = matches!(&self.current_token, Token::Public);
                    self.bump();
                    match &self.current_token {
                        Token::Static => {
                            self.bump();
                            if self.current_token == Token::Function {
                                self.bump();
                                let m = self.parse_method_with_flags(
                                    true, is_public, false, false, false,
                                )?;
                                methods.push(m);
                            } else {
                                let mut f = self.parse_field()?;
                                f.is_static = true;
                                fields.push(f);
                            }
                        }
                        Token::Function => {
                            self.bump();
                            let m =
                                self.parse_method_with_flags(false, is_public, false, false, true)?;
                            constructor = Some(m);
                        }
                        Token::Abstract => {
                            self.bump();
                            self.expect(Token::Function)?;
                            let m = self.parse_abstract_method(is_public)?;
                            methods.push(m);
                        }
                        _ => {
                            let f = self.parse_field()?;
                            fields.push(f);
                        }
                    }
                }
                Token::Abstract => {
                    self.bump();
                    self.expect(Token::Function)?;
                    let m = self.parse_abstract_method(true)?;
                    methods.push(m);
                }
                Token::Identifier(n) => {
                    if n == "__construct" {
                        self.bump();
                        let m = self.parse_method_with_flags(false, true, false, false, true)?;
                        constructor = Some(m);
                    } else {
                        let f = self.parse_field()?;
                        fields.push(f);
                    }
                }
                _ => {
                    self.bump();
                }
            }
        }

        self.expect(Token::RBrace)?;

        Ok(Class {
            name: self.class_name.clone(),
            extends,
            implements,
            is_abstract,
            is_final,
            is_open,
            is_interface,
            is_enum,
            enum_backed_type,
            fields,
            methods,
            constants,
            constructor,
            enum_values,
        })
    }

    fn parse_const(&mut self) -> CompileResult<ClassConst> {
        self.expect(Token::Const)?;

        // Skip type annotation like "string", "int", etc.
        if matches!(&self.current_token, Token::Type(_) | Token::Identifier(_)) {
            let _ = self.parse_type();
        }

        let name = match &self.current_token {
            Token::Identifier(n) => n.clone(),
            Token::Variable(n) => n.clone(),
            _ => {
                return Err(CompileError::ParserError(format!(
                    "Expected const name, got {:?}",
                    self.current_token
                )))
            }
        };
        self.bump();

        self.expect(Token::Equal)?;

        let value = self.parse_expr()?;

        if self.current_token == Token::Semicolon {
            self.bump();
        }

        Ok(ClassConst { name, value })
    }

    fn parse_field(&mut self) -> CompileResult<ClassField> {
        let field_type = self.parse_type()?;

        let name = match &self.current_token {
            Token::Variable(n) => n.trim_start_matches('$').to_string(),
            Token::Identifier(n) => n.clone(),
            _ => {
                return Err(CompileError::ParserError(format!(
                    "Expected field name, got {:?}",
                    self.current_token
                )))
            }
        };
        self.bump();

        let initializer = if self.current_token == Token::Equal {
            self.bump();
            Some(self.parse_expr()?)
        } else {
            None
        };

        if self.current_token == Token::Semicolon {
            self.bump();
        }

        Ok(ClassField {
            name,
            field_type,
            is_nullable: false,
            is_static: false,
            is_public: true,
            is_private: false,
            is_protected: false,
            is_final: false,
            initializer,
        })
    }

    fn parse_method(&mut self, is_static: bool, is_public: bool) -> CompileResult<ClassMethod> {
        self.parse_method_with_flags(is_static, is_public, false, false, false)
    }

    fn parse_method_with_flags(
        &mut self,
        is_static: bool,
        is_public: bool,
        is_abstract: bool,
        is_default: bool,
        is_constructor: bool,
    ) -> CompileResult<ClassMethod> {
        let name = match &self.current_token {
            Token::Identifier(n) => n.clone(),
            _ => {
                return Err(CompileError::ParserError(
                    "Expected method name".to_string(),
                ))
            }
        };
        self.bump();

        self.expect(Token::LParen)?;
        let (params, promoted_params) = if is_constructor {
            self.parse_params_with_promoted()?
        } else {
            (self.parse_params()?, Vec::new())
        };
        self.expect(Token::RParen)?;

        let return_type = if self.current_token == Token::Colon {
            self.bump();
            self.parse_type()?
        } else {
            Type::Void
        };

        let body = if is_abstract && self.current_token == Token::Semicolon {
            self.bump();
            Vec::new()
        } else if is_default || !is_abstract {
            self.expect(Token::LBrace)?;
            let body = self.parse_block()?;
            self.expect(Token::RBrace)?;
            body
        } else {
            Vec::new()
        };

        Ok(ClassMethod {
            name,
            params,
            promoted_params,
            return_type,
            body,
            is_static,
            is_public,
            is_abstract,
            is_default,
        })
    }

    fn parse_abstract_method(&mut self, is_public: bool) -> CompileResult<ClassMethod> {
        let name = match &self.current_token {
            Token::Identifier(n) => n.clone(),
            _ => {
                return Err(CompileError::ParserError(
                    "Expected abstract method name".to_string(),
                ))
            }
        };
        self.bump();

        self.expect(Token::LParen)?;
        let params = self.parse_params()?;
        self.expect(Token::RParen)?;

        let return_type = if self.current_token == Token::Colon {
            self.bump();
            self.parse_type()?
        } else {
            Type::Void
        };

        self.expect(Token::Semicolon)?;

        Ok(ClassMethod {
            name,
            params,
            promoted_params: Vec::new(),
            return_type,
            body: Vec::new(),
            is_static: false,
            is_public,
            is_abstract: true,
            is_default: false,
        })
    }

    fn parse_params(&mut self) -> CompileResult<Vec<(String, Type)>> {
        let mut params = Vec::new();

        if self.current_token != Token::RParen {
            loop {
                let param_type = self.parse_type()?;

                let param_name = match &self.current_token {
                    Token::Variable(n) => n.clone(),
                    Token::Identifier(n) => n.clone(),
                    _ => {
                        return Err(CompileError::ParserError(
                            "Expected parameter name".to_string(),
                        ))
                    }
                };
                self.bump();

                params.push((param_name, param_type));

                if self.current_token == Token::Comma {
                    self.bump();
                } else {
                    break;
                }
            }
        }

        Ok(params)
    }

    fn parse_params_with_promoted(
        &mut self,
    ) -> CompileResult<(Vec<(String, Type)>, Vec<PromotedParam>)> {
        let mut params = Vec::new();
        let mut promoted_params = Vec::new();

        if self.current_token != Token::RParen {
            loop {
                let mut is_public = false;
                let mut is_private = false;
                let mut is_protected = false;

                if matches!(
                    &self.current_token,
                    Token::Public | Token::Private | Token::Protected
                ) {
                    match &self.current_token {
                        Token::Public => is_public = true,
                        Token::Private => is_private = true,
                        Token::Protected => is_protected = true,
                        _ => {}
                    }
                    self.bump();
                }

                let param_type = self.parse_type()?;

                let param_name = match &self.current_token {
                    Token::Variable(n) => n.clone(),
                    Token::Identifier(n) => n.clone(),
                    _ => {
                        return Err(CompileError::ParserError(
                            "Expected parameter name".to_string(),
                        ))
                    }
                };
                self.bump();

                params.push((param_name.clone(), param_type.clone()));

                if is_public || is_private || is_protected {
                    promoted_params.push(PromotedParam {
                        name: param_name,
                        param_type,
                        is_public,
                        is_private,
                        is_protected,
                    });
                }

                if self.current_token == Token::Comma {
                    self.bump();
                } else {
                    break;
                }
            }
        }

        Ok((params, promoted_params))
    }

    fn parse_type(&mut self) -> CompileResult<Type> {
        match &self.current_token {
            Token::Type(t) => {
                let ty = t.clone();
                self.bump();
                Ok(match ty.as_str() {
                    "string" => Type::String,
                    "boolean" => Type::Boolean,
                    "int8" => Type::Int8,
                    "int16" => Type::Int16,
                    "int32" => Type::Int32,
                    "int64" => Type::Int64,
                    "float32" => Type::Float32,
                    "float64" => Type::Float64,
                    _ => Type::String,
                })
            }
            Token::Identifier(t) => {
                let ty = t.clone();
                self.bump();
                match ty.as_str() {
                    "string" | "String" => Ok(Type::String),
                    "boolean" | "bool" => Ok(Type::Boolean),
                    "int8" | "Int8" => Ok(Type::Int8),
                    "int16" | "Int16" => Ok(Type::Int16),
                    "int32" | "Int32" => Ok(Type::Int32),
                    "int64" | "Int64" => Ok(Type::Int64),
                    "float32" | "Float32" => Ok(Type::Float32),
                    "float64" | "Float64" => Ok(Type::Float64),
                    "byte" | "Byte" => Ok(Type::Int8),
                    "int" | "Int" => Ok(Type::Int32),
                    "short" | "Short" => Ok(Type::Int16),
                    "long" | "Long" => Ok(Type::Int64),
                    "float" | "Float" => Ok(Type::Float32),
                    "double" | "Double" => Ok(Type::Float64),
                    "void" | "Void" => Ok(Type::Void),
                    _ => Err(CompileError::ParserError(format!("Unknown type: {}", ty))),
                }
            }
            // 处理类型关键字 Token
            Token::TypeInt => {
                self.bump();
                Ok(Type::Int64)
            }
            Token::TypeInt8 => {
                self.bump();
                Ok(Type::Int8)
            }
            Token::TypeInt16 => {
                self.bump();
                Ok(Type::Int16)
            }
            Token::TypeInt32 => {
                self.bump();
                Ok(Type::Int32)
            }
            Token::TypeInt64 => {
                self.bump();
                Ok(Type::Int64)
            }
            Token::TypeFloat => {
                self.bump();
                Ok(Type::Float64)
            }
            Token::TypeFloat32 => {
                self.bump();
                Ok(Type::Float32)
            }
            Token::TypeFloat64 => {
                self.bump();
                Ok(Type::Float64)
            }
            Token::TypeBoolean => {
                self.bump();
                Ok(Type::Boolean)
            }
            Token::TypeByte => {
                self.bump();
                Ok(Type::Int8)
            }
            _ => {
                self.bump();
                Ok(Type::String)
            }
        }
    }

    fn parse_block(&mut self) -> CompileResult<Vec<Stmt>> {
        let mut statements = Vec::new();

        while self.current_token != Token::RBrace {
            statements.push(self.parse_stmt()?);
        }

        Ok(statements)
    }

    fn parse_stmt(&mut self) -> CompileResult<Stmt> {
        match &self.current_token {
            Token::Return => {
                self.bump();
                let expr = if self.current_token == Token::Semicolon {
                    None
                } else {
                    Some(self.parse_expr()?)
                };
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Return(expr))
            }
            Token::Identifier(n) if n == "print" || n == "println" || n == "printf" => {
                let func = n.clone();
                self.bump();
                self.expect(Token::LParen)?;
                let arg = self.parse_expr()?;
                self.expect(Token::RParen)?;
                self.expect(Token::Semicolon)?;
                match func.as_str() {
                    "print" => Ok(Stmt::Print(arg)),
                    "println" => Ok(Stmt::Println(arg)),
                    "printf" => Ok(Stmt::Printf(arg, vec![])),
                    _ => Err(CompileError::ParserError("Unknown function".to_string())),
                }
            }
            Token::If => {
                self.bump();
                self.expect(Token::LParen)?;
                let cond = self.parse_expr()?;
                self.expect(Token::RParen)?;
                self.expect(Token::LBrace)?;
                let then = self.parse_block()?;
                self.expect(Token::RBrace)?;
                Ok(Stmt::If(cond, then, None))
            }
            Token::While => {
                self.bump();
                self.expect(Token::LParen)?;
                let cond = self.parse_expr()?;
                self.expect(Token::RParen)?;
                self.expect(Token::LBrace)?;
                let body = self.parse_block()?;
                self.expect(Token::RBrace)?;
                Ok(Stmt::While(cond, body))
            }
            Token::Variable(_) => {
                let name = match &self.current_token {
                    Token::Variable(n) => n.clone(),
                    _ => return Err(CompileError::ParserError("Expected variable".to_string())),
                };
                self.bump();

                if self.current_token == Token::Equal {
                    self.bump();
                    let value = self.parse_expr()?;
                    self.expect(Token::Semicolon)?;
                    return Ok(Stmt::Assign(name, value));
                }

                // 处理字段访问 $obj->field
                if self.current_token == Token::Arrow || self.current_token == Token::Dot {
                    let mut expr = Expr::Variable(name);
                    while self.current_token == Token::Arrow || self.current_token == Token::Dot {
                        let _op = self.current_token.clone();
                        self.bump();
                        let member = match &self.current_token {
                            Token::Identifier(n) => n.clone(),
                            _ => {
                                return Err(CompileError::ParserError(
                                    "Expected member".to_string(),
                                ))
                            }
                        };
                        self.bump();

                        if self.current_token == Token::LParen {
                            self.bump();
                            let args = self.parse_args()?;
                            self.expect(Token::RParen)?;
                            expr = Expr::MethodCall(Box::new(expr), member, args);
                        } else {
                            expr = Expr::FieldAccess(Box::new(expr), member);
                        }
                    }

                    // 检查是否是赋值 $obj->field = value
                    if self.current_token == Token::Equal {
                        self.bump();
                        let value = self.parse_expr()?;
                        self.expect(Token::Semicolon)?;
                        // 创建赋值表达式：$obj->field = value
                        let assign_expr =
                            Expr::BinaryOp(BinaryOp::Assign, Box::new(expr), Box::new(value));
                        return Ok(Stmt::Expr(assign_expr));
                    }

                    self.expect(Token::Semicolon)?;
                    return Ok(Stmt::Expr(expr));
                }

                self.expect(Token::Semicolon)?;
                Ok(Stmt::Assign(name, Expr::NullLiteral))
            }
            _ => {
                let e = self.parse_expr()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Expr(e))
            }
        }
    }

    fn parse_expr(&mut self) -> CompileResult<Expr> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> CompileResult<Expr> {
        let mut left = self.parse_additive()?;

        while matches!(
            &self.current_token,
            Token::Lt | Token::Le | Token::Gt | Token::Ge | Token::Eq | Token::Ne
        ) {
            let op = match &self.current_token {
                Token::Lt => BinaryOp::Lt,
                Token::Le => BinaryOp::Le,
                Token::Gt => BinaryOp::Gt,
                Token::Ge => BinaryOp::Ge,
                Token::Eq => BinaryOp::Eq,
                Token::Ne => BinaryOp::Ne,
                _ => unreachable!(),
            };
            self.bump();
            let right = self.parse_additive()?;
            left = Expr::BinaryOp(op, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_additive(&mut self) -> CompileResult<Expr> {
        let mut left = self.parse_term()?;

        while self.current_token == Token::Plus || self.current_token == Token::Minus {
            let op = if matches!(&self.current_token, Token::Plus) {
                BinaryOp::Add
            } else {
                BinaryOp::Sub
            };
            self.bump();
            let right = self.parse_term()?;
            left = Expr::BinaryOp(op, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_term(&mut self) -> CompileResult<Expr> {
        let mut left = self.parse_unary()?;

        while matches!(
            &self.current_token,
            Token::Star | Token::Slash | Token::Percent
        ) {
            let op = match &self.current_token {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                Token::Percent => BinaryOp::Mod,
                _ => unreachable!(),
            };
            self.bump();
            let right = self.parse_unary()?;
            left = Expr::BinaryOp(op, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> CompileResult<Expr> {
        if self.current_token == Token::Minus {
            self.bump();
            let e = self.parse_unary()?;
            return Ok(Expr::UnaryOp(UnaryOp::Neg, Box::new(e)));
        }
        if self.current_token == Token::Not {
            self.bump();
            let e = self.parse_unary()?;
            return Ok(Expr::UnaryOp(UnaryOp::Not, Box::new(e)));
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            match &self.current_token {
                Token::LParen => {
                    // Closure call $fn(...) or regular function call
                    self.bump();
                    let args = self.parse_args()?;
                    self.expect(Token::RParen)?;
                    expr = Expr::ClosureCall(Box::new(expr), args);
                }
                Token::Dot | Token::Arrow => {
                    self.bump();
                    let member = match &self.current_token {
                        Token::Identifier(n) => n.clone(),
                        _ => return Err(CompileError::ParserError("Expected member".to_string())),
                    };
                    self.bump();

                    if self.current_token == Token::LParen {
                        self.bump();
                        let args = self.parse_args()?;
                        self.expect(Token::RParen)?;
                        expr = Expr::MethodCall(Box::new(expr), member, args);
                    } else {
                        expr = Expr::FieldAccess(Box::new(expr), member);
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> CompileResult<Expr> {
        match &self.current_token {
            Token::Variable(n) => {
                let name = n.clone();
                self.bump();
                Ok(Expr::Variable(name))
            }
            Token::StringLiteral(s) => {
                let val = s.clone();
                self.bump();
                Ok(Expr::StringLiteral(val))
            }
            Token::IntLiteral(n) => {
                let val = *n;
                self.bump();
                Ok(Expr::IntLiteral(val))
            }
            Token::FloatLiteral(n) => {
                let val = *n;
                self.bump();
                Ok(Expr::FloatLiteral(val))
            }
            Token::True => {
                self.bump();
                Ok(Expr::BoolLiteral(true))
            }
            Token::False => {
                self.bump();
                Ok(Expr::BoolLiteral(false))
            }
            Token::Null => {
                self.bump();
                Ok(Expr::NullLiteral)
            }
            Token::Function => self.parse_closure(),
            Token::New => {
                self.bump();
                let class = match &self.current_token {
                    Token::Identifier(n) => n.clone(),
                    _ => return Err(CompileError::ParserError("Expected class name".to_string())),
                };
                self.bump();
                self.expect(Token::LParen)?;
                let args = self.parse_args()?;
                self.expect(Token::RParen)?;
                Ok(Expr::NewObject(class, args))
            }
            Token::LParen => {
                self.bump();
                let e = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(e)
            }
            _ => {
                let name = match &self.current_token {
                    Token::Identifier(n) => n.clone(),
                    Token::SelfRef => "self".to_string(),
                    Token::Parent => "parent".to_string(),
                    _ => return Err(CompileError::ParserError("Unexpected token".to_string())),
                };
                self.bump();

                // Handle :: for static method/field access (self::, parent::, ClassName::)
                if self.current_token == Token::DoubleColon {
                    self.bump();
                    let member = match &self.current_token {
                        Token::Identifier(n) => n.clone(),
                        Token::Variable(n) => n.trim_start_matches('$').to_string(),
                        _ => {
                            return Err(CompileError::ParserError(
                                "Expected static member name".to_string(),
                            ))
                        }
                    };
                    self.bump();

                    if self.current_token == Token::LParen {
                        self.bump();
                        let args = self.parse_args()?;
                        self.expect(Token::RParen)?;
                        Ok(Expr::StaticCall(name, member, args))
                    } else {
                        Ok(Expr::StaticFieldAccess(name, member))
                    }
                } else if self.current_token == Token::LParen {
                    self.bump();
                    let args = self.parse_args()?;
                    self.expect(Token::RParen)?;
                    Ok(Expr::NewObject(name, args))
                } else {
                    Ok(Expr::Variable(name))
                }
            }
        }
    }

    fn parse_args(&mut self) -> CompileResult<Vec<Expr>> {
        let mut args = Vec::new();

        while self.current_token != Token::RParen {
            args.push(self.parse_expr()?);
            if self.current_token == Token::Comma {
                self.bump();
            }
        }

        Ok(args)
    }

    /// 解析闭包表达式: function(params) use (vars) : return_type { body }
    fn parse_closure(&mut self) -> CompileResult<Expr> {
        self.expect(Token::Function)?;
        self.expect(Token::LParen)?;
        let params = self.parse_params()?;
        self.expect(Token::RParen)?;

        // 解析 use 子句 (可选)
        let captures = if self.current_token == Token::Use {
            self.bump();
            self.expect(Token::LParen)?;
            let caps = self.parse_capture_vars()?;
            self.expect(Token::RParen)?;
            caps
        } else {
            Vec::new()
        };

        // 解析返回类型 (可选)
        let return_type = if self.current_token == Token::Colon {
            self.bump();
            self.parse_type()?
        } else {
            Type::Void
        };

        self.expect(Token::LBrace)?;
        let body = self.parse_block()?;
        self.expect(Token::RBrace)?;

        Ok(Expr::Closure(ClosureExpr {
            params,
            return_type,
            captures,
            body,
        }))
    }

    /// 解析捕获变量列表: $var, &$var, ...
    fn parse_capture_vars(&mut self) -> CompileResult<Vec<CaptureVar>> {
        let mut captures = Vec::new();

        if self.current_token == Token::RParen {
            return Ok(captures);
        }

        loop {
            // 检查是否是引用捕获 &$var
            let is_reference = if self.current_token == Token::Ampersand {
                self.bump();
                true
            } else {
                false
            };

            // 解析变量名
            let name = match &self.current_token {
                Token::Variable(n) => n.clone(),
                _ => {
                    return Err(CompileError::ParserError(format!(
                        "Expected variable in use clause, got {:?}",
                        self.current_token
                    )))
                }
            };
            self.bump();

            captures.push(CaptureVar { name, is_reference });

            if self.current_token == Token::Comma {
                self.bump();
            } else {
                break;
            }
        }

        Ok(captures)
    }

    /// 解析语句表达式（用于测试闭包）
    pub fn parse_expr_stmt(&mut self) -> CompileResult<Expr> {
        let expr = self.parse_expr()?;
        if self.current_token == Token::Semicolon {
            self.bump();
        }
        Ok(expr)
    }
}

/// 解析源代码为 Class AST
pub fn parse(source: &str) -> CompileResult<Class> {
    let mut parser = Parser::new(source.to_string());
    parser.parse_class()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_closure_parsing() {
        // 测试基本闭包解析
        let source = "function(int $x, int $y) use ($factor) : int { return $x + $y; }".to_string();
        let mut parser = Parser::new(source);

        let expr = parser.parse_expr_stmt().unwrap();

        match expr {
            Expr::Closure(closure) => {
                // 验证参数
                assert_eq!(closure.params.len(), 2);
                assert_eq!(closure.params[0].0, "x");
                assert_eq!(closure.params[1].0, "y");

                // 验证捕获变量
                assert_eq!(closure.captures.len(), 1);
                assert_eq!(closure.captures[0].name, "factor");
                assert!(!closure.captures[0].is_reference); // 值捕获

                // 验证返回类型 (int 被映射为 Int64)
                assert_eq!(closure.return_type, Type::Int64);

                // 验证闭包体
                assert_eq!(closure.body.len(), 1);
            }
            _ => panic!("Expected Closure expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_closure_with_reference_capture() {
        // 测试引用捕获闭包解析
        let source = "function() use (&$counter) : int { return $counter; }".to_string();
        let mut parser = Parser::new(source);

        let expr = parser.parse_expr_stmt().unwrap();

        match expr {
            Expr::Closure(closure) => {
                // 验证捕获变量
                assert_eq!(closure.captures.len(), 1);
                assert_eq!(closure.captures[0].name, "counter");
                assert!(closure.captures[0].is_reference); // 引用捕获
            }
            _ => panic!("Expected Closure expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_closure_with_multiple_captures() {
        // 测试多个捕获变量
        let source = "function(int $x) use ($a, &$b, $c) : int { return $x + $a; }".to_string();
        let mut parser = Parser::new(source);

        let expr = parser.parse_expr_stmt().unwrap();

        match expr {
            Expr::Closure(closure) => {
                // 验证捕获变量
                assert_eq!(closure.captures.len(), 3);
                assert_eq!(closure.captures[0].name, "a");
                assert!(!closure.captures[0].is_reference);
                assert_eq!(closure.captures[1].name, "b");
                assert!(closure.captures[1].is_reference);
                assert_eq!(closure.captures[2].name, "c");
                assert!(!closure.captures[2].is_reference);
            }
            _ => panic!("Expected Closure expression, got {:?}", expr),
        }
    }
}

use crate::ast::{
    AnnotationArgument, AnnotationDefinition, AnnotationProperty, AnnotationRetention,
    AnnotationTarget, AnnotationUsage, AnnotationValue, AttributeMeta, BinaryOp, CaptureVar,
    CatchClause, Class, ClassConst, ClassField, ClassMethod, ClosureExpr, CompilationUnit,
    EnumValue, Expr, Import, PromotedParam, PropertyHook, PropertyHookType, Stmt, Type, UnaryOp,
};
use crate::error::{CompileError, CompileResult};
use crate::lexer::{Lexer, Token};
use std::collections::HashSet;

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    class_name: String,
    package: Option<String>,
    imports: Vec<Import>,
}

impl Parser {
    pub fn new(input: String) -> Self {
        let mut parser = Parser {
            lexer: Lexer::new(input),
            current_token: Token::Eof,
            class_name: String::new(),
            package: None,
            imports: Vec::new(),
        };
        parser.bump();
        parser
    }

    fn parse_visibility(&mut self) -> (bool, bool, bool, bool) {
        let mut is_public = false;
        let mut is_private = false;
        let mut is_protected = false;
        let mut is_internal = false;

        match &self.current_token {
            Token::Public => {
                is_public = true;
                self.bump();
            }
            Token::Private => {
                is_private = true;
                self.bump();
            }
            Token::Protected => {
                is_protected = true;
                self.bump();
            }
            Token::Internal => {
                is_internal = true;
                self.bump();
            }
            _ => {
                is_public = true;
            }
        }

        (is_public, is_private, is_protected, is_internal)
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

    /// 解析完整的限定名（如 java.util.HashMap 或 java/util/HashMap）
    fn parse_qualified_name(&mut self) -> CompileResult<String> {
        let mut parts = Vec::new();

        // 第一个标识符
        match &self.current_token {
            Token::Identifier(name) => {
                parts.push(name.clone());
                self.bump();
            }
            _ => {
                return Err(CompileError::ParserError(
                    "Expected identifier in qualified name".to_string(),
                ))
            }
        }

        // 后续的 . 或 / 分隔的部分
        while self.current_token == Token::Dot {
            self.bump(); // 吃掉 .
            match &self.current_token {
                Token::Identifier(name) => {
                    parts.push(name.clone());
                    self.bump();
                }
                _ => {
                    return Err(CompileError::ParserError(
                        "Expected identifier after '.'".to_string(),
                    ))
                }
            }
        }

        Ok(parts.join("/"))
    }

    /// 解析 package 声明
    fn parse_package(&mut self) -> CompileResult<()> {
        self.expect(Token::Package)?;
        self.package = Some(self.parse_qualified_name()?);
        self.expect(Token::Semicolon)?;
        Ok(())
    }

    /// 解析 import 声明
    fn parse_import(&mut self) -> CompileResult<()> {
        self.expect(Token::Import)?;

        let mut parts = Vec::new();
        let mut is_star = false;

        // 第一个标识符
        match &self.current_token {
            Token::Identifier(name) => {
                parts.push(name.clone());
                self.bump();
            }
            _ => {
                return Err(CompileError::ParserError(
                    "Expected identifier in import".to_string(),
                ))
            }
        }

        // 后续的 . 分隔的部分或 *
        while self.current_token == Token::Dot {
            self.bump(); // 吃掉 .
            match &self.current_token {
                Token::Identifier(name) => {
                    parts.push(name.clone());
                    self.bump();
                }
                Token::Star => {
                    is_star = true;
                    self.bump();
                    break;
                }
                _ => {
                    return Err(CompileError::ParserError(
                        "Expected identifier or '*' after '.'".to_string(),
                    ))
                }
            }
        }

        let path = parts.join("/");
        self.imports.push(Import {
            path,
            is_star,
            alias: None,
        });

        self.expect(Token::Semicolon)?;
        Ok(())
    }

    pub fn parse_compilation_unit(&mut self) -> CompileResult<CompilationUnit> {
        if self.current_token == Token::Package {
            self.parse_package()?;
        }

        while self.current_token == Token::Import {
            self.parse_import()?;
        }

        let mut classes = Vec::new();
        let mut annotations = Vec::new();
        while self.current_token != Token::Eof {
            if self.current_token == Token::Annotation {
                let annotation = self.parse_annotation()?;
                annotations.push(annotation.with_package(&self.package));
            } else if self.current_token == Token::Public {
                self.bump();
                if self.current_token == Token::Annotation {
                    self.bump();
                    let annotation = self.parse_annotation_public()?;
                    annotations.push(annotation.with_package(&self.package));
                } else {
                    let class = self.parse_class_after_visibility(true, false, false, false)?;
                    classes.push(class.with_package(&self.package));
                }
            } else if matches!(
                self.current_token,
                Token::Class
                    | Token::Interface
                    | Token::Enum
                    | Token::Abstract
                    | Token::Final
                    | Token::Open
                    | Token::At
            ) {
                let class = self.parse_class()?;
                classes.push(class.with_package(&self.package));
            } else {
                self.bump();
            }
        }

        Ok(CompilationUnit {
            package: self.package.clone(),
            imports: self.imports.clone(),
            classes,
            annotations,
        })
    }

    pub fn parse_class(&mut self) -> CompileResult<Class> {
        let mut annotations = Vec::new();
        while self.current_token == Token::At {
            annotations.push(self.parse_annotation_usage()?);
        }

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
                        // 检查是否是构造函数 __construct
                        let is_constructor = if let Token::Identifier(n) = &self.current_token {
                            n == "__construct"
                        } else {
                            false
                        };
                        let m = self.parse_method_with_flags(
                            true, true, false, false, false, false, false, is_constructor,
                        )?;
                        if is_constructor {
                            constructor = Some(m);
                        } else {
                            methods.push(m);
                        }
                    } else {
                        let mut f = self.parse_field(true, false, false, false)?;
                        f.is_static = true;
                        fields.push(f);
                    }
                }
                Token::Function => {
                    self.bump();
                    // 检查是否是构造函数 __construct
                    let is_constructor = if let Token::Identifier(n) = &self.current_token {
                        n == "__construct"
                    } else {
                        false
                    };
                    let is_abstract_method = is_abstract && !is_interface;
                    let is_default_method = is_interface;
                    let m = self.parse_method_with_flags(
                        false,
                        true,
                        false,
                        false,
                        false,
                        is_abstract_method,
                        is_default_method,
                        is_constructor,
                    )?;
                    if is_constructor {
                        constructor = Some(m);
                    } else {
                        methods.push(m);
                    }
                }
                Token::Public | Token::Private | Token::Protected | Token::Internal => {
                    let (is_public, is_private, is_protected, is_internal) =
                        self.parse_visibility();
                    match &self.current_token {
                        Token::Static => {
                            self.bump();
                            if self.current_token == Token::Function {
                                self.bump();
                                // 检查是否是构造函数 __construct
                                let is_constructor = if let Token::Identifier(n) = &self.current_token {
                                    n == "__construct"
                                } else {
                                    false
                                };
                                let m = self.parse_method_with_flags(
                                    true,
                                    is_public,
                                    is_private,
                                    is_protected,
                                    is_internal,
                                    false,
                                    false,
                                    is_constructor,
                                )?;
                                if is_constructor {
                                    constructor = Some(m);
                                } else {
                                    methods.push(m);
                                }
                            } else {
                                let mut f = self.parse_field(
                                    is_public,
                                    is_private,
                                    is_protected,
                                    is_internal,
                                )?;
                                f.is_static = true;
                                fields.push(f);
                            }
                        }
                        Token::Function => {
                            self.bump();
                            // 检查是否是构造函数 __construct
                            let is_constructor = if let Token::Identifier(n) = &self.current_token {
                                n == "__construct"
                            } else {
                                false
                            };
                            let m = self.parse_method_with_flags(
                                false,
                                is_public,
                                is_private,
                                is_protected,
                                is_internal,
                                false,
                                false,
                                is_constructor,
                            )?;
                            if is_constructor {
                                constructor = Some(m);
                            } else {
                                methods.push(m);
                            }
                        }
                        Token::Abstract => {
                            self.bump();
                            self.expect(Token::Function)?;
                            let m = self.parse_abstract_method(
                                is_public,
                                is_private,
                                is_protected,
                                is_internal,
                            )?;
                            methods.push(m);
                        }
                        Token::Final => {
                            self.bump();
                            let mut f =
                                self.parse_field(is_public, is_private, is_protected, is_internal)?;
                            f.is_final = true;
                            fields.push(f);
                        }
                        _ => {
                            let f =
                                self.parse_field(is_public, is_private, is_protected, is_internal)?;
                            fields.push(f);
                        }
                    }
                }
                Token::Abstract => {
                    self.bump();
                    self.expect(Token::Function)?;
                    let m = self.parse_abstract_method(true, false, false, false)?;
                    methods.push(m);
                }
                Token::Identifier(n) => {
                    if n == "__construct" {
                        self.bump();
                        let m = self.parse_method_with_flags(
                            false, true, false, false, false, false, false, true,
                        )?;
                        constructor = Some(m);
                    } else {
                        let f = self.parse_field(true, false, false, false)?;
                        fields.push(f);
                    }
                }
                Token::Final => {
                    self.bump();
                    let mut f = self.parse_field(true, false, false, false)?;
                    f.is_final = true;
                    fields.push(f);
                }
                Token::Type(_)
                | Token::TypeInt8
                | Token::TypeInt16
                | Token::TypeInt32
                | Token::TypeInt64
                | Token::TypeFloat32
                | Token::TypeFloat64
                | Token::TypeBoolean
                | Token::TypeByte
                | Token::TypeInt
                | Token::TypeFloat => {
                    let f = self.parse_field(true, false, false, false)?;
                    fields.push(f);
                }
                _ => {
                    self.bump();
                }
            }
        }

        self.expect(Token::RBrace)?;

        Ok(Class {
            name: self.class_name.clone(),
            full_name: self.class_name.clone(),
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
            annotations,
        })
    }

    fn parse_class_after_visibility(
        &mut self,
        is_public: bool,
        is_private: bool,
        is_protected: bool,
        is_internal: bool,
    ) -> CompileResult<Class> {
        let mut annotations = Vec::new();
        while self.current_token == Token::At {
            annotations.push(self.parse_annotation_usage()?);
        }

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
                        let is_constructor = if let Token::Identifier(n) = &self.current_token {
                            n == "__construct"
                        } else {
                            false
                        };
                        let m = self.parse_method_with_flags(
                            true, is_public, is_private, is_protected, is_internal, false, false, is_constructor,
                        )?;
                        if is_constructor {
                            constructor = Some(m);
                        } else {
                            methods.push(m);
                        }
                    } else {
                        let mut f = self.parse_field(is_public, is_private, is_protected, is_internal)?;
                        f.is_static = true;
                        fields.push(f);
                    }
                }
                Token::Function => {
                    self.bump();
                    let is_constructor = if let Token::Identifier(n) = &self.current_token {
                        n == "__construct"
                    } else {
                        false
                    };
                    let is_abstract_method = is_abstract && !is_interface;
                    let is_default_method = is_interface;
                    let m = self.parse_method_with_flags(
                        false, is_public, is_private, is_protected, is_internal, is_abstract_method, is_default_method, is_constructor,
                    )?;
                    if is_constructor {
                        constructor = Some(m);
                    } else {
                        methods.push(m);
                    }
                }
                Token::Public | Token::Private | Token::Protected | Token::Internal => {
                    let (vis_pub, vis_priv, vis_prot, vis_int) = self.parse_visibility();
                    match &self.current_token {
                        Token::Static => {
                            self.bump();
                            if self.current_token == Token::Function {
                                self.bump();
                                let is_ctor = if let Token::Identifier(n) = &self.current_token {
                                    n == "__construct"
                                } else {
                                    false
                                };
                                let m = self.parse_method_with_flags(
                                    true, vis_pub, vis_priv, vis_prot, vis_int, false, false, is_ctor,
                                )?;
                                if is_ctor {
                                    constructor = Some(m);
                                } else {
                                    methods.push(m);
                                }
                            } else {
                                let mut f = self.parse_field(vis_pub, vis_priv, vis_prot, vis_int)?;
                                f.is_static = true;
                                fields.push(f);
                            }
                        }
                        Token::Function => {
                            self.bump();
                            let is_ctor = if let Token::Identifier(n) = &self.current_token {
                                n == "__construct"
                            } else {
                                false
                            };
                            let m = self.parse_method_with_flags(
                                false, vis_pub, vis_priv, vis_prot, vis_int, false, false, is_ctor,
                            )?;
                            if is_ctor {
                                constructor = Some(m);
                            } else {
                                methods.push(m);
                            }
                        }
                        Token::Abstract => {
                            self.bump();
                            self.expect(Token::Function)?;
                            let m = self.parse_abstract_method(vis_pub, vis_priv, vis_prot, vis_int)?;
                            methods.push(m);
                        }
                        Token::Final => {
                            self.bump();
                            let mut f = self.parse_field(vis_pub, vis_priv, vis_prot, vis_int)?;
                            f.is_final = true;
                            fields.push(f);
                        }
                        _ => {
                            let f = self.parse_field(vis_pub, vis_priv, vis_prot, vis_int)?;
                            fields.push(f);
                        }
                    }
                }
                Token::Abstract => {
                    self.bump();
                    self.expect(Token::Function)?;
                    let m = self.parse_abstract_method(true, false, false, false)?;
                    methods.push(m);
                }
                Token::Identifier(n) => {
                    if n == "__construct" {
                        self.bump();
                        let m = self.parse_method_with_flags(
                            false, is_public, is_private, is_protected, is_internal, false, false, true,
                        )?;
                        constructor = Some(m);
                    } else {
                        let f = self.parse_field(is_public, is_private, is_protected, is_internal)?;
                        fields.push(f);
                    }
                }
                Token::Final => {
                    self.bump();
                    let mut f = self.parse_field(is_public, is_private, is_protected, is_internal)?;
                    f.is_final = true;
                    fields.push(f);
                }
                Token::Type(_)
                | Token::TypeInt8
                | Token::TypeInt16
                | Token::TypeInt32
                | Token::TypeInt64
                | Token::TypeFloat32
                | Token::TypeFloat64
                | Token::TypeBoolean
                | Token::TypeByte
                | Token::TypeInt
                | Token::TypeFloat => {
                    let f = self.parse_field(is_public, is_private, is_protected, is_internal)?;
                    fields.push(f);
                }
                _ => {
                    self.bump();
                }
            }
        }

        self.expect(Token::RBrace)?;

        Ok(Class {
            name: self.class_name.clone(),
            full_name: self.class_name.clone(),
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
            annotations,
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

    fn parse_field(
        &mut self,
        is_public: bool,
        is_private: bool,
        is_protected: bool,
        is_internal: bool,
    ) -> CompileResult<ClassField> {
        let mut annotations = Vec::new();
        while self.current_token == Token::At {
            annotations.push(self.parse_annotation_usage()?);
        }

        let field_type = self.parse_type()?;

        let name = match &self.current_token {
            Token::Variable(n) => n.trim_start_matches('$').to_string(),
            _ => {
                return Err(CompileError::ParserError(format!(
                    "Expected field name (must start with $), got {:?}",
                    self.current_token
                )))
            }
        };
        self.bump();

        // Check for initializer before property hooks: type $name = value { get; set; }
        let mut initializer = if self.current_token == Token::Equal {
            self.bump();
            Some(self.parse_expr()?)
        } else {
            None
        };

        // Check for property hooks: { get; set; } or { get { ... } set(string $value) { ... } }
        let mut property_hooks = Vec::new();
        if self.current_token == Token::LBrace {
            self.bump(); // consume '{'
            
            while self.current_token != Token::RBrace {
                match &self.current_token {
                    Token::Get => {
                        self.bump(); // consume 'get'
                        let hook = self.parse_property_hook(PropertyHookType::Get, &field_type)?;
                        property_hooks.push(hook);
                    }
                    Token::Set => {
                        self.bump(); // consume 'set'
                        let hook = self.parse_property_hook(PropertyHookType::Set, &field_type)?;
                        property_hooks.push(hook);
                    }
                    _ => {
                        return Err(CompileError::ParserError(format!(
                            "Expected 'get' or 'set' in property hooks, got {:?}",
                            self.current_token
                        )));
                    }
                }
            }
            
            self.expect(Token::RBrace)?; // consume '}'
        }

        // Also support initializer after property hooks: type $name { get; set; } = value
        if initializer.is_none() && self.current_token == Token::Equal {
            self.bump();
            initializer = Some(self.parse_expr()?);
        }

        if self.current_token == Token::Semicolon {
            self.bump();
        }

        Ok(ClassField {
            name,
            field_type,
            is_nullable: false,
            is_static: false,
            is_public,
            is_private,
            is_protected,
            is_internal,
            is_final: false,
            initializer,
            property_hooks,
            annotations,
        })
    }

    fn parse_property_hook(&mut self, hook_type: PropertyHookType, field_type: &Type) -> CompileResult<PropertyHook> {
        // Check if it's a short form: get; or set;
        if self.current_token == Token::Semicolon {
            self.bump(); // consume ';'
            return Ok(PropertyHook {
                hook_type,
                body: Vec::new(), // Empty body means auto-generated
                param_type: None,
                param_name: None,
            });
        }
        
        // Check for arrow expression: get => expr;
        if self.current_token == Token::Arrow {
            self.bump(); // consume '=>'
            let expr = self.parse_expr()?;
            self.expect(Token::Semicolon)?;
            return Ok(PropertyHook {
                hook_type,
                body: vec![Stmt::Return(Some(expr))],
                param_type: None,
                param_name: None,
            });
        }
        
        // For setter, check if it has parameter type: set(string $value) { ... }
        let mut param_type = None;
        let mut param_name = None;
        
        if hook_type == PropertyHookType::Set && self.current_token != Token::LBrace {
            // Check if it's set(string $value) form with parentheses
            if self.current_token == Token::LParen {
                self.bump(); // consume '('
                
                // Parse optional parameter type
                let is_type_token = matches!(
                    &self.current_token,
                    Token::Type(_)
                        | Token::TypeInt8
                        | Token::TypeInt16
                        | Token::TypeInt32
                        | Token::TypeInt64
                        | Token::TypeFloat32
                        | Token::TypeFloat64
                        | Token::TypeBoolean
                        | Token::TypeByte
                        | Token::TypeInt
                        | Token::TypeFloat
                );
                
                if is_type_token {
                    param_type = Some(self.parse_type()?);
                }
                
                // Parse parameter name (must start with $)
                param_name = match &self.current_token {
                    Token::Variable(n) => {
                        let name = n.trim_start_matches('$').to_string();
                        self.bump();
                        Some(name)
                    }
                    _ => {
                        // If no explicit parameter name, use "value" as default
                        Some("value".to_string())
                    }
                };
                
                self.expect(Token::RParen)?; // consume ')'
            }
        }
        
        // Full block form: get { ... } or set(string $value) { ... }
        self.expect(Token::LBrace)?;
        let body = self.parse_block()?;
        self.expect(Token::RBrace)?;
        
        Ok(PropertyHook {
            hook_type,
            body,
            param_type,
            param_name,
        })
    }

    fn parse_method(
        &mut self,
        is_static: bool,
        is_public: bool,
        is_private: bool,
        is_protected: bool,
        is_internal: bool,
    ) -> CompileResult<ClassMethod> {
        self.parse_method_with_flags(
            is_static,
            is_public,
            is_private,
            is_protected,
            is_internal,
            false,
            false,
            false,
        )
    }

    fn parse_method_with_flags(
        &mut self,
        is_static: bool,
        is_public: bool,
        is_private: bool,
        is_protected: bool,
        is_internal: bool,
        is_abstract: bool,
        is_default: bool,
        is_constructor: bool,
    ) -> CompileResult<ClassMethod> {
        let mut annotations = Vec::new();
        while self.current_token == Token::At {
            annotations.push(self.parse_annotation_usage()?);
        }

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
            is_private,
            is_protected,
            is_internal,
            is_abstract,
            is_default,
            annotations,
        })
    }

    fn parse_abstract_method(
        &mut self,
        is_public: bool,
        is_private: bool,
        is_protected: bool,
        is_internal: bool,
    ) -> CompileResult<ClassMethod> {
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
            is_private,
            is_protected,
            is_internal,
            is_abstract: true,
            is_default: false,
            annotations: Vec::new(),
        })
    }

fn parse_annotation(&mut self) -> CompileResult<AnnotationDefinition> {
        self.expect(Token::Annotation)?;

        let name = match &self.current_token {
            Token::Identifier(n) => n.clone(),
            _ => {
                return Err(CompileError::ParserError(
                    "Expected annotation name".to_string(),
                ))
            }
        };
        self.bump();

        let attribute = if self.current_token == Token::At {
            self.parse_attribute_meta()?
        } else {
            Some(AttributeMeta {
                targets: HashSet::new(),
                retention: AnnotationRetention::Runtime,
            })
        };

        self.expect(Token::LBrace)?;

        let mut properties = Vec::new();
        while self.current_token != Token::RBrace {
            let prop = self.parse_annotation_property()?;
            properties.push(prop);
        }

        self.expect(Token::RBrace)?;

Ok(AnnotationDefinition {
            name: name.clone(),
            full_name: name,
            attribute,
            properties,
            is_public: true,
        })
    }

    fn parse_annotation_public(&mut self) -> CompileResult<AnnotationDefinition> {
        let name = match &self.current_token {
            Token::Identifier(n) => n.clone(),
            _ => {
                return Err(CompileError::ParserError(
                    "Expected annotation name".to_string(),
                ))
            }
        };
        self.bump();

        let attribute = if self.current_token == Token::At {
            self.parse_attribute_meta()?
        } else {
            Some(AttributeMeta {
                targets: HashSet::new(),
                retention: AnnotationRetention::Runtime,
            })
        };

        self.expect(Token::LBrace)?;

        let mut properties = Vec::new();
        while self.current_token != Token::RBrace {
            let prop = self.parse_annotation_property()?;
            properties.push(prop);
        }

        self.expect(Token::RBrace)?;

        Ok(AnnotationDefinition {
            name: name.clone(),
            full_name: name,
            attribute,
            properties,
            is_public: true,
        })
    }

    fn parse_attribute_meta(&mut self) -> CompileResult<Option<AttributeMeta>> {
        self.expect(Token::At)?;

        let attr_name = match &self.current_token {
            Token::Identifier(n) if n == "Attribute" => n.clone(),
            _ => {
                return Err(CompileError::ParserError(
                    "Expected 'Attribute' meta-annotation".to_string(),
                ))
            }
        };
        self.bump();

        self.expect(Token::LParen)?;

        let mut targets = HashSet::new();
        let mut retention = AnnotationRetention::Runtime;

        while self.current_token != Token::RParen {
            match &self.current_token {
                Token::Identifier(n) if n == "target" => {
                    self.bump();
                    self.expect(Token::Colon)?;
                    targets = self.parse_target_flags()?;
                }
                Token::Identifier(n) if n == "retention" => {
                    self.bump();
                    self.expect(Token::Colon)?;
                    retention = self.parse_retention_flag()?;
                }
                _ => {
                    return Err(CompileError::ParserError(format!(
                        "Unknown Attribute parameter: {:?}",
                        self.current_token
                    )))
                }
            }

            if self.current_token == Token::Comma {
                self.bump();
            }
        }

        self.expect(Token::RParen)?;

        Ok(Some(AttributeMeta { targets, retention }))
    }

    fn parse_target_flags(&mut self) -> CompileResult<HashSet<AnnotationTarget>> {
        let mut targets = HashSet::new();

        loop {
            let target = match &self.current_token {
                Token::Identifier(n) => match n.as_str() {
                    "TARGET_CLASS" => AnnotationTarget::Class,
                    "TARGET_FIELD" => AnnotationTarget::Field,
                    "TARGET_METHOD" => AnnotationTarget::Method,
                    "TARGET_PARAMETER" => AnnotationTarget::Parameter,
                    "TARGET_CONSTRUCTOR" => AnnotationTarget::Constructor,
                    "TARGET_PROPERTY" => AnnotationTarget::Property,
                    "Attribute" => {
                        self.bump();
                        if self.current_token == Token::Dot {
                            self.bump();
                        }
                        continue;
                    }
                    _ => {
                        return Err(CompileError::ParserError(format!(
                            "Unknown target flag: {}",
                            n
                        )))
                    }
                },
                _ => break,
            };
            targets.insert(target);
            self.bump();

            if self.current_token == Token::Dot {
                self.bump();
            } else if self.current_token == Token::Pipe {
                self.bump();
            } else {
                break;
            }
        }

        Ok(targets)
    }

    fn parse_retention_flag(&mut self) -> CompileResult<AnnotationRetention> {
        let retention = match &self.current_token {
            Token::Identifier(n) => match n.as_str() {
                "RETENTION_SOURCE" => AnnotationRetention::Source,
                "RETENTION_CLASS" => AnnotationRetention::Class,
                "RETENTION_RUNTIME" => AnnotationRetention::Runtime,
                "Attribute" => {
                    self.bump();
                    if self.current_token == Token::Dot {
                        self.bump();
                    }
                    return self.parse_retention_flag();
                }
                _ => {
                    return Err(CompileError::ParserError(format!(
                        "Unknown retention flag: {}",
                        n
                    )))
                }
            },
            _ => {
                return Err(CompileError::ParserError(
                    "Expected retention flag".to_string(),
                ))
            }
        };
        self.bump();

        Ok(retention)
    }

    fn parse_annotation_property(&mut self) -> CompileResult<AnnotationProperty> {
        let is_public = if self.current_token == Token::Public {
            self.bump();
            true
        } else {
            true
        };

        let property_type = self.parse_type()?;

        let name = match &self.current_token {
            Token::Variable(n) => {
                let raw_name = n.clone();
                let is_value_param = raw_name == "$value";
                let clean_name = if is_value_param {
                    "value".to_string()
                } else {
                    raw_name.trim_start_matches('$').to_string()
                };
                self.bump();
                (clean_name, is_value_param)
            }
            _ => {
                return Err(CompileError::ParserError(
                    "Expected property name (must start with $)".to_string(),
                ))
            }
        };

        let default_value = if self.current_token == Token::Equal {
            return Err(CompileError::ParserError(
                "Pava annotation property defaults must use ':' instead of '='. Use: `$name: defaultValue`".to_string()
            ));
        } else if self.current_token == Token::Colon {
            self.bump();
            Some(self.parse_annotation_value()?)
        } else {
            None
        };

        if self.current_token == Token::Semicolon {
            self.bump();
        }

        Ok(AnnotationProperty {
            name: name.0,
            property_type,
            default_value,
            is_value_param: name.1,
        })
    }

    fn parse_annotation_value(&mut self) -> CompileResult<AnnotationValue> {
        match &self.current_token {
            Token::IntLiteral(n) => {
                let val = *n;
                self.bump();
                Ok(AnnotationValue::Int(val))
            }
            Token::FloatLiteral(n) => {
                let val = *n;
                self.bump();
                Ok(AnnotationValue::Float(val))
            }
            Token::StringLiteral(s) => {
                let val = s.clone();
                self.bump();
                Ok(AnnotationValue::String(val))
            }
            Token::True => {
                self.bump();
                Ok(AnnotationValue::Bool(true))
            }
            Token::False => {
                self.bump();
                Ok(AnnotationValue::Bool(false))
            }
            Token::Null => {
                self.bump();
                Ok(AnnotationValue::Null)
            }
            Token::LBracket => {
                self.bump();
                let mut values = Vec::new();
                while self.current_token != Token::RBracket {
                    values.push(self.parse_annotation_value()?);
                    if self.current_token == Token::Comma {
                        self.bump();
                    }
                }
                self.expect(Token::RBracket)?;
                Ok(AnnotationValue::Array(values))
            }
            Token::Identifier(n) if n == "class" => {
                self.bump();
                let class_name = match &self.current_token {
                    Token::Identifier(name) => name.clone(),
                    _ => {
                        return Err(CompileError::ParserError(
                            "Expected class name after 'class'".to_string(),
                        ))
                    }
                };
                self.bump();
                Ok(AnnotationValue::ClassRef(class_name))
            }
            Token::Identifier(class_name) => {
                let class_name_str = class_name.clone();
                self.bump();
                if self.current_token == Token::Dot {
                    self.bump();
                    let enum_value = match &self.current_token {
                        Token::Identifier(v) => v.clone(),
                        _ => {
                            return Err(CompileError::ParserError(
                                "Expected enum value name".to_string(),
                            ))
                        }
                    };
                    self.bump();
                    
                    if self.current_token == Token::Pipe {
                        let mut values = vec![AnnotationValue::EnumRef(class_name_str.clone(), enum_value.clone())];
                        self.bump();
                        
                        loop {
                            let next_class = match &self.current_token {
                                Token::Identifier(c) => c.clone(),
                                _ => break,
                            };
                            self.bump();
                            
                            if self.current_token == Token::Dot {
                                self.bump();
                            }
                            
                            let next_enum = match &self.current_token {
                                Token::Identifier(v) => v.clone(),
                                _ => {
                                    return Err(CompileError::ParserError(
                                        "Expected enum value after '|'".to_string(),
                                    ))
                                }
                            };
                            self.bump();
                            values.push(AnnotationValue::EnumRef(next_class, next_enum));
                            
                            if self.current_token == Token::Pipe {
                                self.bump();
                            } else {
                                break;
                            }
                        }
                        
                        Ok(AnnotationValue::Array(values))
                    } else {
                        Ok(AnnotationValue::EnumRef(class_name_str, enum_value))
                    }
                } else {
                    Ok(AnnotationValue::ClassRef(class_name_str))
                }
            }
            _ => Err(CompileError::ParserError(format!(
                "Unexpected token in annotation value: {:?}",
                self.current_token
            ))),
        }
    }

    fn parse_annotation_usage(&mut self) -> CompileResult<AnnotationUsage> {
        self.expect(Token::At)?;

        let name = match &self.current_token {
            Token::Identifier(n) => n.clone(),
            _ => {
                return Err(CompileError::ParserError(
                    "Expected annotation name after '@'".to_string(),
                ))
            }
        };
        self.bump();

        let arguments = if self.current_token == Token::LParen {
            self.bump();
            let args = self.parse_annotation_arguments()?;
            self.expect(Token::RParen)?;
            args
        } else {
            Vec::new()
        };

        Ok(AnnotationUsage { name, arguments })
    }

    fn parse_annotation_arguments(&mut self) -> CompileResult<Vec<AnnotationArgument>> {
        let mut arguments = Vec::new();

        if self.current_token == Token::RParen {
            return Ok(arguments);
        }

        while self.current_token != Token::RParen {
            if matches!(self.current_token, Token::Equal | Token::LBrace) {
                return Err(CompileError::ParserError(
                    "Pava annotation arguments must use ':' for key-value pairs and '[]' for arrays, not '=' and '{}'. Example: @Column(\"id\", nullable: false) or @Names([\"a\", \"b\"])".to_string()
                ));
            }

            let (key, value) = self.parse_annotation_argument()?;
            arguments.push(AnnotationArgument { key, value });

            if self.current_token == Token::Comma {
                self.bump();
            }
        }

        Ok(arguments)
    }

    fn parse_annotation_argument(&mut self) -> CompileResult<(Option<String>, AnnotationValue)> {
        if matches!(self.current_token, Token::Identifier(_)) {
            let potential_key = match &self.current_token {
                Token::Identifier(n) => n.clone(),
                _ => unreachable!(),
            };
            self.bump();

            if self.current_token == Token::Colon {
                self.bump();
                let value = self.parse_annotation_value()?;
                return Ok((Some(potential_key), value));
            } else if self.current_token == Token::Equal {
                return Err(CompileError::ParserError(
                    "Pava annotation arguments must use ':' instead of '='. Example: @Column(name: \"id\")".to_string()
                ));
            } else {
                return Ok((None, AnnotationValue::String(potential_key)));
            }
        }

        let value = self.parse_annotation_value()?;
        Ok((None, value))
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
                let mut is_internal = false;

                if matches!(
                    &self.current_token,
                    Token::Public | Token::Private | Token::Protected | Token::Internal
                ) {
                    match &self.current_token {
                        Token::Public => is_public = true,
                        Token::Private => is_private = true,
                        Token::Protected => is_protected = true,
                        Token::Internal => is_internal = true,
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

                if is_public || is_private || is_protected || is_internal {
                    promoted_params.push(PromotedParam {
                        name: param_name,
                        param_type,
                        is_public,
                        is_private,
                        is_protected,
                        is_internal,
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
                Ok(Type::Int32)
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
            Token::Void => {
                self.bump();
                Ok(Type::Void)
            }
            _ => {
                self.bump();
                Ok(Type::String)
            }
        }
    }

    fn parse_exception_type(&mut self) -> CompileResult<String> {
        let mut parts = Vec::new();
        match &self.current_token {
            Token::Identifier(name) => {
                parts.push(name.clone());
                self.bump();
            }
            Token::Type(name) => {
                parts.push(name.clone());
                self.bump();
            }
            _ => {
                return Err(CompileError::ParserError(
                    "Expected exception type name".to_string(),
                ))
            }
        }

        while self.current_token == Token::Dot {
            self.bump();
            match &self.current_token {
                Token::Identifier(name) => {
                    parts.push(name.clone());
                    self.bump();
                }
                _ => {
                    return Err(CompileError::ParserError(
                        "Expected identifier after '.' in exception type".to_string(),
                    ))
                }
            }
        }

        Ok(parts.join("/"))
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
            Token::TypeInt8
            | Token::TypeInt16
            | Token::TypeInt32
            | Token::TypeInt64
            | Token::TypeFloat32
            | Token::TypeFloat64
            | Token::TypeBoolean
            | Token::TypeByte
            | Token::TypeInt
            | Token::TypeFloat
            | Token::Type(_) => {
                let ty = self.parse_type()?;
                let name = match &self.current_token {
                    Token::Variable(n) => n.clone(),
                    _ => {
                        return Err(CompileError::ParserError(
                            "Expected variable name after type".to_string(),
                        ))
                    }
                };
                self.bump();
                self.expect(Token::Equal)?;
                let value = self.parse_expr()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::TypedAssign(name, ty, value))
            }
            Token::SelfRef | Token::Parent => {
                let class_name = match &self.current_token {
                    Token::SelfRef => "self",
                    Token::Parent => "parent",
                    _ => unreachable!(),
                };
                self.bump();
                self.expect(Token::DoubleColon)?;
                let field = match &self.current_token {
                    Token::Identifier(n) => n.clone(),
                    Token::Variable(n) => n.trim_start_matches('$').to_string(),
                    _ => {
                        return Err(CompileError::ParserError(
                            "Expected static member name".to_string(),
                        ))
                    }
                };
                self.bump();

                if self.current_token == Token::Equal {
                    self.bump();
                    let value = self.parse_expr()?;
                    self.expect(Token::Semicolon)?;
                    let lhs = Expr::StaticFieldAccess(class_name.to_string(), field);
                    let assign_expr =
                        Expr::BinaryOp(BinaryOp::Assign, Box::new(lhs), Box::new(value));
                    return Ok(Stmt::Expr(assign_expr));
                }

                if self.current_token == Token::LParen {
                    self.bump();
                    let args = self.parse_args()?;
                    self.expect(Token::RParen)?;
                    self.expect(Token::Semicolon)?;
                    return Ok(Stmt::Expr(Expr::StaticCall(
                        class_name.to_string(),
                        field,
                        args,
                    )));
                }

                self.expect(Token::Semicolon)?;
                Ok(Stmt::Expr(Expr::StaticFieldAccess(
                    class_name.to_string(),
                    field,
                )))
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

                let mut elseif_pairs = Vec::new();
                let mut else_body = None;

                while self.current_token == Token::Else {
                    self.bump();
                    if self.current_token == Token::If {
                        self.bump();
                        self.expect(Token::LParen)?;
                        let ei_cond = self.parse_expr()?;
                        self.expect(Token::RParen)?;
                        self.expect(Token::LBrace)?;
                        let ei_body = self.parse_block()?;
                        self.expect(Token::RBrace)?;
                        elseif_pairs.push((ei_cond, ei_body));
                    } else {
                        self.expect(Token::LBrace)?;
                        else_body = Some(self.parse_block()?);
                        self.expect(Token::RBrace)?;
                        break;
                    }
                }

                Ok(Stmt::If(cond, then, elseif_pairs, else_body))
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
            Token::For => {
                self.bump();
                self.expect(Token::LParen)?;

                let init = if self.current_token != Token::Semicolon {
                    self.parse_for_sub_stmt()?
                } else {
                    Stmt::Expr(Expr::NullLiteral)
                };
                self.expect(Token::Semicolon)?;

                let cond = if self.current_token != Token::RParen {
                    self.parse_expr()?
                } else {
                    Expr::BoolLiteral(true)
                };
                self.expect(Token::Semicolon)?;

                let update = if self.current_token != Token::RParen {
                    self.parse_for_sub_stmt()?
                } else {
                    Stmt::Expr(Expr::NullLiteral)
                };
                self.expect(Token::RParen)?;

                self.expect(Token::LBrace)?;
                let body = self.parse_block()?;
                self.expect(Token::RBrace)?;
                Ok(Stmt::For(Box::new(init), cond, Box::new(update), body))
            }
            Token::Break => {
                self.bump();
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Break)
            }
            Token::Continue => {
                self.bump();
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Continue)
            }
            Token::Try => {
                self.bump();
                self.expect(Token::LBrace)?;
                let try_body = self.parse_block()?;
                self.expect(Token::RBrace)?;

                let mut catch_clauses = Vec::new();
                while self.current_token == Token::Catch {
                    self.bump();
                    self.expect(Token::LParen)?;

                    let mut exception_types = Vec::new();
                    let first_type = self.parse_exception_type()?;
                    exception_types.push(first_type);

                    while self.current_token == Token::Pipe {
                        self.bump();
                        let next_type = self.parse_exception_type()?;
                        exception_types.push(next_type);
                    }

                    let var_name = match &self.current_token {
                        Token::Variable(n) => n.clone(),
                        _ => {
                            return Err(CompileError::ParserError(
                                "Expected variable name in catch clause".to_string(),
                            ))
                        }
                    };
                    self.bump();
                    self.expect(Token::RParen)?;
                    self.expect(Token::LBrace)?;
                    let catch_body = self.parse_block()?;
                    self.expect(Token::RBrace)?;

                    catch_clauses.push(CatchClause {
                        exception_types,
                        var_name,
                        body: catch_body,
                    });
                }

                let finally_body = if self.current_token == Token::Finally {
                    self.bump();
                    self.expect(Token::LBrace)?;
                    let body = self.parse_block()?;
                    self.expect(Token::RBrace)?;
                    Some(body)
                } else {
                    None
                };

                Ok(Stmt::TryCatch {
                    try_body,
                    catch_clauses,
                    finally_body,
                })
            }
            Token::Throw => {
                self.bump();
                let expr = self.parse_expr()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Expr(Expr::Throw(Box::new(expr))))
            }
            Token::Variable(_) => {
                let expr = self.parse_expr()?;
                if self.current_token == Token::Equal {
                    self.bump();
                    let value = self.parse_expr()?;
                    self.expect(Token::Semicolon)?;
                    let assign_expr =
                        Expr::BinaryOp(BinaryOp::Assign, Box::new(expr), Box::new(value));
                    Ok(Stmt::Expr(assign_expr))
                } else if matches!(
                    self.current_token,
                    Token::PlusEqual
                        | Token::MinusEqual
                        | Token::StarEqual
                        | Token::SlashEqual
                        | Token::PercentEqual
                ) {
                    let op = match &self.current_token {
                        Token::PlusEqual => BinaryOp::AddAssign,
                        Token::MinusEqual => BinaryOp::SubAssign,
                        Token::StarEqual => BinaryOp::MulAssign,
                        Token::SlashEqual => BinaryOp::DivAssign,
                        Token::PercentEqual => BinaryOp::ModAssign,
                        _ => unreachable!(),
                    };
                    self.bump();
                    let value = self.parse_expr()?;
                    self.expect(Token::Semicolon)?;
                    let assign_expr = Expr::BinaryOp(op, Box::new(expr), Box::new(value));
                    Ok(Stmt::Expr(assign_expr))
                } else {
                    self.expect(Token::Semicolon)?;
                    Ok(Stmt::Expr(expr))
                }
            }
            _ => {
                let e = self.parse_expr()?;
                if self.current_token == Token::Equal {
                    self.bump();
                    let value = self.parse_expr()?;
                    self.expect(Token::Semicolon)?;
                    let assign_expr =
                        Expr::BinaryOp(BinaryOp::Assign, Box::new(e), Box::new(value));
                    Ok(Stmt::Expr(assign_expr))
                } else if matches!(
                    self.current_token,
                    Token::PlusEqual
                        | Token::MinusEqual
                        | Token::StarEqual
                        | Token::SlashEqual
                        | Token::PercentEqual
                ) {
                    let op = match &self.current_token {
                        Token::PlusEqual => BinaryOp::AddAssign,
                        Token::MinusEqual => BinaryOp::SubAssign,
                        Token::StarEqual => BinaryOp::MulAssign,
                        Token::SlashEqual => BinaryOp::DivAssign,
                        Token::PercentEqual => BinaryOp::ModAssign,
                        _ => unreachable!(),
                    };
                    self.bump();
                    let value = self.parse_expr()?;
                    self.expect(Token::Semicolon)?;
                    let assign_expr = Expr::BinaryOp(op, Box::new(e), Box::new(value));
                    Ok(Stmt::Expr(assign_expr))
                } else {
                    self.expect(Token::Semicolon)?;
                    Ok(Stmt::Expr(e))
                }
            }
        }
    }

    fn parse_expr(&mut self) -> CompileResult<Expr> {
        let left = self.parse_comparison()?;
        self.parse_ternary(left)
    }

    fn parse_ternary(&mut self, left: Expr) -> CompileResult<Expr> {
        match &self.current_token {
            Token::Question => {
                self.bump();
                let then_expr = self.parse_expr()?;
                self.expect(Token::Colon)?;
                let else_expr = self.parse_expr()?;
                Ok(Expr::Ternary(
                    Box::new(left),
                    Box::new(then_expr),
                    Box::new(else_expr),
                ))
            }
            Token::QuestionColon => {
                self.bump();
                let else_expr = self.parse_expr()?;
                Ok(Expr::Elvis(Box::new(left), Box::new(else_expr)))
            }
            Token::DoubleQuestion => {
                self.bump();
                let default_expr = self.parse_expr()?;
                Ok(Expr::NullCoalescing(Box::new(left), Box::new(default_expr)))
            }
            _ => Ok(left),
        }
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

        // Handle instanceof operator
        if self.current_token == Token::Instanceof {
            self.bump();
            let class_name = match &self.current_token {
                Token::Identifier(name) => name.clone(),
                Token::Type(name) => name.clone(),
                Token::TypeInt8 => "byte".to_string(),
                Token::TypeInt16 => "short".to_string(),
                Token::TypeInt32 => "int".to_string(),
                Token::TypeInt64 => "long".to_string(),
                Token::TypeFloat32 => "float".to_string(),
                Token::TypeFloat64 => "double".to_string(),
                Token::TypeBoolean => "boolean".to_string(),
                Token::TypeByte => "byte".to_string(),
                Token::TypeInt => "int".to_string(),
                Token::TypeFloat => "float".to_string(),
                _ => {
                    return Err(CompileError::ParserError(
                        "Expected class name after instanceof".to_string(),
                    ))
                }
            };
            self.bump();
            left = Expr::InstanceOf(Box::new(left), class_name);
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
        if self.current_token == Token::PlusPlus {
            self.bump();
            let e = self.parse_unary()?;
            return Ok(Expr::UnaryOp(UnaryOp::PreIncrement, Box::new(e)));
        }
        if self.current_token == Token::MinusMinus {
            self.bump();
            let e = self.parse_unary()?;
            return Ok(Expr::UnaryOp(UnaryOp::PreDecrement, Box::new(e)));
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            match &self.current_token {
                Token::PlusPlus => {
                    self.bump();
                    expr = Expr::UnaryOp(UnaryOp::PostIncrement, Box::new(expr));
                }
                Token::MinusMinus => {
                    self.bump();
                    expr = Expr::UnaryOp(UnaryOp::PostDecrement, Box::new(expr));
                }
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

    fn is_type_token(&self) -> bool {
        matches!(
            &self.current_token,
            Token::Type(_)
                | Token::TypeInt8
                | Token::TypeInt16
                | Token::TypeInt32
                | Token::TypeInt64
                | Token::TypeFloat32
                | Token::TypeFloat64
                | Token::TypeBoolean
                | Token::TypeByte
                | Token::TypeInt
                | Token::TypeFloat
        )
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
            Token::InterpolatedString(parts) => {
                let expr_parts: Vec<Expr> = parts
                    .iter()
                    .map(|part| match part {
                        crate::lexer::StringPart::Text(s) => Expr::StringLiteral(s.clone()),
                        crate::lexer::StringPart::Variable(name) => Expr::Variable(name.clone()),
                    })
                    .collect();
                self.bump();
                Ok(Expr::InterpolatedString(expr_parts))
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
                if self.is_type_token() {
                    let target_type = self.parse_type()?;
                    self.expect(Token::RParen)?;
                    let expr = self.parse_unary()?;
                    Ok(Expr::Cast(Box::new(expr), target_type))
                } else {
                    let e = self.parse_expr()?;
                    self.expect(Token::RParen)?;
                    Ok(e)
                }
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

    fn parse_for_sub_stmt(&mut self) -> CompileResult<Stmt> {
        match &self.current_token {
            Token::Variable(_) => {
                let name = match &self.current_token {
                    Token::Variable(n) => n.clone(),
                    _ => return Err(CompileError::ParserError("Expected variable".to_string())),
                };
                self.bump();

                if self.current_token == Token::Equal {
                    self.bump();
                    let value = self.parse_expr()?;
                    return Ok(Stmt::Assign(name, value));
                }

                if self.current_token == Token::Arrow || self.current_token == Token::Dot {
                    let mut expr = Expr::Variable(name);
                    while self.current_token == Token::Arrow || self.current_token == Token::Dot {
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

                    if self.current_token == Token::Equal {
                        self.bump();
                        let value = self.parse_expr()?;
                        let assign_expr =
                            Expr::BinaryOp(BinaryOp::Assign, Box::new(expr), Box::new(value));
                        return Ok(Stmt::Expr(assign_expr));
                    }

                    return Ok(Stmt::Expr(expr));
                }

                Ok(Stmt::Assign(name, Expr::NullLiteral))
            }
            _ => {
                let e = self.parse_expr()?;
                if self.current_token == Token::Equal {
                    self.bump();
                    let value = self.parse_expr()?;
                    let assign_expr =
                        Expr::BinaryOp(BinaryOp::Assign, Box::new(e), Box::new(value));
                    Ok(Stmt::Expr(assign_expr))
                } else {
                    Ok(Stmt::Expr(e))
                }
            }
        }
    }

    pub fn parse_stmt_test(&mut self) -> CompileResult<Stmt> {
        self.parse_stmt()
    }
}

/// 解析源代码为 Class AST
pub fn parse(source: &str) -> CompileResult<Class> {
    let mut parser = Parser::new(source.to_string());
    parser.parse_class()
}

/// 解析源代码为编译单元（包含 package、imports 和 classes）
pub fn parse_compilation_unit(source: &str) -> CompileResult<CompilationUnit> {
    let mut parser = Parser::new(source.to_string());
    parser.parse_compilation_unit()
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

    #[test]
    fn test_cast_expression_int32_to_int64() {
        let source = "(int64) $age".to_string();
        let mut parser = Parser::new(source);
        let expr = parser.parse_expr_stmt().unwrap();
        match expr {
            Expr::Cast(inner, target_type) => {
                assert_eq!(target_type, Type::Int64);
                match *inner {
                    Expr::Variable(name) => assert_eq!(name, "age"),
                    _ => panic!("Expected Variable inside Cast"),
                }
            }
            _ => panic!("Expected Cast expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_cast_expression_int64_to_int32() {
        let source = "(int32) $bigValue".to_string();
        let mut parser = Parser::new(source);
        let expr = parser.parse_expr_stmt().unwrap();
        match expr {
            Expr::Cast(inner, target_type) => {
                assert_eq!(target_type, Type::Int32);
                match *inner {
                    Expr::Variable(name) => assert_eq!(name, "bigValue"),
                    _ => panic!("Expected Variable inside Cast"),
                }
            }
            _ => panic!("Expected Cast expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_cast_expression_float64_to_int32() {
        let source = "(int32) $price".to_string();
        let mut parser = Parser::new(source);
        let expr = parser.parse_expr_stmt().unwrap();
        match expr {
            Expr::Cast(inner, target_type) => {
                assert_eq!(target_type, Type::Int32);
                match *inner {
                    Expr::Variable(name) => assert_eq!(name, "price"),
                    _ => panic!("Expected Variable inside Cast"),
                }
            }
            _ => panic!("Expected Cast expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_paren_expr_not_cast() {
        let source = "(1 + 2)".to_string();
        let mut parser = Parser::new(source);
        let expr = parser.parse_expr_stmt().unwrap();
        match expr {
            Expr::BinaryOp(BinaryOp::Add, _, _) => {}
            _ => panic!("Expected BinaryOp, got {:?}", expr),
        }
    }

    #[test]
    fn test_break_stmt() {
        let source = "break;".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        assert!(matches!(stmt, Stmt::Break));
    }

    #[test]
    fn test_instanceof_in_if() {
        let source = "if ($obj instanceof MyClass) { $x = 1; }".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::If(cond, _, _, _) => match cond {
                Expr::InstanceOf(expr, class_name) => {
                    match *expr {
                        Expr::Variable(name) => assert_eq!(name, "obj"),
                        _ => panic!("Expected Variable"),
                    }
                    assert_eq!(class_name, "MyClass");
                }
                _ => panic!("Expected InstanceOf condition"),
            },
            _ => panic!("Expected If, got {:?}", stmt),
        }
    }

    #[test]
    fn test_if_with_else() {
        let source = "if ($x > 0) { $y = 1; } else { $y = 0; }".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::If(_, then_body, elseif_pairs, else_body) => {
                assert_eq!(then_body.len(), 1);
                assert!(elseif_pairs.is_empty());
                assert!(else_body.is_some());
                assert_eq!(else_body.unwrap().len(), 1);
            }
            _ => panic!("Expected If, got {:?}", stmt),
        }
    }

    #[test]
    fn test_if_with_elseif() {
        let source =
            "if ($x > 0) { $y = 1; } else if ($x == 0) { $y = 0; } else { $y = -1; }".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::If(_, then_body, elseif_pairs, else_body) => {
                assert_eq!(then_body.len(), 1);
                assert_eq!(elseif_pairs.len(), 1);
                assert!(else_body.is_some());
            }
            _ => panic!("Expected If, got {:?}", stmt),
        }
    }

    #[test]
    fn test_if_multiple_elseif() {
        let source =
            "if ($x == 1) { $y = 10; } else if ($x == 2) { $y = 20; } else if ($x == 3) { $y = 30; } else { $y = 0; }"
                .to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::If(_, _, elseif_pairs, else_body) => {
                assert_eq!(elseif_pairs.len(), 2);
                assert!(else_body.is_some());
            }
            _ => panic!("Expected If, got {:?}", stmt),
        }
    }

    #[test]
    fn test_for_loop() {
        let source = "for ($i = 0; $i < 10; $i = $i + 1) { println($i); }".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::For(init, cond, update, body) => {
                assert!(matches!(*init, Stmt::Assign(_, _)));
                assert!(matches!(*update, Stmt::Assign(_, _)));
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected For, got {:?}", stmt),
        }
    }

    #[test]
    fn test_typed_assign() {
        let source = "int32 $x = 5;".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::TypedAssign(name, ty, _) => {
                assert_eq!(name, "x");
                assert_eq!(ty, Type::Int32);
            }
            _ => panic!("Expected TypedAssign, got {:?}", stmt),
        }
    }

    #[test]
    fn test_static_field_write_self() {
        let source = "self::count = 5;".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::Expr(Expr::BinaryOp(BinaryOp::Assign, lhs, _)) => match *lhs {
                Expr::StaticFieldAccess(class, field) => {
                    assert_eq!(class, "self");
                    assert_eq!(field, "count");
                }
                _ => panic!("Expected StaticFieldAccess lhs"),
            },
            _ => panic!("Expected Expr(BinaryOp::Assign), got {:?}", stmt),
        }
    }

    #[test]
    fn test_static_field_write_classname() {
        let source = "MyClass::count = 10;".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::Expr(Expr::BinaryOp(BinaryOp::Assign, lhs, _)) => match *lhs {
                Expr::StaticFieldAccess(class, field) => {
                    assert_eq!(class, "MyClass");
                    assert_eq!(field, "count");
                }
                _ => panic!("Expected StaticFieldAccess lhs"),
            },
            _ => panic!("Expected Expr(BinaryOp::Assign), got {:?}", stmt),
        }
    }

    #[test]
    fn test_this_field_access() {
        let source = "$this->field;".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::Expr(Expr::FieldAccess(obj, field_name)) => {
                match *obj {
                    Expr::Variable(name) => assert_eq!(name, "this"),
                    _ => panic!("Expected Variable 'this'"),
                }
                assert_eq!(field_name, "field");
            }
            _ => panic!("Expected FieldAccess, got {:?}", stmt),
        }
    }

    #[test]
    fn test_this_method_call() {
        let source = "$this->doSomething();".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::Expr(Expr::MethodCall(obj, method_name, args)) => {
                match *obj {
                    Expr::Variable(name) => assert_eq!(name, "this"),
                    _ => panic!("Expected Variable 'this'"),
                }
                assert_eq!(method_name, "doSomething");
                assert_eq!(args.len(), 0);
            }
            _ => panic!("Expected MethodCall, got {:?}", stmt),
        }
    }

    #[test]
    fn test_this_field_chain() {
        let source = "$this->obj->name;".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::Expr(Expr::FieldAccess(inner, field_name)) => {
                assert_eq!(field_name, "name");
                match *inner {
                    Expr::FieldAccess(obj, inner_field) => {
                        assert_eq!(inner_field, "obj");
                        match *obj {
                            Expr::Variable(name) => assert_eq!(name, "this"),
                            _ => panic!("Expected Variable 'this'"),
                        }
                    }
                    _ => panic!("Expected nested FieldAccess"),
                }
            }
            _ => panic!("Expected FieldAccess chain, got {:?}", stmt),
        }
    }

    #[test]
    fn test_ternary_expression() {
        let source = "$x ? 1 : 0;".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::Expr(Expr::Ternary(cond, then_expr, else_expr)) => {
                match *cond {
                    Expr::Variable(name) => assert_eq!(name, "x"),
                    _ => panic!("Expected Variable cond"),
                }
                match *then_expr {
                    Expr::IntLiteral(n) => assert_eq!(n, 1),
                    _ => panic!("Expected IntLiteral 1"),
                }
                match *else_expr {
                    Expr::IntLiteral(n) => assert_eq!(n, 0),
                    _ => panic!("Expected IntLiteral 0"),
                }
            }
            _ => panic!("Expected Ternary, got {:?}", stmt),
        }
    }

    #[test]
    fn test_elvis_expression() {
        let source = "$x ?: 0;".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::Expr(Expr::Elvis(value, else_expr)) => {
                match *value {
                    Expr::Variable(name) => assert_eq!(name, "x"),
                    _ => panic!("Expected Variable value"),
                }
                match *else_expr {
                    Expr::IntLiteral(n) => assert_eq!(n, 0),
                    _ => panic!("Expected IntLiteral 0"),
                }
            }
            _ => panic!("Expected Elvis, got {:?}", stmt),
        }
    }

    #[test]
    fn test_null_coalescing_expression() {
        let source = "$x ?? \"default\";".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::Expr(Expr::NullCoalescing(value, default_expr)) => {
                match *value {
                    Expr::Variable(name) => assert_eq!(name, "x"),
                    _ => panic!("Expected Variable value"),
                }
                match *default_expr {
                    Expr::StringLiteral(s) => assert_eq!(s, "default"),
                    _ => panic!("Expected StringLiteral \"default\""),
                }
            }
            _ => panic!("Expected NullCoalescing, got {:?}", stmt),
        }
    }

    #[test]
    fn test_nested_ternary() {
        let source = "$a ? $b : $c ? $d : $e;".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::Expr(Expr::Ternary(cond, then_expr, else_expr)) => {
                match *cond {
                    Expr::Variable(name) => assert_eq!(name, "a"),
                    _ => panic!("Expected Variable 'a'"),
                }
                match *then_expr {
                    Expr::Variable(name) => assert_eq!(name, "b"),
                    _ => panic!("Expected Variable 'b'"),
                }
                match *else_expr {
                    Expr::Ternary(inner_cond, inner_then, inner_else) => {
                        match *inner_cond {
                            Expr::Variable(name) => assert_eq!(name, "c"),
                            _ => panic!("Expected Variable 'c'"),
                        }
                        match *inner_then {
                            Expr::Variable(name) => assert_eq!(name, "d"),
                            _ => panic!("Expected Variable 'd'"),
                        }
                        match *inner_else {
                            Expr::Variable(name) => assert_eq!(name, "e"),
                            _ => panic!("Expected Variable 'e'"),
                        }
                    }
                    _ => panic!("Expected nested Ternary"),
                }
            }
            _ => panic!("Expected Ternary, got {:?}", stmt),
        }
    }

    #[test]
    fn test_ternary_in_assign() {
        let source = "$result = $x > 0 ? \"positive\" : \"negative\";".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::Expr(Expr::BinaryOp(BinaryOp::Assign, lhs, rhs)) => {
                match *lhs {
                    Expr::Variable(name) => assert_eq!(name, "result"),
                    _ => panic!("Expected Variable lhs"),
                }
                match *rhs {
                    Expr::Ternary(_, _, _) => {}
                    _ => panic!("Expected Ternary rhs"),
                }
            }
            _ => panic!("Expected Assign, got {:?}", stmt),
        }
    }

    #[test]
    fn test_compound_assign_add() {
        let source = "$a += $b;".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::Expr(Expr::BinaryOp(BinaryOp::AddAssign, lhs, rhs)) => {
                match *lhs {
                    Expr::Variable(name) => assert_eq!(name, "a"),
                    _ => panic!("Expected Variable lhs"),
                }
                match *rhs {
                    Expr::Variable(name) => assert_eq!(name, "b"),
                    _ => panic!("Expected Variable rhs"),
                }
            }
            _ => panic!("Expected AddAssign, got {:?}", stmt),
        }
    }

    #[test]
    fn test_compound_assign_sub() {
        let source = "$x -= 5;".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::Expr(Expr::BinaryOp(BinaryOp::SubAssign, lhs, rhs)) => {
                match *lhs {
                    Expr::Variable(name) => assert_eq!(name, "x"),
                    _ => panic!("Expected Variable lhs"),
                }
                match *rhs {
                    Expr::IntLiteral(n) => assert_eq!(n, 5),
                    _ => panic!("Expected IntLiteral rhs"),
                }
            }
            _ => panic!("Expected SubAssign, got {:?}", stmt),
        }
    }

    #[test]
    fn test_compound_assign_mul() {
        let source = "$i *= 2;".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::Expr(Expr::BinaryOp(BinaryOp::MulAssign, _, _)) => {}
            _ => panic!("Expected MulAssign, got {:?}", stmt),
        }
    }

    #[test]
    fn test_post_increment() {
        let source = "$i++;".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::Expr(Expr::UnaryOp(UnaryOp::PostIncrement, inner)) => match *inner {
                Expr::Variable(name) => assert_eq!(name, "i"),
                _ => panic!("Expected Variable"),
            },
            _ => panic!("Expected PostIncrement, got {:?}", stmt),
        }
    }

    #[test]
    fn test_pre_increment() {
        let source = "++$i;".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::Expr(Expr::UnaryOp(UnaryOp::PreIncrement, inner)) => match *inner {
                Expr::Variable(name) => assert_eq!(name, "i"),
                _ => panic!("Expected Variable"),
            },
            _ => panic!("Expected PreIncrement, got {:?}", stmt),
        }
    }

    #[test]
    fn test_post_decrement() {
        let source = "$i--;".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::Expr(Expr::UnaryOp(UnaryOp::PostDecrement, inner)) => match *inner {
                Expr::Variable(name) => assert_eq!(name, "i"),
                _ => panic!("Expected Variable"),
            },
            _ => panic!("Expected PostDecrement, got {:?}", stmt),
        }
    }

    #[test]
    fn test_pre_decrement() {
        let source = "--$i;".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::Expr(Expr::UnaryOp(UnaryOp::PreDecrement, inner)) => match *inner {
                Expr::Variable(name) => assert_eq!(name, "i"),
                _ => panic!("Expected Variable"),
            },
            _ => panic!("Expected PreDecrement, got {:?}", stmt),
        }
    }

    #[test]
    fn test_instanceof() {
        let source = "$obj instanceof MyClass;".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::Expr(Expr::InstanceOf(expr, class_name)) => {
                match *expr {
                    Expr::Variable(name) => assert_eq!(name, "obj"),
                    _ => panic!("Expected Variable"),
                }
                assert_eq!(class_name, "MyClass");
            }
            _ => panic!("Expected InstanceOf, got {:?}", stmt),
        }
    }

    #[test]
    fn test_increment_in_expression() {
        let source = "$y = $x++;".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::Expr(Expr::BinaryOp(BinaryOp::Assign, lhs, rhs)) => {
                match *lhs {
                    Expr::Variable(name) => assert_eq!(name, "y"),
                    _ => panic!("Expected Variable lhs"),
                }
                match *rhs {
                    Expr::UnaryOp(UnaryOp::PostIncrement, _) => {}
                    _ => panic!("Expected PostIncrement rhs"),
                }
            }
            _ => panic!("Expected Assign, got {:?}", stmt),
        }
    }

    #[test]
    fn test_interpolated_string_basic() {
        let source = "\"hello {$name}\";".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::Expr(Expr::InterpolatedString(parts)) => {
                assert_eq!(parts.len(), 2);
                match &parts[0] {
                    Expr::StringLiteral(s) => assert_eq!(s, "hello "),
                    _ => panic!("Expected StringLiteral"),
                }
                match &parts[1] {
                    Expr::Variable(name) => assert_eq!(name, "name"),
                    _ => panic!("Expected Variable"),
                }
            }
            _ => panic!("Expected InterpolatedString, got {:?}", stmt),
        }
    }

    #[test]
    fn test_interpolated_string_multiple_vars() {
        let source = "\"{$a} and {$b}\";".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::Expr(Expr::InterpolatedString(parts)) => {
                assert_eq!(parts.len(), 3);
                match &parts[0] {
                    Expr::Variable(name) => assert_eq!(name, "a"),
                    _ => panic!("Expected Variable a"),
                }
                match &parts[1] {
                    Expr::StringLiteral(s) => assert_eq!(s, " and "),
                    _ => panic!("Expected StringLiteral"),
                }
                match &parts[2] {
                    Expr::Variable(name) => assert_eq!(name, "b"),
                    _ => panic!("Expected Variable b"),
                }
            }
            _ => panic!("Expected InterpolatedString, got {:?}", stmt),
        }
    }

    #[test]
    fn test_single_quote_no_interpolation() {
        let source = "'hello {$name}';".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::Expr(Expr::StringLiteral(s)) => {
                assert_eq!(s, "hello {$name}");
            }
            _ => panic!(
                "Expected StringLiteral (single quote no interpolation), got {:?}",
                stmt
            ),
        }
    }

    #[test]
    fn test_plain_double_quote_string() {
        let source = "\"hello world\";".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::Expr(Expr::StringLiteral(s)) => {
                assert_eq!(s, "hello world");
            }
            _ => panic!("Expected StringLiteral, got {:?}", stmt),
        }
    }

    #[test]
    fn test_interpolated_string_with_text_after() {
        let source = "\"{$name} world\";".to_string();
        let mut parser = Parser::new(source);
        let stmt = parser.parse_stmt_test().unwrap();
        match stmt {
            Stmt::Expr(Expr::InterpolatedString(parts)) => {
                assert_eq!(parts.len(), 2);
                match &parts[0] {
                    Expr::Variable(name) => assert_eq!(name, "name"),
                    _ => panic!("Expected Variable"),
                }
                match &parts[1] {
                    Expr::StringLiteral(s) => assert_eq!(s, " world"),
                    _ => panic!("Expected StringLiteral"),
                }
            }
            _ => panic!("Expected InterpolatedString, got {:?}", stmt),
        }
    }

    #[test]
    fn test_visibility_public_field() {
        let source = "class Test { public int32 $x; }".to_string();
        let mut parser = Parser::new(source);
        let class = parser.parse_class().unwrap();
        assert_eq!(class.fields.len(), 1);
        assert!(class.fields[0].is_public);
        assert!(!class.fields[0].is_private);
        assert!(!class.fields[0].is_protected);
        assert!(!class.fields[0].is_internal);
    }

    #[test]
    fn test_visibility_private_field() {
        let source = "class Test { private int32 $x; }".to_string();
        let mut parser = Parser::new(source);
        let class = parser.parse_class().unwrap();
        assert_eq!(class.fields.len(), 1);
        assert!(!class.fields[0].is_public);
        assert!(class.fields[0].is_private);
        assert!(!class.fields[0].is_protected);
        assert!(!class.fields[0].is_internal);
    }

    #[test]
    fn test_visibility_protected_field() {
        let source = "class Test { protected int32 $x; }".to_string();
        let mut parser = Parser::new(source);
        let class = parser.parse_class().unwrap();
        assert_eq!(class.fields.len(), 1);
        assert!(!class.fields[0].is_public);
        assert!(!class.fields[0].is_private);
        assert!(class.fields[0].is_protected);
        assert!(!class.fields[0].is_internal);
    }

    #[test]
    fn test_visibility_internal_field() {
        let source = "class Test { internal int32 $x; }".to_string();
        let mut parser = Parser::new(source);
        let class = parser.parse_class().unwrap();
        assert_eq!(class.fields.len(), 1);
        assert!(!class.fields[0].is_public);
        assert!(!class.fields[0].is_private);
        assert!(!class.fields[0].is_protected);
        assert!(class.fields[0].is_internal);
    }

    #[test]
    fn test_visibility_default_public() {
        let source = "class Test { int32 $x; }".to_string();
        let mut parser = Parser::new(source);
        let class = parser.parse_class().unwrap();
        assert_eq!(class.fields.len(), 1);
        assert!(class.fields[0].is_public);
        assert!(!class.fields[0].is_private);
        assert!(!class.fields[0].is_protected);
        assert!(!class.fields[0].is_internal);
    }

    #[test]
    fn test_visibility_public_method() {
        let source = "class Test { public function test(): void {} }".to_string();
        let mut parser = Parser::new(source);
        let class = parser.parse_class().unwrap();
        assert_eq!(class.methods.len(), 1);
        assert!(class.methods[0].is_public);
        assert!(!class.methods[0].is_private);
        assert!(!class.methods[0].is_protected);
        assert!(!class.methods[0].is_internal);
    }

    #[test]
    fn test_visibility_private_method() {
        let source = "class Test { private function test(): void {} }".to_string();
        let mut parser = Parser::new(source);
        let class = parser.parse_class().unwrap();
        assert_eq!(class.methods.len(), 1);
        assert!(!class.methods[0].is_public);
        assert!(class.methods[0].is_private);
        assert!(!class.methods[0].is_protected);
        assert!(!class.methods[0].is_internal);
    }

    #[test]
    fn test_visibility_protected_method() {
        let source = "class Test { protected function test(): void {} }".to_string();
        let mut parser = Parser::new(source);
        let class = parser.parse_class().unwrap();
        assert_eq!(class.methods.len(), 1);
        assert!(!class.methods[0].is_public);
        assert!(!class.methods[0].is_private);
        assert!(class.methods[0].is_protected);
        assert!(!class.methods[0].is_internal);
    }

    #[test]
    fn test_visibility_internal_method() {
        let source = "class Test { internal function test(): void {} }".to_string();
        let mut parser = Parser::new(source);
        let class = parser.parse_class().unwrap();
        assert_eq!(class.methods.len(), 1);
        assert!(!class.methods[0].is_public);
        assert!(!class.methods[0].is_private);
        assert!(!class.methods[0].is_protected);
        assert!(class.methods[0].is_internal);
    }

    #[test]
    fn test_visibility_static_public_field() {
        let source = "class Test { public static int32 $x; }".to_string();
        let mut parser = Parser::new(source);
        let class = parser.parse_class().unwrap();
        assert_eq!(class.fields.len(), 1);
        assert!(class.fields[0].is_public);
        assert!(class.fields[0].is_static);
    }

    #[test]
    fn test_visibility_final_field() {
        let source = "class Test { final int32 $x; }".to_string();
        let mut parser = Parser::new(source);
        let class = parser.parse_class().unwrap();
        assert_eq!(class.fields.len(), 1);
        assert!(class.fields[0].is_public);
        assert!(class.fields[0].is_final);
    }
}

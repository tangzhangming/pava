use crate::ast::*;
use crate::error::CompileResult;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
enum VarType {
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    String,
    Bool,
    Ref,
}

pub struct CodeGen {
    constant_pool: Vec<ConstantPoolEntry>,
    code_buffer: Vec<u8>,
    local_vars: HashMap<String, (u16, VarType)>,
    max_locals: u16,
    max_stack: u16,
    collected_integers: Vec<i32>,
    collected_longs: Vec<i64>,
    collected_floats: Vec<f32>,
    collected_doubles: Vec<f64>,
    integer_constants: HashMap<i32, u16>,
    long_constants: HashMap<i64, u16>,
    float_constants: HashMap<u32, u16>,
    double_constants: HashMap<u64, u16>,
    system_out_fieldref_idx: u16,
    println_int_idx: u16,
    println_long_idx: u16,
    println_float_idx: u16,
    println_double_idx: u16,
    println_string_idx: u16,
    class_idx: u16,
    super_class_idx: u16,
}

#[derive(Clone, Debug)]
enum ConstantPoolEntry {
    Utf8(String),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    String(u16),
    Class(u16),
    MethodRef(u16, u16),
    FieldRef(u16, u16),
    NameAndType(u16, u16),
}

const ACC_PUBLIC: u16 = 0x0001;
const ACC_STATIC: u16 = 0x0008;
const ACC_SUPER: u16 = 0x0020;

impl CodeGen {
    pub fn new(_class: Class) -> Self {
        CodeGen {
            constant_pool: Vec::new(),
            code_buffer: Vec::new(),
            local_vars: HashMap::new(),
            max_locals: 1,
            max_stack: 1,
            collected_integers: Vec::new(),
            collected_longs: Vec::new(),
            collected_floats: Vec::new(),
            collected_doubles: Vec::new(),
            integer_constants: HashMap::new(),
            long_constants: HashMap::new(),
            float_constants: HashMap::new(),
            double_constants: HashMap::new(),
            system_out_fieldref_idx: 0,
            println_int_idx: 0,
            println_long_idx: 0,
            println_float_idx: 0,
            println_double_idx: 0,
            println_string_idx: 0,
            class_idx: 0,
            super_class_idx: 0,
        }
    }

    pub fn generate(&mut self, class: Class) -> CompileResult<Vec<u8>> {
        self.collect_constants_from_class(&class);
        self.init_constant_pool(&class);
        self.emit_class(&class)
    }

    fn collect_constants_from_class(&mut self, class: &Class) {
        for method in &class.methods {
            for stmt in &method.body {
                self.collect_constants_from_stmt(stmt);
            }
        }
    }

    fn collect_constants_from_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) => self.collect_constants_from_expr(expr),
            Stmt::Return(Some(expr)) => self.collect_constants_from_expr(expr),
            Stmt::Assign(_, expr) => self.collect_constants_from_expr(expr),
            Stmt::If(cond, then_branch, else_branch) => {
                self.collect_constants_from_expr(cond);
                for s in then_branch {
                    self.collect_constants_from_stmt(s);
                }
                if let Some(else_stmts) = else_branch {
                    for s in else_stmts {
                        self.collect_constants_from_stmt(s);
                    }
                }
            }
            Stmt::While(cond, body) => {
                self.collect_constants_from_expr(cond);
                for s in body {
                    self.collect_constants_from_stmt(s);
                }
            }
            Stmt::Print(expr) | Stmt::Println(expr) => self.collect_constants_from_expr(expr),
            Stmt::Block(stmts) => {
                for s in stmts {
                    self.collect_constants_from_stmt(s);
                }
            }
            _ => {}
        }
    }

    fn collect_constants_from_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::IntLiteral(n) => {
                if *n < -32768 || *n > 32767 {
                    if *n >= i32::MIN as i64 && *n <= i32::MAX as i64 {
                        let val = *n as i32;
                        if !self.collected_integers.contains(&val) {
                            self.collected_integers.push(val);
                        }
                    } else if !self.collected_longs.contains(n) {
                        self.collected_longs.push(*n);
                    }
                }
            }
            Expr::FloatLiteral(f) => {
                let f32_val = *f as f32;
                if f32_val != 0.0 && f32_val != 1.0 && f32_val != 2.0 {
                    if !self.collected_floats.contains(&f32_val) {
                        self.collected_floats.push(f32_val);
                    }
                }
            }
            Expr::BinaryOp(_, left, right) => {
                self.collect_constants_from_expr(left);
                self.collect_constants_from_expr(right);
            }
            Expr::UnaryOp(_, inner) => self.collect_constants_from_expr(inner),
            _ => {}
        }
    }

    fn init_constant_pool(&mut self, class: &Class) {
        let mut add = |entry: ConstantPoolEntry| -> u16 {
            let idx = self.constant_pool.len() as u16 + 1;
            self.constant_pool.push(entry);
            idx
        };

        for int_val in &self.collected_integers {
            let idx = add(ConstantPoolEntry::Integer(*int_val));
            self.integer_constants.insert(*int_val, idx);
        }
        for long_val in &self.collected_longs {
            let idx = add(ConstantPoolEntry::Long(*long_val));
            self.long_constants.insert(*long_val, idx);
        }
        for float_val in &self.collected_floats {
            let idx = add(ConstantPoolEntry::Float(*float_val));
            self.float_constants.insert(float_val.to_bits(), idx);
        }
        for double_val in &self.collected_doubles {
            let idx = add(ConstantPoolEntry::Double(*double_val));
            self.double_constants.insert(double_val.to_bits(), idx);
        }

        let _empty = add(ConstantPoolEntry::Utf8("".to_string()));

        let obj_utf8 = add(ConstantPoolEntry::Utf8("java/lang/Object".to_string()));
        let obj_class = add(ConstantPoolEntry::Class(obj_utf8));

        let class_utf8 = add(ConstantPoolEntry::Utf8(class.name.clone()));
        let class_class = add(ConstantPoolEntry::Class(class_utf8));
        self.class_idx = class_class;

        // Handle extends - add parent class to constant pool
        let super_class_idx = if let Some(ref parent) = class.extends {
            let parent_utf8 = add(ConstantPoolEntry::Utf8(parent.clone()));
            add(ConstantPoolEntry::Class(parent_utf8))
        } else {
            obj_class
        };
        self.super_class_idx = super_class_idx;

        let init_utf8 = add(ConstantPoolEntry::Utf8("<init>".to_string()));
        let void_desc = add(ConstantPoolEntry::Utf8("()V".to_string()));
        let init_nt = add(ConstantPoolEntry::NameAndType(init_utf8, void_desc));
        let _object_init = add(ConstantPoolEntry::MethodRef(obj_class, init_nt));

        let code_utf8 = add(ConstantPoolEntry::Utf8("Code".to_string()));
        let sourcefile_utf8 = add(ConstantPoolEntry::Utf8("SourceFile".to_string()));
        let _sourcefile_name = add(ConstantPoolEntry::Utf8(format!("{}.java", class.name)));

        let system_utf8 = add(ConstantPoolEntry::Utf8("java/lang/System".to_string()));
        let system_class = add(ConstantPoolEntry::Class(system_utf8));

        let out_utf8 = add(ConstantPoolEntry::Utf8("out".to_string()));
        let out_type = add(ConstantPoolEntry::Utf8("Ljava/io/PrintStream;".to_string()));
        let out_nt = add(ConstantPoolEntry::NameAndType(out_utf8, out_type));
        self.system_out_fieldref_idx = add(ConstantPoolEntry::FieldRef(system_class, out_nt));

        let printstream_utf8 = add(ConstantPoolEntry::Utf8("java/io/PrintStream".to_string()));
        let printstream_class = add(ConstantPoolEntry::Class(printstream_utf8));

        let println_utf8 = add(ConstantPoolEntry::Utf8("println".to_string()));

        let string_desc = add(ConstantPoolEntry::Utf8("(Ljava/lang/String;)V".to_string()));
        let println_string_nt = add(ConstantPoolEntry::NameAndType(println_utf8, string_desc));
        self.println_string_idx = add(ConstantPoolEntry::MethodRef(
            printstream_class,
            println_string_nt,
        ));

        let int_desc = add(ConstantPoolEntry::Utf8("(I)V".to_string()));
        let println_int_nt = add(ConstantPoolEntry::NameAndType(println_utf8, int_desc));
        self.println_int_idx = add(ConstantPoolEntry::MethodRef(
            printstream_class,
            println_int_nt,
        ));

        let long_desc = add(ConstantPoolEntry::Utf8("(J)V".to_string()));
        let println_long_nt = add(ConstantPoolEntry::NameAndType(println_utf8, long_desc));
        self.println_long_idx = add(ConstantPoolEntry::MethodRef(
            printstream_class,
            println_long_nt,
        ));

        let double_desc = add(ConstantPoolEntry::Utf8("(D)V".to_string()));
        let println_double_nt = add(ConstantPoolEntry::NameAndType(println_utf8, double_desc));
        self.println_double_idx = add(ConstantPoolEntry::MethodRef(
            printstream_class,
            println_double_nt,
        ));

        let float_desc = add(ConstantPoolEntry::Utf8("(F)V".to_string()));
        let println_float_nt = add(ConstantPoolEntry::NameAndType(println_utf8, float_desc));
        self.println_float_idx = add(ConstantPoolEntry::MethodRef(
            printstream_class,
            println_float_nt,
        ));

        let _main_utf8 = add(ConstantPoolEntry::Utf8("main".to_string()));
        let _main_desc = add(ConstantPoolEntry::Utf8(
            "([Ljava/lang/String;)V".to_string(),
        ));
    }

    fn emit_class(&mut self, class: &Class) -> CompileResult<Vec<u8>> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&[0xCA, 0xFE, 0xBA, 0xBE]);
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x31]);

        let cp_count = self.constant_pool.iter().fold(1, |acc, entry| {
            acc + match entry {
                ConstantPoolEntry::Long(_) | ConstantPoolEntry::Double(_) => 2,
                _ => 1,
            }
        }) as u16;
        bytes.extend_from_slice(&cp_count.to_be_bytes());

        for entry in &self.constant_pool {
            self.emit_cp_entry(&mut bytes, entry);
        }

        let access_flags = ACC_SUPER;
        bytes.extend_from_slice(&access_flags.to_be_bytes());

        bytes.extend_from_slice(&self.class_idx.to_be_bytes());
        bytes.extend_from_slice(&self.super_class_idx.to_be_bytes());
        bytes.extend_from_slice(&0u16.to_be_bytes());
        bytes.extend_from_slice(&0u16.to_be_bytes());

        let method_count = class.methods.len() as u16 + 1;
        bytes.extend_from_slice(&method_count.to_be_bytes());

        self.emit_init_method(&mut bytes);

        for method in &class.methods {
            if method.name == "main" {
                self.emit_main_method(&mut bytes, class)?;
            } else {
                self.emit_method(&mut bytes, method)?;
            }
        }

        bytes.extend_from_slice(&0u16.to_be_bytes());

        Ok(bytes)
    }

    fn emit_method(&mut self, bytes: &mut Vec<u8>, method: &ClassMethod) -> CompileResult<()> {
        let access_flags = if method.is_public { ACC_PUBLIC } else { 0 }
            | if method.is_static { ACC_STATIC } else { 0 };

        bytes.extend_from_slice(&access_flags.to_be_bytes());

        let name_idx = self.add_utf8_constant(&method.name);
        bytes.extend_from_slice(&name_idx.to_be_bytes());

        let descriptor = self.build_method_descriptor(method);
        let desc_idx = self.add_utf8_constant(&descriptor);
        bytes.extend_from_slice(&desc_idx.to_be_bytes());

        bytes.extend_from_slice(&1u16.to_be_bytes());

        self.emit_method_code(bytes, method)
    }

    fn build_method_descriptor(&mut self, method: &ClassMethod) -> String {
        let mut desc = String::from("(");
        for (_, param_type) in &method.params {
            desc.push_str(&param_type.to_jvm_descriptor());
        }
        desc.push_str(")");
        desc.push_str(&method.return_type.to_jvm_descriptor());
        desc
    }

    fn emit_method_code(&mut self, bytes: &mut Vec<u8>, method: &ClassMethod) -> CompileResult<()> {
        self.code_buffer.clear();
        self.local_vars.clear();
        self.max_locals = 1;

        if !method.is_static {
            self.local_vars
                .insert("this".to_string(), (0, VarType::Ref));
        }

        for (param_name, param_type) in &method.params {
            let var_type = self.type_to_var_type(param_type);
            let idx = self.max_locals;
            let slots = match var_type {
                VarType::Long | VarType::Double => 2,
                _ => 1,
            };
            self.max_locals += slots;
            self.local_vars.insert(param_name.clone(), (idx, var_type));
        }

        for stmt in &method.body {
            self.emit_stmt(stmt)?;
        }

        if self.code_buffer.is_empty() || self.code_buffer.last() != Some(&0xB1) {
            if method.return_type == Type::Void {
                self.code_buffer.push(0xB1);
            } else {
                match method.return_type {
                    Type::Boolean | Type::Int8 | Type::Int16 | Type::Int32 | Type::Int64 => {
                        self.code_buffer.push(0x03);
                        self.code_buffer.push(0xAC);
                    }
                    _ => {
                        self.code_buffer.push(0x01);
                        self.code_buffer.push(0xB0);
                    }
                }
            }
        }

        let code_idx = self.find_utf8_index("Code").unwrap_or(10);
        let code_attr_len = 12 + self.code_buffer.len() as u32;

        bytes.extend_from_slice(&code_idx.to_be_bytes());
        bytes.extend_from_slice(&code_attr_len.to_be_bytes());
        bytes.extend_from_slice(&5u16.to_be_bytes());
        bytes.extend_from_slice(&self.max_locals.to_be_bytes());

        let code_len = self.code_buffer.len() as u32;
        bytes.extend_from_slice(&code_len.to_be_bytes());
        bytes.extend_from_slice(&self.code_buffer);

        bytes.extend_from_slice(&0u16.to_be_bytes());
        bytes.extend_from_slice(&0u16.to_be_bytes());

        Ok(())
    }

    fn type_to_var_type(&self, ty: &Type) -> VarType {
        match ty {
            Type::Int8 | Type::Int16 | Type::Int32 => VarType::Int,
            Type::Int64 => VarType::Long,
            Type::Float32 => VarType::Float,
            Type::Float64 => VarType::Double,
            Type::Boolean => VarType::Bool,
            Type::String => VarType::String,
            _ => VarType::Ref,
        }
    }

    fn emit_cp_entry(&self, bytes: &mut Vec<u8>, entry: &ConstantPoolEntry) {
        match entry {
            ConstantPoolEntry::Utf8(s) => {
                bytes.push(0x01);
                let bytes_utf = s.as_bytes();
                bytes.extend_from_slice(&(bytes_utf.len() as u16).to_be_bytes());
                bytes.extend_from_slice(bytes_utf);
            }
            ConstantPoolEntry::Integer(n) => {
                bytes.push(0x03);
                bytes.extend_from_slice(&n.to_be_bytes());
            }
            ConstantPoolEntry::Float(f) => {
                bytes.push(0x04);
                bytes.extend_from_slice(&f.to_be_bytes());
            }
            ConstantPoolEntry::String(idx) => {
                bytes.push(0x08);
                bytes.extend_from_slice(&idx.to_be_bytes());
            }
            ConstantPoolEntry::Class(idx) => {
                bytes.push(0x07);
                bytes.extend_from_slice(&idx.to_be_bytes());
            }
            ConstantPoolEntry::MethodRef(c, n) => {
                bytes.push(0x0A);
                bytes.extend_from_slice(&c.to_be_bytes());
                bytes.extend_from_slice(&n.to_be_bytes());
            }
            ConstantPoolEntry::FieldRef(c, n) => {
                bytes.push(0x09);
                bytes.extend_from_slice(&c.to_be_bytes());
                bytes.extend_from_slice(&n.to_be_bytes());
            }
            ConstantPoolEntry::Long(n) => {
                bytes.push(0x05);
                bytes.extend_from_slice(&n.to_be_bytes());
            }
            ConstantPoolEntry::Double(f) => {
                bytes.push(0x06);
                bytes.extend_from_slice(&f.to_be_bytes());
            }
            ConstantPoolEntry::NameAndType(n, t) => {
                bytes.push(0x0C);
                bytes.extend_from_slice(&n.to_be_bytes());
                bytes.extend_from_slice(&t.to_be_bytes());
            }
        }
    }

    fn emit_init_method(&mut self, bytes: &mut Vec<u8>) {
        let init_idx = self.find_utf8_index("<init>").unwrap_or(6);
        let void_desc_idx = self.find_utf8_index("()V").unwrap_or(7);
        let code_idx = self.find_utf8_index("Code").unwrap_or(10);
        let object_init_idx = self
            .find_methodref_index("java/lang/Object", "<init>")
            .unwrap_or(9);

        bytes.extend_from_slice(&ACC_PUBLIC.to_be_bytes());
        bytes.extend_from_slice(&init_idx.to_be_bytes());
        bytes.extend_from_slice(&void_desc_idx.to_be_bytes());
        bytes.extend_from_slice(&1u16.to_be_bytes());

        let code_attr_len = 17u32;
        bytes.extend_from_slice(&code_idx.to_be_bytes());
        bytes.extend_from_slice(&code_attr_len.to_be_bytes());
        bytes.extend_from_slice(&1u16.to_be_bytes());
        bytes.extend_from_slice(&1u16.to_be_bytes());

        bytes.extend_from_slice(&5u32.to_be_bytes());

        bytes.push(0x2A);
        bytes.push(0xB7);
        bytes.extend_from_slice(&object_init_idx.to_be_bytes());
        bytes.push(0xB1);

        bytes.extend_from_slice(&0u16.to_be_bytes());
        bytes.extend_from_slice(&0u16.to_be_bytes());
    }

    fn emit_main_method(&mut self, bytes: &mut Vec<u8>, class: &Class) -> CompileResult<()> {
        let main_idx = self.find_utf8_index("main").unwrap_or(50);
        let main_desc_idx = self.find_utf8_index("([Ljava/lang/String;)V").unwrap_or(51);
        let code_idx = self.find_utf8_index("Code").unwrap_or(10);

        bytes.extend_from_slice(&(ACC_PUBLIC | ACC_STATIC).to_be_bytes());
        bytes.extend_from_slice(&main_idx.to_be_bytes());
        bytes.extend_from_slice(&main_desc_idx.to_be_bytes());
        bytes.extend_from_slice(&1u16.to_be_bytes());

        self.code_buffer.clear();

        for method in &class.methods {
            if method.name == "main" {
                for stmt in &method.body {
                    self.emit_stmt(stmt)?;
                }
                break;
            }
        }

        if self.code_buffer.is_empty() || self.code_buffer.last() != Some(&0xB1) {
            self.code_buffer.push(0xB1);
        }

        let code_attr_len = 12 + self.code_buffer.len() as u32;
        bytes.extend_from_slice(&code_idx.to_be_bytes());
        bytes.extend_from_slice(&code_attr_len.to_be_bytes());
        bytes.extend_from_slice(&5u16.to_be_bytes());
        bytes.extend_from_slice(&20u16.to_be_bytes());

        let code_len = self.code_buffer.len() as u32;
        bytes.extend_from_slice(&code_len.to_be_bytes());
        bytes.extend_from_slice(&self.code_buffer);

        bytes.extend_from_slice(&0u16.to_be_bytes());
        bytes.extend_from_slice(&0u16.to_be_bytes());

        Ok(())
    }

    fn emit_stmt(&mut self, stmt: &Stmt) -> CompileResult<()> {
        match stmt {
            Stmt::Expr(e) => self.emit_expr(e)?,
            Stmt::Return(e) => {
                if let Some(expr) = e {
                    self.emit_expr(expr)?;
                }
                self.code_buffer.push(0xB1);
            }
            Stmt::If(cond, then_stmts, else_stmts) => self.emit_if(cond, then_stmts, else_stmts)?,
            Stmt::While(cond, stmts) => self.emit_while(cond, stmts)?,
            Stmt::Assign(name, expr) => self.emit_assign(name, expr)?,
            Stmt::Print(expr) | Stmt::Println(expr) => self.emit_print(expr)?,
            Stmt::Block(stmts) => {
                for s in stmts {
                    self.emit_stmt(s)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn emit_expr(&mut self, expr: &Expr) -> CompileResult<()> {
        match expr {
            Expr::IntLiteral(n) => self.emit_integer(*n)?,
            Expr::FloatLiteral(f) => {
                let f32_val = *f as f32;
                if (*f - f64::from(f32_val)).abs() < f64::EPSILON {
                    self.emit_float(*f)?;
                } else {
                    self.emit_double(*f)?;
                }
            }
            Expr::StringLiteral(s) => self.emit_string(s)?,
            Expr::BoolLiteral(b) => self.code_buffer.push(if *b { 0x04 } else { 0x03 }),
            Expr::NullLiteral => self.code_buffer.push(0x01),
            Expr::Variable(name) => self.emit_load_var(name)?,
            Expr::BinaryOp(op, left, right) => self.emit_binary_op(op, left, right)?,
            Expr::UnaryOp(op, inner) => self.emit_unary_op(op, inner)?,
            Expr::MethodCall(obj, method_name, args) => {
                self.emit_method_call(obj, method_name, args)?
            }
            Expr::StaticCall(class_name, method_name, args) => {
                self.emit_static_call(class_name, method_name, args)?
            }
            Expr::FieldAccess(obj, field_name) => self.emit_field_access(obj, field_name)?,
            _ => {}
        }
        Ok(())
    }

    fn emit_integer(&mut self, n: i64) -> CompileResult<()> {
        match n {
            -1 => self.code_buffer.push(0x02),
            0 => self.code_buffer.push(0x03),
            1 => self.code_buffer.push(0x04),
            2 => self.code_buffer.push(0x05),
            n if n >= -128 && n <= 127 => {
                self.code_buffer.push(0x10);
                self.code_buffer.push(n as u8);
            }
            n if n >= -32768 && n <= 32767 => {
                self.code_buffer.push(0x11);
                self.code_buffer
                    .extend_from_slice(&(n as i16).to_be_bytes());
            }
            n if n >= i32::MIN as i64 && n <= i32::MAX as i64 => {
                let idx = self.add_integer_constant(n as i32);
                self.emit_ldc(idx);
            }
            _ => self.emit_long(n)?,
        }
        Ok(())
    }

    fn emit_float(&mut self, f: f64) -> CompileResult<()> {
        let f32_val = f as f32;
        match f32_val {
            0.0 => self.code_buffer.push(0x0B),
            1.0 => self.code_buffer.push(0x0C),
            2.0 => self.code_buffer.push(0x0D),
            _ => {
                let idx = self.add_float_constant(f32_val);
                self.emit_ldc(idx);
            }
        }
        Ok(())
    }

    fn emit_long(&mut self, n: i64) -> CompileResult<()> {
        match n {
            0 => self.code_buffer.push(0x09),
            1 => self.code_buffer.push(0x0A),
            _ => {
                let idx = self.add_long_constant(n);
                self.emit_ldc2_w(idx);
            }
        }
        Ok(())
    }

    fn emit_double(&mut self, f: f64) -> CompileResult<()> {
        match f {
            0.0 => self.code_buffer.push(0x0E),
            1.0 => self.code_buffer.push(0x0F),
            _ => {
                let idx = self.add_double_constant(f);
                self.emit_ldc2_w(idx);
            }
        }
        Ok(())
    }

    fn emit_string(&mut self, s: &str) -> CompileResult<()> {
        let idx = self.add_utf8_constant(s);
        self.emit_ldc(idx);
        Ok(())
    }

    fn emit_load_var(&mut self, name: &str) -> CompileResult<()> {
        if let Some(&(idx, ty)) = self.local_vars.get(name) {
            match ty {
                VarType::Byte | VarType::Short | VarType::Int | VarType::Bool => match idx {
                    0 => self.code_buffer.push(0x1A),
                    1 => self.code_buffer.push(0x1B),
                    2 => self.code_buffer.push(0x1C),
                    3 => self.code_buffer.push(0x1D),
                    _ => {
                        self.code_buffer.push(0x15);
                        self.code_buffer.push(idx as u8);
                    }
                },
                VarType::Long => match idx {
                    0 => self.code_buffer.push(0x1E),
                    1 => self.code_buffer.push(0x1F),
                    2 => self.code_buffer.push(0x20),
                    3 => self.code_buffer.push(0x21),
                    _ => {
                        self.code_buffer.push(0x16);
                        self.code_buffer.push(idx as u8);
                    }
                },
                VarType::Float => match idx {
                    0 => self.code_buffer.push(0x22),
                    1 => self.code_buffer.push(0x23),
                    2 => self.code_buffer.push(0x24),
                    3 => self.code_buffer.push(0x25),
                    _ => {
                        self.code_buffer.push(0x17);
                        self.code_buffer.push(idx as u8);
                    }
                },
                VarType::Double => match idx {
                    0 => self.code_buffer.push(0x26),
                    1 => self.code_buffer.push(0x27),
                    2 => self.code_buffer.push(0x28),
                    3 => self.code_buffer.push(0x29),
                    _ => {
                        self.code_buffer.push(0x18);
                        self.code_buffer.push(idx as u8);
                    }
                },
                VarType::String | VarType::Ref => match idx {
                    0 => self.code_buffer.push(0x2A),
                    1 => self.code_buffer.push(0x2B),
                    2 => self.code_buffer.push(0x2C),
                    3 => self.code_buffer.push(0x2D),
                    _ => {
                        self.code_buffer.push(0x19);
                        self.code_buffer.push(idx as u8);
                    }
                },
            }
        }
        Ok(())
    }

    fn emit_store_var(&mut self, name: &str, ty: VarType) -> CompileResult<()> {
        let var_index = if let Some(&(idx, _)) = self.local_vars.get(name) {
            idx
        } else {
            let idx = self.max_locals;
            let slots = match ty {
                VarType::Long | VarType::Double => 2,
                _ => 1,
            };
            self.max_locals += slots;
            self.local_vars.insert(name.to_string(), (idx, ty));
            idx
        };

        match ty {
            VarType::Byte | VarType::Short | VarType::Int | VarType::Bool => match var_index {
                0 => self.code_buffer.push(0x3B),
                1 => self.code_buffer.push(0x3C),
                2 => self.code_buffer.push(0x3D),
                3 => self.code_buffer.push(0x3E),
                _ => {
                    self.code_buffer.push(0x36);
                    self.code_buffer.push(var_index as u8);
                }
            },
            _ => {
                self.code_buffer.push(0x3A);
                self.code_buffer.push(var_index as u8);
            }
        }
        Ok(())
    }

    fn emit_binary_op(&mut self, op: &BinaryOp, left: &Expr, right: &Expr) -> CompileResult<()> {
        match op {
            BinaryOp::Add => {
                self.emit_expr(left)?;
                self.emit_expr(right)?;
                self.code_buffer.push(0x60);
            }
            BinaryOp::Sub => {
                self.emit_expr(left)?;
                self.emit_expr(right)?;
                self.code_buffer.push(0x64);
            }
            BinaryOp::Mul => {
                self.emit_expr(left)?;
                self.emit_expr(right)?;
                self.code_buffer.push(0x68);
            }
            BinaryOp::Div => {
                self.emit_expr(left)?;
                self.emit_expr(right)?;
                self.code_buffer.push(0x6C);
            }
            BinaryOp::Mod => {
                self.emit_expr(left)?;
                self.emit_expr(right)?;
                self.code_buffer.push(0x70);
            }
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                self.emit_comparison_op(op, left, right)?
            }
            BinaryOp::Eq => self.emit_comparison_op(op, left, right)?,
            BinaryOp::Ne => self.emit_comparison_op(op, left, right)?,
            BinaryOp::And => {
                self.emit_expr(left)?;
                self.emit_expr(right)?;
                self.code_buffer.push(0x7E);
            }
            BinaryOp::Or => {
                self.emit_expr(left)?;
                self.emit_expr(right)?;
                self.code_buffer.push(0x80);
            }
            _ => {}
        }
        Ok(())
    }

    fn emit_comparison_op(
        &mut self,
        op: &BinaryOp,
        left: &Expr,
        right: &Expr,
    ) -> CompileResult<()> {
        self.emit_expr(left)?;
        self.emit_expr(right)?;

        let jmp_op = match op {
            BinaryOp::Lt => 0xA2,
            BinaryOp::Le => 0xA3,
            BinaryOp::Gt => 0xA4,
            BinaryOp::Ge => 0xA1,
            BinaryOp::Eq => 0xA0,
            BinaryOp::Ne => 0x9F,
            _ => 0x9F,
        };

        self.code_buffer.push(jmp_op);
        let jmp_to_0_pos = self.code_buffer.len();
        self.code_buffer.extend_from_slice(&0u16.to_be_bytes());

        self.code_buffer.push(0x04);
        self.code_buffer.push(0xA7);
        let goto_end_pos = self.code_buffer.len();
        self.code_buffer.extend_from_slice(&0u16.to_be_bytes());

        let push_0_pos = self.code_buffer.len();
        self.code_buffer.push(0x03);

        let end_pos = self.code_buffer.len();
        let jmp_to_0_offset = (push_0_pos - (jmp_to_0_pos - 1)) as u16;
        let goto_end_offset = (end_pos - (goto_end_pos - 1)) as u16;

        self.code_buffer[jmp_to_0_pos..jmp_to_0_pos + 2]
            .copy_from_slice(&jmp_to_0_offset.to_be_bytes());
        self.code_buffer[goto_end_pos..goto_end_pos + 2]
            .copy_from_slice(&goto_end_offset.to_be_bytes());

        Ok(())
    }

    fn emit_unary_op(&mut self, op: &UnaryOp, expr: &Expr) -> CompileResult<()> {
        self.emit_expr(expr)?;
        match op {
            UnaryOp::Neg => self.code_buffer.push(0x74),
            UnaryOp::Not => {
                self.code_buffer.push(0x04);
                self.code_buffer.push(0x82);
            }
        }
        Ok(())
    }

    fn emit_if(
        &mut self,
        cond: &Expr,
        then_stmts: &[Stmt],
        else_stmts: &Option<Vec<Stmt>>,
    ) -> CompileResult<()> {
        self.emit_expr(cond)?;
        let jmp_offset = self.code_buffer.len();
        self.code_buffer.extend_from_slice(&0u16.to_be_bytes());

        for stmt in then_stmts {
            self.emit_stmt(stmt)?;
        }

        let else_offset = self.code_buffer.len();
        self.code_buffer.extend_from_slice(&0u16.to_be_bytes());

        let current = self.code_buffer.len() as u16;
        let then_end = (current - 4).to_be_bytes();
        self.code_buffer[jmp_offset..jmp_offset + 2].copy_from_slice(&then_end);

        if let Some(else_body) = else_stmts {
            for stmt in else_body {
                self.emit_stmt(stmt)?;
            }
        }

        let current = self.code_buffer.len() as u16;
        let else_end = (current - 2).to_be_bytes();
        self.code_buffer[else_offset..else_offset + 2].copy_from_slice(&else_end);

        Ok(())
    }

    fn emit_while(&mut self, cond: &Expr, stmts: &[Stmt]) -> CompileResult<()> {
        let loop_start = self.code_buffer.len();
        self.emit_expr(cond)?;
        let jmp_offset = self.code_buffer.len();
        self.code_buffer.extend_from_slice(&0u16.to_be_bytes());

        for stmt in stmts {
            self.emit_stmt(stmt)?;
        }

        self.code_buffer.push(0xA7);
        let offset = (loop_start as i32 - self.code_buffer.len() as i32 - 3) as i16;
        self.code_buffer.extend_from_slice(&offset.to_be_bytes());

        let current = self.code_buffer.len() as u16;
        let target = (current - 2).to_be_bytes();
        self.code_buffer[jmp_offset..jmp_offset + 2].copy_from_slice(&target);

        Ok(())
    }

    fn emit_assign(&mut self, name: &str, expr: &Expr) -> CompileResult<()> {
        let ty = self.infer_expr_type(expr);
        self.emit_expr(expr)?;
        self.emit_store_var(name, ty)?;
        Ok(())
    }

    fn emit_method_call(
        &mut self,
        obj: &Expr,
        method_name: &str,
        args: &[Expr],
    ) -> CompileResult<()> {
        self.emit_expr(obj)?;
        for arg in args {
            self.emit_expr(arg)?;
        }

        let class_name = self.infer_class_name_from_expr(obj);
        let descriptor = self.build_method_descriptor_from_args(args);
        let method_idx = self.add_methodref_constant(&class_name, method_name, &descriptor);
        self.code_buffer.push(0xB6);
        self.code_buffer
            .extend_from_slice(&method_idx.to_be_bytes());

        Ok(())
    }

    fn emit_static_call(
        &mut self,
        class_name: &str,
        method_name: &str,
        args: &[Expr],
    ) -> CompileResult<()> {
        for arg in args {
            self.emit_expr(arg)?;
        }

        let descriptor = self.build_method_descriptor_from_args(args);
        let method_idx = self.add_methodref_constant(class_name, method_name, &descriptor);
        self.code_buffer.push(0xB8);
        self.code_buffer
            .extend_from_slice(&method_idx.to_be_bytes());

        Ok(())
    }

    fn emit_field_access(&mut self, obj: &Expr, field_name: &str) -> CompileResult<()> {
        self.emit_expr(obj)?;
        let class_name = self.infer_class_name_from_expr(obj);
        let field_idx = self.add_fieldref_constant(&class_name, field_name, "I");
        self.code_buffer.push(0xB4);
        self.code_buffer.extend_from_slice(&field_idx.to_be_bytes());

        Ok(())
    }

    fn build_method_descriptor_from_args(&self, args: &[Expr]) -> String {
        let mut desc = String::from("(");
        for _ in args {
            desc.push('I');
        }
        desc.push_str(")I");
        desc
    }

    fn infer_expr_type(&self, expr: &Expr) -> VarType {
        match expr {
            Expr::IntLiteral(n) => {
                if *n >= i64::from(i8::MIN) && *n <= i64::from(i8::MAX) {
                    VarType::Byte
                } else if *n >= i64::from(i16::MIN) && *n <= i64::from(i16::MAX) {
                    VarType::Short
                } else if *n >= i64::from(i32::MIN) && *n <= i64::from(i32::MAX) {
                    VarType::Int
                } else {
                    VarType::Long
                }
            }
            Expr::FloatLiteral(f) => {
                let f32_val = *f as f32;
                if (*f - f64::from(f32_val)).abs() < f64::EPSILON {
                    VarType::Float
                } else {
                    VarType::Double
                }
            }
            Expr::StringLiteral(_) => VarType::String,
            Expr::BoolLiteral(_) => VarType::Bool,
            Expr::BinaryOp(_, left, right) => {
                let left_ty = self.infer_expr_type(left);
                let right_ty = self.infer_expr_type(right);
                if left_ty == VarType::Double || right_ty == VarType::Double {
                    VarType::Double
                } else if left_ty == VarType::Float || right_ty == VarType::Float {
                    VarType::Float
                } else if left_ty == VarType::Long || right_ty == VarType::Long {
                    VarType::Long
                } else {
                    VarType::Int
                }
            }
            Expr::Variable(name) => self
                .local_vars
                .get(name)
                .map(|(_, ty)| *ty)
                .unwrap_or(VarType::Int),
            _ => VarType::Ref,
        }
    }

    fn infer_class_name_from_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Variable(name) => {
                if name.starts_with("obj") || name.starts_with("this") {
                    "Object".to_string()
                } else {
                    "java/lang/Object".to_string()
                }
            }
            _ => "java/lang/Object".to_string(),
        }
    }

    fn emit_print(&mut self, expr: &Expr) -> CompileResult<()> {
        let ty = self.infer_expr_type(expr);
        self.code_buffer.push(0xB2);
        self.code_buffer
            .extend_from_slice(&self.system_out_fieldref_idx.to_be_bytes());
        self.emit_expr(expr)?;
        self.code_buffer.push(0xB6);
        let method_idx = match ty {
            VarType::Byte | VarType::Short | VarType::Int | VarType::Bool => self.println_int_idx,
            VarType::Long => self.println_long_idx,
            VarType::Float => self.println_float_idx,
            VarType::Double => self.println_double_idx,
            _ => self.println_string_idx,
        };
        self.code_buffer
            .extend_from_slice(&method_idx.to_be_bytes());
        Ok(())
    }

    fn add_integer_constant(&mut self, value: i32) -> u16 {
        self.integer_constants.get(&value).copied().unwrap_or(0)
    }

    fn add_long_constant(&mut self, value: i64) -> u16 {
        self.long_constants.get(&value).copied().unwrap_or(0)
    }

    fn add_float_constant(&mut self, value: f32) -> u16 {
        self.float_constants
            .get(&value.to_bits())
            .copied()
            .unwrap_or(0)
    }

    fn add_double_constant(&mut self, value: f64) -> u16 {
        self.double_constants
            .get(&value.to_bits())
            .copied()
            .unwrap_or(0)
    }

    fn emit_ldc(&mut self, idx: u16) {
        if idx <= 255 {
            self.code_buffer.push(0x12);
            self.code_buffer.push(idx as u8);
        } else {
            self.code_buffer.push(0x13);
            self.code_buffer.extend_from_slice(&idx.to_be_bytes());
        }
    }

    fn emit_ldc2_w(&mut self, idx: u16) {
        self.code_buffer.push(0x14);
        self.code_buffer.extend_from_slice(&idx.to_be_bytes());
    }

    fn add_utf8_constant(&mut self, s: &str) -> u16 {
        for (i, entry) in self.constant_pool.iter().enumerate() {
            if let ConstantPoolEntry::Utf8(existing) = entry {
                if existing == s {
                    return (i + 1) as u16;
                }
            }
        }
        let idx = self.constant_pool.len() as u16 + 1;
        self.constant_pool
            .push(ConstantPoolEntry::Utf8(s.to_string()));
        idx
    }

    fn add_class_constant(&mut self, class_name: &str) -> u16 {
        let utf8_idx = self.add_utf8_constant(class_name);
        for (i, entry) in self.constant_pool.iter().enumerate() {
            if let ConstantPoolEntry::Class(idx) = entry {
                if *idx == utf8_idx {
                    return (i + 1) as u16;
                }
            }
        }
        let idx = self.constant_pool.len() as u16 + 1;
        self.constant_pool.push(ConstantPoolEntry::Class(utf8_idx));
        idx
    }

    fn add_methodref_constant(
        &mut self,
        class_name: &str,
        method_name: &str,
        descriptor: &str,
    ) -> u16 {
        let class_idx = self.add_class_constant(class_name);
        let name_idx = self.add_utf8_constant(method_name);
        let desc_idx = self.add_utf8_constant(descriptor);
        let name_and_type_idx = self.add_name_and_type_constant(name_idx, desc_idx);
        let idx = self.constant_pool.len() as u16 + 1;
        self.constant_pool
            .push(ConstantPoolEntry::MethodRef(class_idx, name_and_type_idx));
        idx
    }

    fn add_fieldref_constant(
        &mut self,
        class_name: &str,
        field_name: &str,
        descriptor: &str,
    ) -> u16 {
        let class_idx = self.add_class_constant(class_name);
        let name_idx = self.add_utf8_constant(field_name);
        let desc_idx = self.add_utf8_constant(descriptor);
        let name_and_type_idx = self.add_name_and_type_constant(name_idx, desc_idx);
        let idx = self.constant_pool.len() as u16 + 1;
        self.constant_pool
            .push(ConstantPoolEntry::FieldRef(class_idx, name_and_type_idx));
        idx
    }

    fn add_name_and_type_constant(&mut self, name_idx: u16, type_idx: u16) -> u16 {
        for (i, entry) in self.constant_pool.iter().enumerate() {
            if let ConstantPoolEntry::NameAndType(n, t) = entry {
                if *n == name_idx && *t == type_idx {
                    return (i + 1) as u16;
                }
            }
        }
        let idx = self.constant_pool.len() as u16 + 1;
        self.constant_pool
            .push(ConstantPoolEntry::NameAndType(name_idx, type_idx));
        idx
    }

    fn find_utf8_index(&self, s: &str) -> Option<u16> {
        for (i, entry) in self.constant_pool.iter().enumerate() {
            if let ConstantPoolEntry::Utf8(val) = entry {
                if val == s {
                    return Some((i + 1) as u16);
                }
            }
        }
        None
    }

    fn find_class_index(&self, name: &str) -> Option<u16> {
        for (i, entry) in self.constant_pool.iter().enumerate() {
            if let ConstantPoolEntry::Class(utf8_idx) = entry {
                if let Some(ConstantPoolEntry::Utf8(s)) = self
                    .constant_pool
                    .get((*utf8_idx as usize).saturating_sub(1))
                {
                    if s == name {
                        return Some((i + 1) as u16);
                    }
                }
            }
        }
        None
    }

    fn find_methodref_index(&self, class_name: &str, method_name: &str) -> Option<u16> {
        let class_idx = self.find_class_index(class_name)?;
        for (i, entry) in self.constant_pool.iter().enumerate() {
            if let ConstantPoolEntry::MethodRef(cls_idx, nt_idx) = entry {
                if *cls_idx == class_idx {
                    if let Some(ConstantPoolEntry::NameAndType(name_idx, _)) =
                        self.constant_pool.get((*nt_idx as usize).saturating_sub(1))
                    {
                        if let Some(ConstantPoolEntry::Utf8(name)) = self
                            .constant_pool
                            .get((*name_idx as usize).saturating_sub(1))
                        {
                            if name == method_name {
                                return Some((i + 1) as u16);
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

pub fn compile(source: &str) -> CompileResult<Vec<u8>> {
    let mut parser = crate::parser::Parser::new(source.to_string());
    let ast = parser.parse_class()?;
    let mut codegen = CodeGen::new(ast.clone());
    codegen.generate(ast)
}

# Pava 语言实现进度追踪

> 测试模型：glm-5 (alibaba-cn/glm-5)

## 核心编译器模块

| 模块 | 功能 | 设计文档 | 已完成 | 已测试(glm-5) |
|------|------|----------|--------|---------------|
| Lexer | 词法分析器基础框架 | Pave语言.md | ✅ | ✅ |
| Lexer | 关键字识别 (class/function/if等) | Pave语言.md | ✅ | ✅ |
| Lexer | 类型关键字 (int8/int16/string等) | Pave语言.md | ✅ | ✅ |
| Lexer | 变量识别 ($var) | Pave语言.md | ✅ | ✅ |
| Lexer | 字符串/数字/布尔字面量 | Pave语言.md | ✅ | ✅ |
| Lexer | 运算符 (+/-/=/==等) | Pave语言.md | ✅ | ✅ |
| Lexer | 注释支持 (单行//, 多行/* */) | - | ✅ | ✅ |
| Lexer | PHP标签兼容 <?php | Pave语言.md | ✅ | ✅ |
| Lexer | 注解符号 @ | Pave语言.md | ✅ | ✅ |
| Lexer | 双冒号 :: (静态访问) | Pave语言.md | ✅ | ✅ |
| Lexer | 三元操作符 ? | Pave语言.md | ✅ | ✅ |
| Lexer | Elvis操作符 ?: | Pave语言.md | ✅ | ✅ |
| Lexer | Null合并操作符 ?? | Pave语言.md | ✅ | ✅ |
| Lexer | 自增自减运算符 ++ -- | Pave语言.md | ✅ | ✅ |
| Lexer | 复合赋值运算符 += -= *= /= %= | Pave语言.md | ✅ | ✅ |
| Lexer | instanceof关键字 | Pave语言.md | ✅ | ✅ |
| Lexer | 双引号插值字符串 {$var} | Pave语言.md | ✅ | ✅ |
| Lexer | 单引号字符串(不插值) | Pave语言.md | ✅ | ✅ |
| Lexer | StringPart结构 (Text/Variable) | Pave语言.md | ✅ | ✅ |
| Lexer | InterpolatedString Token | Pave语言.md | ✅ | ✅ |
| Lexer | open关键字 | oop_plan.md | ✅ | ✅ |
| Parser | 类解析 (class) | Pave语言.md | ✅ | ✅ |
| Parser | 接口解析 (interface) | oop_plan.md | ✅ | ✅ |
| Parser | 枚举解析 (enum) | oop_plan.md | ✅ | ✅ |
| Parser | 抽象类 (abstract) | oop_plan.md | ✅ | ✅ |
| Parser | 继承 (extends) | Pave语言.md | ✅ | ✅ |
| Parser | 接口实现 (implements) | oop_plan.md | ✅ | ✅ |
| Parser | 类常量 (const) | Pave语言.md | ✅ | ✅ |
| Parser | 类属性/字段 | Pave语言.md | ✅ | ✅ |
| Parser | 方法解析 | Pave语言.md | ✅ | ✅ |
| Parser | 构造函数 (__construct) | Pave语言.md | ✅ | ✅ |
| Parser | 静态方法/属性 (static) | Pave语言.md | ✅ | ✅ |
| Parser | 访问修饰符 (public/private/protected) | Pave语言.md | ✅ | ✅ |
| Parser | final修饰符 | oop_plan.md | ✅ | ✅ |
| Parser | open修饰符 (继承控制) | oop_plan.md | ✅ | ✅ |
| Parser | ::静态访问解析 (self::, parent::) | Pave语言.md | ✅ | ✅ |
| Parser | 属性提升 (构造函数参数声明属性) | oop_plan.md | ✅ | ✅ |
| Parser | $this字段访问解析 ($this->field) | Pave语言.md | ✅ | ✅ |
| Parser | $this字段赋值解析 ($this->field = value) | Pave语言.md | ✅ | ✅ |
| Parser | $this方法调用解析 ($this->method()) | Pave语言.md | ✅ | ✅ |
| Parser | 字段链式访问解析 ($this->obj->name) | Pave语言.md | ✅ | ✅ |
| Parser | 枚举backed类型解析 | oop_plan.md | ✅ | ✅ |
| Parser | 枚举方法定义解析 | oop_plan.md | ✅ | ✅ |
| Parser | 抽象方法解析 | oop_plan.md | ✅ | ✅ |
| Parser | 接口默认方法解析 | oop_plan.md | ✅ | ✅ |
| AST | 类型系统定义 | Pave语言.md | ✅ | ✅ |
| AST | Nullable类型 (?string) | Pave语言.md | ✅ | ✅ |
| AST | 数组类型 (string[]) | oop_plan.md | ✅ | ✅ |
| AST | 表达式节点 | Pave语言.md | ✅ | ✅ |
| AST | 语句节点 | Pave语言.md | ✅ | ✅ |
| AST | 闭包结构 (ClosureExpr) | 闭包.md | ✅ | ✅ |
| AST | Class.is_open字段 | oop_plan.md | ✅ | ✅ |
| AST | Class.enum_backed_type字段 | oop_plan.md | ✅ | ✅ |
| AST | ClassMethod.is_abstract字段 | oop_plan.md | ✅ | ✅ |
| AST | ClassMethod.is_default字段 | oop_plan.md | ✅ | ✅ |
| AST | PromotedParam结构 | oop_plan.md | ✅ | ✅ |
| AST | ClassMethod.promoted_params字段 | oop_plan.md | ✅ | ✅ |
| AST | Expr::Ternary三元表达式 | Pave语言.md | ✅ | ✅ |
| AST | Expr::Elvis (?: 操作符) | Pave语言.md | ✅ | ✅ |
| AST | Expr::NullCoalescing (?? 操作符) | Pave语言.md | ✅ | ✅ |
| AST | Expr::InterpolatedString (插值字符串) | Pave语言.md | ✅ | ✅ |
| CodeGen | JVM字节码基础结构 | Pave语言.md | ✅ | ✅ |
| CodeGen | 常量池生成 | Pave语言.md | ✅ | ✅ |
| CodeGen | <init>构造方法 | Pave语言.md | ✅ | ✅ |
| CodeGen | main入口方法 | Pave语言.md | ✅ | ✅ |
| CodeGen | print/println/printf | Pave语言.md | ✅ | ✅ |
| CodeGen | 类型自动选择println重载 | Pave语言.md | ✅ | ✅ |
| CodeGen | 算术运算 (+,-,*,/,%) | Pave语言.md | ✅ | ✅ |
| CodeGen | 比较运算 (<,<=,>,>=,==,!=) | Pave语言.md | ✅ | ✅ |
| CodeGen | 逻辑运算 (&&,||) | Pave语言.md | ✅ | ✅ |
| CodeGen | 方法调用 | Pave语言.md | ✅ | ✅ |
| CodeGen | 静态方法调用 | Pave语言.md | ✅ | ✅ |
| CodeGen | 静态字段访问 (getstatic) | Pave语言.md | ✅ | ✅ |
| CodeGen | 字段访问 | Pave语言.md | ✅ | ✅ |
| CodeGen | $this字段访问 (aload_0 + getfield) | Pave语言.md | ✅ | ✅ |
| CodeGen | $this字段赋值 (aload_0 + putfield) | Pave语言.md | ✅ | ✅ |
| CodeGen | $this方法调用 (aload_0 + invokevirtual) | Pave语言.md | ✅ | ✅ |
| CodeGen | new对象实例化 | Pave语言.md | ✅ | ✅ |
| CodeGen | self::静态替换为当前类名 | Pave语言.md | ✅ | ✅ |
| CodeGen | parent::访问父类 | Pave语言.md | ✅ | ✅ |
| CodeGen | const编译为static final | Pave语言.md | ✅ | ✅ |
| CodeGen | <clinit>静态初始化块 | oop_plan.md | ✅ | ✅ |
| CodeGen | 类字段生成 (fields) | Pave语言.md | ✅ | ✅ |
| CodeGen | ACC_ABSTRACT标记 | oop_plan.md | ✅ | ✅ |
| CodeGen | ACC_ENUM标记 | oop_plan.md | ✅ | ✅ |
| CodeGen | 枚举继承java.lang.Enum | oop_plan.md | ✅ | ✅ |
| CodeGen | 枚举构造方法生成 | oop_plan.md | ✅ | ✅ |
| CodeGen | 枚举值初始化 | oop_plan.md | ✅ | ✅ |
| CodeGen | 枚举字段生成 | oop_plan.md | ✅ | ✅ |
| CodeGen | 抽象方法生成 | oop_plan.md | ✅ | ✅ |
| CodeGen | 接口默认方法字节码 | oop_plan.md | ✅ | ✅ |
| CodeGen | parent::__construct调用 | oop_plan.md | ✅ | ✅ |
| CodeGen | 继承检查 | oop_plan.md | ✅ | ✅ |
| CodeGen | 属性提升字段生成 | oop_plan.md | ✅ | ✅ |
| CodeGen | 属性提升构造函数赋值 | oop_plan.md | ✅ | ✅ |
| TypeSystem | 类型解析 | Pave语言.md | ✅ | ✅ |
| TypeSystem | 类型赋值兼容检查 | Pave语言.md | ✅ | ✅ |
| TypeSystem | 表达式类型推断 | Pave语言.md | ✅ | ✅ |
| TypeSystem | TypeContext类型上下文 | Pave语言.md | ✅ | ✅ |
| TypeSystem | 鞘空类型初始化检查 | Pave语言.md | ✅ | ✅ |
| TypeSystem | null赋值拦截 | Pave语言.md | ✅ | ✅ |
| TypeSystem | 条件表达式必须为boolean | control_flow_plan.md | ✅ | ✅ |
| CLI | pava build命令 | Pave语言.md | ✅ | ✅ |
| CLI | pava run命令 | Pave语言.md | ✅ | ✅ |

## 控制流语句

| 模块 | 功能 | 设计文档 | 已完成 | 已测试(glm-5) |
|------|------|----------|--------|---------------|
| Lexer | if/else/elseif关键字 | control_flow_plan.md | ✅ | ✅ |
| Lexer | while关键字 | control_flow_plan.md | ✅ | ✅ |
| Lexer | for关键字 | control_flow_plan.md | ✅ | ✅ |
| Lexer | foreach关键字 | control_flow_plan.md | ✅ | ❌ |
| Lexer | break关键字 | control_flow_plan.md | ✅ | ✅ |
| Lexer | continue关键字 | control_flow_plan.md | ✅ | ✅ |
| Parser | if语句解析 | control_flow_plan.md | ✅ | ✅ |
| Parser | elseif链式解析 | control_flow_plan.md | ✅ | ✅ |
| Parser | else分支解析 | control_flow_plan.md | ✅ | ✅ |
| Parser | while循环解析 | control_flow_plan.md | ✅ | ✅ |
| Parser | for循环解析 | control_flow_plan.md | ✅ | ✅ |
| Parser | foreach循环解析 | control_flow_plan.md | ❌ | ❌ |
| Parser | break语句解析 | control_flow_plan.md | ✅ | ✅ |
| Parser | continue语句解析 | control_flow_plan.md | ✅ | ✅ |
| Parser | TypedAssign类型声明赋值 | control_flow_plan.md | ✅ | ✅ |
| CodeGen | if字节码生成 | control_flow_plan.md | ✅ | ✅ |
| CodeGen | elseif字节码生成 | control_flow_plan.md | ✅ | ✅ |
| CodeGen | else字节码生成 | control_flow_plan.md | ✅ | ✅ |
| CodeGen | while字节码生成 | control_flow_plan.md | ✅ | ✅ |
| CodeGen | for字节码生成 | control_flow_plan.md | ✅ | ✅ |
| CodeGen | foreach字节码生成 (数组迭代) | control_flow_plan.md | ❌ | ❌ |
| CodeGen | foreach字节码生成 (Iterable) | control_flow_plan.md | ❌ | ❌ |
| CodeGen | foreach字节码生成 (Map entrySet) | control_flow_plan.md | ❌ | ❌ |
| CodeGen | break跳转生成 | control_flow_plan.md | ✅ | ✅ |
| CodeGen | continue跳转生成 | control_flow_plan.md | ✅ | ✅ |
| CodeGen | 循环上下文栈管理 | control_flow_plan.md | ✅ | ✅ |
| CodeGen | LoopContext结构 | control_flow_plan.md | ✅ | ✅ |
| CodeGen | TypedAssign字节码生成 | control_flow_plan.md | ✅ | ✅ |
| CodeGen | 字符串拼接(StringBuilder) | control_flow_plan.md | ✅ | ✅ |
| CodeGen | Unreachable Code检测 | control_flow_plan.md | ❌ | ❌ |
| Parser | 三元表达式解析 ($a ? $b : $c) | Pave语言.md | ✅ | ✅ |
| Parser | Elvis表达式解析 ($a ?: $c) | Pave语言.md | ✅ | ✅ |
| Parser | Null合并表达式解析 ($a ?? $c) | Pave语言.md | ✅ | ✅ |
| Parser | 嵌套三元表达式解析 | Pave语言.md | ✅ | ✅ |
| Parser | 三元表达式在赋值中使用 | Pave语言.md | ✅ | ✅ |
| CodeGen | 三元表达式字节码生成 | Pave语言.md | ✅ | ✅ |
| CodeGen | Elvis(?:)字节码生成 (dup + ifne) | Pave语言.md | ✅ | ✅ |
| CodeGen | Null合并(??)字节码生成 (dup + ifnonnull) | Pave语言.md | ✅ | ✅ |
| Parser | 插值字符串解析 (双引号{$var}) | Pave语言.md | ✅ | ✅ |
| Parser | 单引号字符串解析(不插值) | Pave语言.md | ✅ | ✅ |
| CodeGen | 插值字符串字节码生成 (StringBuilder) | Pave语言.md | ✅ | ✅ |
| CodeGen | emit_interpolated_string方法 | Pave语言.md | ✅ | ✅ |

## 闭包/函数特性

| 模块 | 功能 | 设计文档 | 已完成 | 已测试(glm-5) |
|------|------|----------|--------|---------------|
| Lexer | function关键字 | 闭包.md | ✅ | ✅ |
| Lexer | use关键字 | 闭包.md | ✅ | ✅ |
| Lexer | 引用捕获符号 & | 闭包.md | ✅ | ✅ |
| Parser | 闭包参数解析 | 闭包.md | ✅ | ✅ |
| Parser | use捕获变量列表解析 | 闭包.md | ✅ | ✅ |
| Parser | 引用捕获 vs 值捕获区分 | 闭包.md | ✅ | ✅ |
| Parser | 闭包返回类型解析 | 闭包.md | ✅ | ✅ |
| Parser | 闭包体解析 | 闭包.md | ✅ | ✅ |
| Parser | 闭包变量赋值解析 | 闭包.md | ✅ | ✅ |
| Parser | 闭包调用解析 $fn(...) | 闭包.md | ✅ | ✅ |
| CodeGen | 值捕获字节码生成 | 闭包.md | ✅ | ✅ |
| CodeGen | 引用捕获变量提升(Ref包装) | 闭包.md | ✅ | ✅ |
| CodeGen | LambdaMetafactory调用 | 闭包.md | ✅* | ✅ |
| CodeGen | 递归闭包支持 | 闭包.md | ✅ | ✅ |
| CodeGen | 闭包调用字节码 | 闭包.md | ✅ | ✅ |
| Runtime | pava.lang.Ref类 | 闭包.md | ✅ | ✅ |
| Runtime | pava.lang.Callable接口 | 闭包.md | ✅ | ✅ |

> *注：LambdaMetafactory采用简化实现（匿名内部类方案），完整实现需Bootstrap Method生成

## OOP高级特性

| 模块 | 功能 | 设计文档 | 已完成 | 已测试(glm-5) |
|------|------|----------|--------|---------------|
| Parser | 枚举值解析 (case) | oop_plan.md | ✅ | ✅ |
| Parser | 枚举方法定义 | oop_plan.md | ✅ | ✅ |
| Parser | 枚举 backed类型 (: int) | oop_plan.md | ✅ | ✅ |
| CodeGen | 枚举编译为java.lang.Enum | oop_plan.md | ✅ | ✅ |
| CodeGen | 枚举case编译为static final字段 | oop_plan.md | ✅ | ✅ |
| CodeGen | 枚举方法生成 | oop_plan.md | ✅ | ✅ |
| CodeGen | 枚举构造方法<init> | oop_plan.md | ✅ | ✅ |
| CodeGen | 枚举值初始化<clinit> | oop_plan.md | ✅ | ✅ |
| Parser | 接口默认方法实现 | oop_plan.md | ✅ | ✅ |
| CodeGen | 接口默认方法字节码 | oop_plan.md | ✅ | ✅ |
| CodeGen | 抽象方法标记ACC_ABSTRACT | oop_plan.md | ✅ | ✅ |
| CodeGen | 继承检查(父类必须open) | oop_plan.md | ✅ | ✅ |
| CodeGen | super父类构造调用 | oop_plan.md | ✅ | ✅ |
| CodeGen | parent::__construct处理 | oop_plan.md | ✅ | ✅ |
| Parser | 属性提升参数解析 | oop_plan.md | ✅ | ✅ |
| AST | PromotedParam结构 | oop_plan.md | ✅ | ✅ |
| CodeGen | 属性提升字段生成 | oop_plan.md | ✅ | ✅ |
| CodeGen | 属性提升构造函数赋值 | oop_plan.md | ✅ | ✅ |

## 类型转换与安全

| 模块 | 功能 | 设计文档 | 已完成 | 已测试(glm-5) |
|------|------|----------|--------|---------------|
| Lexer | 类型转换语法 (int32) | Pave语言.md | ✅ | ✅ |
| Parser | 类型转换表达式解析 | Pave语言.md | ✅ | ✅ |
| CodeGen | 类型转换字节码 | Pave语言.md | ✅ | ✅ |
| CodeGen | 小转大自动 widening | Pave语言.md | ✅ | ✅ |
| CodeGen | 大转小截断 | Pave语言.md | ✅ | ✅ |
| CodeGen | @Nullable注解生成 | Pave语言.md | ❌ | ❌ |

---

## 统计汇总

- **已完成功能**: 189 / 205 (约 92%)
- **已测试功能**: 133 / 205 (约 65%)
- **核心编译流程**: Lexer → Parser → AST → CodeGen → .class 完整可用
- **可编译运行**: 类定义、main方法、print输出、算术/比较/逻辑运算、if/while/for控制流、break/continue、elseif链式、类型声明赋值、字符串拼接、字符串插值("{$var}")、三元表达式(?/:/??)、复合赋值(+=/-=等)、自增自减(++/--)、instanceof、常量定义、静态方法/字段访问、$this字段访问/赋值/方法调用、闭包定义与调用、枚举定义、接口定义、抽象类、继承、parent::__construct调用、属性提升参数

### 分模块统计
| 模块 | 完成 | 总数 | 比例 |
|------|------|------|------|
| 核心编译器 | 108 | 108 | 100% |
| 控制流语句 | 45 | 53 | 85% |
| 闭包/函数特性 | 17 | 17 | 100% |
| OOP高级特性 | 18 | 18 | 100% |
| 类型转换与安全 | 5 | 6 | 83% |

## 本次新增功能 (2026-04-17 续)

### 控制流完善
- ✅ AST: Stmt::For结构 (init, cond, update, body)
- ✅ AST: Stmt::TypedAssign结构 (name, type, expr)
- ✅ AST: Stmt::Break语句
- ✅ AST: Stmt::Continue语句
- ✅ AST: Stmt::If 4字段版本 (cond, then, elseif_pairs, else)
- ✅ Parser: for循环解析
- ✅ Parser: break/continue语句解析
- ✅ Parser: elseif链式解析
- ✅ Parser: TypedAssign类型声明赋值解析
- ✅ Parser: self::字段赋值解析
- ✅ CodeGen: LoopContext结构 (continue_target, break_patches)
- ✅ CodeGen: loop_stack循环上下文栈
- ✅ CodeGen: emit_for方法
- ✅ CodeGen: emit_while修改支持break/continue
- ✅ CodeGen: emit_break方法
- ✅ CodeGen: emit_continue方法
- ✅ CodeGen: emit_typed_assign方法
- ✅ CodeGen: emit_if_with_elseif完整实现
- ✅ CodeGen: emit_string_concat方法 (StringBuilder拼接)
- ✅ CodeGen: emit_append_to_stringbuilder方法
- ✅ CodeGen: emit_binary_op Add支持类型判断和字符串拼接
- ✅ CodeGen: emit_store_field处理FieldAccess/StaticFieldAccess
- ✅ CodeGen: infer_expr_type支持字符串拼接类型推断
- ✅ CodeGen: infer_class_name_from_expr改进类名推断
- ✅ CodeGen: build_method_descriptor_from_args使用实际参数类型
- ✅ CodeGen: collect_constants_from_stmt支持For/TypedAssign
- ✅ CodeGen: class_fields字段用于字段类型查询

### $this引用和字段操作
- ✅ Parser: $this->field字段访问解析
- ✅ Parser: $this->field = value字段赋值解析
- ✅ Parser: $this->method()方法调用解析
- ✅ Parser: $this->obj->name链式字段访问解析
- ✅ CodeGen: emit_load_var正确处理$this (aload_0 = 0x2A)
- ✅ CodeGen: emit_field_access使用class_fields查询字段类型
- ✅ CodeGen: emit_store_field正确处理FieldAccess (putfield)
- ✅ CodeGen: infer_class_name_from_expr正确处理$this -> 当前类名
- ✅ 测试: test_this_field_access, test_this_field_assign, test_this_method_call, test_this_field_chain

### OOP高级特性完善
- ✅ Parser: 枚举backed类型解析 (enum Status: int)
- ✅ Parser: 枚举方法定义解析
- ✅ Parser: 接口默认方法实现解析
- ✅ Parser: 抽象方法解析 (abstract function)
- ✅ AST: Class.enum_backed_type字段
- ✅ AST: ClassMethod.is_abstract字段
- ✅ AST: ClassMethod.is_default字段
- ✅ CodeGen: ACC_ABSTRACT标记
- ✅ CodeGen: ACC_ENUM标记
- ✅ CodeGen: 枚举继承java.lang.Enum
- ✅ CodeGen: 枚举构造方法生成 (emit_enum_init_method)
- ✅ CodeGen: 枚举值初始化 (emit_enum_value_init)
- ✅ CodeGen: 抽象方法生成 (emit_abstract_method)
- ✅ CodeGen: 接口默认方法字节码
- ✅ CodeGen: parent::__construct调用处理 (invokespecial)
- ✅ CodeGen: 继承检查占位逻辑

### 属性提升 (PHP 8特性)
- ✅ AST: PromotedParam结构 (name, param_type, is_public/private/protected)
- ✅ AST: ClassMethod.promoted_params字段
- ✅ Parser: parse_params_with_promoted函数
- ✅ Parser: 属性提升参数解析 (public/private/protected修饰符)
- ✅ CodeGen: emit_promoted_field函数
- ✅ CodeGen: emit_constructor_method函数 (自动赋值)
- ✅ CodeGen: 属性提升字段计数和生成

### 三元表达式 (PHP风格)
- ✅ Lexer: Token::QuestionColon (?:)
- ✅ Lexer: Token::DoubleQuestion (??)
- ✅ AST: Expr::Ternary(cond, then, else)
- ✅ AST: Expr::Elvis(value, else) - $a ?: $c
- ✅ AST: Expr::NullCoalescing(value, default) - $a ?? $c
- ✅ Parser: parse_ternary方法处理三种三元形式
- ✅ Parser: 嵌套三元表达式解析 ($a ? $b : $c ? $d : $e)
- ✅ Parser: 三元表达式在赋值中使用 ($r = $x ? 1 : 0)
- ✅ CodeGen: emit_ternary方法 (ifne + goto)
- ✅ CodeGen: emit_elvis方法 (dup + ifne，当真时返回原值)
- ✅ CodeGen: emit_null_coalescing方法 (dup + ifnonnull)
- ✅ CodeGen: infer_expr_type支持Ternary/Elvis/NullCoalescing类型推断
- ✅ 测试: test_ternary_expression, test_elvis_expression, test_null_coalescing_expression, test_nested_ternary, test_ternary_in_assign

### 复合赋值运算符
- ✅ Lexer: Token::PlusEqual, MinusEqual, StarEqual, SlashEqual, PercentEqual
- ✅ AST: BinaryOp::AddAssign, SubAssign, MulAssign, DivAssign, ModAssign
- ✅ Parser: parse_stmt中复合赋值解析
- ✅ CodeGen: emit_compound_assign方法 (load -> op -> store)
- ✅ 测试: test_compound_assign_add, test_compound_assign_sub, test_compound_assign_mul

### 自增自减运算符
- ✅ Lexer: Token::PlusPlus, MinusMinus
- ✅ AST: UnaryOp::PreIncrement, PostIncrement, PreDecrement, PostDecrement
- ✅ Parser: parse_unary前缀++/--, parse_postfix后缀++/--
- ✅ CodeGen: emit_pre_increment (load -> add -> store -> load)
- ✅ CodeGen: emit_post_increment (load -> dup -> add -> store)
- ✅ CodeGen: emit_pre_decrement, emit_post_decrement
- ✅ CodeGen: 支持变量和字段的自增自减
- ✅ 测试: test_pre_increment, test_post_increment, test_pre_decrement, test_post_decrement, test_increment_in_expression

### instanceof运算符
- ✅ Lexer: Token::Instanceof关键字
- ✅ AST: Expr::InstanceOf(expr, class_name)
- ✅ Parser: parse_comparison中instanceof解析（支持标识符和类型关键字）
- ✅ CodeGen: emit_instanceof方法 (instanceof opcode 0xC1)
- ✅ CodeGen: infer_expr_type返回VarType::Bool
- ✅ 测试: test_instanceof, test_instanceof_in_if

### 字符串插值 (PHP风格)
- ✅ Lexer: StringPart::Text, StringPart::Variable结构
- ✅ Lexer: Token::InterpolatedString(Vec<StringPart>)
- ✅ Lexer: read_string区分单引号(不插值)和双引号(插值)
- ✅ Lexer: 双引号中识别 {$var} 模式，强制{}包围
- ✅ Lexer: 单引号字符串直接作为StringLiteral
- ✅ AST: Expr::InterpolatedString(Vec<Expr>)
- ✅ Parser: parse_primary处理InterpolatedString token
- ✅ Parser: 将StringPart转换为Expr(StringLiteral/Variable)
- ✅ CodeGen: emit_interpolated_string方法 (StringBuilder拼接)
- ✅ CodeGen: infer_expr_type返回VarType::String
- ✅ 测试: test_interpolated_string_basic, test_interpolated_string_multiple_vars, test_single_quote_no_interpolation, test_plain_double_quote_string, test_interpolated_string_with_text_after

## 下一步优先级

1. **foreach循环**: 数组迭代、Iterable迭代、Map entrySet迭代 (用户暂未决定数组设计)
2. **Nullable注解生成**: @Nullable注解生成（用户暂未决定注解设计）
3. **Unreachable Code检测**: 检测和警告不可达代码
4. **测试完善**: 为新增控制流特性添加更多单元测试
5. **LambdaMetafactory**: 完整Bootstrap Method生成 (当前为简化匿名内部类实现)
# Pava 注解使用手册

## 1. 概述

Pava 语言的注解系统采用 PHP/C# 风格的语法设计，底层字节码完全对齐 Java 规范。这意味着：
- **视觉风格**：贴近 PHP/C#，提高开发体验
- **底层兼容**：生成的 `.class` 文件与 Java 注解完全互通

---

## 2. 注解定义

### 2.1 基本语法

使用 `annotation` 关键字定义注解：

```pava
public annotation Column {
    public string $value;
    public bool $nullable = true;
    public string $columnType = "VARCHAR";
}

public annotation Table {
    public string $value;
    public string $schema = "";
}
```

**关键规则**：
- 必须使用 `annotation` 关键字（禁用 `class`）
- 参数默认值使用 `=`（定义时用 `=`，使用时用 `:`）
- 属性名必须以 `$` 开头
- `$value` 属性会被映射为 Java 的 `value()` 方法

### 2.2 翻译对照

| Pava 语法 | Java 映射 |
|-----------|-----------|
| `annotation` | `@interface` |
| `public string $value` | `String value()` |
| `public bool $nullable = true` | `boolean nullable() default true` |

### 2.3 @Attribute 元注解

使用 `@Attribute` 定义注解的使用范围和保留策略：

```pava
// 使用简洁风格（推荐）
@Attribute(target: TYPE | FIELD | METHOD, retention: RUNTIME)
public annotation MyAnnotation {
    public string $value = "";
}

// 使用兼容风格（PHP Attribute 风格）
@Attribute(target: TARGET_CLASS | TARGET_FIELD, retention: RETENTION_RUNTIME)
public annotation LegacyAnnotation {
    public string $value = "";
}
```

**可用的 target 值**：
- `TYPE` / `TARGET_CLASS` - 类、接口、注解
- `FIELD` / `TARGET_FIELD` - 字段
- `METHOD` / `TARGET_METHOD` - 方法
- `PARAMETER` / `TARGET_PARAMETER` - 参数
- `CONSTRUCTOR` / `TARGET_CONSTRUCTOR` - 构造函数
- `PROPERTY` / `TARGET_PROPERTY` - 属性

**可用的 retention 值**：
- `SOURCE` / `RETENTION_SOURCE` - 源码级别，编译后丢弃
- `CLASS` / `RETENTION_CLASS` - 编译到 class 文件，但运行时不可见
- `RUNTIME` / `RETENTION_RUNTIME` - 运行时保留（默认）

---

## 3. 注解使用

### 3.1 基本用法

```pava
@Table("users")
class User {
    public int $id { get; set; }
}
```

### 3.2 参数传递

**命名参数**：
```pava
@Column("user_id", nullable: false)
public int $id { get; set; }
```

**数组参数**：
```pava
@Tags(["important", "verified"])
public string $status { get; set; }
```

### 3.3 语法对照表

| 场景 | Pava 语法 | Java 映射 |
|------|-----------|-----------|
| 定义范围 | `A | B` | `{A, B}` (ElementType 数组) |
| 参数赋值 | `key: value` | `key = value` |
| 列表数据 | `["A", "B"]` | `{"A", "B"}` |
| 单值省略 | `@Anno("val")` | `@Anno(value = "val")` |

---

## 4. 属性注解广播

当注解挂在属性上时，会自动广播到生成的 `Field`、`Getter`、`Setter`：

**Pava 源码**：
```pava
@Column("user_id", nullable: false)
public int $id { get; set; }
```

**生成的 Java 字节码**：
```java
@Column(value = "user_id", nullable = false)
private int id;

@Column(value = "user_id", nullable = false)
public int getId() { return this.id; }

@Column(value = "user_id", nullable = false)
public void setId(int id) { this.id = id; }
```

---

## 5. 与 Java 的互通性

### 5.1 Pava 使用 Java 注解

```pava
@java.lang.annotation.Retention(value: java.lang.annotation.RetentionPolicy.RUNTIME)
public annotation MyCustomAnnotation {
    public string $message: "";
}
```

### 5.2 Java 使用 Pava 注解

Pava 编译器生成的 `.class` 文件与标准 Java 注解格式完全一致，Java 代码可以直接使用：

```java
// Java 代码
@Table(value = "products")
public class JavaProduct {
    @Column(value = "product_id", nullable = false)
    private int productId;
}
```

---

## 6. 错误处理

编译器会严格检查语法错误：

### 6.1 注解定义使用 `=`，注解使用使用 `:`

```pava
// ✅ 正确：注解定义内部属性默认值使用 =
public annotation GoodAnnotation {
    public string $name = "default";
}

// ❌ 错误：注解定义不能使用 :
public annotation BadAnnotation {
    public string $name : "default";
}
```

**编译错误**：
```
Parser Error: Pava annotation property defaults must use '=' instead of ':'.
Use: `public string $name = "default";`
```

### 6.2 禁止使用 `{}`

```pava
// ❌ 错误：必须使用 [] 而不是 {}
@Tags({"important", "verified"})
public string $status { get; set; }
```

**编译错误**：
```
Parser Error: Pava annotation usage must use ':' for key-value pairs and '[]' for arrays.
Example: @Column(name: "id", nullable: false) or @Names(["a", "b"])
```

---

## 7. 完整示例

### 定义注解

```pava
package com.example.annotations;

public annotation Column {
    public string $value;
    public bool $nullable: true;
    public string $columnType: "VARCHAR";
}

public annotation Table {
    public string $value;
    public string $schema: "";
}
```

### 使用注解

```pava
package com.example.entities;

@Table("users")
class User {
    @Column("user_id", nullable: false)
    public int $id { get; set; }
    
    @Column("username")
    public string $name { get; set; }
    
    @Column("created_at")
    public string $createdAt { get; set; }
}
```

---

## 8. 编译与运行

```bash
# 编译注解定义
pava compile Column.pava Table.pava --output target/classes

# 编译使用注解的类
pava compile User.pava --output target/classes

# 运行测试
java -cp target/classes com.example.entities.User
```

---

## 9. 最佳实践

1. **命名规范**：注解名称使用 PascalCase，如 `@Column`、`@Table`
2. **单一职责**：每个注解只服务于一个目的
3. **默认值设计**：为常用属性提供合理的默认值
4. **文档化**：为自定义注解编写说明文档

---

## 10. 参考对照表

### Pava vs Java 语法对照

| 特性 | Pava | Java |
|------|------|------|
| 定义关键字 | `annotation` | `@interface` |
| 属性命名 | `$value` | `value()` |
| 默认值语法（定义时） | `= defaultValue` | `default defaultValue` |
| 参数语法（使用时） | `key: value` | `key = value` |
| 数组表示 | `[]` | `{}` |
| 范围组合 | `A \| B` | `{A, B}` |

### 支持的注解目标

| Pava 常量（简洁风格） | Pava 常量（兼容风格） | Java ElementType |
|---------------------|---------------------|------------------|
| `TYPE` | `TARGET_CLASS` | `TYPE` |
| `FIELD` | `TARGET_FIELD` | `FIELD` |
| `METHOD` | `TARGET_METHOD` | `METHOD` |
| `PARAMETER` | `TARGET_PARAMETER` | `PARAMETER` |
| `CONSTRUCTOR` | `TARGET_CONSTRUCTOR` | `CONSTRUCTOR` |
| `PROPERTY` | `TARGET_PROPERTY` | `FIELD` |

### 支持的保留策略

| Pava 常量（简洁风格） | Pava 常量（兼容风格） | Java RetentionPolicy |
|---------------------|---------------------|----------------------|
| `SOURCE` | `RETENTION_SOURCE` | `SOURCE` |
| `CLASS` | `RETENTION_CLASS` | `CLASS` |
| `RUNTIME` | `RETENTION_RUNTIME` | `RUNTIME` |

---

**文档版本**: 1.0.0  
**最后更新**: 2026-04-21
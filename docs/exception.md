# 异常处理

Pava的异常处理语法与PHP一致，但底层使用JVM异常机制。

## try-catch-finally

```pava
try {
    // 可能抛异常的代码
} catch (Exception $e) {
    // 处理异常
} finally {
    // 无论是否异常都会执行（可选）
}
```

### 多catch分支

按顺序匹配，子类异常应放在父类之前：

```pava
try {
    int32 $x = 10 / 0;
} catch (ArithmeticException $e) {
    println("除零错误");
} catch (Exception $e) {
    println("其他错误");
}
```

### multi-catch

用一个catch捕获多种异常：

```pava
try {
    // ...
} catch (IllegalArgumentException | ArithmeticException $e) {
    println("捕获多种异常");
}
```

### finally

finally块总是执行，无论是否发生异常：

```pava
try {
    // 可能抛异常
} finally {
    // 清理资源
}
```

## throw

抛出异常：

```pava
throw new IllegalArgumentException("参数错误");

// 重新抛出捕获的异常
throw $e;
```

## 常用异常类

Pava直接使用Java异常类，无需import：

| 类名 | JVM类 |
|------|-------|
| Exception | java.lang.Exception |
| RuntimeException | java.lang.RuntimeException |
| IllegalArgumentException | java.lang.IllegalArgumentException |
| ArithmeticException | java.lang.ArithmeticException |
| NullPointerException | java.lang.NullPointerException |
| IndexOutOfBoundsException | java.lang.IndexOutOfBoundsException |

## Nothing类型

throw表达式返回Nothing类型（类似Kotlin）。Nothing是所有类型的子类型，永不返回：

```pava
// throw表达式可用于任何期望返回值的位置
public function fail(): int32 {
    throw new Exception("失败");
}
```

## 与PHP的差异

- 异常类使用Java异常体系，不是PHP异常
- catch变量需声明类型（强类型）
- JVM会自动抛出运行时异常（如除零、空指针）
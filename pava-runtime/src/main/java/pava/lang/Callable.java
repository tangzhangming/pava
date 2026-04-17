package pava.lang;

/**
 * 统一的闭包调用接口
 * 所有 Pava 闭包都实现此接口
 */
@FunctionalInterface
public interface Callable {
    /**
     * 调用闭包
     * @param args 变长参数列表
     * @return 返回值
     */
    Object call(Object... args);
}

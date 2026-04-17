package pava.lang;

/**
 * 用于包装原始类型和对象引用的容器类。
 * 支持引用捕获，允许闭包内部修改外部变量。
 */
public final class Ref {
    public Object value;

    public Ref() {
        this.value = null;
    }

    public Ref(Object value) {
        this.value = value;
    }

    /**
     * 获取整数值
     */
    public int getInt() {
        return (Integer) value;
    }

    /**
     * 设置整数值
     */
    public void setInt(int val) {
        this.value = val;
    }

    /**
     * 获取长整数值
     */
    public long getLong() {
        return (Long) value;
    }

    /**
     * 设置长整数值
     */
    public void setLong(long val) {
        this.value = val;
    }

    /**
     * 获取浮点值
     */
    public double getDouble() {
        return (Double) value;
    }

    /**
     * 设置浮点值
     */
    public void setDouble(double val) {
        this.value = val;
    }

    /**
     * 获取布尔值
     */
    public boolean getBool() {
        return (Boolean) value;
    }

    /**
     * 设置布尔值
     */
    public void setBool(boolean val) {
        this.value = val;
    }

    /**
     * 获取字符串值
     */
    public String getString() {
        return (String) value;
    }

    /**
     * 设置字符串值
     */
    public void setString(String val) {
        this.value = val;
    }

    @Override
    public String toString() {
        return value != null ? value.toString() : "null";
    }
}

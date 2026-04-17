package pava.lang;

/**
 * Reference wrapper for supporting mutable variable capture in closures.
 * Used when a closure captures a variable by reference (use (&$var)).
 * 
 * Design per Pava closure specification:
 * - Wraps any value (primitive via boxing, or object reference)
 * - Allows modification from within the closure body
 * - Enables recursive closure patterns where the closure references itself
 */
public final class Ref {
    public Object value;
    
    public Ref() {
        this.value = null;
    }
    
    public Ref(Object value) {
        this.value = value;
    }
    
    public Object get() {
        return this.value;
    }
    
    public void set(Object value) {
        this.value = value;
    }
    
    @Override
    public String toString() {
        return "Ref(" + value + ")";
    }
}
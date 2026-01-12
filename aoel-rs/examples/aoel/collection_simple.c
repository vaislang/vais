#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <stdint.h>

// AOEL Runtime Types
typedef enum { VAL_INT, VAL_FLOAT, VAL_BOOL, VAL_STRING, VAL_ARRAY, VAL_VOID } ValueType;

typedef struct Value {
    ValueType type;
    union {
        int64_t i;
        double f;
        bool b;
        char* s;
        struct { struct Value* items; size_t len; size_t cap; } arr;
    } data;
} Value;

// Value constructors
static Value val_int(int64_t i) { Value v; v.type = VAL_INT; v.data.i = i; return v; }
static Value val_float(double f) { Value v; v.type = VAL_FLOAT; v.data.f = f; return v; }
static Value val_bool(bool b) { Value v; v.type = VAL_BOOL; v.data.b = b; return v; }
static Value val_void(void) { Value v; v.type = VAL_VOID; return v; }

// Arithmetic operations
static Value val_add(Value a, Value b) {
    if (a.type == VAL_INT && b.type == VAL_INT) return val_int(a.data.i + b.data.i);
    if (a.type == VAL_FLOAT || b.type == VAL_FLOAT) {
        double af = a.type == VAL_FLOAT ? a.data.f : (double)a.data.i;
        double bf = b.type == VAL_FLOAT ? b.data.f : (double)b.data.i;
        return val_float(af + bf);
    }
    return val_void();
}

static Value val_sub(Value a, Value b) {
    if (a.type == VAL_INT && b.type == VAL_INT) return val_int(a.data.i - b.data.i);
    if (a.type == VAL_FLOAT || b.type == VAL_FLOAT) {
        double af = a.type == VAL_FLOAT ? a.data.f : (double)a.data.i;
        double bf = b.type == VAL_FLOAT ? b.data.f : (double)b.data.i;
        return val_float(af - bf);
    }
    return val_void();
}

static Value val_mul(Value a, Value b) {
    if (a.type == VAL_INT && b.type == VAL_INT) return val_int(a.data.i * b.data.i);
    if (a.type == VAL_FLOAT || b.type == VAL_FLOAT) {
        double af = a.type == VAL_FLOAT ? a.data.f : (double)a.data.i;
        double bf = b.type == VAL_FLOAT ? b.data.f : (double)b.data.i;
        return val_float(af * bf);
    }
    return val_void();
}

static Value val_div(Value a, Value b) {
    if (a.type == VAL_INT && b.type == VAL_INT) return val_int(a.data.i / b.data.i);
    if (a.type == VAL_FLOAT || b.type == VAL_FLOAT) {
        double af = a.type == VAL_FLOAT ? a.data.f : (double)a.data.i;
        double bf = b.type == VAL_FLOAT ? b.data.f : (double)b.data.i;
        return val_float(af / bf);
    }
    return val_void();
}

// Comparison operations
static Value val_lt(Value a, Value b) {
    if (a.type == VAL_INT && b.type == VAL_INT) return val_bool(a.data.i < b.data.i);
    if (a.type == VAL_FLOAT || b.type == VAL_FLOAT) {
        double af = a.type == VAL_FLOAT ? a.data.f : (double)a.data.i;
        double bf = b.type == VAL_FLOAT ? b.data.f : (double)b.data.i;
        return val_bool(af < bf);
    }
    return val_bool(false);
}

static Value val_eq(Value a, Value b) {
    if (a.type != b.type) return val_bool(false);
    if (a.type == VAL_INT) return val_bool(a.data.i == b.data.i);
    if (a.type == VAL_FLOAT) return val_bool(a.data.f == b.data.f);
    if (a.type == VAL_BOOL) return val_bool(a.data.b == b.data.b);
    return val_bool(false);
}

static Value val_ne(Value a, Value b) {
    return val_bool(!val_eq(a, b).data.b);
}

static Value val_lte(Value a, Value b) {
    if (a.type == VAL_INT && b.type == VAL_INT) return val_bool(a.data.i <= b.data.i);
    if (a.type == VAL_FLOAT || b.type == VAL_FLOAT) {
        double af = a.type == VAL_FLOAT ? a.data.f : (double)a.data.i;
        double bf = b.type == VAL_FLOAT ? b.data.f : (double)b.data.i;
        return val_bool(af <= bf);
    }
    return val_bool(false);
}

static Value val_gte(Value a, Value b) {
    if (a.type == VAL_INT && b.type == VAL_INT) return val_bool(a.data.i >= b.data.i);
    if (a.type == VAL_FLOAT || b.type == VAL_FLOAT) {
        double af = a.type == VAL_FLOAT ? a.data.f : (double)a.data.i;
        double bf = b.type == VAL_FLOAT ? b.data.f : (double)b.data.i;
        return val_bool(af >= bf);
    }
    return val_bool(false);
}

// Print
static void val_print(Value v) {
    switch (v.type) {
        case VAL_INT: printf("%lld", (long long)v.data.i); break;
        case VAL_FLOAT: printf("%g", v.data.f); break;
        case VAL_BOOL: printf("%s", v.data.b ? "true" : "false"); break;
        case VAL_STRING: printf("%s", v.data.s); break;
        case VAL_VOID: printf("()"); break;
        case VAL_ARRAY: {
            printf("[");
            for (size_t i = 0; i < v.data.arr.len; i++) {
                if (i > 0) printf(", ");
                val_print(v.data.arr.items[i]);
            }
            printf("]");
            break;
        }
        default: printf("<value>"); break;
    }
}

// Array operations
static Value val_array_new(size_t cap) {
    Value v;
    v.type = VAL_ARRAY;
    v.data.arr.items = (Value*)malloc(cap * sizeof(Value));
    v.data.arr.len = 0;
    v.data.arr.cap = cap;
    return v;
}

static void val_array_push(Value* arr, Value elem) {
    if (arr->data.arr.len >= arr->data.arr.cap) {
        arr->data.arr.cap = arr->data.arr.cap == 0 ? 8 : arr->data.arr.cap * 2;
        arr->data.arr.items = (Value*)realloc(arr->data.arr.items, arr->data.arr.cap * sizeof(Value));
    }
    arr->data.arr.items[arr->data.arr.len++] = elem;
}

static Value val_array_get(Value arr, int64_t idx) {
    if (idx < 0) idx += (int64_t)arr.data.arr.len;
    if (idx < 0 || (size_t)idx >= arr.data.arr.len) return val_void();
    return arr.data.arr.items[idx];
}

static int64_t val_array_len(Value arr) {
    return (int64_t)arr.data.arr.len;
}

static Value aoel___main__(void) {
    Value _stack[256];
    int _sp = 0;
    
    L0: ;
    _stack[_sp++] = val_int(1LL);
    L1: ;
    _stack[_sp++] = val_int(2LL);
    L2: ;
    _stack[_sp++] = val_int(3LL);
    L3: ;
    _stack[_sp++] = val_int(4LL);
    L4: ;
    _stack[_sp++] = val_int(5LL);
    L5: ;
    {
        Value _arr = val_array_new(5);
        _sp -= 5;
        val_array_push(&_arr, _stack[_sp + 0]);
        val_array_push(&_arr, _stack[_sp + 1]);
        val_array_push(&_arr, _stack[_sp + 2]);
        val_array_push(&_arr, _stack[_sp + 3]);
        val_array_push(&_arr, _stack[_sp + 4]);
        _stack[_sp++] = _arr;
    }
    L6: ;
    {
        Value _src_arr = _stack[--_sp];
        Value _result_arr = val_array_new(_src_arr.data.arr.len);
        for (size_t _i = 0; _i < _src_arr.data.arr.len; _i++) {
            Value _ = _src_arr.data.arr.items[_i];
            _stack[_sp++] = _;
            _stack[_sp++] = _;
            _stack[_sp++] = val_int(2LL);
            { Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_mul(a, b); }
            Value _mapped = _stack[--_sp];
            val_array_push(&_result_arr, _mapped);
        }
        _stack[_sp++] = _result_arr;
    }
    if (_sp > 0) return _stack[--_sp];
    return val_void();
}

int main(int argc, char* argv[]) {
    Value result = aoel___main__();
    val_print(result);
    printf("\n");
    return 0;
}

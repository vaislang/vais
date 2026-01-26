// Contract runtime support for VAIS
// This file provides the implementation for contract violation handlers

#include <stdio.h>
#include <stdlib.h>

// Contract violation types
#define CONTRACT_REQUIRES 1
#define CONTRACT_ENSURES 2
#define CONTRACT_INVARIANT 3

// Contract violation handler
// Called when a contract (requires/ensures/invariant) is violated
// kind: type of contract (1=requires, 2=ensures, 3=invariant)
// condition: the condition that failed (as string)
// file: source file name
// line: source line number
// func: function name
long __contract_fail(long kind, const char* condition, const char* file, long line, const char* func) {
    const char* kind_str;
    switch (kind) {
        case CONTRACT_REQUIRES:
            kind_str = "precondition";
            break;
        case CONTRACT_ENSURES:
            kind_str = "postcondition";
            break;
        case CONTRACT_INVARIANT:
            kind_str = "invariant";
            break;
        default:
            kind_str = "contract";
            break;
    }

    fprintf(stderr, "Contract violation: %s failed\n", kind_str);
    fprintf(stderr, "  Condition: %s\n", condition);
    fprintf(stderr, "  Location: %s:%ld in function '%s'\n", file, line, func);

    exit(1);
    return 0;
}

// Panic function for assert/assume violations
// msg: error message to display
long __panic(const char* msg) {
    fprintf(stderr, "panic: %s\n", msg);
    exit(1);
    return 0;
}

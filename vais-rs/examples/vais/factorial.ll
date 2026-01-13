; Vais Generated LLVM IR
; Compile with: clang -O2 output.ll -o output

; Target triple for common platforms
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

; Vais Value type (tagged union)
; Tag: 0=void, 1=bool, 2=int, 3=float, 4=string, 5=array
%Value = type { i8, i64 }

; Format strings
@.str.int = private unnamed_addr constant [5 x i8] c"%lld\00"
@.str.float = private unnamed_addr constant [3 x i8] c"%g\00"
@.str.true = private unnamed_addr constant [5 x i8] c"true\00"
@.str.false = private unnamed_addr constant [6 x i8] c"false\00"
@.str.newline = private unnamed_addr constant [2 x i8] c"\0A\00"

; External declarations
declare i32 @printf(i8*, ...)
declare i8* @malloc(i64)
declare void @free(i8*)
declare double @sqrt(double)
declare double @sin(double)
declare double @cos(double)
declare double @tan(double)
declare double @log(double)
declare double @pow(double, double)
declare double @fabs(double)
declare double @floor(double)
declare double @ceil(double)

; Helper: create int value
define %Value @val_int(i64 %v) {
  %1 = insertvalue %Value { i8 2, i64 undef }, i64 %v, 1
  ret %Value %1
}

; Helper: create float value (as i64 bits)
define %Value @val_float(double %v) {
  %1 = bitcast double %v to i64
  %2 = insertvalue %Value { i8 3, i64 undef }, i64 %1, 1
  ret %Value %2
}

; Helper: create bool value
define %Value @val_bool(i1 %v) {
  %1 = zext i1 %v to i64
  %2 = insertvalue %Value { i8 1, i64 undef }, i64 %1, 1
  ret %Value %2
}

; Helper: create void value
define %Value @val_void() {
  ret %Value { i8 0, i64 0 }
}

; Helper: extract int from value
define i64 @get_int(%Value %v) {
  %1 = extractvalue %Value %v, 1
  ret i64 %1
}

; Helper: extract float from value
define double @get_float(%Value %v) {
  %1 = extractvalue %Value %v, 1
  %2 = bitcast i64 %1 to double
  ret double %2
}

; Helper: extract bool from value
define i1 @get_bool(%Value %v) {
  %1 = extractvalue %Value %v, 1
  %2 = trunc i64 %1 to i1
  ret i1 %2
}

; Helper: get value type tag
define i8 @get_type(%Value %v) {
  %1 = extractvalue %Value %v, 0
  ret i8 %1
}

; Arithmetic: add
define %Value @val_add(%Value %a, %Value %b) {
  %ta = call i8 @get_type(%Value %a)
  %tb = call i8 @get_type(%Value %b)
  %is_int_a = icmp eq i8 %ta, 2
  %is_int_b = icmp eq i8 %tb, 2
  %both_int = and i1 %is_int_a, %is_int_b
  br i1 %both_int, label %int_add, label %float_add
int_add:
  %ia = call i64 @get_int(%Value %a)
  %ib = call i64 @get_int(%Value %b)
  %ir = add i64 %ia, %ib
  %rv1 = call %Value @val_int(i64 %ir)
  ret %Value %rv1
float_add:
  %fa = call double @get_float(%Value %a)
  %fb = call double @get_float(%Value %b)
  %fr = fadd double %fa, %fb
  %rv2 = call %Value @val_float(double %fr)
  ret %Value %rv2
}

; Arithmetic: sub
define %Value @val_sub(%Value %a, %Value %b) {
  %ta = call i8 @get_type(%Value %a)
  %tb = call i8 @get_type(%Value %b)
  %is_int_a = icmp eq i8 %ta, 2
  %is_int_b = icmp eq i8 %tb, 2
  %both_int = and i1 %is_int_a, %is_int_b
  br i1 %both_int, label %int_sub, label %float_sub
int_sub:
  %ia = call i64 @get_int(%Value %a)
  %ib = call i64 @get_int(%Value %b)
  %ir = sub i64 %ia, %ib
  %rv1 = call %Value @val_int(i64 %ir)
  ret %Value %rv1
float_sub:
  %fa = call double @get_float(%Value %a)
  %fb = call double @get_float(%Value %b)
  %fr = fsub double %fa, %fb
  %rv2 = call %Value @val_float(double %fr)
  ret %Value %rv2
}

; Arithmetic: mul
define %Value @val_mul(%Value %a, %Value %b) {
  %ta = call i8 @get_type(%Value %a)
  %tb = call i8 @get_type(%Value %b)
  %is_int_a = icmp eq i8 %ta, 2
  %is_int_b = icmp eq i8 %tb, 2
  %both_int = and i1 %is_int_a, %is_int_b
  br i1 %both_int, label %int_mul, label %float_mul
int_mul:
  %ia = call i64 @get_int(%Value %a)
  %ib = call i64 @get_int(%Value %b)
  %ir = mul i64 %ia, %ib
  %rv1 = call %Value @val_int(i64 %ir)
  ret %Value %rv1
float_mul:
  %fa = call double @get_float(%Value %a)
  %fb = call double @get_float(%Value %b)
  %fr = fmul double %fa, %fb
  %rv2 = call %Value @val_float(double %fr)
  ret %Value %rv2
}

; Arithmetic: div
define %Value @val_div(%Value %a, %Value %b) {
  %ta = call i8 @get_type(%Value %a)
  %tb = call i8 @get_type(%Value %b)
  %is_int_a = icmp eq i8 %ta, 2
  %is_int_b = icmp eq i8 %tb, 2
  %both_int = and i1 %is_int_a, %is_int_b
  br i1 %both_int, label %int_div, label %float_div
int_div:
  %ia = call i64 @get_int(%Value %a)
  %ib = call i64 @get_int(%Value %b)
  %ir = sdiv i64 %ia, %ib
  %rv1 = call %Value @val_int(i64 %ir)
  ret %Value %rv1
float_div:
  %fa = call double @get_float(%Value %a)
  %fb = call double @get_float(%Value %b)
  %fr = fdiv double %fa, %fb
  %rv2 = call %Value @val_float(double %fr)
  ret %Value %rv2
}

; Arithmetic: mod
define %Value @val_mod(%Value %a, %Value %b) {
  %ia = call i64 @get_int(%Value %a)
  %ib = call i64 @get_int(%Value %b)
  %ir = srem i64 %ia, %ib
  %rv = call %Value @val_int(i64 %ir)
  ret %Value %rv
}

; Comparison: lt
define %Value @val_lt(%Value %a, %Value %b) {
  %ia = call i64 @get_int(%Value %a)
  %ib = call i64 @get_int(%Value %b)
  %cmp = icmp slt i64 %ia, %ib
  %rv = call %Value @val_bool(i1 %cmp)
  ret %Value %rv
}

; Comparison: gt
define %Value @val_gt(%Value %a, %Value %b) {
  %ia = call i64 @get_int(%Value %a)
  %ib = call i64 @get_int(%Value %b)
  %cmp = icmp sgt i64 %ia, %ib
  %rv = call %Value @val_bool(i1 %cmp)
  ret %Value %rv
}

; Comparison: le
define %Value @val_le(%Value %a, %Value %b) {
  %ia = call i64 @get_int(%Value %a)
  %ib = call i64 @get_int(%Value %b)
  %cmp = icmp sle i64 %ia, %ib
  %rv = call %Value @val_bool(i1 %cmp)
  ret %Value %rv
}

; Comparison: ge
define %Value @val_ge(%Value %a, %Value %b) {
  %ia = call i64 @get_int(%Value %a)
  %ib = call i64 @get_int(%Value %b)
  %cmp = icmp sge i64 %ia, %ib
  %rv = call %Value @val_bool(i1 %cmp)
  ret %Value %rv
}

; Comparison: eq
define %Value @val_eq(%Value %a, %Value %b) {
  %ia = call i64 @get_int(%Value %a)
  %ib = call i64 @get_int(%Value %b)
  %cmp = icmp eq i64 %ia, %ib
  %rv = call %Value @val_bool(i1 %cmp)
  ret %Value %rv
}

; Comparison: ne
define %Value @val_ne(%Value %a, %Value %b) {
  %ia = call i64 @get_int(%Value %a)
  %ib = call i64 @get_int(%Value %b)
  %cmp = icmp ne i64 %ia, %ib
  %rv = call %Value @val_bool(i1 %cmp)
  ret %Value %rv
}

; Print value
define void @val_print(%Value %v) {
  %tag = call i8 @get_type(%Value %v)
  switch i8 %tag, label %done [
    i8 1, label %print_bool
    i8 2, label %print_int
    i8 3, label %print_float
  ]
print_bool:
  %b = call i1 @get_bool(%Value %v)
  br i1 %b, label %print_true, label %print_false
print_true:
  %str_true = getelementptr [5 x i8], [5 x i8]* @.str.true, i64 0, i64 0
  call i32 (i8*, ...) @printf(i8* %str_true)
  br label %done
print_false:
  %str_false = getelementptr [6 x i8], [6 x i8]* @.str.false, i64 0, i64 0
  call i32 (i8*, ...) @printf(i8* %str_false)
  br label %done
print_int:
  %i = call i64 @get_int(%Value %v)
  %fmt_int = getelementptr [5 x i8], [5 x i8]* @.str.int, i64 0, i64 0
  call i32 (i8*, ...) @printf(i8* %fmt_int, i64 %i)
  br label %done
print_float:
  %f = call double @get_float(%Value %v)
  %fmt_float = getelementptr [3 x i8], [3 x i8]* @.str.float, i64 0, i64 0
  call i32 (i8*, ...) @printf(i8* %fmt_float, double %f)
  br label %done
done:
  ret void
}

; Function: factorial
define %Value @factorial(%Value %n) {
entry:
  ; Stack allocation
  %stack = alloca [64 x %Value]
  %sp = alloca i32
  store i32 0, i32* %sp

  %local_n = alloca %Value
  store %Value %n, %Value* %local_n

  ; Instruction 0: Load("n")
  %t0 = load %Value, %Value* %local_n
  %t1 = load i32, i32* %sp
  %t2 = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 %t1
  store %Value %t0, %Value* %t2
  %t3 = add i32 %t1, 1
  store i32 %t3, i32* %sp
  ; Instruction 1: Const(Int(1))
  %t4 = call %Value @val_int(i64 1)
  %t5 = load i32, i32* %sp
  %t6 = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 %t5
  store %Value %t4, %Value* %t6
  %t7 = add i32 %t5, 1
  store i32 %t7, i32* %sp
  ; Instruction 2: Lte
  %t8 = load i32, i32* %sp
  %t9 = sub i32 %t8, 1
  store i32 %t9, i32* %sp
  %t10 = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 %t9
  %t11 = load %Value, %Value* %t10
  %t12 = load i32, i32* %sp
  %t13 = sub i32 %t12, 1
  store i32 %t13, i32* %sp
  %t14 = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 %t13
  %t15 = load %Value, %Value* %t14
  %t16 = call %Value @val_le(%Value %t15, %Value %t11)
  %t17 = load i32, i32* %sp
  %t18 = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 %t17
  store %Value %t16, %Value* %t18
  %t19 = add i32 %t17, 1
  store i32 %t19, i32* %sp
  ; Instruction 3: JumpIfNot(2)
  %t20 = load i32, i32* %sp
  %t21 = sub i32 %t20, 1
  store i32 %t21, i32* %sp
  %t22 = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 %t21
  %t23 = load %Value, %Value* %t22
  %t24 = call i1 @get_bool(%Value %t23)
  br i1 %t24, label %block0, label %block1
block0:
  ; Instruction 4: Const(Int(1))
  %t25 = call %Value @val_int(i64 1)
  %t26 = load i32, i32* %sp
  %t27 = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 %t26
  store %Value %t25, %Value* %t27
  %t28 = add i32 %t26, 1
  store i32 %t28, i32* %sp
  ; Instruction 5: Jump(6)
  br label %block2
block1:
  ; Instruction 6: Load("n")
  %t29 = load %Value, %Value* %local_n
  %t30 = load i32, i32* %sp
  %t31 = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 %t30
  store %Value %t29, %Value* %t31
  %t32 = add i32 %t30, 1
  store i32 %t32, i32* %sp
  ; Instruction 7: Load("n")
  %t33 = load %Value, %Value* %local_n
  %t34 = load i32, i32* %sp
  %t35 = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 %t34
  store %Value %t33, %Value* %t35
  %t36 = add i32 %t34, 1
  store i32 %t36, i32* %sp
  ; Instruction 8: Const(Int(1))
  %t37 = call %Value @val_int(i64 1)
  %t38 = load i32, i32* %sp
  %t39 = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 %t38
  store %Value %t37, %Value* %t39
  %t40 = add i32 %t38, 1
  store i32 %t40, i32* %sp
  ; Instruction 9: Sub
  %t41 = load i32, i32* %sp
  %t42 = sub i32 %t41, 1
  store i32 %t42, i32* %sp
  %t43 = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 %t42
  %t44 = load %Value, %Value* %t43
  %t45 = load i32, i32* %sp
  %t46 = sub i32 %t45, 1
  store i32 %t46, i32* %sp
  %t47 = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 %t46
  %t48 = load %Value, %Value* %t47
  %t49 = call %Value @val_sub(%Value %t48, %Value %t44)
  %t50 = load i32, i32* %sp
  %t51 = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 %t50
  store %Value %t49, %Value* %t51
  %t52 = add i32 %t50, 1
  store i32 %t52, i32* %sp
  ; Instruction 10: Call("factorial", 1)
  %t53 = load i32, i32* %sp
  %t54 = sub i32 %t53, 1
  store i32 %t54, i32* %sp
  %t55 = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 %t54
  %t56 = load %Value, %Value* %t55
  %t57 = call %Value @factorial(%Value %t56)
  %t58 = load i32, i32* %sp
  %t59 = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 %t58
  store %Value %t57, %Value* %t59
  %t60 = add i32 %t58, 1
  store i32 %t60, i32* %sp
  ; Instruction 11: Mul
  %t61 = load i32, i32* %sp
  %t62 = sub i32 %t61, 1
  store i32 %t62, i32* %sp
  %t63 = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 %t62
  %t64 = load %Value, %Value* %t63
  %t65 = load i32, i32* %sp
  %t66 = sub i32 %t65, 1
  store i32 %t66, i32* %sp
  %t67 = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 %t66
  %t68 = load %Value, %Value* %t67
  %t69 = call %Value @val_mul(%Value %t68, %Value %t64)
  %t70 = load i32, i32* %sp
  %t71 = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 %t70
  store %Value %t69, %Value* %t71
  %t72 = add i32 %t70, 1
  store i32 %t72, i32* %sp
  br label %block2
block2:
  ; Instruction 12: Return
  %t73 = load i32, i32* %sp
  %t74 = sub i32 %t73, 1
  store i32 %t74, i32* %sp
  %t75 = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 %t74
  %t76 = load %Value, %Value* %t75
  ret %Value %t76
block3:
  %t77 = load i32, i32* %sp
  %t78 = icmp sgt i32 %t77, 0
  br i1 %t78, label %return_stack, label %block4
return_stack:
  %t79 = load i32, i32* %sp
  %t80 = sub i32 %t79, 1
  store i32 %t80, i32* %sp
  %t81 = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 %t80
  %t82 = load %Value, %Value* %t81
  ret %Value %t82
block4:
  %t83 = call %Value @val_void()
  ret %Value %t83
}

; Function: __main__
define %Value @__main__() {
entry:
  ; Stack allocation
  %stack = alloca [64 x %Value]
  %sp = alloca i32
  store i32 0, i32* %sp


  ; Instruction 0: Const(Int(10))
  %t0 = call %Value @val_int(i64 10)
  %t1 = load i32, i32* %sp
  %t2 = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 %t1
  store %Value %t0, %Value* %t2
  %t3 = add i32 %t1, 1
  store i32 %t3, i32* %sp
  ; Instruction 1: Call("factorial", 1)
  %t4 = load i32, i32* %sp
  %t5 = sub i32 %t4, 1
  store i32 %t5, i32* %sp
  %t6 = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 %t5
  %t7 = load %Value, %Value* %t6
  %t8 = call %Value @factorial(%Value %t7)
  %t9 = load i32, i32* %sp
  %t10 = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 %t9
  store %Value %t8, %Value* %t10
  %t11 = add i32 %t9, 1
  store i32 %t11, i32* %sp
  br label %block0
block0:
  %t12 = load i32, i32* %sp
  %t13 = icmp sgt i32 %t12, 0
  br i1 %t13, label %return_stack, label %block1
return_stack:
  %t14 = load i32, i32* %sp
  %t15 = sub i32 %t14, 1
  store i32 %t15, i32* %sp
  %t16 = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 %t15
  %t17 = load %Value, %Value* %t16
  ret %Value %t17
block1:
  %t18 = call %Value @val_void()
  ret %Value %t18
}

; Main entry point
define i32 @main() {
  %result = call %Value @__main__()
  call void @val_print(%Value %result)
  %nl = getelementptr [2 x i8], [2 x i8]* @.str.newline, i64 0, i64 0
  call i32 (i8*, ...) @printf(i8* %nl)
  ret i32 0
}

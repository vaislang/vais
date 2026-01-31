; ModuleID = 'contract_macro_test'
source_filename = "<vais>"

declare i32 @rand()
declare i32 @strncmp(i8*, i8*, i64)
declare i32 @printf(i8*, ...)
declare i64 @fseek(i64, i64, i64)
declare i64 @ftell(i64)
declare i64 @vais_gc_set_threshold(i64)
declare i64 @strlen(i8*)
declare i32 @isdigit(i32)
declare i32 @toupper(i32)
declare i32 @fclose(i64)
declare i64 @memcpy_str(i64, i8*, i64)
declare i32 @putchar(i32)
declare i64 @fopen(i8*, i8*)
declare i32 @usleep(i64)
declare i32 @puts(i8*)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @malloc(i64)
declare i32 @strcmp(i8*, i8*)
declare i64 @vais_gc_alloc(i64, i32)
declare i64 @vais_gc_print_stats()
declare i64 @vais_gc_collect()
declare i64 @vais_gc_collections()
declare i32 @sched_yield()
declare i64 @vais_gc_objects_count()
declare i32 @tolower(i32)
declare i64 @strcat(i64, i8*)
declare i64 @fputc(i64, i64)
declare i64 @memcpy(i64, i64, i64)
declare i64 @strcpy(i64, i8*)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @labs(i64)
declare i64 @feof(i64)
declare void @srand(i32)
declare void @free(i64)
declare i64 @fgetc(i64)
declare i64 @vais_gc_init()
declare void @exit(i32)
declare i64 @vais_gc_add_root(i64)
declare i64 @vais_gc_remove_root(i64)
declare i64 @fputs(i8*, i64)
declare i64 @vais_gc_bytes_allocated()
declare i64 @fflush(i64)
declare i64 @fgets(i64, i64, i64)
declare i64 @atol(i8*)
declare double @atof(i8*)
declare i32 @isalpha(i32)
declare double @fabs(double)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare double @sqrt(double)
declare i32 @atoi(i8*)
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

define i64 @safe_divide(i64 %a, i64 %b) {
entry:
  %nonzero_cond_1 = icmp ne i64 %b, 0
  br i1 %nonzero_cond_1, label %nonzero_ok_b_0, label %nonzero_fail_b_0
nonzero_fail_b_0:
  call i64 @__contract_fail(i64 1, i8* getelementptr inbounds ([26 x i8], [26 x i8]* @.str.contract.0, i64 0, i64 0), i8* getelementptr inbounds ([8 x i8], [8 x i8]* @.str.contract.1, i64 0, i64 0), i64 0, i8* getelementptr inbounds ([12 x i8], [12 x i8]* @.str.contract.2, i64 0, i64 0))
  unreachable
nonzero_ok_b_0:
  %2 = sdiv i64 %a, %b
  ret i64 %2
}

define i64 @safe_modulo(i64 %a, i64 %b) {
entry:
  %nonzero_cond_1 = icmp ne i64 %b, 0
  br i1 %nonzero_cond_1, label %nonzero_ok_b_0, label %nonzero_fail_b_0
nonzero_fail_b_0:
  call i64 @__contract_fail(i64 1, i8* getelementptr inbounds ([26 x i8], [26 x i8]* @.str.contract.0, i64 0, i64 0), i8* getelementptr inbounds ([8 x i8], [8 x i8]* @.str.contract.1, i64 0, i64 0), i64 0, i8* getelementptr inbounds ([12 x i8], [12 x i8]* @.str.contract.3, i64 0, i64 0))
  unreachable
nonzero_ok_b_0:
  %2 = srem i64 %a, %b
  ret i64 %2
}

define i64 @auto_infer(i64 %divisor, i64 %modulo) {
entry:
  %nonzero_cond_1 = icmp ne i64 %divisor, 0
  br i1 %nonzero_cond_1, label %nonzero_ok_divisor_0, label %nonzero_fail_divisor_0
nonzero_fail_divisor_0:
  call i64 @__contract_fail(i64 1, i8* getelementptr inbounds ([32 x i8], [32 x i8]* @.str.contract.4, i64 0, i64 0), i8* getelementptr inbounds ([8 x i8], [8 x i8]* @.str.contract.1, i64 0, i64 0), i64 0, i8* getelementptr inbounds ([11 x i8], [11 x i8]* @.str.contract.5, i64 0, i64 0))
  unreachable
nonzero_ok_divisor_0:
  %nonzero_cond_3 = icmp ne i64 %modulo, 0
  br i1 %nonzero_cond_3, label %nonzero_ok_modulo_2, label %nonzero_fail_modulo_2
nonzero_fail_modulo_2:
  call i64 @__contract_fail(i64 1, i8* getelementptr inbounds ([31 x i8], [31 x i8]* @.str.contract.6, i64 0, i64 0), i8* getelementptr inbounds ([8 x i8], [8 x i8]* @.str.contract.1, i64 0, i64 0), i64 0, i8* getelementptr inbounds ([11 x i8], [11 x i8]* @.str.contract.5, i64 0, i64 0))
  unreachable
nonzero_ok_modulo_2:
  %4 = sdiv i64 100, %divisor
  %5 = srem i64 %4, %modulo
  ret i64 %5
}

define i64 @complex_div(i64 %a, i64 %b, i64 %c) {
entry:
  %nonzero_cond_1 = icmp ne i64 %b, 0
  br i1 %nonzero_cond_1, label %nonzero_ok_b_0, label %nonzero_fail_b_0
nonzero_fail_b_0:
  call i64 @__contract_fail(i64 1, i8* getelementptr inbounds ([26 x i8], [26 x i8]* @.str.contract.0, i64 0, i64 0), i8* getelementptr inbounds ([8 x i8], [8 x i8]* @.str.contract.1, i64 0, i64 0), i64 0, i8* getelementptr inbounds ([12 x i8], [12 x i8]* @.str.contract.7, i64 0, i64 0))
  unreachable
nonzero_ok_b_0:
  %nonzero_cond_3 = icmp ne i64 %c, 0
  br i1 %nonzero_cond_3, label %nonzero_ok_c_2, label %nonzero_fail_c_2
nonzero_fail_c_2:
  call i64 @__contract_fail(i64 1, i8* getelementptr inbounds ([26 x i8], [26 x i8]* @.str.contract.8, i64 0, i64 0), i8* getelementptr inbounds ([8 x i8], [8 x i8]* @.str.contract.1, i64 0, i64 0), i64 0, i8* getelementptr inbounds ([12 x i8], [12 x i8]* @.str.contract.7, i64 0, i64 0))
  unreachable
nonzero_ok_c_2:
  %4 = sdiv i64 %a, %b
  %5 = sdiv i64 %a, %c
  %6 = add i64 %4, %5
  ret i64 %6
}

define i64 @main() {
entry:
  %0 = call i64 @safe_divide(i64 10, i64 2)
  %1 = call i64 @safe_modulo(i64 10, i64 3)
  %2 = call i64 @auto_infer(i64 5, i64 3)
  %3 = call i64 @complex_div(i64 100, i64 5, i64 10)
  %4 = add i64 %0, %1
  %5 = add i64 %4, %2
  %6 = add i64 %5, %3
  ret i64 %6
}


; C library function declarations
declare i64 @write(i32, i8*, i64)

; Global constants for runtime functions
@.panic_newline = private unnamed_addr constant [2 x i8] c"\0A\00"

; Runtime panic function (used by assert)
define i64 @__panic(i8* %msg) {
entry:
  ; Calculate message length
  %len = call i64 @strlen(i8* %msg)
  ; Write message to stderr (fd=2)
  %0 = call i64 @write(i32 2, i8* %msg, i64 %len)
  ; Write newline
  %1 = call i64 @write(i32 2, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @.panic_newline, i64 0, i64 0), i64 1)
  call void @exit(i32 1)
  unreachable
}

; Runtime contract failure function
define i64 @__contract_fail(i64 %kind, i8* %condition, i8* %file, i64 %line, i8* %func) {
entry:
  ; Calculate message length
  %len = call i64 @strlen(i8* %condition)
  ; Write contract failure message to stderr (fd=2)
  %0 = call i64 @write(i32 2, i8* %condition, i64 %len)
  ; Write newline
  %1 = call i64 @write(i32 2, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @.panic_newline, i64 0, i64 0), i64 1)
  call void @exit(i32 1)
  unreachable
}

; Helper function: load byte from memory
define i64 @__load_byte(i64 %ptr) {
entry:
  %0 = inttoptr i64 %ptr to i8*
  %1 = load i8, i8* %0
  %2 = zext i8 %1 to i64
  ret i64 %2
}

; Helper function: store byte to memory
define void @__store_byte(i64 %ptr, i64 %val) {
entry:
  %0 = inttoptr i64 %ptr to i8*
  %1 = trunc i64 %val to i8
  store i8 %1, i8* %0
  ret void
}

; Helper function: load i64 from memory
define i64 @__load_i64(i64 %ptr) {
entry:
  %0 = inttoptr i64 %ptr to i64*
  %1 = load i64, i64* %0
  ret i64 %1
}

; Helper function: store i64 to memory
define void @__store_i64(i64 %ptr, i64 %val) {
entry:
  %0 = inttoptr i64 %ptr to i64*
  store i64 %val, i64* %0
  ret void
}

; Helper function: load f64 from memory
define double @__load_f64(i64 %ptr) {
entry:
  %0 = inttoptr i64 %ptr to double*
  %1 = load double, double* %0
  ret double %1
}

; Helper function: store f64 to memory
define void @__store_f64(i64 %ptr, double %val) {
entry:
  %0 = inttoptr i64 %ptr to double*
  store double %val, double* %0
  ret void
}
; Contract runtime declarations
declare void @llvm.assume(i1)

@.str.contract.1 = private unnamed_addr constant [8 x i8] c"unknown\00"
@.str.contract.2 = private unnamed_addr constant [12 x i8] c"safe_divide\00"
@.str.contract.8 = private unnamed_addr constant [26 x i8] c"c != 0 (division by zero)\00"
@.str.contract.7 = private unnamed_addr constant [12 x i8] c"complex_div\00"
@.str.contract.4 = private unnamed_addr constant [32 x i8] c"divisor != 0 (division by zero)\00"
@.str.contract.3 = private unnamed_addr constant [12 x i8] c"safe_modulo\00"
@.str.contract.0 = private unnamed_addr constant [26 x i8] c"b != 0 (division by zero)\00"
@.str.contract.5 = private unnamed_addr constant [11 x i8] c"auto_infer\00"
@.str.contract.6 = private unnamed_addr constant [31 x i8] c"modulo != 0 (division by zero)\00"

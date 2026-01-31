; ModuleID = 'spawn_test'
source_filename = "<vais>"

declare i64 @vais_gc_alloc(i64, i32)
declare i64 @vais_gc_set_threshold(i64)
declare i32 @rand()
declare i32 @putchar(i32)
declare i64 @memcpy(i64, i64, i64)
declare i64 @strcpy(i64, i8*)
declare i64 @fopen(i8*, i8*)
declare i64 @vais_gc_add_root(i64)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @fgets(i64, i64, i64)
declare i64 @fflush(i64)
declare i64 @atol(i64)
declare i64 @vais_gc_collect()
declare i64 @memcpy_str(i64, i8*, i64)
declare i32 @strcmp(i8*, i8*)
declare i32 @atoi(i8*)
declare i64 @fputs(i8*, i64)
declare i64 @vais_gc_init()
declare i64 @malloc(i64)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare i32 @fclose(i64)
declare i64 @fgetc(i64)
declare i32 @puts(i64)
declare double @fabs(double)
declare void @free(i64)
declare double @sqrt(double)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @vais_gc_objects_count()
declare i32 @usleep(i64)
declare i64 @fputc(i64, i64)
declare i32 @printf(i8*, ...)
declare i32 @sched_yield()
declare void @exit(i32)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @vais_gc_bytes_allocated()
declare i64 @vais_gc_print_stats()
declare i64 @labs(i64)
declare i64 @vais_gc_remove_root(i64)
declare double @atof(i8*)
declare i32 @isalpha(i32)
declare i64 @strlen(i8*)
declare i64 @vais_gc_collections()
declare i32 @toupper(i32)
declare void @srand(i32)
declare i32 @isdigit(i32)
declare i32 @tolower(i32)
declare i64 @strcat(i64, i8*)
declare i64 @fseek(i64, i64, i64)
declare i64 @ftell(i64)
declare i64 @feof(i64)
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

@.str.0 = private unnamed_addr constant [19 x i8] c"=== Spawn Test ===\00"
@.str.1 = private unnamed_addr constant [34 x i8] c"Test 1: Spawn and immediate await\00"
@.str.2 = private unnamed_addr constant [22 x i8] c"  slow_compute(10) = \00"
@.str.3 = private unnamed_addr constant [27 x i8] c"Test 2: Direct async calls\00"
@.str.4 = private unnamed_addr constant [21 x i8] c"  slow_compute(5) = \00"
@.str.5 = private unnamed_addr constant [21 x i8] c"  fast_compute(5) = \00"
@.str.6 = private unnamed_addr constant [22 x i8] c"Test 3: Chained async\00"
@.str.7 = private unnamed_addr constant [22 x i8] c"  chain_compute(5) = \00"
@.str.8 = private unnamed_addr constant [28 x i8] c"=== Spawn Test Complete ===\00"

; Async state struct for slow_compute
%slow_compute__AsyncState = type { i64, i64, i64 }

; Create function for async slow_compute
define i64 @slow_compute(i64 %x) {
entry:
  %state_ptr = call i64 @malloc(i64 24)
  %state = inttoptr i64 %state_ptr to %slow_compute__AsyncState*
  %state_field = getelementptr %slow_compute__AsyncState, %slow_compute__AsyncState* %state, i32 0, i32 0
  store i64 0, i64* %state_field
  %param_x_ptr = getelementptr %slow_compute__AsyncState, %slow_compute__AsyncState* %state, i32 0, i32 2
  store i64 %x, i64* %param_x_ptr
  ret i64 %state_ptr
}

; Poll function for async slow_compute
define { i64, i64 } @slow_compute__poll(i64 %state_ptr) {
entry:
  %state = inttoptr i64 %state_ptr to %slow_compute__AsyncState*
  %state_field = getelementptr %slow_compute__AsyncState, %slow_compute__AsyncState* %state, i32 0, i32 0
  %current_state = load i64, i64* %state_field
  %param_x_ptr = getelementptr %slow_compute__AsyncState, %slow_compute__AsyncState* %state, i32 0, i32 2
  %x = load i64, i64* %param_x_ptr
  switch i64 %current_state, label %state_invalid [
    i64 0, label %state_0
  ]

state_0:
  %0 = mul i64 %x, 2
  %result_ptr = getelementptr %slow_compute__AsyncState, %slow_compute__AsyncState* %state, i32 0, i32 1
  store i64 %0, i64* %result_ptr
  store i64 -1, i64* %state_field
  %ret_val = load i64, i64* %result_ptr
  %ret_0 = insertvalue { i64, i64 } undef, i64 1, 0
  %ret_1 = insertvalue { i64, i64 } %ret_0, i64 %ret_val, 1
  ret { i64, i64 } %ret_1

state_invalid:
  %invalid_ret = insertvalue { i64, i64 } undef, i64 0, 0
  ret { i64, i64 } %invalid_ret
}

; Async state struct for fast_compute
%fast_compute__AsyncState = type { i64, i64, i64 }

; Create function for async fast_compute
define i64 @fast_compute(i64 %x) {
entry:
  %state_ptr = call i64 @malloc(i64 24)
  %state = inttoptr i64 %state_ptr to %fast_compute__AsyncState*
  %state_field = getelementptr %fast_compute__AsyncState, %fast_compute__AsyncState* %state, i32 0, i32 0
  store i64 0, i64* %state_field
  %param_x_ptr = getelementptr %fast_compute__AsyncState, %fast_compute__AsyncState* %state, i32 0, i32 2
  store i64 %x, i64* %param_x_ptr
  ret i64 %state_ptr
}

; Poll function for async fast_compute
define { i64, i64 } @fast_compute__poll(i64 %state_ptr) {
entry:
  %state = inttoptr i64 %state_ptr to %fast_compute__AsyncState*
  %state_field = getelementptr %fast_compute__AsyncState, %fast_compute__AsyncState* %state, i32 0, i32 0
  %current_state = load i64, i64* %state_field
  %param_x_ptr = getelementptr %fast_compute__AsyncState, %fast_compute__AsyncState* %state, i32 0, i32 2
  %x = load i64, i64* %param_x_ptr
  switch i64 %current_state, label %state_invalid [
    i64 0, label %state_0
  ]

state_0:
  %0 = add i64 %x, 1
  %result_ptr = getelementptr %fast_compute__AsyncState, %fast_compute__AsyncState* %state, i32 0, i32 1
  store i64 %0, i64* %result_ptr
  store i64 -1, i64* %state_field
  %ret_val = load i64, i64* %result_ptr
  %ret_0 = insertvalue { i64, i64 } undef, i64 1, 0
  %ret_1 = insertvalue { i64, i64 } %ret_0, i64 %ret_val, 1
  ret { i64, i64 } %ret_1

state_invalid:
  %invalid_ret = insertvalue { i64, i64 } undef, i64 0, 0
  ret { i64, i64 } %invalid_ret
}

; Async state struct for chain_compute
%chain_compute__AsyncState = type { i64, i64, i64 }

; Create function for async chain_compute
define i64 @chain_compute(i64 %x) {
entry:
  %state_ptr = call i64 @malloc(i64 24)
  %state = inttoptr i64 %state_ptr to %chain_compute__AsyncState*
  %state_field = getelementptr %chain_compute__AsyncState, %chain_compute__AsyncState* %state, i32 0, i32 0
  store i64 0, i64* %state_field
  %param_x_ptr = getelementptr %chain_compute__AsyncState, %chain_compute__AsyncState* %state, i32 0, i32 2
  store i64 %x, i64* %param_x_ptr
  ret i64 %state_ptr
}

; Poll function for async chain_compute
define { i64, i64 } @chain_compute__poll(i64 %state_ptr) {
entry:
  %state = inttoptr i64 %state_ptr to %chain_compute__AsyncState*
  %state_field = getelementptr %chain_compute__AsyncState, %chain_compute__AsyncState* %state, i32 0, i32 0
  %current_state = load i64, i64* %state_field
  %param_x_ptr = getelementptr %chain_compute__AsyncState, %chain_compute__AsyncState* %state, i32 0, i32 2
  %x = load i64, i64* %param_x_ptr
  switch i64 %current_state, label %state_invalid [
    i64 0, label %state_0
  ]

state_0:
  %0 = call i64 @slow_compute(i64 %x)
  br label %await_poll0

await_poll0:
  %1 = call { i64, i64 } @slow_compute__poll(i64 %0)
  %2 = extractvalue { i64, i64 } %1, 0
  %3 = icmp eq i64 %2, 1
  br i1 %3, label %await_ready1, label %await_pending2

await_pending2:
  br label %await_poll0

await_ready1:
  %4 = extractvalue { i64, i64 } %1, 1
  %6 = call i64 @fast_compute(i64 %4)
  br label %await_poll3

await_poll3:
  %7 = call { i64, i64 } @fast_compute__poll(i64 %6)
  %8 = extractvalue { i64, i64 } %7, 0
  %9 = icmp eq i64 %8, 1
  br i1 %9, label %await_ready4, label %await_pending5

await_pending5:
  br label %await_poll3

await_ready4:
  %10 = extractvalue { i64, i64 } %7, 1
  %12 = add i64 %4, %10
  %result_ptr = getelementptr %chain_compute__AsyncState, %chain_compute__AsyncState* %state, i32 0, i32 1
  store i64 %12, i64* %result_ptr
  store i64 -1, i64* %state_field
  %ret_val = load i64, i64* %result_ptr
  %ret_0 = insertvalue { i64, i64 } undef, i64 1, 0
  %ret_1 = insertvalue { i64, i64 } %ret_0, i64 %ret_val, 1
  ret { i64, i64 } %ret_1

state_invalid:
  %invalid_ret = insertvalue { i64, i64 } undef, i64 0, 0
  ret { i64, i64 } %invalid_ret
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([19 x i8], [19 x i8]* @.str.0, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = trunc i64 10 to i32
  %3 = call i32 @putchar(i32 %2)
  %4 = sext i32 %3 to i64
  %5 = call i32 @puts(i8* getelementptr ([34 x i8], [34 x i8]* @.str.1, i64 0, i64 0))
  %6 = sext i32 %5 to i64
  %7 = call i64 @slow_compute(i64 10)
; Spawned task at %7
  br label %await_poll0

await_poll0:
  %8 = call { i64, i64 } @slow_compute__poll(i64 %7)
  %9 = extractvalue { i64, i64 } %8, 0
  %10 = icmp eq i64 %9, 1
  br i1 %10, label %await_ready1, label %await_pending2

await_pending2:
  br label %await_poll0

await_ready1:
  %11 = extractvalue { i64, i64 } %8, 1
  %12 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.2, i64 0, i64 0))
  %13 = sext i32 %12 to i64
  %14 = sdiv i64 %11, 10
  %15 = add i64 %14, 48
  %16 = trunc i64 %15 to i32
  %17 = call i32 @putchar(i32 %16)
  %18 = sext i32 %17 to i64
  %19 = srem i64 %11, 10
  %20 = add i64 %19, 48
  %21 = trunc i64 %20 to i32
  %22 = call i32 @putchar(i32 %21)
  %23 = sext i32 %22 to i64
  %24 = trunc i64 10 to i32
  %25 = call i32 @putchar(i32 %24)
  %26 = sext i32 %25 to i64
  %27 = call i32 @puts(i8* getelementptr ([27 x i8], [27 x i8]* @.str.3, i64 0, i64 0))
  %28 = sext i32 %27 to i64
  %29 = call i64 @slow_compute(i64 5)
  br label %await_poll3

await_poll3:
  %30 = call { i64, i64 } @slow_compute__poll(i64 %29)
  %31 = extractvalue { i64, i64 } %30, 0
  %32 = icmp eq i64 %31, 1
  br i1 %32, label %await_ready4, label %await_pending5

await_pending5:
  br label %await_poll3

await_ready4:
  %33 = extractvalue { i64, i64 } %30, 1
  %34 = call i64 @fast_compute(i64 5)
  br label %await_poll6

await_poll6:
  %35 = call { i64, i64 } @fast_compute__poll(i64 %34)
  %36 = extractvalue { i64, i64 } %35, 0
  %37 = icmp eq i64 %36, 1
  br i1 %37, label %await_ready7, label %await_pending8

await_pending8:
  br label %await_poll6

await_ready7:
  %38 = extractvalue { i64, i64 } %35, 1
  %39 = call i32 @puts(i8* getelementptr ([21 x i8], [21 x i8]* @.str.4, i64 0, i64 0))
  %40 = sext i32 %39 to i64
  %41 = sdiv i64 %33, 10
  %42 = add i64 %41, 48
  %43 = trunc i64 %42 to i32
  %44 = call i32 @putchar(i32 %43)
  %45 = sext i32 %44 to i64
  %46 = srem i64 %33, 10
  %47 = add i64 %46, 48
  %48 = trunc i64 %47 to i32
  %49 = call i32 @putchar(i32 %48)
  %50 = sext i32 %49 to i64
  %51 = trunc i64 10 to i32
  %52 = call i32 @putchar(i32 %51)
  %53 = sext i32 %52 to i64
  %54 = call i32 @puts(i8* getelementptr ([21 x i8], [21 x i8]* @.str.5, i64 0, i64 0))
  %55 = sext i32 %54 to i64
  %56 = add i64 %38, 48
  %57 = trunc i64 %56 to i32
  %58 = call i32 @putchar(i32 %57)
  %59 = sext i32 %58 to i64
  %60 = trunc i64 10 to i32
  %61 = call i32 @putchar(i32 %60)
  %62 = sext i32 %61 to i64
  %63 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.6, i64 0, i64 0))
  %64 = sext i32 %63 to i64
  %65 = call i64 @chain_compute(i64 5)
  br label %await_poll9

await_poll9:
  %66 = call { i64, i64 } @chain_compute__poll(i64 %65)
  %67 = extractvalue { i64, i64 } %66, 0
  %68 = icmp eq i64 %67, 1
  br i1 %68, label %await_ready10, label %await_pending11

await_pending11:
  br label %await_poll9

await_ready10:
  %69 = extractvalue { i64, i64 } %66, 1
  %70 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.7, i64 0, i64 0))
  %71 = sext i32 %70 to i64
  %72 = sdiv i64 %69, 10
  %73 = add i64 %72, 48
  %74 = trunc i64 %73 to i32
  %75 = call i32 @putchar(i32 %74)
  %76 = sext i32 %75 to i64
  %77 = srem i64 %69, 10
  %78 = add i64 %77, 48
  %79 = trunc i64 %78 to i32
  %80 = call i32 @putchar(i32 %79)
  %81 = sext i32 %80 to i64
  %82 = trunc i64 10 to i32
  %83 = call i32 @putchar(i32 %82)
  %84 = sext i32 %83 to i64
  %85 = call i32 @puts(i8* getelementptr ([28 x i8], [28 x i8]* @.str.8, i64 0, i64 0))
  %86 = sext i32 %85 to i64
  ret i64 0
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

; ModuleID = 'runtime_test'
source_filename = "<vais>"

declare double @atof(i8*)
declare i32 @fclose(i64)
declare i32 @printf(i8*, ...)
declare void @exit(i32)
declare i32 @atoi(i8*)
declare void @srand(i32)
declare i32 @isalpha(i32)
declare i64 @strcat(i64, i8*)
declare i64 @fgets(i64, i64, i64)
declare i64 @fputs(i8*, i64)
declare i64 @strcpy(i64, i8*)
declare i64 @vais_gc_add_root(i64)
declare i64 @vais_gc_alloc(i64, i32)
declare i64 @vais_gc_init()
declare i64 @fflush(i64)
declare i32 @rand()
declare i32 @usleep(i64)
declare i64 @labs(i64)
declare double @sqrt(double)
declare i32 @puts(i64)
declare i64 @fread(i64, i64, i64, i64)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare i32 @putchar(i32)
declare i64 @memcpy(i64, i64, i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @strlen(i8*)
declare i64 @vais_gc_remove_root(i64)
declare i64 @atol(i8*)
declare i32 @toupper(i32)
declare i64 @fputc(i64, i64)
declare i64 @feof(i64)
declare i64 @vais_gc_set_threshold(i64)
declare i64 @fseek(i64, i64, i64)
declare i32 @tolower(i32)
declare i32 @isdigit(i32)
declare i64 @vais_gc_collect()
declare void @free(i64)
declare i64 @vais_gc_objects_count()
declare i64 @vais_gc_collections()
declare i64 @fopen(i8*, i8*)
declare i64 @ftell(i64)
declare i64 @vais_gc_print_stats()
declare i32 @strcmp(i8*, i8*)
declare i64 @malloc(i64)
declare i64 @vais_gc_bytes_allocated()
declare i64 @memcpy_str(i64, i8*, i64)
declare i32 @sched_yield()
declare i64 @fgetc(i64)
declare double @fabs(double)
declare i32 @strncmp(i8*, i8*, i64)
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

@.str.0 = private unnamed_addr constant [27 x i8] c"=== Async Runtime Test ===\00"
@.str.1 = private unnamed_addr constant [27 x i8] c"Testing sequential awaits:\00"
@.str.2 = private unnamed_addr constant [24 x i8] c"  compute_double(10) = \00"
@.str.3 = private unnamed_addr constant [24 x i8] c"  compute_double(21) = \00"
@.str.4 = private unnamed_addr constant [24 x i8] c"  compute_sum(1,2,3) = \00"
@.str.5 = private unnamed_addr constant [12 x i8] c"Total sum: \00"
@.str.6 = private unnamed_addr constant [22 x i8] c"=== Test Complete ===\00"

; Async state struct for compute_double
%compute_double__AsyncState = type { i64, i64, i64 }

; Create function for async compute_double
define i64 @compute_double(i64 %x) {
entry:
  %state_ptr = call i64 @malloc(i64 24)
  %state = inttoptr i64 %state_ptr to %compute_double__AsyncState*
  %state_field = getelementptr %compute_double__AsyncState, %compute_double__AsyncState* %state, i32 0, i32 0
  store i64 0, i64* %state_field
  %param_x_ptr = getelementptr %compute_double__AsyncState, %compute_double__AsyncState* %state, i32 0, i32 2
  store i64 %x, i64* %param_x_ptr
  ret i64 %state_ptr
}

; Poll function for async compute_double
define { i64, i64 } @compute_double__poll(i64 %state_ptr) {
entry:
  %state = inttoptr i64 %state_ptr to %compute_double__AsyncState*
  %state_field = getelementptr %compute_double__AsyncState, %compute_double__AsyncState* %state, i32 0, i32 0
  %current_state = load i64, i64* %state_field
  %param_x_ptr = getelementptr %compute_double__AsyncState, %compute_double__AsyncState* %state, i32 0, i32 2
  %x = load i64, i64* %param_x_ptr
  switch i64 %current_state, label %state_invalid [
    i64 0, label %state_0
  ]

state_0:
  %0 = mul i64 %x, 2
  %result_ptr = getelementptr %compute_double__AsyncState, %compute_double__AsyncState* %state, i32 0, i32 1
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

; Async state struct for compute_sum
%compute_sum__AsyncState = type { i64, i64, i64, i64, i64 }

; Create function for async compute_sum
define i64 @compute_sum(i64 %a, i64 %b, i64 %c) {
entry:
  %state_ptr = call i64 @malloc(i64 40)
  %state = inttoptr i64 %state_ptr to %compute_sum__AsyncState*
  %state_field = getelementptr %compute_sum__AsyncState, %compute_sum__AsyncState* %state, i32 0, i32 0
  store i64 0, i64* %state_field
  %param_a_ptr = getelementptr %compute_sum__AsyncState, %compute_sum__AsyncState* %state, i32 0, i32 2
  store i64 %a, i64* %param_a_ptr
  %param_b_ptr = getelementptr %compute_sum__AsyncState, %compute_sum__AsyncState* %state, i32 0, i32 3
  store i64 %b, i64* %param_b_ptr
  %param_c_ptr = getelementptr %compute_sum__AsyncState, %compute_sum__AsyncState* %state, i32 0, i32 4
  store i64 %c, i64* %param_c_ptr
  ret i64 %state_ptr
}

; Poll function for async compute_sum
define { i64, i64 } @compute_sum__poll(i64 %state_ptr) {
entry:
  %state = inttoptr i64 %state_ptr to %compute_sum__AsyncState*
  %state_field = getelementptr %compute_sum__AsyncState, %compute_sum__AsyncState* %state, i32 0, i32 0
  %current_state = load i64, i64* %state_field
  %param_a_ptr = getelementptr %compute_sum__AsyncState, %compute_sum__AsyncState* %state, i32 0, i32 2
  %a = load i64, i64* %param_a_ptr
  %param_b_ptr = getelementptr %compute_sum__AsyncState, %compute_sum__AsyncState* %state, i32 0, i32 3
  %b = load i64, i64* %param_b_ptr
  %param_c_ptr = getelementptr %compute_sum__AsyncState, %compute_sum__AsyncState* %state, i32 0, i32 4
  %c = load i64, i64* %param_c_ptr
  switch i64 %current_state, label %state_invalid [
    i64 0, label %state_0
  ]

state_0:
  %0 = add i64 %a, %b
  %1 = add i64 %0, %c
  %result_ptr = getelementptr %compute_sum__AsyncState, %compute_sum__AsyncState* %state, i32 0, i32 1
  store i64 %1, i64* %result_ptr
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
  %0 = call i32 @puts(i8* getelementptr ([27 x i8], [27 x i8]* @.str.0, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = trunc i64 10 to i32
  %3 = call i32 @putchar(i32 %2)
  %4 = sext i32 %3 to i64
  %5 = call i32 @puts(i8* getelementptr ([27 x i8], [27 x i8]* @.str.1, i64 0, i64 0))
  %6 = sext i32 %5 to i64
  %7 = call i64 @compute_double(i64 10)
  br label %await_poll0

await_poll0:
  %8 = call { i64, i64 } @compute_double__poll(i64 %7)
  %9 = extractvalue { i64, i64 } %8, 0
  %10 = icmp eq i64 %9, 1
  br i1 %10, label %await_ready1, label %await_pending2

await_pending2:
  br label %await_poll0

await_ready1:
  %11 = extractvalue { i64, i64 } %8, 1
  %12 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.2, i64 0, i64 0))
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
  %27 = call i64 @compute_double(i64 21)
  br label %await_poll3

await_poll3:
  %28 = call { i64, i64 } @compute_double__poll(i64 %27)
  %29 = extractvalue { i64, i64 } %28, 0
  %30 = icmp eq i64 %29, 1
  br i1 %30, label %await_ready4, label %await_pending5

await_pending5:
  br label %await_poll3

await_ready4:
  %31 = extractvalue { i64, i64 } %28, 1
  %32 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.3, i64 0, i64 0))
  %33 = sext i32 %32 to i64
  %34 = sdiv i64 %31, 10
  %35 = add i64 %34, 48
  %36 = trunc i64 %35 to i32
  %37 = call i32 @putchar(i32 %36)
  %38 = sext i32 %37 to i64
  %39 = srem i64 %31, 10
  %40 = add i64 %39, 48
  %41 = trunc i64 %40 to i32
  %42 = call i32 @putchar(i32 %41)
  %43 = sext i32 %42 to i64
  %44 = trunc i64 10 to i32
  %45 = call i32 @putchar(i32 %44)
  %46 = sext i32 %45 to i64
  %47 = call i64 @compute_sum(i64 1, i64 2, i64 3)
  br label %await_poll6

await_poll6:
  %48 = call { i64, i64 } @compute_sum__poll(i64 %47)
  %49 = extractvalue { i64, i64 } %48, 0
  %50 = icmp eq i64 %49, 1
  br i1 %50, label %await_ready7, label %await_pending8

await_pending8:
  br label %await_poll6

await_ready7:
  %51 = extractvalue { i64, i64 } %48, 1
  %52 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.4, i64 0, i64 0))
  %53 = sext i32 %52 to i64
  %54 = add i64 %51, 48
  %55 = trunc i64 %54 to i32
  %56 = call i32 @putchar(i32 %55)
  %57 = sext i32 %56 to i64
  %58 = trunc i64 10 to i32
  %59 = call i32 @putchar(i32 %58)
  %60 = sext i32 %59 to i64
  %61 = trunc i64 10 to i32
  %62 = call i32 @putchar(i32 %61)
  %63 = sext i32 %62 to i64
  %64 = add i64 %11, %31
  %65 = add i64 %64, %51
  %66 = call i32 @puts(i8* getelementptr ([12 x i8], [12 x i8]* @.str.5, i64 0, i64 0))
  %67 = sext i32 %66 to i64
  %68 = sdiv i64 %65, 10
  %69 = add i64 %68, 48
  %70 = trunc i64 %69 to i32
  %71 = call i32 @putchar(i32 %70)
  %72 = sext i32 %71 to i64
  %73 = srem i64 %65, 10
  %74 = add i64 %73, 48
  %75 = trunc i64 %74 to i32
  %76 = call i32 @putchar(i32 %75)
  %77 = sext i32 %76 to i64
  %78 = trunc i64 10 to i32
  %79 = call i32 @putchar(i32 %78)
  %80 = sext i32 %79 to i64
  %81 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.6, i64 0, i64 0))
  %82 = sext i32 %81 to i64
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

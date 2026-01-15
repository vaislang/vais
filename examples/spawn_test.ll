; ModuleID = 'spawn_test'
source_filename = "<vais>"

declare i64 @ftell(i64)
declare i64 @fputc(i64, i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i32 @strcmp(i8*, i8*)
declare i64 @fgetc(i64)
declare i64 @fseek(i64, i64, i64)
declare i32 @usleep(i64)
declare i64 @feof(i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @fopen(i8*, i8*)
declare i64 @memcpy(i64, i64, i64)
declare i32 @printf(i8*)
declare i32 @fclose(i64)
declare i64 @fflush(i64)
declare i64 @fgets(i64, i64, i64)
declare i64 @fread(i64, i64, i64, i64)
declare i32 @puts(i64)
declare void @free(i64)
declare i64 @malloc(i64)
declare void @exit(i32)
declare i64 @strlen(i64)
declare i32 @sched_yield()
declare i64 @fputs(i8*, i64)
declare i32 @putchar(i32)
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
  %a.5 = alloca i64
  store i64 %4, i64* %a.5
  %6 = load i64, i64* %a.5
  %7 = call i64 @fast_compute(i64 %6)
  br label %await_poll3

await_poll3:
  %8 = call { i64, i64 } @fast_compute__poll(i64 %7)
  %9 = extractvalue { i64, i64 } %8, 0
  %10 = icmp eq i64 %9, 1
  br i1 %10, label %await_ready4, label %await_pending5

await_pending5:
  br label %await_poll3

await_ready4:
  %11 = extractvalue { i64, i64 } %8, 1
  %b.12 = alloca i64
  store i64 %11, i64* %b.12
  %13 = load i64, i64* %a.5
  %14 = load i64, i64* %b.12
  %15 = add i64 %13, %14
  %result_ptr = getelementptr %chain_compute__AsyncState, %chain_compute__AsyncState* %state, i32 0, i32 1
  store i64 %15, i64* %result_ptr
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
  %1 = trunc i64 10 to i32
  %2 = call i32 @putchar(i32 %1)
  %3 = call i32 @puts(i8* getelementptr ([34 x i8], [34 x i8]* @.str.1, i64 0, i64 0))
  %4 = call i64 @slow_compute(i64 10)
; Spawned task at %4
  br label %await_poll0

await_poll0:
  %5 = call { i64, i64 } @slow_compute__poll(i64 %4)
  %6 = extractvalue { i64, i64 } %5, 0
  %7 = icmp eq i64 %6, 1
  br i1 %7, label %await_ready1, label %await_pending2

await_pending2:
  br label %await_poll0

await_ready1:
  %8 = extractvalue { i64, i64 } %5, 1
  %result1.9 = alloca i64
  store i64 %8, i64* %result1.9
  %10 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.2, i64 0, i64 0))
  %11 = load i64, i64* %result1.9
  %12 = sdiv i64 %11, 10
  %13 = add i64 %12, 48
  %14 = trunc i64 %13 to i32
  %15 = call i32 @putchar(i32 %14)
  %16 = load i64, i64* %result1.9
  %17 = srem i64 %16, 10
  %18 = add i64 %17, 48
  %19 = trunc i64 %18 to i32
  %20 = call i32 @putchar(i32 %19)
  %21 = trunc i64 10 to i32
  %22 = call i32 @putchar(i32 %21)
  %23 = call i32 @puts(i8* getelementptr ([27 x i8], [27 x i8]* @.str.3, i64 0, i64 0))
  %24 = call i64 @slow_compute(i64 5)
  br label %await_poll3

await_poll3:
  %25 = call { i64, i64 } @slow_compute__poll(i64 %24)
  %26 = extractvalue { i64, i64 } %25, 0
  %27 = icmp eq i64 %26, 1
  br i1 %27, label %await_ready4, label %await_pending5

await_pending5:
  br label %await_poll3

await_ready4:
  %28 = extractvalue { i64, i64 } %25, 1
  %result_a.29 = alloca i64
  store i64 %28, i64* %result_a.29
  %30 = call i64 @fast_compute(i64 5)
  br label %await_poll6

await_poll6:
  %31 = call { i64, i64 } @fast_compute__poll(i64 %30)
  %32 = extractvalue { i64, i64 } %31, 0
  %33 = icmp eq i64 %32, 1
  br i1 %33, label %await_ready7, label %await_pending8

await_pending8:
  br label %await_poll6

await_ready7:
  %34 = extractvalue { i64, i64 } %31, 1
  %result_b.35 = alloca i64
  store i64 %34, i64* %result_b.35
  %36 = call i32 @puts(i8* getelementptr ([21 x i8], [21 x i8]* @.str.4, i64 0, i64 0))
  %37 = load i64, i64* %result_a.29
  %38 = sdiv i64 %37, 10
  %39 = add i64 %38, 48
  %40 = trunc i64 %39 to i32
  %41 = call i32 @putchar(i32 %40)
  %42 = load i64, i64* %result_a.29
  %43 = srem i64 %42, 10
  %44 = add i64 %43, 48
  %45 = trunc i64 %44 to i32
  %46 = call i32 @putchar(i32 %45)
  %47 = trunc i64 10 to i32
  %48 = call i32 @putchar(i32 %47)
  %49 = call i32 @puts(i8* getelementptr ([21 x i8], [21 x i8]* @.str.5, i64 0, i64 0))
  %50 = load i64, i64* %result_b.35
  %51 = add i64 %50, 48
  %52 = trunc i64 %51 to i32
  %53 = call i32 @putchar(i32 %52)
  %54 = trunc i64 10 to i32
  %55 = call i32 @putchar(i32 %54)
  %56 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.6, i64 0, i64 0))
  %57 = call i64 @chain_compute(i64 5)
  br label %await_poll9

await_poll9:
  %58 = call { i64, i64 } @chain_compute__poll(i64 %57)
  %59 = extractvalue { i64, i64 } %58, 0
  %60 = icmp eq i64 %59, 1
  br i1 %60, label %await_ready10, label %await_pending11

await_pending11:
  br label %await_poll9

await_ready10:
  %61 = extractvalue { i64, i64 } %58, 1
  %result3.62 = alloca i64
  store i64 %61, i64* %result3.62
  %63 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.7, i64 0, i64 0))
  %64 = load i64, i64* %result3.62
  %65 = sdiv i64 %64, 10
  %66 = add i64 %65, 48
  %67 = trunc i64 %66 to i32
  %68 = call i32 @putchar(i32 %67)
  %69 = load i64, i64* %result3.62
  %70 = srem i64 %69, 10
  %71 = add i64 %70, 48
  %72 = trunc i64 %71 to i32
  %73 = call i32 @putchar(i32 %72)
  %74 = trunc i64 10 to i32
  %75 = call i32 @putchar(i32 %74)
  %76 = call i32 @puts(i8* getelementptr ([28 x i8], [28 x i8]* @.str.8, i64 0, i64 0))
  ret i64 0
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

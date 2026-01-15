; ModuleID = 'runtime_test'
source_filename = "<vais>"

declare i64 @fseek(i64, i64, i64)
declare i64 @fgetc(i64)
declare i64 @memcpy(i64, i64, i64)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @fputs(i8*, i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i32 @fclose(i64)
declare i64 @fputc(i64, i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @strlen(i64)
declare i64 @feof(i64)
declare i32 @strcmp(i8*, i8*)
declare i32 @usleep(i64)
declare i32 @sched_yield()
declare i32 @puts(i8*)
declare i64 @fgets(i64, i64, i64)
declare i32 @putchar(i32)
declare i64 @fflush(i64)
declare i64 @ftell(i64)
declare i32 @printf(i8*)
declare i64 @malloc(i64)
declare i64 @fopen(i8*, i8*)
declare void @exit(i32)
declare void @free(i64)
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
  %1 = trunc i64 10 to i32
  %2 = call i32 @putchar(i32 %1)
  %3 = call i32 @puts(i8* getelementptr ([27 x i8], [27 x i8]* @.str.1, i64 0, i64 0))
  %4 = call i64 @compute_double(i64 10)
  br label %await_poll0

await_poll0:
  %5 = call { i64, i64 } @compute_double__poll(i64 %4)
  %6 = extractvalue { i64, i64 } %5, 0
  %7 = icmp eq i64 %6, 1
  br i1 %7, label %await_ready1, label %await_pending2

await_pending2:
  br label %await_poll0

await_ready1:
  %8 = extractvalue { i64, i64 } %5, 1
  %result1.9 = alloca i64
  store i64 %8, i64* %result1.9
  %10 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.2, i64 0, i64 0))
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
  %23 = call i64 @compute_double(i64 21)
  br label %await_poll3

await_poll3:
  %24 = call { i64, i64 } @compute_double__poll(i64 %23)
  %25 = extractvalue { i64, i64 } %24, 0
  %26 = icmp eq i64 %25, 1
  br i1 %26, label %await_ready4, label %await_pending5

await_pending5:
  br label %await_poll3

await_ready4:
  %27 = extractvalue { i64, i64 } %24, 1
  %result2.28 = alloca i64
  store i64 %27, i64* %result2.28
  %29 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.3, i64 0, i64 0))
  %30 = load i64, i64* %result2.28
  %31 = sdiv i64 %30, 10
  %32 = add i64 %31, 48
  %33 = trunc i64 %32 to i32
  %34 = call i32 @putchar(i32 %33)
  %35 = load i64, i64* %result2.28
  %36 = srem i64 %35, 10
  %37 = add i64 %36, 48
  %38 = trunc i64 %37 to i32
  %39 = call i32 @putchar(i32 %38)
  %40 = trunc i64 10 to i32
  %41 = call i32 @putchar(i32 %40)
  %42 = call i64 @compute_sum(i64 1, i64 2, i64 3)
  br label %await_poll6

await_poll6:
  %43 = call { i64, i64 } @compute_sum__poll(i64 %42)
  %44 = extractvalue { i64, i64 } %43, 0
  %45 = icmp eq i64 %44, 1
  br i1 %45, label %await_ready7, label %await_pending8

await_pending8:
  br label %await_poll6

await_ready7:
  %46 = extractvalue { i64, i64 } %43, 1
  %result3.47 = alloca i64
  store i64 %46, i64* %result3.47
  %48 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.4, i64 0, i64 0))
  %49 = load i64, i64* %result3.47
  %50 = add i64 %49, 48
  %51 = trunc i64 %50 to i32
  %52 = call i32 @putchar(i32 %51)
  %53 = trunc i64 10 to i32
  %54 = call i32 @putchar(i32 %53)
  %55 = trunc i64 10 to i32
  %56 = call i32 @putchar(i32 %55)
  %57 = load i64, i64* %result1.9
  %58 = load i64, i64* %result2.28
  %59 = add i64 %57, %58
  %60 = load i64, i64* %result3.47
  %61 = add i64 %59, %60
  %total.62 = alloca i64
  store i64 %61, i64* %total.62
  %63 = call i32 @puts(i8* getelementptr ([12 x i8], [12 x i8]* @.str.5, i64 0, i64 0))
  %64 = load i64, i64* %total.62
  %65 = sdiv i64 %64, 10
  %66 = add i64 %65, 48
  %67 = trunc i64 %66 to i32
  %68 = call i32 @putchar(i32 %67)
  %69 = load i64, i64* %total.62
  %70 = srem i64 %69, 10
  %71 = add i64 %70, 48
  %72 = trunc i64 %71 to i32
  %73 = call i32 @putchar(i32 %72)
  %74 = trunc i64 10 to i32
  %75 = call i32 @putchar(i32 %74)
  %76 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.6, i64 0, i64 0))
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

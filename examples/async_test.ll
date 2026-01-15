; ModuleID = 'async_test'
source_filename = "<vais>"

declare i64 @fputc(i64, i64)
declare i64 @fputs(i8*, i64)
declare void @free(i64)
declare i32 @fclose(i64)
declare i64 @fflush(i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @strlen(i64)
declare i64 @fopen(i8*, i8*)
declare i32 @puts(i64)
declare i32 @putchar(i32)
declare i32 @printf(i8*)
declare i64 @fread(i64, i64, i64, i64)
declare i32 @strcmp(i8*, i8*)
declare void @exit(i32)
declare i64 @memcpy(i64, i64, i64)
declare i64 @fgetc(i64)
declare i64 @feof(i64)
declare i64 @ftell(i64)
declare i64 @malloc(i64)
declare i64 @fgets(i64, i64, i64)
declare i64 @fseek(i64, i64, i64)
@.str.0 = private unnamed_addr constant [37 x i8] c"Testing async/await with coroutines:\00"
@.str.1 = private unnamed_addr constant [20 x i8] c"compute(21).await =\00"
@.str.2 = private unnamed_addr constant [26 x i8] c"add_values(10, 5).await =\00"
@.str.3 = private unnamed_addr constant [21 x i8] c"Async test complete!\00"

; Async state struct for compute
%compute__AsyncState = type { i64, i64, i64 }

; Create function for async compute
define i64 @compute(i64 %x) {
entry:
  %state_ptr = call i64 @malloc(i64 24)
  %state = inttoptr i64 %state_ptr to %compute__AsyncState*
  %state_field = getelementptr %compute__AsyncState, %compute__AsyncState* %state, i32 0, i32 0
  store i64 0, i64* %state_field
  %param_x_ptr = getelementptr %compute__AsyncState, %compute__AsyncState* %state, i32 0, i32 2
  store i64 %x, i64* %param_x_ptr
  ret i64 %state_ptr
}

; Poll function for async compute
define { i64, i64 } @compute__poll(i64 %state_ptr) {
entry:
  %state = inttoptr i64 %state_ptr to %compute__AsyncState*
  %state_field = getelementptr %compute__AsyncState, %compute__AsyncState* %state, i32 0, i32 0
  %current_state = load i64, i64* %state_field
  %param_x_ptr = getelementptr %compute__AsyncState, %compute__AsyncState* %state, i32 0, i32 2
  %x = load i64, i64* %param_x_ptr
  switch i64 %current_state, label %state_invalid [
    i64 0, label %state_0
  ]

state_0:
  %0 = mul i64 %x, 2
  %result_ptr = getelementptr %compute__AsyncState, %compute__AsyncState* %state, i32 0, i32 1
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

; Async state struct for add_values
%add_values__AsyncState = type { i64, i64, i64, i64 }

; Create function for async add_values
define i64 @add_values(i64 %a, i64 %b) {
entry:
  %state_ptr = call i64 @malloc(i64 32)
  %state = inttoptr i64 %state_ptr to %add_values__AsyncState*
  %state_field = getelementptr %add_values__AsyncState, %add_values__AsyncState* %state, i32 0, i32 0
  store i64 0, i64* %state_field
  %param_a_ptr = getelementptr %add_values__AsyncState, %add_values__AsyncState* %state, i32 0, i32 2
  store i64 %a, i64* %param_a_ptr
  %param_b_ptr = getelementptr %add_values__AsyncState, %add_values__AsyncState* %state, i32 0, i32 3
  store i64 %b, i64* %param_b_ptr
  ret i64 %state_ptr
}

; Poll function for async add_values
define { i64, i64 } @add_values__poll(i64 %state_ptr) {
entry:
  %state = inttoptr i64 %state_ptr to %add_values__AsyncState*
  %state_field = getelementptr %add_values__AsyncState, %add_values__AsyncState* %state, i32 0, i32 0
  %current_state = load i64, i64* %state_field
  %param_a_ptr = getelementptr %add_values__AsyncState, %add_values__AsyncState* %state, i32 0, i32 2
  %a = load i64, i64* %param_a_ptr
  %param_b_ptr = getelementptr %add_values__AsyncState, %add_values__AsyncState* %state, i32 0, i32 3
  %b = load i64, i64* %param_b_ptr
  switch i64 %current_state, label %state_invalid [
    i64 0, label %state_0
  ]

state_0:
  %0 = add i64 %a, %b
  %result_ptr = getelementptr %add_values__AsyncState, %add_values__AsyncState* %state, i32 0, i32 1
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

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([37 x i8], [37 x i8]* @.str.0, i64 0, i64 0))
  %1 = call i64 @compute(i64 21)
  br label %await_poll0

await_poll0:
  %2 = call { i64, i64 } @compute__poll(i64 %1)
  %3 = extractvalue { i64, i64 } %2, 0
  %4 = icmp eq i64 %3, 1
  br i1 %4, label %await_ready1, label %await_pending2

await_pending2:
  br label %await_poll0

await_ready1:
  %5 = extractvalue { i64, i64 } %2, 1
  %result.6 = alloca i64
  store i64 %5, i64* %result.6
  %7 = call i32 @puts(i8* getelementptr ([20 x i8], [20 x i8]* @.str.1, i64 0, i64 0))
  %8 = load i64, i64* %result.6
  %9 = sdiv i64 %8, 10
  %10 = add i64 %9, 48
  %11 = trunc i64 %10 to i32
  %12 = call i32 @putchar(i32 %11)
  %13 = load i64, i64* %result.6
  %14 = srem i64 %13, 10
  %15 = add i64 %14, 48
  %16 = trunc i64 %15 to i32
  %17 = call i32 @putchar(i32 %16)
  %18 = trunc i64 10 to i32
  %19 = call i32 @putchar(i32 %18)
  %20 = call i64 @add_values(i64 10, i64 5)
  br label %await_poll3

await_poll3:
  %21 = call { i64, i64 } @add_values__poll(i64 %20)
  %22 = extractvalue { i64, i64 } %21, 0
  %23 = icmp eq i64 %22, 1
  br i1 %23, label %await_ready4, label %await_pending5

await_pending5:
  br label %await_poll3

await_ready4:
  %24 = extractvalue { i64, i64 } %21, 1
  %sum.25 = alloca i64
  store i64 %24, i64* %sum.25
  %26 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.2, i64 0, i64 0))
  %27 = load i64, i64* %sum.25
  %28 = sdiv i64 %27, 10
  %29 = add i64 %28, 48
  %30 = trunc i64 %29 to i32
  %31 = call i32 @putchar(i32 %30)
  %32 = load i64, i64* %sum.25
  %33 = srem i64 %32, 10
  %34 = add i64 %33, 48
  %35 = trunc i64 %34 to i32
  %36 = call i32 @putchar(i32 %35)
  %37 = trunc i64 10 to i32
  %38 = call i32 @putchar(i32 %37)
  %39 = call i32 @puts(i8* getelementptr ([21 x i8], [21 x i8]* @.str.3, i64 0, i64 0))
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

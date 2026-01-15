; ModuleID = 'closure_simple'
source_filename = "<vais>"

declare i32 @putchar(i32)
declare i64 @memcpy(i64, i64, i64)
declare i32 @printf(i8*)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @feof(i64)
declare i64 @fopen(i8*, i8*)
declare i64 @ftell(i64)
declare i64 @fputs(i8*, i64)
declare void @exit(i32)
declare i64 @strlen(i64)
declare i64 @fputc(i64, i64)
declare i32 @puts(i8*)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @malloc(i64)
declare i64 @fseek(i64, i64, i64)
declare i32 @fclose(i64)
declare i64 @fgetc(i64)
declare i64 @fflush(i64)
declare void @free(i64)
declare i64 @fgets(i64, i64, i64)
@.str.0 = private unnamed_addr constant [21 x i8] c"=== Closure Test ===\00"
@.str.1 = private unnamed_addr constant [23 x i8] c"Test 1: Simple capture\00"
@.str.2 = private unnamed_addr constant [26 x i8] c"Test 2: Multiple captures\00"
@.str.3 = private unnamed_addr constant [13 x i8] c"=== Done ===\00"

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([21 x i8], [21 x i8]* @.str.0, i64 0, i64 0))
  %1 = call i32 @puts(i8* getelementptr ([23 x i8], [23 x i8]* @.str.1, i64 0, i64 0))
  %multiplier.2 = alloca i64
  store i64 10, i64* %multiplier.2
  %3 = load i64, i64* %multiplier.2
  %scale.4 = alloca i64
  store i64 ptrtoint (i64 (i64, i64)* @__lambda_0 to i64), i64* %scale.4
  %5 = load i64, i64* %scale.4
  %6 = inttoptr i64 %5 to i64 (i64, i64)*
  %7 = call i64 %6(i64 %3, i64 5)
  %result1.8 = alloca i64
  store i64 %7, i64* %result1.8
  %9 = load i64, i64* %result1.8
  %10 = sdiv i64 %9, 10
  %11 = add i64 %10, 48
  %12 = trunc i64 %11 to i32
  %13 = call i32 @putchar(i32 %12)
  %14 = load i64, i64* %result1.8
  %15 = srem i64 %14, 10
  %16 = add i64 %15, 48
  %17 = trunc i64 %16 to i32
  %18 = call i32 @putchar(i32 %17)
  %19 = trunc i64 10 to i32
  %20 = call i32 @putchar(i32 %19)
  %21 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.2, i64 0, i64 0))
  %base.22 = alloca i64
  store i64 20, i64* %base.22
  %offset.23 = alloca i64
  store i64 3, i64* %offset.23
  %24 = load i64, i64* %base.22
  %25 = load i64, i64* %offset.23
  %compute.26 = alloca i64
  store i64 ptrtoint (i64 (i64, i64, i64)* @__lambda_1 to i64), i64* %compute.26
  %27 = load i64, i64* %compute.26
  %28 = inttoptr i64 %27 to i64 (i64, i64, i64)*
  %29 = call i64 %28(i64 %24, i64 %25, i64 7)
  %result2.30 = alloca i64
  store i64 %29, i64* %result2.30
  %31 = load i64, i64* %result2.30
  %32 = sdiv i64 %31, 10
  %33 = add i64 %32, 48
  %34 = trunc i64 %33 to i32
  %35 = call i32 @putchar(i32 %34)
  %36 = load i64, i64* %result2.30
  %37 = srem i64 %36, 10
  %38 = add i64 %37, 48
  %39 = trunc i64 %38 to i32
  %40 = call i32 @putchar(i32 %39)
  %41 = trunc i64 10 to i32
  %42 = call i32 @putchar(i32 %41)
  %43 = call i32 @puts(i8* getelementptr ([13 x i8], [13 x i8]* @.str.3, i64 0, i64 0))
  ret i64 0
}


define i64 @__lambda_0(i64 %__cap_multiplier, i64 %x) {
entry:
  %0 = mul i64 %x, %__cap_multiplier
  ret i64 %0
}

define i64 @__lambda_1(i64 %__cap_base, i64 %__cap_offset, i64 %x) {
entry:
  %0 = add i64 %__cap_base, %x
  %1 = add i64 %0, %__cap_offset
  ret i64 %1
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

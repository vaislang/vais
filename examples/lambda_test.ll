; ModuleID = 'lambda_test'
source_filename = "<vais>"

declare i32 @printf(i8*)
declare i64 @fopen(i8*, i8*)
declare i32 @fclose(i64)
declare i64 @fputs(i8*, i64)
declare i64 @fseek(i64, i64, i64)
declare i64 @strlen(i64)
declare i64 @memcpy(i64, i64, i64)
declare i32 @puts(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare void @free(i64)
declare i64 @malloc(i64)
declare i64 @fgetc(i64)
declare i64 @fputc(i64, i64)
declare i64 @fflush(i64)
declare i64 @feof(i64)
declare i64 @fread(i64, i64, i64, i64)
declare void @exit(i32)
declare i64 @fgets(i64, i64, i64)
declare i64 @ftell(i64)
declare i32 @putchar(i32)
@.str.0 = private unnamed_addr constant [28 x i8] c"Testing lambda expressions:\00"
@.str.1 = private unnamed_addr constant [13 x i8] c"add_one(5) =\00"
@.str.2 = private unnamed_addr constant [12 x i8] c"double(7) =\00"
@.str.3 = private unnamed_addr constant [12 x i8] c"add(3, 4) =\00"

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([28 x i8], [28 x i8]* @.str.0, i64 0, i64 0))
  %add_one.1 = alloca i64
  store i64 ptrtoint (i64 (i64)* @__lambda_0 to i64), i64* %add_one.1
  %2 = load i64, i64* %add_one.1
  %3 = inttoptr i64 %2 to i64 (i64)*
  %4 = call i64 %3(i64 5)
  %result1.5 = alloca i64
  store i64 %4, i64* %result1.5
  %6 = call i32 @puts(i8* getelementptr ([13 x i8], [13 x i8]* @.str.1, i64 0, i64 0))
  %7 = load i64, i64* %result1.5
  %8 = add i64 %7, 48
  %9 = trunc i64 %8 to i32
  %10 = call i32 @putchar(i32 %9)
  %11 = trunc i64 10 to i32
  %12 = call i32 @putchar(i32 %11)
  %double.13 = alloca i64
  store i64 ptrtoint (i64 (i64)* @__lambda_1 to i64), i64* %double.13
  %14 = load i64, i64* %double.13
  %15 = inttoptr i64 %14 to i64 (i64)*
  %16 = call i64 %15(i64 7)
  %result2.17 = alloca i64
  store i64 %16, i64* %result2.17
  %18 = call i32 @puts(i8* getelementptr ([12 x i8], [12 x i8]* @.str.2, i64 0, i64 0))
  %19 = load i64, i64* %result2.17
  %20 = sdiv i64 %19, 10
  %21 = add i64 %20, 48
  %22 = trunc i64 %21 to i32
  %23 = call i32 @putchar(i32 %22)
  %24 = load i64, i64* %result2.17
  %25 = srem i64 %24, 10
  %26 = add i64 %25, 48
  %27 = trunc i64 %26 to i32
  %28 = call i32 @putchar(i32 %27)
  %29 = trunc i64 10 to i32
  %30 = call i32 @putchar(i32 %29)
  %add.31 = alloca i64
  store i64 ptrtoint (i64 (i64, i64)* @__lambda_2 to i64), i64* %add.31
  %32 = load i64, i64* %add.31
  %33 = inttoptr i64 %32 to i64 (i64, i64)*
  %34 = call i64 %33(i64 3, i64 4)
  %result3.35 = alloca i64
  store i64 %34, i64* %result3.35
  %36 = call i32 @puts(i8* getelementptr ([12 x i8], [12 x i8]* @.str.3, i64 0, i64 0))
  %37 = load i64, i64* %result3.35
  %38 = add i64 %37, 48
  %39 = trunc i64 %38 to i32
  %40 = call i32 @putchar(i32 %39)
  %41 = trunc i64 10 to i32
  %42 = call i32 @putchar(i32 %41)
  ret i64 0
}


define i64 @__lambda_0(i64 %x) {
entry:
  %0 = add i64 %x, 1
  ret i64 %0
}

define i64 @__lambda_1(i64 %x) {
entry:
  %0 = mul i64 %x, 2
  ret i64 %0
}

define i64 @__lambda_2(i64 %a, i64 %b) {
entry:
  %0 = add i64 %a, %b
  ret i64 %0
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

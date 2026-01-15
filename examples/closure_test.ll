; ModuleID = 'closure_test'
source_filename = "<vais>"

declare i64 @fopen(i8*, i8*)
declare i64 @fgetc(i64)
declare i64 @fputs(i8*, i64)
declare void @free(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i32 @putchar(i32)
declare i64 @fputc(i64, i64)
declare i32 @fclose(i64)
declare i64 @feof(i64)
declare i64 @fread(i64, i64, i64, i64)
declare i32 @puts(i64)
declare i64 @strlen(i64)
declare i64 @ftell(i64)
declare void @exit(i32)
declare i64 @fgets(i64, i64, i64)
declare i64 @fflush(i64)
declare i32 @printf(i8*)
declare i64 @malloc(i64)
declare i64 @memcpy(i64, i64, i64)
declare i64 @fseek(i64, i64, i64)
@.str.0 = private unnamed_addr constant [21 x i8] c"=== Closure Test ===\00"
@.str.1 = private unnamed_addr constant [23 x i8] c"Test 1: Simple capture\00"
@.str.2 = private unnamed_addr constant [29 x i8] c"scale(5) with multiplier=10:\00"
@.str.3 = private unnamed_addr constant [26 x i8] c"Test 2: Multiple captures\00"
@.str.4 = private unnamed_addr constant [36 x i8] c"compute(3) with base=100, offset=7:\00"
@.str.5 = private unnamed_addr constant [27 x i8] c"Test 3: Nested value usage\00"
@.str.6 = private unnamed_addr constant [25 x i8] c"triple(2) with factor=5:\00"
@.str.7 = private unnamed_addr constant [13 x i8] c"=== Done ===\00"

define i64 @print_num(i64 %n) {
entry:
  %0 = icmp sge i64 %n, 100
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %then0, label %else1
then0:
  %3 = sdiv i64 %n, 100
  %4 = add i64 %3, 48
  %5 = trunc i64 %4 to i32
  %6 = call i32 @putchar(i32 %5)
  br label %merge2
else1:
  br label %merge2
merge2:
  %7 = phi i64 [ %6, %then0 ], [ 0, %else1 ]
  %8 = icmp sge i64 %n, 10
  %9 = zext i1 %8 to i64
  %10 = icmp ne i64 %9, 0
  br i1 %10, label %then3, label %else4
then3:
  %11 = sdiv i64 %n, 10
  %12 = srem i64 %11, 10
  %13 = add i64 %12, 48
  %14 = trunc i64 %13 to i32
  %15 = call i32 @putchar(i32 %14)
  br label %merge5
else4:
  br label %merge5
merge5:
  %16 = phi i64 [ %15, %then3 ], [ 0, %else4 ]
  %17 = srem i64 %n, 10
  %18 = add i64 %17, 48
  %19 = trunc i64 %18 to i32
  %20 = call i32 @putchar(i32 %19)
  ret i64 0
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([21 x i8], [21 x i8]* @.str.0, i64 0, i64 0))
  %1 = call i32 @puts(i8* getelementptr ([23 x i8], [23 x i8]* @.str.1, i64 0, i64 0))
  %multiplier.2 = alloca i64
  store i64 10, i64* %multiplier.2
  %3 = load i64, i64* %multiplier.2
  %scale.4 = alloca i64
  store i64 ptrtoint (i64 (i64, i64)* @__lambda_0 to i64), i64* %scale.4
  %5 = load i64, i64* %scale
  %6 = inttoptr i64 %5 to i64 (i64)*
  %7 = call i64 %6(i64 5)
  %result1.8 = alloca i64
  store i64 %7, i64* %result1.8
  %9 = call i32 @puts(i8* getelementptr ([29 x i8], [29 x i8]* @.str.2, i64 0, i64 0))
  %10 = load i64, i64* %result1.8
  %11 = call i64 @print_num(i64 %10)
  %12 = trunc i64 10 to i32
  %13 = call i32 @putchar(i32 %12)
  %14 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.3, i64 0, i64 0))
  %base.15 = alloca i64
  store i64 100, i64* %base.15
  %offset.16 = alloca i64
  store i64 7, i64* %offset.16
  %17 = load i64, i64* %base.15
  %18 = load i64, i64* %offset.16
  %compute.19 = alloca i64
  store i64 ptrtoint (i64 (i64, i64, i64)* @__lambda_1 to i64), i64* %compute.19
  %20 = load i64, i64* %compute
  %21 = inttoptr i64 %20 to i64 (i64)*
  %22 = call i64 %21(i64 3)
  %result2.23 = alloca i64
  store i64 %22, i64* %result2.23
  %24 = call i32 @puts(i8* getelementptr ([36 x i8], [36 x i8]* @.str.4, i64 0, i64 0))
  %25 = load i64, i64* %result2.23
  %26 = call i64 @print_num(i64 %25)
  %27 = trunc i64 10 to i32
  %28 = call i32 @putchar(i32 %27)
  %29 = call i32 @puts(i8* getelementptr ([27 x i8], [27 x i8]* @.str.5, i64 0, i64 0))
  %factor.30 = alloca i64
  store i64 5, i64* %factor.30
  %31 = load i64, i64* %factor.30
  %triple.32 = alloca i64
  store i64 ptrtoint (i64 (i64, i64)* @__lambda_2 to i64), i64* %triple.32
  %33 = load i64, i64* %triple
  %34 = inttoptr i64 %33 to i64 (i64)*
  %35 = call i64 %34(i64 2)
  %result3.36 = alloca i64
  store i64 %35, i64* %result3.36
  %37 = call i32 @puts(i8* getelementptr ([25 x i8], [25 x i8]* @.str.6, i64 0, i64 0))
  %38 = load i64, i64* %result3.36
  %39 = call i64 @print_num(i64 %38)
  %40 = trunc i64 10 to i32
  %41 = call i32 @putchar(i32 %40)
  %42 = call i32 @puts(i8* getelementptr ([13 x i8], [13 x i8]* @.str.7, i64 0, i64 0))
  ret i64 0
}


define i64 @__lambda_0(i64 %__cap_multiplier, i64 %x) {
entry:
  %0 = mul i64 %x, %__cap_multiplier
  ret i64 %0
}

define i64 @__lambda_1(i64 %__cap_base, i64 %__cap_offset, i64 %x) {
entry:
  %0 = mul i64 %x, %__cap_offset
  %1 = add i64 %__cap_base, %0
  ret i64 %1
}

define i64 @__lambda_2(i64 %__cap_factor, i64 %n) {
entry:
  %0 = add i64 %n, %n
  %1 = add i64 %0, %n
  %2 = add i64 %1, %__cap_factor
  ret i64 %2
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

; ModuleID = 'opt_test'
source_filename = "<vais>"

declare void @free(i64)
declare i32 @fclose(i64)
declare i64 @fgets(i64, i64, i64)
declare i32 @strcmp(i8*, i8*)
declare i64 @malloc(i64)
declare i64 @fopen(i8*, i8*)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @fgetc(i64)
declare i64 @fflush(i64)
declare i64 @ftell(i64)
declare i32 @puts(i8*)
declare i64 @fread(i64, i64, i64, i64)
declare i32 @printf(i8*)
declare i64 @fseek(i64, i64, i64)
declare i64 @fputc(i64, i64)
declare i32 @putchar(i32)
declare void @exit(i32)
declare i64 @strlen(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @feof(i64)
declare i64 @fputs(i8*, i64)
declare i64 @memcpy(i64, i64, i64)
@.str.0 = private unnamed_addr constant [23 x i8] c"Testing optimizations:\00"
@.str.1 = private unnamed_addr constant [23 x i8] c"constant_fold_test() =\00"
@.str.2 = private unnamed_addr constant [27 x i8] c"strength_reduce_test(10) =\00"

define i64 @constant_fold_test() {
entry:
  %0 = add i64 0, 30  ; folded from 10 add 20
  %a.1 = alloca i64
  store i64 %0, i64* %a.1
  %2 = load i64, i64* %a.1
  %3 = shl i64 %2, 1  ; strength reduced from mul by 2
  %b.4 = alloca i64
  store i64 %3, i64* %b.4
  %5 = load i64, i64* %b.4
  ret i64 %5
}

define i64 @strength_reduce_test(i64 %x) {
entry:
  %0 = shl i64 %x, 3  ; strength reduced from mul by 8
  %y.1 = alloca i64
  store i64 %0, i64* %y.1
  %2 = load i64, i64* %y.1
  %3 = ashr i64 %2, 2  ; strength reduced from div by 4
  %z.4 = alloca i64
  store i64 %3, i64* %z.4
  %5 = load i64, i64* %z.4
  ret i64 %5
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([23 x i8], [23 x i8]* @.str.0, i64 0, i64 0))
  %1 = call i64 @constant_fold_test()
  %r1.2 = alloca i64
  store i64 %1, i64* %r1.2
  %3 = call i32 @puts(i8* getelementptr ([23 x i8], [23 x i8]* @.str.1, i64 0, i64 0))
  %4 = load i64, i64* %r1.2
  %5 = sdiv i64 %4, 10
  %6 = add i64 %5, 48
  %7 = trunc i64 %6 to i32
  %8 = call i32 @putchar(i32 %7)
  %9 = load i64, i64* %r1.2
  %10 = srem i64 %9, 10
  %11 = add i64 %10, 48
  %12 = trunc i64 %11 to i32
  %13 = call i32 @putchar(i32 %12)
  %14 = trunc i64 10 to i32
  %15 = call i32 @putchar(i32 %14)
  %16 = call i64 @strength_reduce_test(i64 10)
  %r2.17 = alloca i64
  store i64 %16, i64* %r2.17
  %18 = call i32 @puts(i8* getelementptr ([27 x i8], [27 x i8]* @.str.2, i64 0, i64 0))
  %19 = load i64, i64* %r2.17
  %20 = sdiv i64 %19, 10
  %21 = add i64 %20, 48
  %22 = trunc i64 %21 to i32
  %23 = call i32 @putchar(i32 %22)
  %24 = load i64, i64* %r2.17
  %25 = srem i64 %24, 10
  %26 = add i64 %25, 48
  %27 = trunc i64 %26 to i32
  %28 = call i32 @putchar(i32 %27)
  %29 = trunc i64 10 to i32
  %30 = call i32 @putchar(i32 %29)
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

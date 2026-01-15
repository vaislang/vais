; ModuleID = 'pattern_match_test'
source_filename = "<vais>"

declare i32 @puts(i64)
declare void @free(i64)
declare i64 @fputc(i64, i64)
declare i64 @ftell(i64)
declare i32 @putchar(i32)
declare i64 @malloc(i64)
declare void @exit(i32)
declare i64 @memcpy(i64, i64, i64)
declare i64 @fgets(i64, i64, i64)
declare i64 @fseek(i64, i64, i64)
declare i64 @fflush(i64)
declare i32 @fclose(i64)
declare i64 @fopen(i8*, i8*)
declare i64 @fgetc(i64)
declare i32 @printf(i8*)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @strlen(i64)
declare i64 @fputs(i8*, i64)
declare i64 @feof(i64)
@.str.0 = private unnamed_addr constant [26 x i8] c"Testing pattern matching:\00"
@.str.1 = private unnamed_addr constant [15 x i8] c"color_code(0):\00"
@.str.2 = private unnamed_addr constant [13 x i8] c"describe(0):\00"
@.str.3 = private unnamed_addr constant [14 x i8] c"describe(41):\00"

define i64 @color_code(i64 %c) {
entry:
  switch i64 %c, label %match.default1 [
    i64 0, label %match.arm2
    i64 1, label %match.arm3
  ]
match.arm2:
  br label %match.merge0
match.arm3:
  br label %match.merge0
match.default1:
  br label %match.merge0
match.merge0:
  %0 = phi i64 [ 255, %match.arm2 ], [ 65280, %match.arm3 ], [ 0, %match.default1 ]
  ret i64 %0
}

define i64 @describe(i64 %n) {
entry:
  br label %match.check1
match.check1:
  %0 = icmp eq i64 %n, 0
  br i1 %0, label %match.arm3, label %match.check2
match.arm3:
  br label %match.merge0
match.check2:
  br i1 1, label %match.arm4, label %match.merge0
match.arm4:
  %x.1 = alloca i64
  store i64 %n, i64* %x.1
  %2 = load i64, i64* %x.1
  %3 = add i64 %2, 1
  br label %match.merge0
match.merge0:
  %4 = phi i64 [ 100, %match.arm3 ], [ %3, %match.arm4 ]
  ret i64 %4
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.0, i64 0, i64 0))
  %1 = call i64 @color_code(i64 0)
  %c.2 = alloca i64
  store i64 %1, i64* %c.2
  %3 = call i32 @puts(i8* getelementptr ([15 x i8], [15 x i8]* @.str.1, i64 0, i64 0))
  %4 = load i64, i64* %c.2
  %5 = sdiv i64 %4, 100
  %6 = add i64 %5, 48
  %7 = trunc i64 %6 to i32
  %8 = call i32 @putchar(i32 %7)
  %9 = load i64, i64* %c.2
  %10 = sdiv i64 %9, 10
  %11 = srem i64 %10, 10
  %12 = add i64 %11, 48
  %13 = trunc i64 %12 to i32
  %14 = call i32 @putchar(i32 %13)
  %15 = load i64, i64* %c.2
  %16 = srem i64 %15, 10
  %17 = add i64 %16, 48
  %18 = trunc i64 %17 to i32
  %19 = call i32 @putchar(i32 %18)
  %20 = trunc i64 10 to i32
  %21 = call i32 @putchar(i32 %20)
  %22 = call i64 @describe(i64 0)
  %d.23 = alloca i64
  store i64 %22, i64* %d.23
  %24 = call i32 @puts(i8* getelementptr ([13 x i8], [13 x i8]* @.str.2, i64 0, i64 0))
  %25 = load i64, i64* %d.23
  %26 = sdiv i64 %25, 100
  %27 = add i64 %26, 48
  %28 = trunc i64 %27 to i32
  %29 = call i32 @putchar(i32 %28)
  %30 = load i64, i64* %d.23
  %31 = sdiv i64 %30, 10
  %32 = srem i64 %31, 10
  %33 = add i64 %32, 48
  %34 = trunc i64 %33 to i32
  %35 = call i32 @putchar(i32 %34)
  %36 = load i64, i64* %d.23
  %37 = srem i64 %36, 10
  %38 = add i64 %37, 48
  %39 = trunc i64 %38 to i32
  %40 = call i32 @putchar(i32 %39)
  %41 = trunc i64 10 to i32
  %42 = call i32 @putchar(i32 %41)
  %43 = call i64 @describe(i64 41)
  %e.44 = alloca i64
  store i64 %43, i64* %e.44
  %45 = call i32 @puts(i8* getelementptr ([14 x i8], [14 x i8]* @.str.3, i64 0, i64 0))
  %46 = load i64, i64* %e.44
  %47 = sdiv i64 %46, 10
  %48 = add i64 %47, 48
  %49 = trunc i64 %48 to i32
  %50 = call i32 @putchar(i32 %49)
  %51 = load i64, i64* %e.44
  %52 = srem i64 %51, 10
  %53 = add i64 %52, 48
  %54 = trunc i64 %53 to i32
  %55 = call i32 @putchar(i32 %54)
  %56 = trunc i64 10 to i32
  %57 = call i32 @putchar(i32 %56)
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

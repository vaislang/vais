; ModuleID = 'match_binding'
source_filename = "<vais>"

declare i64 @fopen(i8*, i8*)
declare i32 @puts(i8*)
declare void @free(i64)
declare i32 @fclose(i64)
declare i64 @memcpy(i64, i64, i64)
declare i32 @printf(i8*)
declare i64 @fputc(i64, i64)
declare i64 @malloc(i64)
declare i64 @fgets(i64, i64, i64)
declare i64 @fflush(i64)
declare i64 @feof(i64)
declare void @exit(i32)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @fgetc(i64)
declare i32 @putchar(i32)
declare i64 @fseek(i64, i64, i64)
declare i64 @fputs(i8*, i64)
declare i64 @ftell(i64)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @strlen(i64)
@.str.0 = private unnamed_addr constant [10 x i8] c"check(0):\00"
@.str.1 = private unnamed_addr constant [10 x i8] c"check(1):\00"
@.str.2 = private unnamed_addr constant [25 x i8] c"check(5) - should be 50:\00"

define i64 @check(i64 %n) {
entry:
  br label %match.check1
match.check1:
  %0 = icmp eq i64 %n, 0
  br i1 %0, label %match.arm3, label %match.check2
match.arm3:
  br label %match.merge0
match.check2:
  %1 = icmp eq i64 %n, 1
  br i1 %1, label %match.arm5, label %match.check4
match.arm5:
  br label %match.merge0
match.check4:
  br i1 1, label %match.arm6, label %match.merge0
match.arm6:
  %x.2 = alloca i64
  store i64 %n, i64* %x.2
  %3 = load i64, i64* %x.2
  %4 = mul i64 %3, 10
  br label %match.merge0
match.merge0:
  %5 = phi i64 [ 100, %match.arm3 ], [ 200, %match.arm5 ], [ %4, %match.arm6 ]
  ret i64 %5
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([10 x i8], [10 x i8]* @.str.0, i64 0, i64 0))
  %1 = call i64 @check(i64 0)
  %r.2 = alloca i64
  store i64 %1, i64* %r.2
  %3 = load i64, i64* %r.2
  %4 = sdiv i64 %3, 100
  %5 = add i64 %4, 48
  %6 = trunc i64 %5 to i32
  %7 = call i32 @putchar(i32 %6)
  %8 = trunc i64 10 to i32
  %9 = call i32 @putchar(i32 %8)
  %10 = call i32 @puts(i8* getelementptr ([10 x i8], [10 x i8]* @.str.1, i64 0, i64 0))
  %11 = call i64 @check(i64 1)
  %r2.12 = alloca i64
  store i64 %11, i64* %r2.12
  %13 = load i64, i64* %r2.12
  %14 = sdiv i64 %13, 100
  %15 = add i64 %14, 48
  %16 = trunc i64 %15 to i32
  %17 = call i32 @putchar(i32 %16)
  %18 = trunc i64 10 to i32
  %19 = call i32 @putchar(i32 %18)
  %20 = call i32 @puts(i8* getelementptr ([25 x i8], [25 x i8]* @.str.2, i64 0, i64 0))
  %21 = call i64 @check(i64 5)
  %r3.22 = alloca i64
  store i64 %21, i64* %r3.22
  %23 = load i64, i64* %r3.22
  %24 = sdiv i64 %23, 10
  %25 = add i64 %24, 48
  %26 = trunc i64 %25 to i32
  %27 = call i32 @putchar(i32 %26)
  %28 = load i64, i64* %r3.22
  %29 = srem i64 %28, 10
  %30 = add i64 %29, 48
  %31 = trunc i64 %30 to i32
  %32 = call i32 @putchar(i32 %31)
  %33 = trunc i64 10 to i32
  %34 = call i32 @putchar(i32 %33)
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

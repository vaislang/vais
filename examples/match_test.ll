; ModuleID = 'match_test'
source_filename = "<vais>"

declare i32 @printf(i8*)
declare i32 @puts(i8*)
declare i64 @memcpy(i64, i64, i64)
declare i32 @fclose(i64)
declare i64 @fgets(i64, i64, i64)
declare i32 @putchar(i32)
declare void @free(i64)
declare i64 @feof(i64)
declare i64 @ftell(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare void @exit(i32)
declare i64 @strlen(i64)
declare i64 @fopen(i8*, i8*)
declare i64 @fgetc(i64)
declare i64 @fputs(i8*, i64)
declare i64 @fseek(i64, i64, i64)
declare i64 @fflush(i64)
declare i64 @fputc(i64, i64)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @malloc(i64)
@.str.0 = private unnamed_addr constant [22 x i8] c"Testing match with 0:\00"
@.str.1 = private unnamed_addr constant [22 x i8] c"Testing match with 1:\00"
@.str.2 = private unnamed_addr constant [22 x i8] c"Testing match with 5:\00"

define i64 @describe(i64 %n) {
entry:
  switch i64 %n, label %match.default1 [
    i64 0, label %match.arm2
    i64 1, label %match.arm3
    i64 2, label %match.arm4
  ]
match.arm2:
  br label %match.merge0
match.arm3:
  br label %match.merge0
match.arm4:
  br label %match.merge0
match.default1:
  br label %match.merge0
match.merge0:
  %0 = phi i64 [ 0, %match.arm2 ], [ 1, %match.arm3 ], [ 2, %match.arm4 ], [ %n, %match.default1 ]
  ret i64 %0
}

define i64 @main() {
entry:
  %0 = call i64 @describe(i64 0)
  %r.1 = alloca i64
  store i64 %0, i64* %r.1
  %2 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.0, i64 0, i64 0))
  %3 = load i64, i64* %r.1
  %4 = add i64 %3, 48
  %5 = trunc i64 %4 to i32
  %6 = call i32 @putchar(i32 %5)
  %7 = trunc i64 10 to i32
  %8 = call i32 @putchar(i32 %7)
  %9 = call i64 @describe(i64 1)
  store i64 %9, i64* %r.1
  %10 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.1, i64 0, i64 0))
  %11 = load i64, i64* %r.1
  %12 = add i64 %11, 48
  %13 = trunc i64 %12 to i32
  %14 = call i32 @putchar(i32 %13)
  %15 = trunc i64 10 to i32
  %16 = call i32 @putchar(i32 %15)
  %17 = call i64 @describe(i64 5)
  store i64 %17, i64* %r.1
  %18 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.2, i64 0, i64 0))
  %19 = load i64, i64* %r.1
  %20 = add i64 %19, 48
  %21 = trunc i64 %20 to i32
  %22 = call i32 @putchar(i32 %21)
  %23 = trunc i64 10 to i32
  %24 = call i32 @putchar(i32 %23)
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

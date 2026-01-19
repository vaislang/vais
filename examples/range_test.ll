; ModuleID = 'range_test'
source_filename = "<vais>"

declare i32 @printf(i8*)
declare i64 @fopen(i8*, i8*)
declare i32 @fclose(i64)
declare void @exit(i32)
declare i64 @fseek(i64, i64, i64)
declare i64 @memcpy(i64, i64, i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @ftell(i64)
declare i64 @fputs(i8*, i64)
declare i32 @puts(i64)
declare i32 @strcmp(i8*, i8*)
declare i64 @feof(i64)
declare i32 @putchar(i32)
declare i32 @sched_yield()
declare i64 @fputc(i64, i64)
declare i32 @usleep(i64)
declare i64 @fread(i64, i64, i64, i64)
declare void @free(i64)
declare i64 @strlen(i64)
declare i64 @fflush(i64)
declare i64 @fgets(i64, i64, i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @malloc(i64)
declare i64 @fgetc(i64)
@.str.0 = private unnamed_addr constant [25 x i8] c"Testing range iteration:\00"
@.str.1 = private unnamed_addr constant [10 x i8] c"Sum 0..5:\00"

define i64 @test_range() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([25 x i8], [25 x i8]* @.str.0, i64 0, i64 0))
  %sum.1 = alloca i64
  store i64 0, i64* %sum.1
  br label %loop.start0
loop.start0:
  %2 = icmp ne i64 0, 0
  br i1 %2, label %loop.body1, label %loop.end2
loop.body1:
  %3 = load i64, i64* %sum.1
  %4 = add i64 %3, @i
  store i64 %4, i64* %sum.1
  %5 = add i64 @i, 48
  %6 = trunc i64 %5 to i32
  %7 = call i32 @putchar(i32 %6)
  %8 = trunc i64 32 to i32
  %9 = call i32 @putchar(i32 %8)
  br label %loop.start0
loop.end2:
  %10 = trunc i64 10 to i32
  %11 = call i32 @putchar(i32 %10)
  %12 = call i32 @puts(i8* getelementptr ([10 x i8], [10 x i8]* @.str.1, i64 0, i64 0))
  %13 = load i64, i64* %sum.1
  %14 = call i64 @print_num(i64 %13)
  %15 = trunc i64 10 to i32
  %16 = call i32 @putchar(i32 %15)
  %17 = load i64, i64* %sum.1
  ret i64 %17
}

define i64 @print_num(i64 %n) {
entry:
  %0 = icmp sge i64 %n, 10
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %then0, label %else1
then0:
  %3 = sdiv i64 %n, 10
  %4 = add i64 %3, 48
  %5 = trunc i64 %4 to i32
  %6 = call i32 @putchar(i32 %5)
  br label %merge2
else1:
  br label %merge2
merge2:
  %7 = phi i64 [ 0, %then0 ], [ 0, %else1 ]
  %8 = srem i64 %n, 10
  %9 = add i64 %8, 48
  %10 = trunc i64 %9 to i32
  %11 = call i32 @putchar(i32 %10)
  ret i64 0
}

define i64 @main() {
entry:
  %0 = call i64 @test_range()
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

; ModuleID = 'file_test'
source_filename = "<vais>"

declare i64 @feof(i64)
declare i64 @fseek(i64, i64, i64)
declare i64 @strlen(i64)
declare i64 @fgetc(i64)
declare void @free(i64)
declare void @exit(i32)
declare i64 @fread(i64, i64, i64, i64)
declare i32 @puts(i8*)
declare i64 @memcpy(i64, i64, i64)
declare i32 @fclose(i64)
declare i64 @fopen(i8*, i8*)
declare i64 @malloc(i64)
declare i32 @printf(i8*)
declare i64 @fgets(i64, i64, i64)
declare i64 @fflush(i64)
declare i64 @ftell(i64)
declare i32 @putchar(i32)
declare i64 @fputc(i64, i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @fputs(i8*, i64)
@.str.0 = private unnamed_addr constant [22 x i8] c"=== File I/O Test ===\00"
@.str.1 = private unnamed_addr constant [23 x i8] c"Writing to test.txt...\00"
@.str.2 = private unnamed_addr constant [9 x i8] c"test.txt\00"
@.str.3 = private unnamed_addr constant [2 x i8] c"w\00"
@.str.4 = private unnamed_addr constant [17 x i8] c"Hello from VAIS!\00"
@.str.5 = private unnamed_addr constant [10 x i8] c"Write OK!\00"
@.str.6 = private unnamed_addr constant [14 x i8] c"Write FAILED!\00"
@.str.7 = private unnamed_addr constant [25 x i8] c"Reading from test.txt...\00"
@.str.8 = private unnamed_addr constant [9 x i8] c"test.txt\00"
@.str.9 = private unnamed_addr constant [2 x i8] c"r\00"
@.str.10 = private unnamed_addr constant [12 x i8] c"First char:\00"
@.str.11 = private unnamed_addr constant [9 x i8] c"Read OK!\00"
@.str.12 = private unnamed_addr constant [13 x i8] c"Read FAILED!\00"
@.str.13 = private unnamed_addr constant [13 x i8] c"=== Done ===\00"

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.0, i64 0, i64 0))
  %1 = call i32 @puts(i8* getelementptr ([23 x i8], [23 x i8]* @.str.1, i64 0, i64 0))
  %2 = call i64 @fopen(i8* getelementptr ([9 x i8], [9 x i8]* @.str.2, i64 0, i64 0), i8* getelementptr ([2 x i8], [2 x i8]* @.str.3, i64 0, i64 0))
  %h.3 = alloca i64
  store i64 %2, i64* %h.3
  %4 = load i64, i64* %h.3
  %5 = icmp ne i64 %4, 0
  %6 = zext i1 %5 to i64
  %7 = icmp ne i64 %6, 0
  br i1 %7, label %then0, label %else1
then0:
  %8 = load i64, i64* %h.3
  %9 = call i64 @fputs(i8* getelementptr ([17 x i8], [17 x i8]* @.str.4, i64 0, i64 0), i64 %8)
  %10 = load i64, i64* %h.3
  %11 = call i32 @fclose(i64 %10)
  %12 = call i32 @puts(i8* getelementptr ([10 x i8], [10 x i8]* @.str.5, i64 0, i64 0))
  br label %merge2
else1:
  %13 = call i32 @puts(i8* getelementptr ([14 x i8], [14 x i8]* @.str.6, i64 0, i64 0))
  br label %merge2
merge2:
  %14 = phi i64 [ 0, %then0 ], [ 0, %else1 ]
  %15 = call i32 @puts(i8* getelementptr ([25 x i8], [25 x i8]* @.str.7, i64 0, i64 0))
  %16 = call i64 @fopen(i8* getelementptr ([9 x i8], [9 x i8]* @.str.8, i64 0, i64 0), i8* getelementptr ([2 x i8], [2 x i8]* @.str.9, i64 0, i64 0))
  %h2.17 = alloca i64
  store i64 %16, i64* %h2.17
  %18 = load i64, i64* %h2.17
  %19 = icmp ne i64 %18, 0
  %20 = zext i1 %19 to i64
  %21 = icmp ne i64 %20, 0
  br i1 %21, label %then3, label %else4
then3:
  %22 = load i64, i64* %h2.17
  %23 = call i64 @fgetc(i64 %22)
  %c.24 = alloca i64
  store i64 %23, i64* %c.24
  %25 = load i64, i64* %c.24
  %26 = icmp sge i64 %25, 0
  %27 = zext i1 %26 to i64
  %28 = icmp ne i64 %27, 0
  br i1 %28, label %then6, label %else7
then6:
  %29 = call i32 @puts(i8* getelementptr ([12 x i8], [12 x i8]* @.str.10, i64 0, i64 0))
  %30 = load i64, i64* %c.24
  %31 = trunc i64 %30 to i32
  %32 = call i32 @putchar(i32 %31)
  %33 = trunc i64 10 to i32
  %34 = call i32 @putchar(i32 %33)
  br label %merge8
else7:
  br label %merge8
merge8:
  %35 = phi i64 [ 0, %then6 ], [ 0, %else7 ]
  %36 = load i64, i64* %h2.17
  %37 = call i32 @fclose(i64 %36)
  %38 = call i32 @puts(i8* getelementptr ([9 x i8], [9 x i8]* @.str.11, i64 0, i64 0))
  br label %merge5
else4:
  %39 = call i32 @puts(i8* getelementptr ([13 x i8], [13 x i8]* @.str.12, i64 0, i64 0))
  br label %merge5
merge5:
  %40 = phi i64 [ 0, %then3 ], [ 0, %else4 ]
  %41 = call i32 @puts(i8* getelementptr ([13 x i8], [13 x i8]* @.str.13, i64 0, i64 0))
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

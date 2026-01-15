; ModuleID = 'loop_break_test'
source_filename = "<vais>"

declare void @free(i64)
declare i32 @printf(i8*)
declare i64 @strlen(i64)
declare i64 @memcpy(i64, i64, i64)
declare i32 @puts(i64)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @malloc(i64)
declare i64 @fgetc(i64)
declare i64 @fputc(i64, i64)
declare i64 @fputs(i8*, i64)
declare i64 @fgets(i64, i64, i64)
declare i64 @fflush(i64)
declare i64 @feof(i64)
declare i64 @fseek(i64, i64, i64)
declare void @exit(i32)
declare i64 @fopen(i8*, i8*)
declare i64 @fwrite(i64, i64, i64, i64)
declare i32 @fclose(i64)
declare i32 @putchar(i32)
declare i64 @ftell(i64)
@.str.0 = private unnamed_addr constant [24 x i8] c"=== Loop Break Test ===\00"
@.str.1 = private unnamed_addr constant [24 x i8] c"Test 1: Break from loop\00"
@.str.2 = private unnamed_addr constant [13 x i8] c"=== Done ===\00"

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.0, i64 0, i64 0))
  %1 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.1, i64 0, i64 0))
  %i.2 = alloca i64
  store i64 0, i64* %i.2
  br label %loop.start0
loop.start0:
  %3 = load i64, i64* %i.2
  %4 = icmp sge i64 %3, 5
  %5 = zext i1 %4 to i64
  %6 = icmp ne i64 %5, 0
  br i1 %6, label %then3, label %else4
then3:
  br label %loop.end2
else4:
  br label %merge5
merge5:
  %7 = phi i64 [ 0, %else4 ]
  %8 = load i64, i64* %i.2
  %9 = add i64 %8, 48
  %10 = trunc i64 %9 to i32
  %11 = call i32 @putchar(i32 %10)
  %12 = load i64, i64* %i.2
  %13 = add i64 %12, 1
  store i64 %13, i64* %i.2
  br label %loop.start0
loop.end2:
  %14 = trunc i64 10 to i32
  %15 = call i32 @putchar(i32 %14)
  %16 = call i32 @puts(i8* getelementptr ([13 x i8], [13 x i8]* @.str.2, i64 0, i64 0))
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

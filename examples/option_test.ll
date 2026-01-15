; ModuleID = 'option_test'
source_filename = "<vais>"

%Option = type { i32, { i64 } }
declare i64 @memcpy(i64, i64, i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @malloc(i64)
declare i64 @fgetc(i64)
declare i32 @puts(i8*)
declare void @free(i64)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @fputc(i64, i64)
declare i32 @putchar(i32)
declare i64 @fgets(i64, i64, i64)
declare i64 @fseek(i64, i64, i64)
declare i64 @fflush(i64)
declare void @exit(i32)
declare i64 @strlen(i64)
declare i32 @fclose(i64)
declare i64 @fputs(i8*, i64)
declare i64 @fopen(i8*, i8*)
declare i64 @ftell(i64)
declare i64 @feof(i64)
declare i32 @printf(i8*)
@.str.0 = private unnamed_addr constant [21 x i8] c"Testing Option type:\00"
@.str.1 = private unnamed_addr constant [9 x i8] c"10 / 2 =\00"
@.str.2 = private unnamed_addr constant [9 x i8] c"10 / 0 =\00"

define i64 @divide(i64 %a, i64 %b) {
entry:
  %0 = icmp eq i64 %b, 0
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %then0, label %else1
then0:
  br label %merge2
else1:
  %3 = sdiv i64 %a, %b
  br label %merge2
merge2:
  %4 = phi i64 [ 0, %then0 ], [ %3, %else1 ]
  ret i64 %4
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([21 x i8], [21 x i8]* @.str.0, i64 0, i64 0))
  %1 = call i64 @divide(i64 10, i64 2)
  %r1.2 = alloca i64
  store i64 %1, i64* %r1.2
  %3 = call i32 @puts(i8* getelementptr ([9 x i8], [9 x i8]* @.str.1, i64 0, i64 0))
  %4 = load i64, i64* %r1.2
  %5 = add i64 %4, 48
  %6 = trunc i64 %5 to i32
  %7 = call i32 @putchar(i32 %6)
  %8 = trunc i64 10 to i32
  %9 = call i32 @putchar(i32 %8)
  %10 = call i64 @divide(i64 10, i64 0)
  %r2.11 = alloca i64
  store i64 %10, i64* %r2.11
  %12 = call i32 @puts(i8* getelementptr ([9 x i8], [9 x i8]* @.str.2, i64 0, i64 0))
  %13 = load i64, i64* %r2.11
  %14 = add i64 %13, 48
  %15 = trunc i64 %14 to i32
  %16 = call i32 @putchar(i32 %15)
  %17 = trunc i64 10 to i32
  %18 = call i32 @putchar(i32 %17)
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

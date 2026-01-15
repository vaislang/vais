; ModuleID = 'spawn_test'
source_filename = "<vais>"

declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @fgetc(i64)
declare i64 @fgets(i64, i64, i64)
declare i64 @fflush(i64)
declare i32 @printf(i8*)
declare i64 @feof(i64)
declare void @free(i64)
declare i64 @fputs(i8*, i64)
declare i32 @puts(i8*)
declare i32 @fclose(i64)
declare i64 @fseek(i64, i64, i64)
declare i64 @malloc(i64)
declare void @exit(i32)
declare i64 @strlen(i64)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @memcpy(i64, i64, i64)
declare i64 @fputc(i64, i64)
declare i64 @fopen(i8*, i8*)
declare i64 @ftell(i64)
declare i32 @putchar(i32)
@.str.0 = private unnamed_addr constant [30 x i8] c"Testing spawn (sync for now):\00"
@.str.1 = private unnamed_addr constant [20 x i8] c"spawn { work(5) } =\00"

define i64 @work(i64 %x) {
entry:
  %0 = mul i64 %x, %x
  ret i64 %0
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([30 x i8], [30 x i8]* @.str.0, i64 0, i64 0))
  %1 = call i64 @work(i64 5)
  %result.2 = alloca i64
  store i64 %1, i64* %result.2
  %3 = call i32 @puts(i8* getelementptr ([20 x i8], [20 x i8]* @.str.1, i64 0, i64 0))
  %4 = load i64, i64* %result.2
  %5 = sdiv i64 %4, 10
  %6 = add i64 %5, 48
  %7 = trunc i64 %6 to i32
  %8 = call i32 @putchar(i32 %7)
  %9 = load i64, i64* %result.2
  %10 = srem i64 %9, 10
  %11 = add i64 %10, 48
  %12 = trunc i64 %11 to i32
  %13 = call i32 @putchar(i32 %12)
  %14 = trunc i64 10 to i32
  %15 = call i32 @putchar(i32 %14)
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

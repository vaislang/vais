; ModuleID = 'test_import'
source_filename = "<vais>"

declare void @free(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i32 @fclose(i64)
declare i64 @fgets(i64, i64, i64)
declare i64 @fopen(i8*, i8*)
declare i64 @fgetc(i64)
declare i32 @printf(i8*)
declare void @exit(i32)
declare i32 @puts(i8*)
declare i64 @fputc(i64, i64)
declare i64 @memcpy(i64, i64, i64)
declare i64 @fputs(i8*, i64)
declare i64 @fflush(i64)
declare i32 @putchar(i32)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @strlen(i64)
declare i64 @ftell(i64)
declare i64 @feof(i64)
declare i64 @fseek(i64, i64, i64)
declare i64 @malloc(i64)
define i64 @test_func(i64 %a, i64 %b) {
entry:
  %0 = add i64 %a, %b
  ret i64 %0
}

define i64 @main() {
entry:
  %0 = call i64 @test_func(i64 10, i64 20)
  %result.1 = alloca i64
  store i64 %0, i64* %result.1
  %2 = load i64, i64* %result.1
  %3 = sdiv i64 %2, 10
  %4 = add i64 %3, 48
  %5 = trunc i64 %4 to i32
  %6 = call i32 @putchar(i32 %5)
  %7 = load i64, i64* %result.1
  %8 = srem i64 %7, 10
  %9 = add i64 %8, 48
  %10 = trunc i64 %9 to i32
  %11 = call i32 @putchar(i32 %10)
  %12 = trunc i64 10 to i32
  %13 = call i32 @putchar(i32 %12)
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

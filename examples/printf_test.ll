; ModuleID = 'printf_test'
source_filename = "<vais>"

declare i64 @memcpy(i64, i64, i64)
declare i64 @fputc(i64, i64)
declare i64 @malloc(i64)
declare void @free(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @feof(i64)
declare i32 @puts(i64)
declare i32 @fclose(i64)
declare i64 @fseek(i64, i64, i64)
declare i64 @ftell(i64)
declare i64 @strlen(i64)
declare i64 @fgetc(i64)
declare i32 @putchar(i32)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @fputs(i8*, i64)
declare void @exit(i32)
declare i32 @printf(i8*)
declare i64 @fgets(i64, i64, i64)
declare i64 @fflush(i64)
declare i64 @fopen(i8*, i8*)
@.str.0 = private unnamed_addr constant [17 x i8] c"Hello from Vais!\00"
@.str.1 = private unnamed_addr constant [16 x i8] c"The answer is: \00"

define i64 @main() {
entry:
  %x.0 = alloca i64
  store i64 42, i64* %x.0
  %1 = call i32 @puts(i8* getelementptr ([17 x i8], [17 x i8]* @.str.0, i64 0, i64 0))
  %2 = call i32 @printf(i8* getelementptr ([16 x i8], [16 x i8]* @.str.1, i64 0, i64 0))
  %3 = trunc i64 52 to i32
  %4 = call i32 @putchar(i32 %3)
  %5 = trunc i64 50 to i32
  %6 = call i32 @putchar(i32 %5)
  %7 = trunc i64 10 to i32
  %8 = call i32 @putchar(i32 %7)
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

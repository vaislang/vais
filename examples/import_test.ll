; ModuleID = 'import_test'
source_filename = "<vais>"

declare i64 @strlen(i64)
declare i32 @puts(i64)
declare i64 @ftell(i64)
declare void @exit(i32)
declare i64 @fgets(i64, i64, i64)
declare i64 @fflush(i64)
declare i64 @feof(i64)
declare i64 @memcpy(i64, i64, i64)
declare i64 @fopen(i8*, i8*)
declare i64 @fread(i64, i64, i64, i64)
declare i32 @printf(i8*)
declare i64 @fputc(i64, i64)
declare i64 @fseek(i64, i64, i64)
declare void @free(i64)
declare i64 @fgetc(i64)
declare i64 @fputs(i8*, i64)
declare i64 @malloc(i64)
declare i32 @putchar(i32)
declare i64 @fwrite(i64, i64, i64, i64)
declare i32 @fclose(i64)
@.str.0 = private unnamed_addr constant [17 x i8] c"Testing imports:\00"
@.str.1 = private unnamed_addr constant [17 x i8] c"add(10, 20) = 30\00"
@.str.2 = private unnamed_addr constant [15 x i8] c"mul(5, 6) = 30\00"
@.str.3 = private unnamed_addr constant [15 x i8] c"square(7) = 49\00"

define i64 @add(i64 %a, i64 %b) {
entry:
  %0 = add i64 %a, %b
  ret i64 %0
}

define i64 @mul(i64 %a, i64 %b) {
entry:
  %0 = mul i64 %a, %b
  ret i64 %0
}

define i64 @square(i64 %x) {
entry:
  %0 = mul i64 %x, %x
  ret i64 %0
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([17 x i8], [17 x i8]* @.str.0, i64 0, i64 0))
  %1 = call i64 @add(i64 10, i64 20)
  %result.2 = alloca i64
  store i64 %1, i64* %result.2
  %3 = call i32 @puts(i8* getelementptr ([17 x i8], [17 x i8]* @.str.1, i64 0, i64 0))
  %4 = call i64 @mul(i64 5, i64 6)
  %result2.5 = alloca i64
  store i64 %4, i64* %result2.5
  %6 = call i32 @puts(i8* getelementptr ([15 x i8], [15 x i8]* @.str.2, i64 0, i64 0))
  %7 = call i64 @square(i64 7)
  %result3.8 = alloca i64
  store i64 %7, i64* %result3.8
  %9 = call i32 @puts(i8* getelementptr ([15 x i8], [15 x i8]* @.str.3, i64 0, i64 0))
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

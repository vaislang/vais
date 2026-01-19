; ModuleID = 'enum_struct_variant_test'
source_filename = "<vais>"

%Color = type { i32, { i64, i64, i64 } }
%Message = type { i32, { i64, i64, i64 } }
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @ftell(i64)
declare i32 @sched_yield()
declare i64 @fread(i64, i64, i64, i64)
declare i64 @fseek(i64, i64, i64)
declare i32 @puts(i64)
declare i64 @fgetc(i64)
declare i32 @fclose(i64)
declare i64 @fopen(i8*, i8*)
declare i64 @malloc(i64)
declare i64 @memcpy(i64, i64, i64)
declare i64 @fgets(i64, i64, i64)
declare i32 @strcmp(i8*, i8*)
declare i32 @usleep(i64)
declare i64 @fputs(i8*, i64)
declare void @free(i64)
declare i64 @fflush(i64)
declare i32 @putchar(i32)
declare i64 @fputc(i64, i64)
declare i64 @feof(i64)
declare i64 @strlen(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare void @exit(i32)
declare i32 @printf(i8*)
@.str.0 = private unnamed_addr constant [44 x i8] c"=== Enum Struct Variant Type Check Test ===\00"
@.str.1 = private unnamed_addr constant [42 x i8] c"Test 1: Enum with struct variants defined\00"
@.str.2 = private unnamed_addr constant [48 x i8] c"Color enum has: Red, Green, Blue, Custom{r,g,b}\00"
@.str.3 = private unnamed_addr constant [66 x i8] c"Message enum has: Quit, Move{x,y}, Write(i64), ChangeColor{r,g,b}\00"
@.str.4 = private unnamed_addr constant [33 x i8] c"Test 2: Type checking successful\00"
@.str.5 = private unnamed_addr constant [8 x i8] c"Result:\00"
@.str.6 = private unnamed_addr constant [49 x i8] c"=== Struct variant registration test passed! ===\00"

define i64 @get_red() {
entry:
  ret i64 0
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([44 x i8], [44 x i8]* @.str.0, i64 0, i64 0))
  %1 = trunc i64 10 to i32
  %2 = call i32 @putchar(i32 %1)
  %3 = call i32 @puts(i8* getelementptr ([42 x i8], [42 x i8]* @.str.1, i64 0, i64 0))
  %4 = call i32 @puts(i8* getelementptr ([48 x i8], [48 x i8]* @.str.2, i64 0, i64 0))
  %5 = call i32 @puts(i8* getelementptr ([66 x i8], [66 x i8]* @.str.3, i64 0, i64 0))
  %6 = trunc i64 10 to i32
  %7 = call i32 @putchar(i32 %6)
  %8 = call i32 @puts(i8* getelementptr ([33 x i8], [33 x i8]* @.str.4, i64 0, i64 0))
  %9 = call i64 @get_red()
  %result.10 = alloca i64
  store i64 %9, i64* %result.10
  %11 = call i32 @puts(i8* getelementptr ([8 x i8], [8 x i8]* @.str.5, i64 0, i64 0))
  %12 = load i64, i64* %result.10
  %13 = add i64 %12, 48
  %14 = trunc i64 %13 to i32
  %15 = call i32 @putchar(i32 %14)
  %16 = trunc i64 10 to i32
  %17 = call i32 @putchar(i32 %16)
  %18 = call i32 @puts(i8* getelementptr ([49 x i8], [49 x i8]* @.str.6, i64 0, i64 0))
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

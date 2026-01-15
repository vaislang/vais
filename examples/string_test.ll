; ModuleID = 'string_test'
source_filename = "<vais>"

%String = type { i64, i64, i64 }
declare i32 @puts(i64)
declare i64 @fputc(i64, i64)
declare i32 @putchar(i32)
declare void @free(i64)
declare i64 @strlen(i64)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @fopen(i8*, i8*)
declare i64 @malloc(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @ftell(i64)
declare i32 @printf(i8*)
declare i64 @fgetc(i64)
declare i64 @fseek(i64, i64, i64)
declare void @exit(i32)
declare i64 @fgets(i64, i64, i64)
declare i64 @fputs(i8*, i64)
declare i64 @fflush(i64)
declare i32 @fclose(i64)
declare i64 @feof(i64)
declare i64 @memcpy(i64, i64, i64)
@.str.0 = private unnamed_addr constant [21 x i8] c"Testing String type:\00"
@.str.1 = private unnamed_addr constant [33 x i8] c"Created string, pushing chars...\00"
@.str.2 = private unnamed_addr constant [16 x i8] c"String content:\00"
@.str.3 = private unnamed_addr constant [15 x i8] c"String length:\00"
@.str.4 = private unnamed_addr constant [6 x i8] c"Done!\00"

define i64 @String_len(%String* %self) {
entry:
  %0 = getelementptr %String, %String* %self, i32 0, i32 1
  %1 = load i64, i64* %0
  ret i64 %1
}

define i64 @String_push_char(%String* %self, i64 %c) {
entry:
  %0 = getelementptr %String, %String* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = getelementptr %String, %String* %self, i32 0, i32 1
  %3 = load i64, i64* %2
  %4 = add i64 %1, %3
  %ptr.5 = alloca i64
  store i64 %4, i64* %ptr.5
  %6 = load i64, i64* %ptr.5
  call void @__store_byte(i64 %6, i64 %c)
  %7 = getelementptr %String, %String* %self, i32 0, i32 1
  %8 = load i64, i64* %7
  %9 = add i64 %8, 1
  %10 = getelementptr %String, %String* %self, i32 0, i32 1
  store i64 %9, i64* %10
  %11 = getelementptr %String, %String* %self, i32 0, i32 0
  %12 = load i64, i64* %11
  %13 = getelementptr %String, %String* %self, i32 0, i32 1
  %14 = load i64, i64* %13
  %15 = add i64 %12, %14
  call void @__store_byte(i64 %15, i64 0)
  %16 = getelementptr %String, %String* %self, i32 0, i32 1
  %17 = load i64, i64* %16
  ret i64 %17
}

define i64 @String_print(%String* %self) {
entry:
  %0 = getelementptr %String, %String* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = inttoptr i64 %1 to i8*
  %3 = call i32 @puts(i8* %2)
  ret i64 0
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([21 x i8], [21 x i8]* @.str.0, i64 0, i64 0))
  %1 = call i8* @malloc(i64 64)
  %2 = ptrtoint i8* %1 to i64
  %data.3 = alloca i64
  store i64 %2, i64* %data.3
  %4 = load i64, i64* %data.3
  call void @__store_byte(i64 %4, i64 0)
  %5 = alloca %String
  %6 = load i64, i64* %data.3
  %7 = getelementptr %String, %String* %5, i32 0, i32 0
  store i64 %6, i64* %7
  %8 = getelementptr %String, %String* %5, i32 0, i32 1
  store i64 0, i64* %8
  %9 = getelementptr %String, %String* %5, i32 0, i32 2
  store i64 64, i64* %9
  %s.10 = alloca %String*
  store %String* %5, %String** %s.10
  %11 = call i32 @puts(i8* getelementptr ([33 x i8], [33 x i8]* @.str.1, i64 0, i64 0))
  %12 = load %String*, %String** %s.10
  %13 = call i64 @String_push_char(%String* %12, i64 72)
  %14 = load %String*, %String** %s.10
  %15 = call i64 @String_push_char(%String* %14, i64 101)
  %16 = load %String*, %String** %s.10
  %17 = call i64 @String_push_char(%String* %16, i64 108)
  %18 = load %String*, %String** %s.10
  %19 = call i64 @String_push_char(%String* %18, i64 108)
  %20 = load %String*, %String** %s.10
  %21 = call i64 @String_push_char(%String* %20, i64 111)
  %22 = call i32 @puts(i8* getelementptr ([16 x i8], [16 x i8]* @.str.2, i64 0, i64 0))
  %23 = load %String*, %String** %s.10
  %24 = call i64 @String_print(%String* %23)
  %25 = call i32 @puts(i8* getelementptr ([15 x i8], [15 x i8]* @.str.3, i64 0, i64 0))
  %26 = load %String*, %String** %s.10
  %27 = call i64 @String_len(%String* %26)
  %len.28 = alloca i64
  store i64 %27, i64* %len.28
  %29 = load i64, i64* %len.28
  %30 = add i64 %29, 48
  %31 = trunc i64 %30 to i32
  %32 = call i32 @putchar(i32 %31)
  %33 = trunc i64 10 to i32
  %34 = call i32 @putchar(i32 %33)
  %35 = load i64, i64* %data.3
  %36 = inttoptr i64 %35 to i8*
  call void @free(i8* %36)
  %37 = call i32 @puts(i8* getelementptr ([6 x i8], [6 x i8]* @.str.4, i64 0, i64 0))
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

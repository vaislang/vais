; ModuleID = 'trait_test'
source_filename = "<vais>"

%Counter = type { i64 }
declare i64 @fgets(i64, i64, i64)
declare i64 @feof(i64)
declare i32 @fclose(i64)
declare i32 @sched_yield()
declare i64 @memcpy(i64, i64, i64)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @fseek(i64, i64, i64)
declare i64 @fputc(i64, i64)
declare i64 @ftell(i64)
declare i64 @fputs(i8*, i64)
declare void @exit(i32)
declare i32 @strcmp(i8*, i8*)
declare i64 @strlen(i64)
declare i32 @usleep(i64)
declare i64 @fgetc(i64)
declare i32 @puts(i8*)
declare i32 @printf(i8*)
declare i32 @putchar(i32)
declare i64 @fflush(i64)
declare i64 @malloc(i64)
declare void @free(i64)
declare i64 @fopen(i8*, i8*)
@.str.0 = private unnamed_addr constant [15 x i8] c"Counter value:\00"
@.str.1 = private unnamed_addr constant [25 x i8] c"Testing traits and impl:\00"
@.str.2 = private unnamed_addr constant [14 x i8] c"increment() =\00"
@.str.3 = private unnamed_addr constant [11 x i8] c"double() =\00"

define i64 @Counter_print(%Counter* %self) {
entry:
  %0 = call i32 @puts(i8* getelementptr ([15 x i8], [15 x i8]* @.str.0, i64 0, i64 0))
  %1 = getelementptr %Counter, %Counter* %self, i32 0, i32 0
  %2 = load i64, i64* %1
  %3 = sdiv i64 %2, 10
  %4 = add i64 %3, 48
  %5 = trunc i64 %4 to i32
  %6 = call i32 @putchar(i32 %5)
  %7 = getelementptr %Counter, %Counter* %self, i32 0, i32 0
  %8 = load i64, i64* %7
  %9 = srem i64 %8, 10
  %10 = add i64 %9, 48
  %11 = trunc i64 %10 to i32
  %12 = call i32 @putchar(i32 %11)
  %13 = trunc i64 10 to i32
  %14 = call i32 @putchar(i32 %13)
  ret i64 0
}

define i64 @Counter_increment(%Counter* %self) {
entry:
  %0 = getelementptr %Counter, %Counter* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = add i64 %1, 1
  ret i64 %2
}

define i64 @Counter_double(%Counter* %self) {
entry:
  %0 = getelementptr %Counter, %Counter* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = mul i64 %1, 2
  ret i64 %2
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([25 x i8], [25 x i8]* @.str.1, i64 0, i64 0))
  %1 = alloca %Counter
  %2 = getelementptr %Counter, %Counter* %1, i32 0, i32 0
  store i64 42, i64* %2
  %c.3 = alloca %Counter*
  store %Counter* %1, %Counter** %c.3
  %4 = load %Counter*, %Counter** %c.3
  %5 = call i64 @Counter_print(%Counter* %4)
  %6 = load %Counter*, %Counter** %c.3
  %7 = call i64 @Counter_increment(%Counter* %6)
  %inc.8 = alloca i64
  store i64 %7, i64* %inc.8
  %9 = call i32 @puts(i8* getelementptr ([14 x i8], [14 x i8]* @.str.2, i64 0, i64 0))
  %10 = load i64, i64* %inc.8
  %11 = sdiv i64 %10, 10
  %12 = add i64 %11, 48
  %13 = trunc i64 %12 to i32
  %14 = call i32 @putchar(i32 %13)
  %15 = load i64, i64* %inc.8
  %16 = srem i64 %15, 10
  %17 = add i64 %16, 48
  %18 = trunc i64 %17 to i32
  %19 = call i32 @putchar(i32 %18)
  %20 = trunc i64 10 to i32
  %21 = call i32 @putchar(i32 %20)
  %22 = load %Counter*, %Counter** %c.3
  %23 = call i64 @Counter_double(%Counter* %22)
  %dbl.24 = alloca i64
  store i64 %23, i64* %dbl.24
  %25 = call i32 @puts(i8* getelementptr ([11 x i8], [11 x i8]* @.str.3, i64 0, i64 0))
  %26 = load i64, i64* %dbl.24
  %27 = sdiv i64 %26, 10
  %28 = add i64 %27, 48
  %29 = trunc i64 %28 to i32
  %30 = call i32 @putchar(i32 %29)
  %31 = load i64, i64* %dbl.24
  %32 = srem i64 %31, 10
  %33 = add i64 %32, 48
  %34 = trunc i64 %33 to i32
  %35 = call i32 @putchar(i32 %34)
  %36 = trunc i64 10 to i32
  %37 = call i32 @putchar(i32 %36)
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

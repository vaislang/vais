; ModuleID = 'method_test'
source_filename = "<vais>"

%Point = type { i64, i64 }
declare i64 @fputs(i8*, i64)
declare i32 @putchar(i32)
declare i32 @usleep(i64)
declare i32 @printf(i8*)
declare i64 @strlen(i64)
declare void @free(i64)
declare i64 @memcpy(i64, i64, i64)
declare i64 @ftell(i64)
declare i64 @feof(i64)
declare i32 @fclose(i64)
declare i64 @fseek(i64, i64, i64)
declare i64 @fgets(i64, i64, i64)
declare i32 @sched_yield()
declare i64 @fopen(i8*, i8*)
declare i64 @fread(i64, i64, i64, i64)
declare void @exit(i32)
declare i64 @malloc(i64)
declare i32 @puts(i64)
declare i64 @fflush(i64)
declare i64 @fgetc(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i32 @strcmp(i8*, i8*)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @fputc(i64, i64)
@.str.0 = private unnamed_addr constant [24 x i8] c"Testing struct methods:\00"
@.str.1 = private unnamed_addr constant [10 x i8] c"p.sum() =\00"
@.str.2 = private unnamed_addr constant [13 x i8] c"p.scale(2) =\00"

define i64 @Point_sum(%Point* %self) {
entry:
  %0 = getelementptr %Point, %Point* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = getelementptr %Point, %Point* %self, i32 0, i32 1
  %3 = load i64, i64* %2
  %4 = add i64 %1, %3
  ret i64 %4
}

define i64 @Point_scale(%Point* %self, i64 %factor) {
entry:
  %0 = getelementptr %Point, %Point* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = getelementptr %Point, %Point* %self, i32 0, i32 1
  %3 = load i64, i64* %2
  %4 = add i64 %1, %3
  %5 = mul i64 %4, %factor
  ret i64 %5
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.0, i64 0, i64 0))
  %1 = alloca %Point
  %2 = getelementptr %Point, %Point* %1, i32 0, i32 0
  store i64 10, i64* %2
  %3 = getelementptr %Point, %Point* %1, i32 0, i32 1
  store i64 20, i64* %3
  %p.4 = alloca %Point*
  store %Point* %1, %Point** %p.4
  %5 = load %Point*, %Point** %p.4
  %6 = call i64 @Point_sum(%Point* %5)
  %s.7 = alloca i64
  store i64 %6, i64* %s.7
  %8 = call i32 @puts(i8* getelementptr ([10 x i8], [10 x i8]* @.str.1, i64 0, i64 0))
  %9 = load i64, i64* %s.7
  %10 = sdiv i64 %9, 10
  %11 = add i64 %10, 48
  %12 = trunc i64 %11 to i32
  %13 = call i32 @putchar(i32 %12)
  %14 = load i64, i64* %s.7
  %15 = srem i64 %14, 10
  %16 = add i64 %15, 48
  %17 = trunc i64 %16 to i32
  %18 = call i32 @putchar(i32 %17)
  %19 = trunc i64 10 to i32
  %20 = call i32 @putchar(i32 %19)
  %21 = load %Point*, %Point** %p.4
  %22 = call i64 @Point_scale(%Point* %21, i64 2)
  %scaled.23 = alloca i64
  store i64 %22, i64* %scaled.23
  %24 = call i32 @puts(i8* getelementptr ([13 x i8], [13 x i8]* @.str.2, i64 0, i64 0))
  %25 = load i64, i64* %scaled.23
  %26 = sdiv i64 %25, 10
  %27 = add i64 %26, 48
  %28 = trunc i64 %27 to i32
  %29 = call i32 @putchar(i32 %28)
  %30 = load i64, i64* %scaled.23
  %31 = srem i64 %30, 10
  %32 = add i64 %31, 48
  %33 = trunc i64 %32 to i32
  %34 = call i32 @putchar(i32 %33)
  %35 = trunc i64 10 to i32
  %36 = call i32 @putchar(i32 %35)
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

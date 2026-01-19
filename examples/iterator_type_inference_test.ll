; ModuleID = 'iterator_type_inference_test'
source_filename = "<vais>"

%Counter = type { i64, i64 }
declare i64 @strlen(i64)
declare i64 @fflush(i64)
declare void @exit(i32)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @fgets(i64, i64, i64)
declare void @free(i64)
declare i32 @puts(i8*)
declare i64 @memcpy(i64, i64, i64)
declare i64 @fgetc(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i32 @strcmp(i8*, i8*)
declare i32 @putchar(i32)
declare i32 @sched_yield()
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @fopen(i8*, i8*)
declare i64 @fputs(i8*, i64)
declare i64 @feof(i64)
declare i32 @printf(i8*)
declare i64 @malloc(i64)
declare i32 @fclose(i64)
declare i64 @fseek(i64, i64, i64)
declare i64 @ftell(i64)
declare i64 @fputc(i64, i64)
declare i32 @usleep(i64)
@.str.0 = private unnamed_addr constant [33 x i8] c"Testing iterator type inference:\00"
@.str.1 = private unnamed_addr constant [17 x i8] c"Range iteration:\00"
@.str.2 = private unnamed_addr constant [17 x i8] c"Custom iterator:\00"
@.str.3 = private unnamed_addr constant [27 x i8] c"Type inference successful!\00"

define i64 @Counter_next(%Counter* %self) {
entry:
  %0 = getelementptr %Counter, %Counter* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = getelementptr %Counter, %Counter* %self, i32 0, i32 1
  %3 = load i64, i64* %2
  %4 = icmp slt i64 %1, %3
  %5 = zext i1 %4 to i64
  %6 = icmp ne i64 %5, 0
  br i1 %6, label %then0, label %else1
then0:
  %7 = getelementptr %Counter, %Counter* %self, i32 0, i32 0
  %8 = load i64, i64* %7
  %val.9 = alloca i64
  store i64 %8, i64* %val.9
  %10 = getelementptr %Counter, %Counter* %self, i32 0, i32 0
  %11 = load i64, i64* %10
  %12 = add i64 %11, 1
  %13 = getelementptr %Counter, %Counter* %self, i32 0, i32 0
  store i64 %12, i64* %13
  %14 = load i64, i64* %val.9
  br label %merge2
else1:
  %15 = sub i64 0, 1
  br label %merge2
merge2:
  %16 = phi i64 [ %14, %then0 ], [ %15, %else1 ]
  ret i64 %16
}

define i64 @test_iteration() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([33 x i8], [33 x i8]* @.str.0, i64 0, i64 0))
  %1 = call i32 @puts(i8* getelementptr ([17 x i8], [17 x i8]* @.str.1, i64 0, i64 0))
  %r.2 = alloca i64
  store i64 0, i64* %r.2
  %sum1.3 = alloca i64
  store i64 0, i64* %sum1.3
  br label %loop.start0
loop.start0:
  %4 = load i64, i64* %r.2
  %5 = icmp ne i64 %4, 0
  br i1 %5, label %loop.body1, label %loop.end2
loop.body1:
  %6 = add i64 @i, 48
  %7 = trunc i64 %6 to i32
  %8 = call i32 @putchar(i32 %7)
  %9 = trunc i64 32 to i32
  %10 = call i32 @putchar(i32 %9)
  %11 = load i64, i64* %sum1.3
  %12 = add i64 %11, @i
  store i64 %12, i64* %sum1.3
  br label %loop.start0
loop.end2:
  %13 = trunc i64 10 to i32
  %14 = call i32 @putchar(i32 %13)
  %15 = call i32 @puts(i8* getelementptr ([17 x i8], [17 x i8]* @.str.2, i64 0, i64 0))
  %16 = alloca %Counter
  %17 = getelementptr %Counter, %Counter* %16, i32 0, i32 0
  store i64 0, i64* %17
  %18 = getelementptr %Counter, %Counter* %16, i32 0, i32 1
  store i64 5, i64* %18
  %counter.19 = alloca %Counter*
  store %Counter* %16, %Counter** %counter.19
  %sum2.20 = alloca i64
  store i64 0, i64* %sum2.20
  br label %loop.start3
loop.start3:
  %21 = load %Counter*, %Counter** %counter.19
  %22 = call i64 @Counter_next(%Counter* %21)
  %val.23 = alloca i64
  store i64 %22, i64* %val.23
  %24 = load i64, i64* %val.23
  %25 = icmp slt i64 %24, 0
  %26 = zext i1 %25 to i64
  %27 = icmp ne i64 %26, 0
  br i1 %27, label %then6, label %else7
then6:
  br label %loop.end5
else7:
  br label %merge8
merge8:
  %28 = add i64 0, 0
  %29 = load i64, i64* %sum2.20
  %30 = load i64, i64* %val.23
  %31 = add i64 %29, %30
  store i64 %31, i64* %sum2.20
  %32 = load i64, i64* %val.23
  %33 = add i64 %32, 48
  %34 = trunc i64 %33 to i32
  %35 = call i32 @putchar(i32 %34)
  %36 = trunc i64 32 to i32
  %37 = call i32 @putchar(i32 %36)
  br label %loop.start3
loop.end5:
  %38 = trunc i64 10 to i32
  %39 = call i32 @putchar(i32 %38)
  %40 = call i32 @puts(i8* getelementptr ([27 x i8], [27 x i8]* @.str.3, i64 0, i64 0))
  ret i64 0
}

define i64 @main() {
entry:
  %0 = call i64 @test_iteration()
  ret i64 %0
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

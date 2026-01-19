; ModuleID = 'comprehensive_iterator_test'
source_filename = "<vais>"

%FibIter = type { i64, i64, i64, i64 }
%RangeIter = type { i64, i64 }
declare i64 @fputc(i64, i64)
declare i64 @fflush(i64)
declare i64 @memcpy(i64, i64, i64)
declare i64 @fgets(i64, i64, i64)
declare i64 @fseek(i64, i64, i64)
declare i64 @feof(i64)
declare i64 @fopen(i8*, i8*)
declare i32 @printf(i8*)
declare i64 @strlen(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i32 @putchar(i32)
declare i64 @fread(i64, i64, i64, i64)
declare i32 @puts(i64)
declare void @exit(i32)
declare i64 @fgetc(i64)
declare i64 @fputs(i8*, i64)
declare i64 @ftell(i64)
declare i32 @fclose(i64)
declare void @free(i64)
declare i32 @strcmp(i8*, i8*)
declare i32 @usleep(i64)
declare i64 @malloc(i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i32 @sched_yield()
@.str.0 = private unnamed_addr constant [11 x i8] c"RangeIter:\00"
@.str.1 = private unnamed_addr constant [5 x i8] c" to \00"
@.str.2 = private unnamed_addr constant [36 x i8] c"=== Comprehensive Iterator Test ===\00"
@.str.3 = private unnamed_addr constant [23 x i8] c"Test 1: Struct methods\00"
@.str.4 = private unnamed_addr constant [31 x i8] c"Test 2: Iterator trait methods\00"
@.str.5 = private unnamed_addr constant [10 x i8] c"Has more:\00"
@.str.6 = private unnamed_addr constant [13 x i8] c"Next values:\00"
@.str.7 = private unnamed_addr constant [27 x i8] c"Test 3: Fibonacci Iterator\00"
@.str.8 = private unnamed_addr constant [34 x i8] c"Test 4: New iterator from scratch\00"
@.str.9 = private unnamed_addr constant [6 x i8] c"Sum: \00"
@.str.10 = private unnamed_addr constant [25 x i8] c"=== All Tests Passed ===\00"

define i64 @RangeIter_next(%RangeIter* %self) {
entry:
  %0 = getelementptr %RangeIter, %RangeIter* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = getelementptr %RangeIter, %RangeIter* %self, i32 0, i32 1
  %3 = load i64, i64* %2
  %4 = icmp slt i64 %1, %3
  %5 = zext i1 %4 to i64
  %6 = icmp ne i64 %5, 0
  br i1 %6, label %then0, label %else1
then0:
  %7 = getelementptr %RangeIter, %RangeIter* %self, i32 0, i32 0
  %8 = load i64, i64* %7
  %val.9 = alloca i64
  store i64 %8, i64* %val.9
  %10 = getelementptr %RangeIter, %RangeIter* %self, i32 0, i32 0
  %11 = load i64, i64* %10
  %12 = add i64 %11, 1
  %13 = getelementptr %RangeIter, %RangeIter* %self, i32 0, i32 0
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

define i64 @RangeIter_has_more(%RangeIter* %self) {
entry:
  %0 = getelementptr %RangeIter, %RangeIter* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = getelementptr %RangeIter, %RangeIter* %self, i32 0, i32 1
  %3 = load i64, i64* %2
  %4 = icmp slt i64 %1, %3
  %5 = zext i1 %4 to i64
  %6 = icmp ne i64 %5, 0
  br i1 %6, label %then0, label %else1
then0:
  br label %merge2
else1:
  br label %merge2
merge2:
  %7 = phi i64 [ 1, %then0 ], [ 0, %else1 ]
  ret i64 %7
}

define i64 @RangeIter_print(%RangeIter* %self) {
entry:
  %0 = call i32 @puts(i8* getelementptr ([11 x i8], [11 x i8]* @.str.0, i64 0, i64 0))
  %1 = getelementptr %RangeIter, %RangeIter* %self, i32 0, i32 0
  %2 = load i64, i64* %1
  %3 = add i64 %2, 48
  %4 = trunc i64 %3 to i32
  %5 = call i32 @putchar(i32 %4)
  %6 = call i32 @puts(i8* getelementptr ([5 x i8], [5 x i8]* @.str.1, i64 0, i64 0))
  %7 = getelementptr %RangeIter, %RangeIter* %self, i32 0, i32 1
  %8 = load i64, i64* %7
  %9 = add i64 %8, 48
  %10 = trunc i64 %9 to i32
  %11 = call i32 @putchar(i32 %10)
  %12 = trunc i64 10 to i32
  %13 = call i32 @putchar(i32 %12)
  ret i64 0
}

define %RangeIter @RangeIter_new(i64 %start, i64 %end) {
entry:
  %0 = alloca %RangeIter
  %1 = getelementptr %RangeIter, %RangeIter* %0, i32 0, i32 0
  store i64 %start, i64* %1
  %2 = getelementptr %RangeIter, %RangeIter* %0, i32 0, i32 1
  store i64 %end, i64* %2
  %ret.3 = load %RangeIter, %RangeIter* %0
  ret %RangeIter %ret.3
}

define i64 @FibIter_next(%FibIter* %self) {
entry:
  %0 = getelementptr %FibIter, %FibIter* %self, i32 0, i32 2
  %1 = load i64, i64* %0
  %2 = getelementptr %FibIter, %FibIter* %self, i32 0, i32 3
  %3 = load i64, i64* %2
  %4 = icmp sge i64 %1, %3
  %5 = zext i1 %4 to i64
  %6 = icmp ne i64 %5, 0
  br i1 %6, label %then0, label %else1
then0:
  %7 = sub i64 0, 1
  br label %merge2
else1:
  %8 = getelementptr %FibIter, %FibIter* %self, i32 0, i32 0
  %9 = load i64, i64* %8
  %current.10 = alloca i64
  store i64 %9, i64* %current.10
  %11 = getelementptr %FibIter, %FibIter* %self, i32 0, i32 1
  %12 = load i64, i64* %11
  %13 = getelementptr %FibIter, %FibIter* %self, i32 0, i32 0
  store i64 %12, i64* %13
  %14 = load i64, i64* %current.10
  %15 = getelementptr %FibIter, %FibIter* %self, i32 0, i32 1
  %16 = load i64, i64* %15
  %17 = add i64 %14, %16
  %18 = getelementptr %FibIter, %FibIter* %self, i32 0, i32 1
  store i64 %17, i64* %18
  %19 = getelementptr %FibIter, %FibIter* %self, i32 0, i32 2
  %20 = load i64, i64* %19
  %21 = add i64 %20, 1
  %22 = getelementptr %FibIter, %FibIter* %self, i32 0, i32 2
  store i64 %21, i64* %22
  %23 = load i64, i64* %current.10
  br label %merge2
merge2:
  %24 = phi i64 [ %7, %then0 ], [ %23, %else1 ]
  ret i64 %24
}

define i64 @FibIter_has_more(%FibIter* %self) {
entry:
  %0 = getelementptr %FibIter, %FibIter* %self, i32 0, i32 2
  %1 = load i64, i64* %0
  %2 = getelementptr %FibIter, %FibIter* %self, i32 0, i32 3
  %3 = load i64, i64* %2
  %4 = icmp slt i64 %1, %3
  %5 = zext i1 %4 to i64
  %6 = icmp ne i64 %5, 0
  br i1 %6, label %then0, label %else1
then0:
  br label %merge2
else1:
  br label %merge2
merge2:
  %7 = phi i64 [ 1, %then0 ], [ 0, %else1 ]
  ret i64 %7
}

define %FibIter @FibIter_new(i64 %max) {
entry:
  %0 = alloca %FibIter
  %1 = getelementptr %FibIter, %FibIter* %0, i32 0, i32 0
  store i64 0, i64* %1
  %2 = getelementptr %FibIter, %FibIter* %0, i32 0, i32 1
  store i64 1, i64* %2
  %3 = getelementptr %FibIter, %FibIter* %0, i32 0, i32 2
  store i64 0, i64* %3
  %4 = getelementptr %FibIter, %FibIter* %0, i32 0, i32 3
  store i64 %max, i64* %4
  %ret.5 = load %FibIter, %FibIter* %0
  ret %FibIter %ret.5
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([36 x i8], [36 x i8]* @.str.2, i64 0, i64 0))
  %1 = trunc i64 10 to i32
  %2 = call i32 @putchar(i32 %1)
  %3 = call i32 @puts(i8* getelementptr ([23 x i8], [23 x i8]* @.str.3, i64 0, i64 0))
  %4 = call %RangeIter @RangeIter_new(i64 0, i64 5)
  %r.5.struct = alloca %RangeIter
  store %RangeIter %4, %RangeIter* %r.5.struct
  %r.5 = alloca %RangeIter*
  store %RangeIter* %r.5.struct, %RangeIter** %r.5
  %6 = load %RangeIter*, %RangeIter** %r.5
  %7 = call i64 @RangeIter_print(%RangeIter* %6)
  %8 = trunc i64 10 to i32
  %9 = call i32 @putchar(i32 %8)
  %10 = call i32 @puts(i8* getelementptr ([31 x i8], [31 x i8]* @.str.4, i64 0, i64 0))
  %11 = call i32 @puts(i8* getelementptr ([10 x i8], [10 x i8]* @.str.5, i64 0, i64 0))
  %12 = load %RangeIter*, %RangeIter** %r.5
  %13 = call i64 @RangeIter_has_more(%RangeIter* %12)
  %status.14 = alloca i64
  store i64 %13, i64* %status.14
  %15 = load i64, i64* %status.14
  %16 = add i64 %15, 48
  %17 = trunc i64 %16 to i32
  %18 = call i32 @putchar(i32 %17)
  %19 = trunc i64 10 to i32
  %20 = call i32 @putchar(i32 %19)
  %21 = call i32 @puts(i8* getelementptr ([13 x i8], [13 x i8]* @.str.6, i64 0, i64 0))
  br label %loop.start0
loop.start0:
  %22 = load %RangeIter*, %RangeIter** %r.5
  %23 = call i64 @RangeIter_next(%RangeIter* %22)
  %val.24 = alloca i64
  store i64 %23, i64* %val.24
  %25 = load i64, i64* %val.24
  %26 = icmp slt i64 %25, 0
  %27 = zext i1 %26 to i64
  %28 = icmp ne i64 %27, 0
  br i1 %28, label %then3, label %else4
then3:
  br label %loop.end2
else4:
  br label %merge5
merge5:
  %29 = add i64 0, 0
  %30 = load i64, i64* %val.24
  %31 = add i64 %30, 48
  %32 = trunc i64 %31 to i32
  %33 = call i32 @putchar(i32 %32)
  %34 = trunc i64 32 to i32
  %35 = call i32 @putchar(i32 %34)
  br label %loop.start0
loop.end2:
  %36 = trunc i64 10 to i32
  %37 = call i32 @putchar(i32 %36)
  %38 = trunc i64 10 to i32
  %39 = call i32 @putchar(i32 %38)
  %40 = call i32 @puts(i8* getelementptr ([27 x i8], [27 x i8]* @.str.7, i64 0, i64 0))
  %41 = call %FibIter @FibIter_new(i64 8)
  %fib.42.struct = alloca %FibIter
  store %FibIter %41, %FibIter* %fib.42.struct
  %fib.42 = alloca %FibIter*
  store %FibIter* %fib.42.struct, %FibIter** %fib.42
  br label %loop.start6
loop.start6:
  %43 = load %FibIter*, %FibIter** %fib.42
  %44 = call i64 @FibIter_has_more(%FibIter* %43)
  %45 = icmp eq i64 %44, 0
  %46 = zext i1 %45 to i64
  %47 = icmp ne i64 %46, 0
  br i1 %47, label %then9, label %else10
then9:
  br label %loop.end8
else10:
  br label %merge11
merge11:
  %48 = add i64 0, 0
  %49 = load %FibIter*, %FibIter** %fib.42
  %50 = call i64 @FibIter_next(%FibIter* %49)
  %val.51 = alloca i64
  store i64 %50, i64* %val.51
  %52 = load i64, i64* %val.51
  %53 = add i64 %52, 48
  %54 = trunc i64 %53 to i32
  %55 = call i32 @putchar(i32 %54)
  %56 = trunc i64 32 to i32
  %57 = call i32 @putchar(i32 %56)
  br label %loop.start6
loop.end8:
  %58 = trunc i64 10 to i32
  %59 = call i32 @putchar(i32 %58)
  %60 = trunc i64 10 to i32
  %61 = call i32 @putchar(i32 %60)
  %62 = call i32 @puts(i8* getelementptr ([34 x i8], [34 x i8]* @.str.8, i64 0, i64 0))
  %63 = call %RangeIter @RangeIter_new(i64 5, i64 8)
  %r2.64.struct = alloca %RangeIter
  store %RangeIter %63, %RangeIter* %r2.64.struct
  %r2.64 = alloca %RangeIter*
  store %RangeIter* %r2.64.struct, %RangeIter** %r2.64
  %sum.65 = alloca i64
  store i64 0, i64* %sum.65
  br label %loop.start12
loop.start12:
  %66 = load %RangeIter*, %RangeIter** %r2.64
  %67 = call i64 @RangeIter_has_more(%RangeIter* %66)
  %68 = icmp eq i64 %67, 0
  %69 = zext i1 %68 to i64
  %70 = icmp ne i64 %69, 0
  br i1 %70, label %then15, label %else16
then15:
  br label %loop.end14
else16:
  br label %merge17
merge17:
  %71 = add i64 0, 0
  %72 = load %RangeIter*, %RangeIter** %r2.64
  %73 = call i64 @RangeIter_next(%RangeIter* %72)
  %val.74 = alloca i64
  store i64 %73, i64* %val.74
  %75 = load i64, i64* %sum.65
  %76 = load i64, i64* %val.74
  %77 = add i64 %75, %76
  store i64 %77, i64* %sum.65
  br label %loop.start12
loop.end14:
  %78 = call i32 @puts(i8* getelementptr ([6 x i8], [6 x i8]* @.str.9, i64 0, i64 0))
  %79 = load i64, i64* %sum.65
  %80 = add i64 %79, 48
  %81 = trunc i64 %80 to i32
  %82 = call i32 @putchar(i32 %81)
  %83 = trunc i64 10 to i32
  %84 = call i32 @putchar(i32 %83)
  %85 = trunc i64 10 to i32
  %86 = call i32 @putchar(i32 %85)
  %87 = call i32 @puts(i8* getelementptr ([25 x i8], [25 x i8]* @.str.10, i64 0, i64 0))
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

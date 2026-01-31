; ModuleID = 'comprehensive_iterator_test'
source_filename = "<vais>"

%FibIter = type { i64, i64, i64, i64 }
%RangeIter = type { i64, i64 }
declare i64 @vais_gc_alloc(i64, i32)
declare i64 @vais_gc_bytes_allocated()
declare i32 @isdigit(i32)
declare void @exit(i32)
declare void @free(i64)
declare i64 @labs(i64)
declare i64 @memcpy_str(i64, i8*, i64)
declare i64 @fseek(i64, i64, i64)
declare i64 @fwrite(i64, i64, i64, i64)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare i64 @fputs(i8*, i64)
declare i32 @isalpha(i32)
declare i32 @printf(i8*, ...)
declare i32 @usleep(i64)
declare i64 @strlen(i8*)
declare i32 @rand()
declare i32 @puts(i8*)
declare i64 @fopen(i8*, i8*)
declare i64 @vais_gc_collections()
declare i64 @vais_gc_collect()
declare i32 @toupper(i32)
declare double @fabs(double)
declare i64 @fflush(i64)
declare i32 @sched_yield()
declare i64 @strcat(i64, i8*)
declare i32 @strcmp(i8*, i8*)
declare i64 @malloc(i64)
declare i64 @vais_gc_objects_count()
declare i64 @strcpy(i64, i8*)
declare i64 @atol(i8*)
declare i32 @fclose(i64)
declare i32 @atoi(i8*)
declare i64 @ftell(i64)
declare i32 @strncmp(i8*, i8*, i64)
declare double @sqrt(double)
declare i32 @tolower(i32)
declare i32 @putchar(i32)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @vais_gc_add_root(i64)
declare double @atof(i8*)
declare i64 @feof(i64)
declare i64 @vais_gc_remove_root(i64)
declare i64 @memcpy(i64, i64, i64)
declare i64 @vais_gc_set_threshold(i64)
declare i64 @vais_gc_print_stats()
declare i64 @fgets(i64, i64, i64)
declare i64 @fputc(i64, i64)
declare i64 @fgetc(i64)
declare void @srand(i32)
declare i64 @vais_gc_init()
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

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
  %9 = getelementptr %RangeIter, %RangeIter* %self, i32 0, i32 0
  %10 = load i64, i64* %9
  %11 = add i64 %10, 1
  %12 = getelementptr %RangeIter, %RangeIter* %self, i32 0, i32 0
  store i64 %11, i64* %12
  br label %merge2
else1:
  %13 = sub i64 0, 1
  br label %merge2
merge2:
  %14 = phi i64 [ %8, %then0 ], [ %13, %else1 ]
  ret i64 %14
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
  %1 = sext i32 %0 to i64
  %2 = getelementptr %RangeIter, %RangeIter* %self, i32 0, i32 0
  %3 = load i64, i64* %2
  %4 = add i64 %3, 48
  %5 = trunc i64 %4 to i32
  %6 = call i32 @putchar(i32 %5)
  %7 = sext i32 %6 to i64
  %8 = call i32 @puts(i8* getelementptr ([5 x i8], [5 x i8]* @.str.1, i64 0, i64 0))
  %9 = sext i32 %8 to i64
  %10 = getelementptr %RangeIter, %RangeIter* %self, i32 0, i32 1
  %11 = load i64, i64* %10
  %12 = add i64 %11, 48
  %13 = trunc i64 %12 to i32
  %14 = call i32 @putchar(i32 %13)
  %15 = sext i32 %14 to i64
  %16 = trunc i64 10 to i32
  %17 = call i32 @putchar(i32 %16)
  %18 = sext i32 %17 to i64
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
  %10 = getelementptr %FibIter, %FibIter* %self, i32 0, i32 1
  %11 = load i64, i64* %10
  %12 = getelementptr %FibIter, %FibIter* %self, i32 0, i32 0
  store i64 %11, i64* %12
  %13 = getelementptr %FibIter, %FibIter* %self, i32 0, i32 1
  %14 = load i64, i64* %13
  %15 = add i64 %9, %14
  %16 = getelementptr %FibIter, %FibIter* %self, i32 0, i32 1
  store i64 %15, i64* %16
  %17 = getelementptr %FibIter, %FibIter* %self, i32 0, i32 2
  %18 = load i64, i64* %17
  %19 = add i64 %18, 1
  %20 = getelementptr %FibIter, %FibIter* %self, i32 0, i32 2
  store i64 %19, i64* %20
  br label %merge2
merge2:
  %21 = phi i64 [ %7, %then0 ], [ %9, %else1 ]
  ret i64 %21
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
  %1 = sext i32 %0 to i64
  %2 = trunc i64 10 to i32
  %3 = call i32 @putchar(i32 %2)
  %4 = sext i32 %3 to i64
  %5 = call i32 @puts(i8* getelementptr ([23 x i8], [23 x i8]* @.str.3, i64 0, i64 0))
  %6 = sext i32 %5 to i64
  %7 = call %RangeIter @RangeIter_new(i64 0, i64 5)
  %r.8.struct = alloca %RangeIter
  store %RangeIter %7, %RangeIter* %r.8.struct
  %r.8 = alloca %RangeIter*
  store %RangeIter* %r.8.struct, %RangeIter** %r.8
  %9 = load %RangeIter*, %RangeIter** %r.8
  %10 = call i64 @RangeIter_print(%RangeIter* %9)
  %11 = trunc i64 10 to i32
  %12 = call i32 @putchar(i32 %11)
  %13 = sext i32 %12 to i64
  %14 = call i32 @puts(i8* getelementptr ([31 x i8], [31 x i8]* @.str.4, i64 0, i64 0))
  %15 = sext i32 %14 to i64
  %16 = call i32 @puts(i8* getelementptr ([10 x i8], [10 x i8]* @.str.5, i64 0, i64 0))
  %17 = sext i32 %16 to i64
  %18 = load %RangeIter*, %RangeIter** %r.8
  %19 = call i64 @RangeIter_has_more(%RangeIter* %18)
  %20 = add i64 %19, 48
  %21 = trunc i64 %20 to i32
  %22 = call i32 @putchar(i32 %21)
  %23 = sext i32 %22 to i64
  %24 = trunc i64 10 to i32
  %25 = call i32 @putchar(i32 %24)
  %26 = sext i32 %25 to i64
  %27 = call i32 @puts(i8* getelementptr ([13 x i8], [13 x i8]* @.str.6, i64 0, i64 0))
  %28 = sext i32 %27 to i64
  br label %loop.start0
loop.start0:
  %29 = load %RangeIter*, %RangeIter** %r.8
  %30 = call i64 @RangeIter_next(%RangeIter* %29)
  %31 = icmp slt i64 %30, 0
  %32 = zext i1 %31 to i64
  %33 = icmp ne i64 %32, 0
  br i1 %33, label %then3, label %else4
then3:
  br label %loop.end2
else4:
  br label %merge5
merge5:
  %34 = add i64 0, 0
  %35 = add i64 %30, 48
  %36 = trunc i64 %35 to i32
  %37 = call i32 @putchar(i32 %36)
  %38 = sext i32 %37 to i64
  %39 = trunc i64 32 to i32
  %40 = call i32 @putchar(i32 %39)
  %41 = sext i32 %40 to i64
  br label %loop.start0
loop.end2:
  %42 = trunc i64 10 to i32
  %43 = call i32 @putchar(i32 %42)
  %44 = sext i32 %43 to i64
  %45 = trunc i64 10 to i32
  %46 = call i32 @putchar(i32 %45)
  %47 = sext i32 %46 to i64
  %48 = call i32 @puts(i8* getelementptr ([27 x i8], [27 x i8]* @.str.7, i64 0, i64 0))
  %49 = sext i32 %48 to i64
  %50 = call %FibIter @FibIter_new(i64 8)
  %fib.51.struct = alloca %FibIter
  store %FibIter %50, %FibIter* %fib.51.struct
  %fib.51 = alloca %FibIter*
  store %FibIter* %fib.51.struct, %FibIter** %fib.51
  br label %loop.start6
loop.start6:
  %52 = load %FibIter*, %FibIter** %fib.51
  %53 = call i64 @FibIter_has_more(%FibIter* %52)
  %54 = icmp eq i64 %53, 0
  %55 = zext i1 %54 to i64
  %56 = icmp ne i64 %55, 0
  br i1 %56, label %then9, label %else10
then9:
  br label %loop.end8
else10:
  br label %merge11
merge11:
  %57 = add i64 0, 0
  %58 = load %FibIter*, %FibIter** %fib.51
  %59 = call i64 @FibIter_next(%FibIter* %58)
  %60 = add i64 %59, 48
  %61 = trunc i64 %60 to i32
  %62 = call i32 @putchar(i32 %61)
  %63 = sext i32 %62 to i64
  %64 = trunc i64 32 to i32
  %65 = call i32 @putchar(i32 %64)
  %66 = sext i32 %65 to i64
  br label %loop.start6
loop.end8:
  %67 = trunc i64 10 to i32
  %68 = call i32 @putchar(i32 %67)
  %69 = sext i32 %68 to i64
  %70 = trunc i64 10 to i32
  %71 = call i32 @putchar(i32 %70)
  %72 = sext i32 %71 to i64
  %73 = call i32 @puts(i8* getelementptr ([34 x i8], [34 x i8]* @.str.8, i64 0, i64 0))
  %74 = sext i32 %73 to i64
  %75 = call %RangeIter @RangeIter_new(i64 5, i64 8)
  %r2.76.struct = alloca %RangeIter
  store %RangeIter %75, %RangeIter* %r2.76.struct
  %r2.76 = alloca %RangeIter*
  store %RangeIter* %r2.76.struct, %RangeIter** %r2.76
  %sum.77 = alloca i64
  store i64 0, i64* %sum.77
  br label %loop.start12
loop.start12:
  %78 = load %RangeIter*, %RangeIter** %r2.76
  %79 = call i64 @RangeIter_has_more(%RangeIter* %78)
  %80 = icmp eq i64 %79, 0
  %81 = zext i1 %80 to i64
  %82 = icmp ne i64 %81, 0
  br i1 %82, label %then15, label %else16
then15:
  br label %loop.end14
else16:
  br label %merge17
merge17:
  %83 = add i64 0, 0
  %84 = load %RangeIter*, %RangeIter** %r2.76
  %85 = call i64 @RangeIter_next(%RangeIter* %84)
  %86 = load i64, i64* %sum.77
  %87 = add i64 %86, %85
  store i64 %87, i64* %sum.77
  br label %loop.start12
loop.end14:
  %88 = call i32 @puts(i8* getelementptr ([6 x i8], [6 x i8]* @.str.9, i64 0, i64 0))
  %89 = sext i32 %88 to i64
  %90 = load i64, i64* %sum.77
  %91 = add i64 %90, 48
  %92 = trunc i64 %91 to i32
  %93 = call i32 @putchar(i32 %92)
  %94 = sext i32 %93 to i64
  %95 = trunc i64 10 to i32
  %96 = call i32 @putchar(i32 %95)
  %97 = sext i32 %96 to i64
  %98 = trunc i64 10 to i32
  %99 = call i32 @putchar(i32 %98)
  %100 = sext i32 %99 to i64
  %101 = call i32 @puts(i8* getelementptr ([25 x i8], [25 x i8]* @.str.10, i64 0, i64 0))
  %102 = sext i32 %101 to i64
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

; Helper function: load f64 from memory
define double @__load_f64(i64 %ptr) {
entry:
  %0 = inttoptr i64 %ptr to double*
  %1 = load double, double* %0
  ret double %1
}

; Helper function: store f64 to memory
define void @__store_f64(i64 %ptr, double %val) {
entry:
  %0 = inttoptr i64 %ptr to double*
  store double %val, double* %0
  ret void
}

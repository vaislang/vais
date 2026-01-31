; ModuleID = 'set_test'
source_filename = "<vais>"

declare i32 @atoi(i8*)
declare i64 @fputs(i8*, i64)
declare i64 @fflush(i64)
declare i64 @strlen(i8*)
declare i32 @isdigit(i32)
declare i64 @fopen(i8*, i8*)
declare i64 @vais_gc_set_threshold(i64)
declare i32 @usleep(i64)
declare double @atof(i8*)
declare void @free(i64)
declare i64 @vais_gc_init()
declare i64 @strcpy(i64, i8*)
declare i64 @vais_gc_add_root(i64)
declare i32 @puts(i64)
declare i64 @memcpy(i64, i64, i64)
declare i32 @fclose(i64)
declare i64 @vais_gc_collect()
declare i64 @strcat(i64, i8*)
declare i64 @feof(i64)
declare i64 @vais_gc_collections()
declare i64 @fgets(i64, i64, i64)
declare i64 @fseek(i64, i64, i64)
declare double @sqrt(double)
declare i64 @fgetc(i64)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i32 @printf(i8*, ...)
declare i32 @sched_yield()
declare i64 @vais_gc_bytes_allocated()
declare i64 @atol(i64)
declare i64 @fputc(i64, i64)
declare i32 @putchar(i32)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare i32 @toupper(i32)
declare void @srand(i32)
declare i32 @strcmp(i8*, i8*)
declare i64 @labs(i64)
declare double @fabs(double)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @memcpy_str(i64, i8*, i64)
declare i64 @vais_gc_print_stats()
declare i64 @vais_gc_remove_root(i64)
declare i64 @vais_gc_objects_count()
declare i64 @malloc(i64)
declare void @exit(i32)
declare i32 @isalpha(i32)
declare i64 @ftell(i64)
declare i64 @vais_gc_alloc(i64, i32)
declare i32 @rand()
declare i32 @tolower(i32)
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

@.str.0 = private unnamed_addr constant [29 x i8] c"Test 1: Basic Set Operations\00"
@.str.1 = private unnamed_addr constant [31 x i8] c"  Inserted 10, 20, 30. Count: \00"
@.str.2 = private unnamed_addr constant [16 x i8] c"  Contains 10: \00"
@.str.3 = private unnamed_addr constant [16 x i8] c"  Contains 99: \00"
@.str.4 = private unnamed_addr constant [34 x i8] c"  Duplicate insert(10) returned: \00"
@.str.5 = private unnamed_addr constant [9 x i8] c"  PASSED\00"
@.str.6 = private unnamed_addr constant [22 x i8] c"Test 2: Element Count\00"
@.str.7 = private unnamed_addr constant [31 x i8] c"  Inserted 5 values, counted: \00"
@.str.8 = private unnamed_addr constant [9 x i8] c"  PASSED\00"
@.str.9 = private unnamed_addr constant [24 x i8] c"Test 3: Negative Values\00"
@.str.10 = private unnamed_addr constant [17 x i8] c"  Contains -10: \00"
@.str.11 = private unnamed_addr constant [15 x i8] c"  Contains 5: \00"
@.str.12 = private unnamed_addr constant [9 x i8] c"  PASSED\00"
@.str.13 = private unnamed_addr constant [28 x i8] c"=== Set Collection Test ===\00"
@.str.14 = private unnamed_addr constant [29 x i8] c"=== All Set Tests PASSED ===\00"
@.str.15 = private unnamed_addr constant [26 x i8] c"=== Some Tests FAILED ===\00"
@.str.16 = private unnamed_addr constant [26 x i8] c"=== Some Tests FAILED ===\00"
@.str.17 = private unnamed_addr constant [26 x i8] c"=== Some Tests FAILED ===\00"

define i64 @print_num(i64 %n) {
entry:
  %0 = icmp sge i64 %n, 100
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %then0, label %else1
then0:
  %3 = sdiv i64 %n, 100
  %4 = add i64 %3, 48
  %5 = trunc i64 %4 to i32
  %6 = call i32 @putchar(i32 %5)
  %7 = sext i32 %6 to i64
  br label %merge2
else1:
  br label %merge2
merge2:
  %8 = add i64 0, 0
  %9 = icmp sge i64 %n, 10
  %10 = zext i1 %9 to i64
  %11 = icmp ne i64 %10, 0
  br i1 %11, label %then3, label %else4
then3:
  %12 = sdiv i64 %n, 10
  %13 = srem i64 %12, 10
  %14 = add i64 %13, 48
  %15 = trunc i64 %14 to i32
  %16 = call i32 @putchar(i32 %15)
  %17 = sext i32 %16 to i64
  br label %merge5
else4:
  br label %merge5
merge5:
  %18 = add i64 0, 0
  %19 = srem i64 %n, 10
  %20 = add i64 %19, 48
  %21 = trunc i64 %20 to i32
  %22 = call i32 @putchar(i32 %21)
  %23 = sext i32 %22 to i64
  ret i64 0
}

define i64 @init_buckets(i64 %buckets, i64 %cap, i64 %i) {
entry:
  %0 = icmp sge i64 %i, %cap
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %then0, label %else1
then0:
  br label %merge2
else1:
  %3 = mul i64 %i, 8
  %4 = add i64 %buckets, %3
  call void @__store_i64(i64 %4, i64 0)
  %5 = add i64 %i, 1
  %6 = call i64 @init_buckets(i64 %buckets, i64 %cap, i64 %5)
  br label %merge2
merge2:
  %7 = phi i64 [ 0, %then0 ], [ %6, %else1 ]
  ret i64 %7
}

define i64 @set_hash(i64 %value, i64 %cap) {
entry:
  %0 = icmp slt i64 %value, 0
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %then0, label %else1
then0:
  %3 = sub i64 0, %value
  br label %merge2
else1:
  br label %merge2
merge2:
  %4 = phi i64 [ %3, %then0 ], [ %value, %else1 ]
  %5 = srem i64 %4, %cap
  ret i64 %5
}

define i64 @set_search_chain(i64 %entry_ptr, i64 %value) {
entry:
  %0 = icmp eq i64 %entry_ptr, 0
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %then0, label %else1
then0:
  br label %merge2
else1:
  %3 = call i64 @__load_i64(i64 %entry_ptr)
  %4 = icmp eq i64 %3, %value
  %5 = zext i1 %4 to i64
  %6 = icmp ne i64 %5, 0
  br i1 %6, label %then3, label %else4
then3:
  br label %merge5
else4:
  %7 = add i64 %entry_ptr, 8
  %8 = call i64 @__load_i64(i64 %7)
  %9 = call i64 @set_search_chain(i64 %8, i64 %value)
  br label %merge5
merge5:
  %10 = phi i64 [ 1, %then3 ], [ %9, %else4 ]
  br label %merge2
merge2:
  %11 = phi i64 [ 0, %then0 ], [ %10, %merge5 ]
  ret i64 %11
}

define i64 @set_contains(i64 %buckets, i64 %cap, i64 %value) {
entry:
  %0 = call i64 @set_hash(i64 %value, i64 %cap)
  %1 = mul i64 %0, 8
  %2 = add i64 %buckets, %1
  %3 = call i64 @__load_i64(i64 %2)
  %4 = call i64 @set_search_chain(i64 %3, i64 %value)
  ret i64 %4
}

define i64 @set_insert(i64 %buckets, i64 %cap, i64 %value) {
entry:
  %0 = call i64 @set_contains(i64 %buckets, i64 %cap, i64 %value)
  %1 = icmp eq i64 %0, 1
  %2 = zext i1 %1 to i64
  %3 = icmp ne i64 %2, 0
  br i1 %3, label %then0, label %else1
then0:
  br label %merge2
else1:
  %4 = call i64 @set_hash(i64 %value, i64 %cap)
  %5 = mul i64 %4, 8
  %6 = add i64 %buckets, %5
  %7 = call i64 @__load_i64(i64 %6)
  %8 = call i8* @malloc(i64 16)
  %9 = ptrtoint i8* %8 to i64
  call void @__store_i64(i64 %9, i64 %value)
  %10 = add i64 %9, 8
  call void @__store_i64(i64 %10, i64 %7)
  %11 = mul i64 %4, 8
  %12 = add i64 %buckets, %11
  call void @__store_i64(i64 %12, i64 %9)
  br label %merge2
merge2:
  %13 = phi i64 [ 0, %then0 ], [ 1, %else1 ]
  ret i64 %13
}

define i64 @count_chain(i64 %entry_ptr) {
entry:
  %0 = icmp eq i64 %entry_ptr, 0
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %then0, label %else1
then0:
  br label %merge2
else1:
  %3 = add i64 %entry_ptr, 8
  %4 = call i64 @__load_i64(i64 %3)
  %5 = call i64 @count_chain(i64 %4)
  %6 = add i64 1, %5
  br label %merge2
merge2:
  %7 = phi i64 [ 0, %then0 ], [ %6, %else1 ]
  ret i64 %7
}

define i64 @set_count(i64 %buckets, i64 %cap, i64 %i) {
entry:
  %0 = icmp sge i64 %i, %cap
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %then0, label %else1
then0:
  br label %merge2
else1:
  %3 = mul i64 %i, 8
  %4 = add i64 %buckets, %3
  %5 = call i64 @__load_i64(i64 %4)
  %6 = call i64 @count_chain(i64 %5)
  %7 = add i64 %i, 1
  %8 = call i64 @set_count(i64 %buckets, i64 %cap, i64 %7)
  %9 = add i64 %6, %8
  br label %merge2
merge2:
  %10 = phi i64 [ 0, %then0 ], [ %9, %else1 ]
  ret i64 %10
}

define i64 @test_basic() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([29 x i8], [29 x i8]* @.str.0, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = mul i64 8, 8
  %3 = call i8* @malloc(i64 %2)
  %4 = ptrtoint i8* %3 to i64
  %5 = call i64 @init_buckets(i64 %4, i64 8, i64 0)
  %6 = call i64 @set_insert(i64 %4, i64 8, i64 10)
  %7 = call i64 @set_insert(i64 %4, i64 8, i64 20)
  %8 = call i64 @set_insert(i64 %4, i64 8, i64 30)
  %9 = add i64 %6, %7
  %10 = add i64 %9, %8
  %11 = call i32 @puts(i8* getelementptr ([31 x i8], [31 x i8]* @.str.1, i64 0, i64 0))
  %12 = sext i32 %11 to i64
  %13 = call i64 @print_num(i64 %10)
  %14 = trunc i64 10 to i32
  %15 = call i32 @putchar(i32 %14)
  %16 = sext i32 %15 to i64
  %17 = call i64 @set_contains(i64 %4, i64 8, i64 10)
  %18 = call i64 @set_contains(i64 %4, i64 8, i64 20)
  %19 = call i64 @set_contains(i64 %4, i64 8, i64 99)
  %20 = call i32 @puts(i8* getelementptr ([16 x i8], [16 x i8]* @.str.2, i64 0, i64 0))
  %21 = sext i32 %20 to i64
  %22 = call i64 @print_num(i64 %17)
  %23 = trunc i64 10 to i32
  %24 = call i32 @putchar(i32 %23)
  %25 = sext i32 %24 to i64
  %26 = call i32 @puts(i8* getelementptr ([16 x i8], [16 x i8]* @.str.3, i64 0, i64 0))
  %27 = sext i32 %26 to i64
  %28 = call i64 @print_num(i64 %19)
  %29 = trunc i64 10 to i32
  %30 = call i32 @putchar(i32 %29)
  %31 = sext i32 %30 to i64
  %32 = call i64 @set_insert(i64 %4, i64 8, i64 10)
  %33 = call i32 @puts(i8* getelementptr ([34 x i8], [34 x i8]* @.str.4, i64 0, i64 0))
  %34 = sext i32 %33 to i64
  %35 = call i64 @print_num(i64 %32)
  %36 = trunc i64 10 to i32
  %37 = call i32 @putchar(i32 %36)
  %38 = sext i32 %37 to i64
  %39 = icmp eq i64 %17, 1
  %40 = zext i1 %39 to i64
  %41 = icmp ne i64 %40, 0
  br i1 %41, label %then0, label %else1
then0:
  %42 = icmp eq i64 %18, 1
  %43 = zext i1 %42 to i64
  %44 = icmp ne i64 %43, 0
  br i1 %44, label %then3, label %else4
then3:
  %45 = icmp eq i64 %19, 0
  %46 = zext i1 %45 to i64
  %47 = icmp ne i64 %46, 0
  br i1 %47, label %then6, label %else7
then6:
  %48 = icmp eq i64 %32, 0
  %49 = zext i1 %48 to i64
  %50 = icmp ne i64 %49, 0
  br i1 %50, label %then9, label %else10
then9:
  %51 = call i32 @puts(i8* getelementptr ([9 x i8], [9 x i8]* @.str.5, i64 0, i64 0))
  %52 = sext i32 %51 to i64
  %53 = trunc i64 10 to i32
  %54 = call i32 @putchar(i32 %53)
  %55 = sext i32 %54 to i64
  br label %merge11
else10:
  br label %merge11
merge11:
  %56 = phi i64 [ 1, %then9 ], [ 0, %else10 ]
  br label %merge8
else7:
  br label %merge8
merge8:
  %57 = phi i64 [ %56, %merge11 ], [ 0, %else7 ]
  br label %merge5
else4:
  br label %merge5
merge5:
  %58 = phi i64 [ %57, %merge8 ], [ 0, %else4 ]
  br label %merge2
else1:
  br label %merge2
merge2:
  %59 = phi i64 [ %58, %merge5 ], [ 0, %else1 ]
  ret i64 %59
}

define i64 @test_count() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.6, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = mul i64 16, 8
  %3 = call i8* @malloc(i64 %2)
  %4 = ptrtoint i8* %3 to i64
  %5 = call i64 @init_buckets(i64 %4, i64 16, i64 0)
  %6 = call i64 @set_insert(i64 %4, i64 16, i64 1)
  %7 = call i64 @set_insert(i64 %4, i64 16, i64 2)
  %8 = call i64 @set_insert(i64 %4, i64 16, i64 3)
  %9 = call i64 @set_insert(i64 %4, i64 16, i64 4)
  %10 = call i64 @set_insert(i64 %4, i64 16, i64 5)
  %11 = call i64 @set_count(i64 %4, i64 16, i64 0)
  %12 = call i32 @puts(i8* getelementptr ([31 x i8], [31 x i8]* @.str.7, i64 0, i64 0))
  %13 = sext i32 %12 to i64
  %14 = call i64 @print_num(i64 %11)
  %15 = trunc i64 10 to i32
  %16 = call i32 @putchar(i32 %15)
  %17 = sext i32 %16 to i64
  %18 = icmp eq i64 %11, 5
  %19 = zext i1 %18 to i64
  %20 = icmp ne i64 %19, 0
  br i1 %20, label %then0, label %else1
then0:
  %21 = call i32 @puts(i8* getelementptr ([9 x i8], [9 x i8]* @.str.8, i64 0, i64 0))
  %22 = sext i32 %21 to i64
  %23 = trunc i64 10 to i32
  %24 = call i32 @putchar(i32 %23)
  %25 = sext i32 %24 to i64
  br label %merge2
else1:
  br label %merge2
merge2:
  %26 = phi i64 [ 1, %then0 ], [ 0, %else1 ]
  ret i64 %26
}

define i64 @test_negative() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.9, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = mul i64 8, 8
  %3 = call i8* @malloc(i64 %2)
  %4 = ptrtoint i8* %3 to i64
  %5 = call i64 @init_buckets(i64 %4, i64 8, i64 0)
  %6 = sub i64 0, 10
  %7 = call i64 @set_insert(i64 %4, i64 8, i64 %6)
  %8 = sub i64 0, 20
  %9 = call i64 @set_insert(i64 %4, i64 8, i64 %8)
  %10 = call i64 @set_insert(i64 %4, i64 8, i64 5)
  %11 = sub i64 0, 10
  %12 = call i64 @set_contains(i64 %4, i64 8, i64 %11)
  %13 = call i64 @set_contains(i64 %4, i64 8, i64 5)
  %14 = call i32 @puts(i8* getelementptr ([17 x i8], [17 x i8]* @.str.10, i64 0, i64 0))
  %15 = sext i32 %14 to i64
  %16 = call i64 @print_num(i64 %12)
  %17 = trunc i64 10 to i32
  %18 = call i32 @putchar(i32 %17)
  %19 = sext i32 %18 to i64
  %20 = call i32 @puts(i8* getelementptr ([15 x i8], [15 x i8]* @.str.11, i64 0, i64 0))
  %21 = sext i32 %20 to i64
  %22 = call i64 @print_num(i64 %13)
  %23 = trunc i64 10 to i32
  %24 = call i32 @putchar(i32 %23)
  %25 = sext i32 %24 to i64
  %26 = icmp eq i64 %12, 1
  %27 = zext i1 %26 to i64
  %28 = icmp ne i64 %27, 0
  br i1 %28, label %then0, label %else1
then0:
  %29 = icmp eq i64 %13, 1
  %30 = zext i1 %29 to i64
  %31 = icmp ne i64 %30, 0
  br i1 %31, label %then3, label %else4
then3:
  %32 = call i32 @puts(i8* getelementptr ([9 x i8], [9 x i8]* @.str.12, i64 0, i64 0))
  %33 = sext i32 %32 to i64
  %34 = trunc i64 10 to i32
  %35 = call i32 @putchar(i32 %34)
  %36 = sext i32 %35 to i64
  br label %merge5
else4:
  br label %merge5
merge5:
  %37 = phi i64 [ 1, %then3 ], [ 0, %else4 ]
  br label %merge2
else1:
  br label %merge2
merge2:
  %38 = phi i64 [ %37, %merge5 ], [ 0, %else1 ]
  ret i64 %38
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([28 x i8], [28 x i8]* @.str.13, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = trunc i64 10 to i32
  %3 = call i32 @putchar(i32 %2)
  %4 = sext i32 %3 to i64
  %5 = call i64 @test_basic()
  %6 = call i64 @test_count()
  %7 = call i64 @test_negative()
  %8 = trunc i64 10 to i32
  %9 = call i32 @putchar(i32 %8)
  %10 = sext i32 %9 to i64
  %11 = icmp eq i64 %5, 1
  %12 = zext i1 %11 to i64
  %13 = icmp ne i64 %12, 0
  br i1 %13, label %then0, label %else1
then0:
  %14 = icmp eq i64 %6, 1
  %15 = zext i1 %14 to i64
  %16 = icmp ne i64 %15, 0
  br i1 %16, label %then3, label %else4
then3:
  %17 = icmp eq i64 %7, 1
  %18 = zext i1 %17 to i64
  %19 = icmp ne i64 %18, 0
  br i1 %19, label %then6, label %else7
then6:
  %20 = call i32 @puts(i8* getelementptr ([29 x i8], [29 x i8]* @.str.14, i64 0, i64 0))
  %21 = sext i32 %20 to i64
  br label %merge8
else7:
  %22 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.15, i64 0, i64 0))
  %23 = sext i32 %22 to i64
  br label %merge8
merge8:
  %24 = phi i64 [ 0, %then6 ], [ 1, %else7 ]
  br label %merge5
else4:
  %25 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.16, i64 0, i64 0))
  %26 = sext i32 %25 to i64
  br label %merge5
merge5:
  %27 = phi i64 [ %24, %merge8 ], [ 1, %else4 ]
  br label %merge2
else1:
  %28 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.17, i64 0, i64 0))
  %29 = sext i32 %28 to i64
  br label %merge2
merge2:
  %30 = phi i64 [ %27, %merge5 ], [ 1, %else1 ]
  ret i64 %30
}


; C library function declarations
declare i64 @write(i32, i8*, i64)

; Global constants for runtime functions
@.panic_newline = private unnamed_addr constant [2 x i8] c"\0A\00"

; Runtime panic function (used by assert)
define i64 @__panic(i8* %msg) {
entry:
  ; Calculate message length
  %len = call i64 @strlen(i8* %msg)
  ; Write message to stderr (fd=2)
  %0 = call i64 @write(i32 2, i8* %msg, i64 %len)
  ; Write newline
  %1 = call i64 @write(i32 2, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @.panic_newline, i64 0, i64 0), i64 1)
  call void @exit(i32 1)
  unreachable
}

; Runtime contract failure function
define i64 @__contract_fail(i64 %kind, i8* %condition, i8* %file, i64 %line, i8* %func) {
entry:
  ; Calculate message length
  %len = call i64 @strlen(i8* %condition)
  ; Write contract failure message to stderr (fd=2)
  %0 = call i64 @write(i32 2, i8* %condition, i64 %len)
  ; Write newline
  %1 = call i64 @write(i32 2, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @.panic_newline, i64 0, i64 0), i64 1)
  call void @exit(i32 1)
  unreachable
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

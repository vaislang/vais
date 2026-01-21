; ModuleID = 'set_test'
source_filename = "<vais>"

declare i32 @sched_yield()
declare i64 @fopen(i8*, i8*)
declare i64 @malloc(i64)
declare i64 @memcpy(i64, i64, i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @feof(i64)
declare i32 @puts(i8*)
declare i64 @ftell(i64)
declare i64 @strlen(i64)
declare i32 @usleep(i64)
declare i64 @fputc(i64, i64)
declare i64 @fseek(i64, i64, i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @fgets(i64, i64, i64)
declare i64 @fputs(i8*, i64)
declare i32 @strcmp(i8*, i8*)
declare i32 @putchar(i32)
declare i64 @fflush(i64)
declare i32 @printf(i8*)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @fgetc(i64)
declare void @exit(i32)
declare i32 @fclose(i64)
declare void @free(i64)
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
  %d1.4 = alloca i64
  store i64 %3, i64* %d1.4
  %5 = load i64, i64* %d1.4
  %6 = add i64 %5, 48
  %7 = trunc i64 %6 to i32
  %8 = call i32 @putchar(i32 %7)
  br label %merge2
else1:
  br label %merge2
merge2:
  %9 = add i64 0, 0
  %10 = icmp sge i64 %n, 10
  %11 = zext i1 %10 to i64
  %12 = icmp ne i64 %11, 0
  br i1 %12, label %then3, label %else4
then3:
  %13 = sdiv i64 %n, 10
  %14 = srem i64 %13, 10
  %d2.15 = alloca i64
  store i64 %14, i64* %d2.15
  %16 = load i64, i64* %d2.15
  %17 = add i64 %16, 48
  %18 = trunc i64 %17 to i32
  %19 = call i32 @putchar(i32 %18)
  br label %merge5
else4:
  br label %merge5
merge5:
  %20 = add i64 0, 0
  %21 = srem i64 %n, 10
  %d3.22 = alloca i64
  store i64 %21, i64* %d3.22
  %23 = load i64, i64* %d3.22
  %24 = add i64 %23, 48
  %25 = trunc i64 %24 to i32
  %26 = call i32 @putchar(i32 %25)
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
  %abs_v.5 = alloca i64
  store i64 %4, i64* %abs_v.5
  %6 = load i64, i64* %abs_v.5
  %7 = srem i64 %6, %cap
  ret i64 %7
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
  %entry_value.4 = alloca i64
  store i64 %3, i64* %entry_value.4
  %5 = load i64, i64* %entry_value.4
  %6 = icmp eq i64 %5, %value
  %7 = zext i1 %6 to i64
  %8 = icmp ne i64 %7, 0
  br i1 %8, label %then3, label %else4
then3:
  br label %merge5
else4:
  %9 = add i64 %entry_ptr, 8
  %10 = call i64 @__load_i64(i64 %9)
  %next.11 = alloca i64
  store i64 %10, i64* %next.11
  %12 = load i64, i64* %next.11
  %13 = call i64 @set_search_chain(i64 %12, i64 %value)
  br label %merge5
merge5:
  %14 = phi i64 [ 1, %then3 ], [ %13, %else4 ]
  br label %merge2
merge2:
  %15 = phi i64 [ 0, %then0 ], [ %14, %merge5 ]
  ret i64 %15
}

define i64 @set_contains(i64 %buckets, i64 %cap, i64 %value) {
entry:
  %0 = call i64 @set_hash(i64 %value, i64 %cap)
  %idx.1 = alloca i64
  store i64 %0, i64* %idx.1
  %2 = load i64, i64* %idx.1
  %3 = mul i64 %2, 8
  %4 = add i64 %buckets, %3
  %5 = call i64 @__load_i64(i64 %4)
  %entry_ptr.6 = alloca i64
  store i64 %5, i64* %entry_ptr.6
  %7 = load i64, i64* %entry_ptr.6
  %8 = call i64 @set_search_chain(i64 %7, i64 %value)
  ret i64 %8
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
  %idx.5 = alloca i64
  store i64 %4, i64* %idx.5
  %6 = load i64, i64* %idx.5
  %7 = mul i64 %6, 8
  %8 = add i64 %buckets, %7
  %9 = call i64 @__load_i64(i64 %8)
  %old_head.10 = alloca i64
  store i64 %9, i64* %old_head.10
  %11 = call i8* @malloc(i64 16)
  %12 = ptrtoint i8* %11 to i64
  %new_entry.13 = alloca i64
  store i64 %12, i64* %new_entry.13
  %14 = load i64, i64* %new_entry.13
  call void @__store_i64(i64 %14, i64 %value)
  %15 = load i64, i64* %new_entry.13
  %16 = add i64 %15, 8
  %17 = load i64, i64* %old_head.10
  call void @__store_i64(i64 %16, i64 %17)
  %18 = load i64, i64* %idx.5
  %19 = mul i64 %18, 8
  %20 = add i64 %buckets, %19
  %21 = load i64, i64* %new_entry.13
  call void @__store_i64(i64 %20, i64 %21)
  br label %merge2
merge2:
  %22 = phi i64 [ 0, %then0 ], [ 1, %else1 ]
  ret i64 %22
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
  %next.5 = alloca i64
  store i64 %4, i64* %next.5
  %6 = load i64, i64* %next.5
  %7 = call i64 @count_chain(i64 %6)
  %8 = add i64 1, %7
  br label %merge2
merge2:
  %9 = phi i64 [ 0, %then0 ], [ %8, %else1 ]
  ret i64 %9
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
  %entry_ptr.6 = alloca i64
  store i64 %5, i64* %entry_ptr.6
  %7 = load i64, i64* %entry_ptr.6
  %8 = call i64 @count_chain(i64 %7)
  %9 = add i64 %i, 1
  %10 = call i64 @set_count(i64 %buckets, i64 %cap, i64 %9)
  %11 = add i64 %8, %10
  br label %merge2
merge2:
  %12 = phi i64 [ 0, %then0 ], [ %11, %else1 ]
  ret i64 %12
}

define i64 @test_basic() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([29 x i8], [29 x i8]* @.str.0, i64 0, i64 0))
  %cap.1 = alloca i64
  store i64 8, i64* %cap.1
  %2 = load i64, i64* %cap.1
  %3 = mul i64 %2, 8
  %4 = call i8* @malloc(i64 %3)
  %5 = ptrtoint i8* %4 to i64
  %buckets.6 = alloca i64
  store i64 %5, i64* %buckets.6
  %7 = load i64, i64* %buckets.6
  %8 = load i64, i64* %cap.1
  %9 = call i64 @init_buckets(i64 %7, i64 %8, i64 0)
  %10 = load i64, i64* %buckets.6
  %11 = load i64, i64* %cap.1
  %12 = call i64 @set_insert(i64 %10, i64 %11, i64 10)
  %r1.13 = alloca i64
  store i64 %12, i64* %r1.13
  %14 = load i64, i64* %buckets.6
  %15 = load i64, i64* %cap.1
  %16 = call i64 @set_insert(i64 %14, i64 %15, i64 20)
  %r2.17 = alloca i64
  store i64 %16, i64* %r2.17
  %18 = load i64, i64* %buckets.6
  %19 = load i64, i64* %cap.1
  %20 = call i64 @set_insert(i64 %18, i64 %19, i64 30)
  %r3.21 = alloca i64
  store i64 %20, i64* %r3.21
  %22 = load i64, i64* %r1.13
  %23 = load i64, i64* %r2.17
  %24 = add i64 %22, %23
  %25 = load i64, i64* %r3.21
  %26 = add i64 %24, %25
  %size.27 = alloca i64
  store i64 %26, i64* %size.27
  %28 = call i32 @puts(i8* getelementptr ([31 x i8], [31 x i8]* @.str.1, i64 0, i64 0))
  %29 = load i64, i64* %size.27
  %30 = call i64 @print_num(i64 %29)
  %31 = trunc i64 10 to i32
  %32 = call i32 @putchar(i32 %31)
  %33 = load i64, i64* %buckets.6
  %34 = load i64, i64* %cap.1
  %35 = call i64 @set_contains(i64 %33, i64 %34, i64 10)
  %has10.36 = alloca i64
  store i64 %35, i64* %has10.36
  %37 = load i64, i64* %buckets.6
  %38 = load i64, i64* %cap.1
  %39 = call i64 @set_contains(i64 %37, i64 %38, i64 20)
  %has20.40 = alloca i64
  store i64 %39, i64* %has20.40
  %41 = load i64, i64* %buckets.6
  %42 = load i64, i64* %cap.1
  %43 = call i64 @set_contains(i64 %41, i64 %42, i64 99)
  %has99.44 = alloca i64
  store i64 %43, i64* %has99.44
  %45 = call i32 @puts(i8* getelementptr ([16 x i8], [16 x i8]* @.str.2, i64 0, i64 0))
  %46 = load i64, i64* %has10.36
  %47 = call i64 @print_num(i64 %46)
  %48 = trunc i64 10 to i32
  %49 = call i32 @putchar(i32 %48)
  %50 = call i32 @puts(i8* getelementptr ([16 x i8], [16 x i8]* @.str.3, i64 0, i64 0))
  %51 = load i64, i64* %has99.44
  %52 = call i64 @print_num(i64 %51)
  %53 = trunc i64 10 to i32
  %54 = call i32 @putchar(i32 %53)
  %55 = load i64, i64* %buckets.6
  %56 = load i64, i64* %cap.1
  %57 = call i64 @set_insert(i64 %55, i64 %56, i64 10)
  %dup.58 = alloca i64
  store i64 %57, i64* %dup.58
  %59 = call i32 @puts(i8* getelementptr ([34 x i8], [34 x i8]* @.str.4, i64 0, i64 0))
  %60 = load i64, i64* %dup.58
  %61 = call i64 @print_num(i64 %60)
  %62 = trunc i64 10 to i32
  %63 = call i32 @putchar(i32 %62)
  %64 = load i64, i64* %has10.36
  %65 = icmp eq i64 %64, 1
  %66 = zext i1 %65 to i64
  %67 = icmp ne i64 %66, 0
  br i1 %67, label %then0, label %else1
then0:
  %68 = load i64, i64* %has20.40
  %69 = icmp eq i64 %68, 1
  %70 = zext i1 %69 to i64
  %71 = icmp ne i64 %70, 0
  br i1 %71, label %then3, label %else4
then3:
  %72 = load i64, i64* %has99.44
  %73 = icmp eq i64 %72, 0
  %74 = zext i1 %73 to i64
  %75 = icmp ne i64 %74, 0
  br i1 %75, label %then6, label %else7
then6:
  %76 = load i64, i64* %dup.58
  %77 = icmp eq i64 %76, 0
  %78 = zext i1 %77 to i64
  %79 = icmp ne i64 %78, 0
  br i1 %79, label %then9, label %else10
then9:
  %80 = call i32 @puts(i8* getelementptr ([9 x i8], [9 x i8]* @.str.5, i64 0, i64 0))
  %81 = trunc i64 10 to i32
  %82 = call i32 @putchar(i32 %81)
  br label %merge11
else10:
  br label %merge11
merge11:
  %83 = phi i64 [ 1, %then9 ], [ 0, %else10 ]
  br label %merge8
else7:
  br label %merge8
merge8:
  %84 = phi i64 [ %83, %merge11 ], [ 0, %else7 ]
  br label %merge5
else4:
  br label %merge5
merge5:
  %85 = phi i64 [ %84, %merge8 ], [ 0, %else4 ]
  br label %merge2
else1:
  br label %merge2
merge2:
  %86 = phi i64 [ %85, %merge5 ], [ 0, %else1 ]
  ret i64 %86
}

define i64 @test_count() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.6, i64 0, i64 0))
  %cap.1 = alloca i64
  store i64 16, i64* %cap.1
  %2 = load i64, i64* %cap.1
  %3 = mul i64 %2, 8
  %4 = call i8* @malloc(i64 %3)
  %5 = ptrtoint i8* %4 to i64
  %buckets.6 = alloca i64
  store i64 %5, i64* %buckets.6
  %7 = load i64, i64* %buckets.6
  %8 = load i64, i64* %cap.1
  %9 = call i64 @init_buckets(i64 %7, i64 %8, i64 0)
  %10 = load i64, i64* %buckets.6
  %11 = load i64, i64* %cap.1
  %12 = call i64 @set_insert(i64 %10, i64 %11, i64 1)
  %13 = load i64, i64* %buckets.6
  %14 = load i64, i64* %cap.1
  %15 = call i64 @set_insert(i64 %13, i64 %14, i64 2)
  %16 = load i64, i64* %buckets.6
  %17 = load i64, i64* %cap.1
  %18 = call i64 @set_insert(i64 %16, i64 %17, i64 3)
  %19 = load i64, i64* %buckets.6
  %20 = load i64, i64* %cap.1
  %21 = call i64 @set_insert(i64 %19, i64 %20, i64 4)
  %22 = load i64, i64* %buckets.6
  %23 = load i64, i64* %cap.1
  %24 = call i64 @set_insert(i64 %22, i64 %23, i64 5)
  %25 = load i64, i64* %buckets.6
  %26 = load i64, i64* %cap.1
  %27 = call i64 @set_count(i64 %25, i64 %26, i64 0)
  %total.28 = alloca i64
  store i64 %27, i64* %total.28
  %29 = call i32 @puts(i8* getelementptr ([31 x i8], [31 x i8]* @.str.7, i64 0, i64 0))
  %30 = load i64, i64* %total.28
  %31 = call i64 @print_num(i64 %30)
  %32 = trunc i64 10 to i32
  %33 = call i32 @putchar(i32 %32)
  %34 = load i64, i64* %total.28
  %35 = icmp eq i64 %34, 5
  %36 = zext i1 %35 to i64
  %37 = icmp ne i64 %36, 0
  br i1 %37, label %then0, label %else1
then0:
  %38 = call i32 @puts(i8* getelementptr ([9 x i8], [9 x i8]* @.str.8, i64 0, i64 0))
  %39 = trunc i64 10 to i32
  %40 = call i32 @putchar(i32 %39)
  br label %merge2
else1:
  br label %merge2
merge2:
  %41 = phi i64 [ 1, %then0 ], [ 0, %else1 ]
  ret i64 %41
}

define i64 @test_negative() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.9, i64 0, i64 0))
  %cap.1 = alloca i64
  store i64 8, i64* %cap.1
  %2 = load i64, i64* %cap.1
  %3 = mul i64 %2, 8
  %4 = call i8* @malloc(i64 %3)
  %5 = ptrtoint i8* %4 to i64
  %buckets.6 = alloca i64
  store i64 %5, i64* %buckets.6
  %7 = load i64, i64* %buckets.6
  %8 = load i64, i64* %cap.1
  %9 = call i64 @init_buckets(i64 %7, i64 %8, i64 0)
  %10 = load i64, i64* %buckets.6
  %11 = load i64, i64* %cap.1
  %12 = sub i64 0, 10
  %13 = call i64 @set_insert(i64 %10, i64 %11, i64 %12)
  %14 = load i64, i64* %buckets.6
  %15 = load i64, i64* %cap.1
  %16 = sub i64 0, 20
  %17 = call i64 @set_insert(i64 %14, i64 %15, i64 %16)
  %18 = load i64, i64* %buckets.6
  %19 = load i64, i64* %cap.1
  %20 = call i64 @set_insert(i64 %18, i64 %19, i64 5)
  %21 = load i64, i64* %buckets.6
  %22 = load i64, i64* %cap.1
  %23 = sub i64 0, 10
  %24 = call i64 @set_contains(i64 %21, i64 %22, i64 %23)
  %has_neg10.25 = alloca i64
  store i64 %24, i64* %has_neg10.25
  %26 = load i64, i64* %buckets.6
  %27 = load i64, i64* %cap.1
  %28 = call i64 @set_contains(i64 %26, i64 %27, i64 5)
  %has_5.29 = alloca i64
  store i64 %28, i64* %has_5.29
  %30 = call i32 @puts(i8* getelementptr ([17 x i8], [17 x i8]* @.str.10, i64 0, i64 0))
  %31 = load i64, i64* %has_neg10.25
  %32 = call i64 @print_num(i64 %31)
  %33 = trunc i64 10 to i32
  %34 = call i32 @putchar(i32 %33)
  %35 = call i32 @puts(i8* getelementptr ([15 x i8], [15 x i8]* @.str.11, i64 0, i64 0))
  %36 = load i64, i64* %has_5.29
  %37 = call i64 @print_num(i64 %36)
  %38 = trunc i64 10 to i32
  %39 = call i32 @putchar(i32 %38)
  %40 = load i64, i64* %has_neg10.25
  %41 = icmp eq i64 %40, 1
  %42 = zext i1 %41 to i64
  %43 = icmp ne i64 %42, 0
  br i1 %43, label %then0, label %else1
then0:
  %44 = load i64, i64* %has_5.29
  %45 = icmp eq i64 %44, 1
  %46 = zext i1 %45 to i64
  %47 = icmp ne i64 %46, 0
  br i1 %47, label %then3, label %else4
then3:
  %48 = call i32 @puts(i8* getelementptr ([9 x i8], [9 x i8]* @.str.12, i64 0, i64 0))
  %49 = trunc i64 10 to i32
  %50 = call i32 @putchar(i32 %49)
  br label %merge5
else4:
  br label %merge5
merge5:
  %51 = phi i64 [ 1, %then3 ], [ 0, %else4 ]
  br label %merge2
else1:
  br label %merge2
merge2:
  %52 = phi i64 [ %51, %merge5 ], [ 0, %else1 ]
  ret i64 %52
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([28 x i8], [28 x i8]* @.str.13, i64 0, i64 0))
  %1 = trunc i64 10 to i32
  %2 = call i32 @putchar(i32 %1)
  %3 = call i64 @test_basic()
  %r1.4 = alloca i64
  store i64 %3, i64* %r1.4
  %5 = call i64 @test_count()
  %r2.6 = alloca i64
  store i64 %5, i64* %r2.6
  %7 = call i64 @test_negative()
  %r3.8 = alloca i64
  store i64 %7, i64* %r3.8
  %9 = trunc i64 10 to i32
  %10 = call i32 @putchar(i32 %9)
  %11 = load i64, i64* %r1.4
  %12 = icmp eq i64 %11, 1
  %13 = zext i1 %12 to i64
  %14 = icmp ne i64 %13, 0
  br i1 %14, label %then0, label %else1
then0:
  %15 = load i64, i64* %r2.6
  %16 = icmp eq i64 %15, 1
  %17 = zext i1 %16 to i64
  %18 = icmp ne i64 %17, 0
  br i1 %18, label %then3, label %else4
then3:
  %19 = load i64, i64* %r3.8
  %20 = icmp eq i64 %19, 1
  %21 = zext i1 %20 to i64
  %22 = icmp ne i64 %21, 0
  br i1 %22, label %then6, label %else7
then6:
  %23 = call i32 @puts(i8* getelementptr ([29 x i8], [29 x i8]* @.str.14, i64 0, i64 0))
  br label %merge8
else7:
  %24 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.15, i64 0, i64 0))
  br label %merge8
merge8:
  %25 = phi i64 [ 0, %then6 ], [ 1, %else7 ]
  br label %merge5
else4:
  %26 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.16, i64 0, i64 0))
  br label %merge5
merge5:
  %27 = phi i64 [ %25, %merge8 ], [ 1, %else4 ]
  br label %merge2
else1:
  %28 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.17, i64 0, i64 0))
  br label %merge2
merge2:
  %29 = phi i64 [ %27, %merge5 ], [ 1, %else1 ]
  ret i64 %29
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

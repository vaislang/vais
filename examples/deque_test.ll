; ModuleID = 'deque_test'
source_filename = "<vais>"

declare i64 @fopen(i8*, i8*)
declare i64 @memcpy(i64, i64, i64)
declare double @sqrt(double)
declare i64 @vais_gc_set_threshold(i64)
declare i32 @printf(i8*, ...)
declare i64 @fflush(i64)
declare i64 @vais_gc_alloc(i64, i32)
declare i64 @strcat(i64, i8*)
declare i32 @atoi(i8*)
declare i64 @vais_gc_objects_count()
declare i32 @isalpha(i32)
declare i32 @puts(i64)
declare i64 @strcpy(i64, i8*)
declare i64 @strlen(i8*)
declare i32 @fclose(i64)
declare i32 @toupper(i32)
declare double @fabs(double)
declare i32 @rand()
declare i64 @fputc(i64, i64)
declare i64 @vais_gc_collect()
declare i64 @vais_gc_remove_root(i64)
declare i32 @isdigit(i32)
declare i64 @fgetc(i64)
declare i32 @putchar(i32)
declare i64 @fseek(i64, i64, i64)
declare i64 @vais_gc_init()
declare void @srand(i32)
declare i64 @fgets(i64, i64, i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i32 @sched_yield()
declare i64 @vais_gc_bytes_allocated()
declare void @free(i64)
declare i64 @vais_gc_collections()
declare i64 @labs(i64)
declare i64 @fread(i64, i64, i64, i64)
declare void @exit(i32)
declare i64 @vais_gc_add_root(i64)
declare i64 @atol(i8*)
declare i64 @memcpy_str(i64, i8*, i64)
declare double @atof(i8*)
declare i32 @usleep(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @fputs(i8*, i64)
declare i32 @tolower(i32)
declare i64 @feof(i64)
declare i64 @vais_gc_print_stats()
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare i64 @ftell(i64)
declare i32 @strcmp(i8*, i8*)
declare i64 @malloc(i64)
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

@.str.0 = private unnamed_addr constant [29 x i8] c"Test 1: Push Back Operations\00"
@.str.1 = private unnamed_addr constant [20 x i8] c"  Pushed 10, 20, 30\00"
@.str.2 = private unnamed_addr constant [11 x i8] c"  Length: \00"
@.str.3 = private unnamed_addr constant [10 x i8] c"  Front: \00"
@.str.4 = private unnamed_addr constant [9 x i8] c"  Back: \00"
@.str.5 = private unnamed_addr constant [9 x i8] c"  PASSED\00"
@.str.6 = private unnamed_addr constant [30 x i8] c"Test 2: Push Front Operations\00"
@.str.7 = private unnamed_addr constant [24 x i8] c"  Push front 10, 20, 30\00"
@.str.8 = private unnamed_addr constant [11 x i8] c"  Length: \00"
@.str.9 = private unnamed_addr constant [25 x i8] c"  Front (should be 30): \00"
@.str.10 = private unnamed_addr constant [24 x i8] c"  Back (should be 10): \00"
@.str.11 = private unnamed_addr constant [9 x i8] c"  PASSED\00"
@.str.12 = private unnamed_addr constant [23 x i8] c"Test 3: Pop Operations\00"
@.str.13 = private unnamed_addr constant [17 x i8] c"  Pushed 1,2,3,4\00"
@.str.14 = private unnamed_addr constant [14 x i8] c"  Pop front: \00"
@.str.15 = private unnamed_addr constant [13 x i8] c"  Pop back: \00"
@.str.16 = private unnamed_addr constant [21 x i8] c"  Remaining length: \00"
@.str.17 = private unnamed_addr constant [9 x i8] c"  PASSED\00"
@.str.18 = private unnamed_addr constant [29 x i8] c"Test 4: Circular Wrap-around\00"
@.str.19 = private unnamed_addr constant [31 x i8] c"  After wrap-around operations\00"
@.str.20 = private unnamed_addr constant [11 x i8] c"  Length: \00"
@.str.21 = private unnamed_addr constant [10 x i8] c"  Front: \00"
@.str.22 = private unnamed_addr constant [9 x i8] c"  Back: \00"
@.str.23 = private unnamed_addr constant [9 x i8] c"  PASSED\00"
@.str.24 = private unnamed_addr constant [30 x i8] c"=== Deque Collection Test ===\00"
@.str.25 = private unnamed_addr constant [31 x i8] c"=== All Deque Tests PASSED ===\00"
@.str.26 = private unnamed_addr constant [26 x i8] c"=== Some Tests FAILED ===\00"
@.str.27 = private unnamed_addr constant [26 x i8] c"=== Some Tests FAILED ===\00"
@.str.28 = private unnamed_addr constant [26 x i8] c"=== Some Tests FAILED ===\00"
@.str.29 = private unnamed_addr constant [26 x i8] c"=== Some Tests FAILED ===\00"

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

define i64 @deque_create(i64 %capacity) {
entry:
  %0 = call i8* @malloc(i64 40)
  %1 = ptrtoint i8* %0 to i64
  %2 = mul i64 %capacity, 8
  %3 = call i8* @malloc(i64 %2)
  %4 = ptrtoint i8* %3 to i64
  call void @__store_i64(i64 %1, i64 %4)
  %5 = add i64 %1, 8
  call void @__store_i64(i64 %5, i64 0)
  %6 = add i64 %1, 16
  call void @__store_i64(i64 %6, i64 0)
  %7 = add i64 %1, 24
  call void @__store_i64(i64 %7, i64 0)
  %8 = add i64 %1, 32
  call void @__store_i64(i64 %8, i64 %capacity)
  ret i64 %1
}

define i64 @deque_len(i64 %dq) {
entry:
  %0 = add i64 %dq, 24
  %1 = call i64 @__load_i64(i64 %0)
  ret i64 %1
}

define i64 @deque_cap(i64 %dq) {
entry:
  %0 = add i64 %dq, 32
  %1 = call i64 @__load_i64(i64 %0)
  ret i64 %1
}

define i64 @deque_is_empty(i64 %dq) {
entry:
  %0 = call i64 @deque_len(i64 %dq)
  %1 = icmp eq i64 %0, 0
  %2 = zext i1 %1 to i64
  %3 = icmp ne i64 %2, 0
  br i1 %3, label %then0, label %else1
then0:
  br label %merge2
else1:
  br label %merge2
merge2:
  %4 = phi i64 [ 1, %then0 ], [ 0, %else1 ]
  ret i64 %4
}

define i64 @deque_push_back(i64 %dq, i64 %value) {
entry:
  %0 = call i64 @__load_i64(i64 %dq)
  %1 = add i64 %dq, 16
  %2 = call i64 @__load_i64(i64 %1)
  %3 = add i64 %dq, 24
  %4 = call i64 @__load_i64(i64 %3)
  %5 = add i64 %dq, 32
  %6 = call i64 @__load_i64(i64 %5)
  %7 = mul i64 %2, 8
  %8 = add i64 %0, %7
  call void @__store_i64(i64 %8, i64 %value)
  %9 = add i64 %2, 1
  %10 = icmp sge i64 %9, %6
  %11 = zext i1 %10 to i64
  %12 = icmp ne i64 %11, 0
  br i1 %12, label %then0, label %else1
then0:
  br label %merge2
else1:
  br label %merge2
merge2:
  %13 = phi i64 [ 0, %then0 ], [ %9, %else1 ]
  %14 = add i64 %dq, 16
  call void @__store_i64(i64 %14, i64 %13)
  %15 = add i64 %dq, 24
  %16 = add i64 %4, 1
  call void @__store_i64(i64 %15, i64 %16)
  %17 = add i64 %4, 1
  ret i64 %17
}

define i64 @deque_push_front(i64 %dq, i64 %value) {
entry:
  %0 = call i64 @__load_i64(i64 %dq)
  %1 = add i64 %dq, 8
  %2 = call i64 @__load_i64(i64 %1)
  %3 = add i64 %dq, 24
  %4 = call i64 @__load_i64(i64 %3)
  %5 = add i64 %dq, 32
  %6 = call i64 @__load_i64(i64 %5)
  %7 = icmp eq i64 %2, 0
  %8 = zext i1 %7 to i64
  %9 = icmp ne i64 %8, 0
  br i1 %9, label %then0, label %else1
then0:
  %10 = sub i64 %6, 1
  br label %merge2
else1:
  %11 = sub i64 %2, 1
  br label %merge2
merge2:
  %12 = phi i64 [ %10, %then0 ], [ %11, %else1 ]
  %13 = add i64 %dq, 8
  call void @__store_i64(i64 %13, i64 %12)
  %14 = mul i64 %12, 8
  %15 = add i64 %0, %14
  call void @__store_i64(i64 %15, i64 %value)
  %16 = add i64 %dq, 24
  %17 = add i64 %4, 1
  call void @__store_i64(i64 %16, i64 %17)
  %18 = add i64 %4, 1
  ret i64 %18
}

define i64 @deque_pop_back(i64 %dq) {
entry:
  %0 = add i64 %dq, 24
  %1 = call i64 @__load_i64(i64 %0)
  %2 = icmp eq i64 %1, 0
  %3 = zext i1 %2 to i64
  %4 = icmp ne i64 %3, 0
  br i1 %4, label %then0, label %else1
then0:
  br label %merge2
else1:
  %5 = call i64 @__load_i64(i64 %dq)
  %6 = add i64 %dq, 16
  %7 = call i64 @__load_i64(i64 %6)
  %8 = add i64 %dq, 32
  %9 = call i64 @__load_i64(i64 %8)
  %10 = icmp eq i64 %7, 0
  %11 = zext i1 %10 to i64
  %12 = icmp ne i64 %11, 0
  br i1 %12, label %then3, label %else4
then3:
  %13 = sub i64 %9, 1
  br label %merge5
else4:
  %14 = sub i64 %7, 1
  br label %merge5
merge5:
  %15 = phi i64 [ %13, %then3 ], [ %14, %else4 ]
  %16 = add i64 %dq, 16
  call void @__store_i64(i64 %16, i64 %15)
  %17 = mul i64 %15, 8
  %18 = add i64 %5, %17
  %19 = call i64 @__load_i64(i64 %18)
  %20 = add i64 %dq, 24
  %21 = sub i64 %1, 1
  call void @__store_i64(i64 %20, i64 %21)
  br label %merge2
merge2:
  %22 = phi i64 [ 0, %then0 ], [ %19, %merge5 ]
  ret i64 %22
}

define i64 @deque_pop_front(i64 %dq) {
entry:
  %0 = add i64 %dq, 24
  %1 = call i64 @__load_i64(i64 %0)
  %2 = icmp eq i64 %1, 0
  %3 = zext i1 %2 to i64
  %4 = icmp ne i64 %3, 0
  br i1 %4, label %then0, label %else1
then0:
  br label %merge2
else1:
  %5 = call i64 @__load_i64(i64 %dq)
  %6 = add i64 %dq, 8
  %7 = call i64 @__load_i64(i64 %6)
  %8 = add i64 %dq, 32
  %9 = call i64 @__load_i64(i64 %8)
  %10 = mul i64 %7, 8
  %11 = add i64 %5, %10
  %12 = call i64 @__load_i64(i64 %11)
  %13 = add i64 %7, 1
  %14 = icmp sge i64 %13, %9
  %15 = zext i1 %14 to i64
  %16 = icmp ne i64 %15, 0
  br i1 %16, label %then3, label %else4
then3:
  br label %merge5
else4:
  br label %merge5
merge5:
  %17 = phi i64 [ 0, %then3 ], [ %13, %else4 ]
  %18 = add i64 %dq, 8
  call void @__store_i64(i64 %18, i64 %17)
  %19 = add i64 %dq, 24
  %20 = sub i64 %1, 1
  call void @__store_i64(i64 %19, i64 %20)
  br label %merge2
merge2:
  %21 = phi i64 [ 0, %then0 ], [ %12, %merge5 ]
  ret i64 %21
}

define i64 @deque_front(i64 %dq) {
entry:
  %0 = add i64 %dq, 24
  %1 = call i64 @__load_i64(i64 %0)
  %2 = icmp eq i64 %1, 0
  %3 = zext i1 %2 to i64
  %4 = icmp ne i64 %3, 0
  br i1 %4, label %then0, label %else1
then0:
  br label %merge2
else1:
  %5 = call i64 @__load_i64(i64 %dq)
  %6 = add i64 %dq, 8
  %7 = call i64 @__load_i64(i64 %6)
  %8 = mul i64 %7, 8
  %9 = add i64 %5, %8
  %10 = call i64 @__load_i64(i64 %9)
  br label %merge2
merge2:
  %11 = phi i64 [ 0, %then0 ], [ %10, %else1 ]
  ret i64 %11
}

define i64 @deque_back(i64 %dq) {
entry:
  %0 = add i64 %dq, 24
  %1 = call i64 @__load_i64(i64 %0)
  %2 = icmp eq i64 %1, 0
  %3 = zext i1 %2 to i64
  %4 = icmp ne i64 %3, 0
  br i1 %4, label %then0, label %else1
then0:
  br label %merge2
else1:
  %5 = call i64 @__load_i64(i64 %dq)
  %6 = add i64 %dq, 16
  %7 = call i64 @__load_i64(i64 %6)
  %8 = add i64 %dq, 32
  %9 = call i64 @__load_i64(i64 %8)
  %10 = icmp eq i64 %7, 0
  %11 = zext i1 %10 to i64
  %12 = icmp ne i64 %11, 0
  br i1 %12, label %then3, label %else4
then3:
  %13 = sub i64 %9, 1
  br label %merge5
else4:
  %14 = sub i64 %7, 1
  br label %merge5
merge5:
  %15 = phi i64 [ %13, %then3 ], [ %14, %else4 ]
  %16 = mul i64 %15, 8
  %17 = add i64 %5, %16
  %18 = call i64 @__load_i64(i64 %17)
  br label %merge2
merge2:
  %19 = phi i64 [ 0, %then0 ], [ %18, %merge5 ]
  ret i64 %19
}

define i64 @test_push_back() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([29 x i8], [29 x i8]* @.str.0, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = call i64 @deque_create(i64 8)
  %3 = call i64 @deque_push_back(i64 %2, i64 10)
  %4 = call i64 @deque_push_back(i64 %2, i64 20)
  %5 = call i64 @deque_push_back(i64 %2, i64 30)
  %6 = call i64 @deque_len(i64 %2)
  %7 = call i64 @deque_front(i64 %2)
  %8 = call i64 @deque_back(i64 %2)
  %9 = call i32 @puts(i8* getelementptr ([20 x i8], [20 x i8]* @.str.1, i64 0, i64 0))
  %10 = sext i32 %9 to i64
  %11 = call i32 @puts(i8* getelementptr ([11 x i8], [11 x i8]* @.str.2, i64 0, i64 0))
  %12 = sext i32 %11 to i64
  %13 = call i64 @print_num(i64 %6)
  %14 = trunc i64 10 to i32
  %15 = call i32 @putchar(i32 %14)
  %16 = sext i32 %15 to i64
  %17 = call i32 @puts(i8* getelementptr ([10 x i8], [10 x i8]* @.str.3, i64 0, i64 0))
  %18 = sext i32 %17 to i64
  %19 = call i64 @print_num(i64 %7)
  %20 = trunc i64 10 to i32
  %21 = call i32 @putchar(i32 %20)
  %22 = sext i32 %21 to i64
  %23 = call i32 @puts(i8* getelementptr ([9 x i8], [9 x i8]* @.str.4, i64 0, i64 0))
  %24 = sext i32 %23 to i64
  %25 = call i64 @print_num(i64 %8)
  %26 = trunc i64 10 to i32
  %27 = call i32 @putchar(i32 %26)
  %28 = sext i32 %27 to i64
  %29 = icmp eq i64 %6, 3
  %30 = zext i1 %29 to i64
  %31 = icmp ne i64 %30, 0
  br i1 %31, label %then0, label %else1
then0:
  %32 = icmp eq i64 %7, 10
  %33 = zext i1 %32 to i64
  %34 = icmp ne i64 %33, 0
  br i1 %34, label %then3, label %else4
then3:
  %35 = icmp eq i64 %8, 30
  %36 = zext i1 %35 to i64
  %37 = icmp ne i64 %36, 0
  br i1 %37, label %then6, label %else7
then6:
  %38 = call i32 @puts(i8* getelementptr ([9 x i8], [9 x i8]* @.str.5, i64 0, i64 0))
  %39 = sext i32 %38 to i64
  %40 = trunc i64 10 to i32
  %41 = call i32 @putchar(i32 %40)
  %42 = sext i32 %41 to i64
  br label %merge8
else7:
  br label %merge8
merge8:
  %43 = phi i64 [ 1, %then6 ], [ 0, %else7 ]
  br label %merge5
else4:
  br label %merge5
merge5:
  %44 = phi i64 [ %43, %merge8 ], [ 0, %else4 ]
  br label %merge2
else1:
  br label %merge2
merge2:
  %45 = phi i64 [ %44, %merge5 ], [ 0, %else1 ]
  ret i64 %45
}

define i64 @test_push_front() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([30 x i8], [30 x i8]* @.str.6, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = call i64 @deque_create(i64 8)
  %3 = call i64 @deque_push_front(i64 %2, i64 10)
  %4 = call i64 @deque_push_front(i64 %2, i64 20)
  %5 = call i64 @deque_push_front(i64 %2, i64 30)
  %6 = call i64 @deque_len(i64 %2)
  %7 = call i64 @deque_front(i64 %2)
  %8 = call i64 @deque_back(i64 %2)
  %9 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.7, i64 0, i64 0))
  %10 = sext i32 %9 to i64
  %11 = call i32 @puts(i8* getelementptr ([11 x i8], [11 x i8]* @.str.8, i64 0, i64 0))
  %12 = sext i32 %11 to i64
  %13 = call i64 @print_num(i64 %6)
  %14 = trunc i64 10 to i32
  %15 = call i32 @putchar(i32 %14)
  %16 = sext i32 %15 to i64
  %17 = call i32 @puts(i8* getelementptr ([25 x i8], [25 x i8]* @.str.9, i64 0, i64 0))
  %18 = sext i32 %17 to i64
  %19 = call i64 @print_num(i64 %7)
  %20 = trunc i64 10 to i32
  %21 = call i32 @putchar(i32 %20)
  %22 = sext i32 %21 to i64
  %23 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.10, i64 0, i64 0))
  %24 = sext i32 %23 to i64
  %25 = call i64 @print_num(i64 %8)
  %26 = trunc i64 10 to i32
  %27 = call i32 @putchar(i32 %26)
  %28 = sext i32 %27 to i64
  %29 = icmp eq i64 %6, 3
  %30 = zext i1 %29 to i64
  %31 = icmp ne i64 %30, 0
  br i1 %31, label %then0, label %else1
then0:
  %32 = icmp eq i64 %7, 30
  %33 = zext i1 %32 to i64
  %34 = icmp ne i64 %33, 0
  br i1 %34, label %then3, label %else4
then3:
  %35 = icmp eq i64 %8, 10
  %36 = zext i1 %35 to i64
  %37 = icmp ne i64 %36, 0
  br i1 %37, label %then6, label %else7
then6:
  %38 = call i32 @puts(i8* getelementptr ([9 x i8], [9 x i8]* @.str.11, i64 0, i64 0))
  %39 = sext i32 %38 to i64
  %40 = trunc i64 10 to i32
  %41 = call i32 @putchar(i32 %40)
  %42 = sext i32 %41 to i64
  br label %merge8
else7:
  br label %merge8
merge8:
  %43 = phi i64 [ 1, %then6 ], [ 0, %else7 ]
  br label %merge5
else4:
  br label %merge5
merge5:
  %44 = phi i64 [ %43, %merge8 ], [ 0, %else4 ]
  br label %merge2
else1:
  br label %merge2
merge2:
  %45 = phi i64 [ %44, %merge5 ], [ 0, %else1 ]
  ret i64 %45
}

define i64 @test_pop_operations() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([23 x i8], [23 x i8]* @.str.12, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = call i64 @deque_create(i64 8)
  %3 = call i64 @deque_push_back(i64 %2, i64 1)
  %4 = call i64 @deque_push_back(i64 %2, i64 2)
  %5 = call i64 @deque_push_back(i64 %2, i64 3)
  %6 = call i64 @deque_push_back(i64 %2, i64 4)
  %7 = call i64 @deque_pop_front(i64 %2)
  %8 = call i64 @deque_pop_back(i64 %2)
  %9 = call i32 @puts(i8* getelementptr ([17 x i8], [17 x i8]* @.str.13, i64 0, i64 0))
  %10 = sext i32 %9 to i64
  %11 = call i32 @puts(i8* getelementptr ([14 x i8], [14 x i8]* @.str.14, i64 0, i64 0))
  %12 = sext i32 %11 to i64
  %13 = call i64 @print_num(i64 %7)
  %14 = trunc i64 10 to i32
  %15 = call i32 @putchar(i32 %14)
  %16 = sext i32 %15 to i64
  %17 = call i32 @puts(i8* getelementptr ([13 x i8], [13 x i8]* @.str.15, i64 0, i64 0))
  %18 = sext i32 %17 to i64
  %19 = call i64 @print_num(i64 %8)
  %20 = trunc i64 10 to i32
  %21 = call i32 @putchar(i32 %20)
  %22 = sext i32 %21 to i64
  %23 = call i64 @deque_len(i64 %2)
  %24 = call i32 @puts(i8* getelementptr ([21 x i8], [21 x i8]* @.str.16, i64 0, i64 0))
  %25 = sext i32 %24 to i64
  %26 = call i64 @print_num(i64 %23)
  %27 = trunc i64 10 to i32
  %28 = call i32 @putchar(i32 %27)
  %29 = sext i32 %28 to i64
  %30 = icmp eq i64 %7, 1
  %31 = zext i1 %30 to i64
  %32 = icmp ne i64 %31, 0
  br i1 %32, label %then0, label %else1
then0:
  %33 = icmp eq i64 %8, 4
  %34 = zext i1 %33 to i64
  %35 = icmp ne i64 %34, 0
  br i1 %35, label %then3, label %else4
then3:
  %36 = icmp eq i64 %23, 2
  %37 = zext i1 %36 to i64
  %38 = icmp ne i64 %37, 0
  br i1 %38, label %then6, label %else7
then6:
  %39 = call i32 @puts(i8* getelementptr ([9 x i8], [9 x i8]* @.str.17, i64 0, i64 0))
  %40 = sext i32 %39 to i64
  %41 = trunc i64 10 to i32
  %42 = call i32 @putchar(i32 %41)
  %43 = sext i32 %42 to i64
  br label %merge8
else7:
  br label %merge8
merge8:
  %44 = phi i64 [ 1, %then6 ], [ 0, %else7 ]
  br label %merge5
else4:
  br label %merge5
merge5:
  %45 = phi i64 [ %44, %merge8 ], [ 0, %else4 ]
  br label %merge2
else1:
  br label %merge2
merge2:
  %46 = phi i64 [ %45, %merge5 ], [ 0, %else1 ]
  ret i64 %46
}

define i64 @test_wrap_around() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([29 x i8], [29 x i8]* @.str.18, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = call i64 @deque_create(i64 4)
  %3 = call i64 @deque_push_back(i64 %2, i64 1)
  %4 = call i64 @deque_push_back(i64 %2, i64 2)
  %5 = call i64 @deque_pop_front(i64 %2)
  %6 = call i64 @deque_pop_front(i64 %2)
  %7 = call i64 @deque_push_back(i64 %2, i64 10)
  %8 = call i64 @deque_push_back(i64 %2, i64 20)
  %9 = call i64 @deque_push_back(i64 %2, i64 30)
  %10 = call i64 @deque_front(i64 %2)
  %11 = call i64 @deque_back(i64 %2)
  %12 = call i64 @deque_len(i64 %2)
  %13 = call i32 @puts(i8* getelementptr ([31 x i8], [31 x i8]* @.str.19, i64 0, i64 0))
  %14 = sext i32 %13 to i64
  %15 = call i32 @puts(i8* getelementptr ([11 x i8], [11 x i8]* @.str.20, i64 0, i64 0))
  %16 = sext i32 %15 to i64
  %17 = call i64 @print_num(i64 %12)
  %18 = trunc i64 10 to i32
  %19 = call i32 @putchar(i32 %18)
  %20 = sext i32 %19 to i64
  %21 = call i32 @puts(i8* getelementptr ([10 x i8], [10 x i8]* @.str.21, i64 0, i64 0))
  %22 = sext i32 %21 to i64
  %23 = call i64 @print_num(i64 %10)
  %24 = trunc i64 10 to i32
  %25 = call i32 @putchar(i32 %24)
  %26 = sext i32 %25 to i64
  %27 = call i32 @puts(i8* getelementptr ([9 x i8], [9 x i8]* @.str.22, i64 0, i64 0))
  %28 = sext i32 %27 to i64
  %29 = call i64 @print_num(i64 %11)
  %30 = trunc i64 10 to i32
  %31 = call i32 @putchar(i32 %30)
  %32 = sext i32 %31 to i64
  %33 = icmp eq i64 %12, 3
  %34 = zext i1 %33 to i64
  %35 = icmp ne i64 %34, 0
  br i1 %35, label %then0, label %else1
then0:
  %36 = icmp eq i64 %10, 10
  %37 = zext i1 %36 to i64
  %38 = icmp ne i64 %37, 0
  br i1 %38, label %then3, label %else4
then3:
  %39 = icmp eq i64 %11, 30
  %40 = zext i1 %39 to i64
  %41 = icmp ne i64 %40, 0
  br i1 %41, label %then6, label %else7
then6:
  %42 = call i32 @puts(i8* getelementptr ([9 x i8], [9 x i8]* @.str.23, i64 0, i64 0))
  %43 = sext i32 %42 to i64
  %44 = trunc i64 10 to i32
  %45 = call i32 @putchar(i32 %44)
  %46 = sext i32 %45 to i64
  br label %merge8
else7:
  br label %merge8
merge8:
  %47 = phi i64 [ 1, %then6 ], [ 0, %else7 ]
  br label %merge5
else4:
  br label %merge5
merge5:
  %48 = phi i64 [ %47, %merge8 ], [ 0, %else4 ]
  br label %merge2
else1:
  br label %merge2
merge2:
  %49 = phi i64 [ %48, %merge5 ], [ 0, %else1 ]
  ret i64 %49
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([30 x i8], [30 x i8]* @.str.24, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = trunc i64 10 to i32
  %3 = call i32 @putchar(i32 %2)
  %4 = sext i32 %3 to i64
  %5 = call i64 @test_push_back()
  %6 = call i64 @test_push_front()
  %7 = call i64 @test_pop_operations()
  %8 = call i64 @test_wrap_around()
  %9 = trunc i64 10 to i32
  %10 = call i32 @putchar(i32 %9)
  %11 = sext i32 %10 to i64
  %12 = icmp eq i64 %5, 1
  %13 = zext i1 %12 to i64
  %14 = icmp ne i64 %13, 0
  br i1 %14, label %then0, label %else1
then0:
  %15 = icmp eq i64 %6, 1
  %16 = zext i1 %15 to i64
  %17 = icmp ne i64 %16, 0
  br i1 %17, label %then3, label %else4
then3:
  %18 = icmp eq i64 %7, 1
  %19 = zext i1 %18 to i64
  %20 = icmp ne i64 %19, 0
  br i1 %20, label %then6, label %else7
then6:
  %21 = icmp eq i64 %8, 1
  %22 = zext i1 %21 to i64
  %23 = icmp ne i64 %22, 0
  br i1 %23, label %then9, label %else10
then9:
  %24 = call i32 @puts(i8* getelementptr ([31 x i8], [31 x i8]* @.str.25, i64 0, i64 0))
  %25 = sext i32 %24 to i64
  br label %merge11
else10:
  %26 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.26, i64 0, i64 0))
  %27 = sext i32 %26 to i64
  br label %merge11
merge11:
  %28 = phi i64 [ 0, %then9 ], [ 1, %else10 ]
  br label %merge8
else7:
  %29 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.27, i64 0, i64 0))
  %30 = sext i32 %29 to i64
  br label %merge8
merge8:
  %31 = phi i64 [ %28, %merge11 ], [ 1, %else7 ]
  br label %merge5
else4:
  %32 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.28, i64 0, i64 0))
  %33 = sext i32 %32 to i64
  br label %merge5
merge5:
  %34 = phi i64 [ %31, %merge8 ], [ 1, %else4 ]
  br label %merge2
else1:
  %35 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.29, i64 0, i64 0))
  %36 = sext i32 %35 to i64
  br label %merge2
merge2:
  %37 = phi i64 [ %34, %merge5 ], [ 1, %else1 ]
  ret i64 %37
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

; ModuleID = 'deque_test'
source_filename = "<vais>"

declare i64 @fgets(i64, i64, i64)
declare i32 @puts(i8*)
declare i64 @ftell(i64)
declare i32 @fclose(i64)
declare i32 @printf(i8*)
declare i64 @fread(i64, i64, i64, i64)
declare i32 @usleep(i64)
declare i32 @strcmp(i8*, i8*)
declare i64 @feof(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @fseek(i64, i64, i64)
declare i64 @fopen(i8*, i8*)
declare i64 @memcpy(i64, i64, i64)
declare i32 @putchar(i32)
declare i64 @fputc(i64, i64)
declare i64 @fputs(i8*, i64)
declare void @free(i64)
declare void @exit(i32)
declare i64 @fgetc(i64)
declare i64 @fflush(i64)
declare i64 @strlen(i64)
declare i32 @sched_yield()
declare i64 @malloc(i64)
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

define i64 @deque_create(i64 %capacity) {
entry:
  %0 = call i8* @malloc(i64 40)
  %1 = ptrtoint i8* %0 to i64
  %dq.2 = alloca i64
  store i64 %1, i64* %dq.2
  %3 = mul i64 %capacity, 8
  %4 = call i8* @malloc(i64 %3)
  %5 = ptrtoint i8* %4 to i64
  %data.6 = alloca i64
  store i64 %5, i64* %data.6
  %7 = load i64, i64* %dq.2
  %8 = load i64, i64* %data.6
  call void @__store_i64(i64 %7, i64 %8)
  %9 = load i64, i64* %dq.2
  %10 = add i64 %9, 8
  call void @__store_i64(i64 %10, i64 0)
  %11 = load i64, i64* %dq.2
  %12 = add i64 %11, 16
  call void @__store_i64(i64 %12, i64 0)
  %13 = load i64, i64* %dq.2
  %14 = add i64 %13, 24
  call void @__store_i64(i64 %14, i64 0)
  %15 = load i64, i64* %dq.2
  %16 = add i64 %15, 32
  call void @__store_i64(i64 %16, i64 %capacity)
  %17 = load i64, i64* %dq.2
  ret i64 %17
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
  %data.1 = alloca i64
  store i64 %0, i64* %data.1
  %2 = add i64 %dq, 16
  %3 = call i64 @__load_i64(i64 %2)
  %tail.4 = alloca i64
  store i64 %3, i64* %tail.4
  %5 = add i64 %dq, 24
  %6 = call i64 @__load_i64(i64 %5)
  %len.7 = alloca i64
  store i64 %6, i64* %len.7
  %8 = add i64 %dq, 32
  %9 = call i64 @__load_i64(i64 %8)
  %cap.10 = alloca i64
  store i64 %9, i64* %cap.10
  %11 = load i64, i64* %data.1
  %12 = load i64, i64* %tail.4
  %13 = mul i64 %12, 8
  %14 = add i64 %11, %13
  call void @__store_i64(i64 %14, i64 %value)
  %15 = load i64, i64* %tail.4
  %16 = add i64 %15, 1
  %new_tail.17 = alloca i64
  store i64 %16, i64* %new_tail.17
  %18 = load i64, i64* %new_tail.17
  %19 = load i64, i64* %cap.10
  %20 = icmp sge i64 %18, %19
  %21 = zext i1 %20 to i64
  %22 = icmp ne i64 %21, 0
  br i1 %22, label %then0, label %else1
then0:
  br label %merge2
else1:
  %23 = load i64, i64* %new_tail.17
  br label %merge2
merge2:
  %24 = phi i64 [ 0, %then0 ], [ %23, %else1 ]
  %wrapped_tail.25 = alloca i64
  store i64 %24, i64* %wrapped_tail.25
  %26 = add i64 %dq, 16
  %27 = load i64, i64* %wrapped_tail.25
  call void @__store_i64(i64 %26, i64 %27)
  %28 = add i64 %dq, 24
  %29 = load i64, i64* %len.7
  %30 = add i64 %29, 1
  call void @__store_i64(i64 %28, i64 %30)
  %31 = load i64, i64* %len.7
  %32 = add i64 %31, 1
  ret i64 %32
}

define i64 @deque_push_front(i64 %dq, i64 %value) {
entry:
  %0 = call i64 @__load_i64(i64 %dq)
  %data.1 = alloca i64
  store i64 %0, i64* %data.1
  %2 = add i64 %dq, 8
  %3 = call i64 @__load_i64(i64 %2)
  %head.4 = alloca i64
  store i64 %3, i64* %head.4
  %5 = add i64 %dq, 24
  %6 = call i64 @__load_i64(i64 %5)
  %len.7 = alloca i64
  store i64 %6, i64* %len.7
  %8 = add i64 %dq, 32
  %9 = call i64 @__load_i64(i64 %8)
  %cap.10 = alloca i64
  store i64 %9, i64* %cap.10
  %11 = load i64, i64* %head.4
  %12 = icmp eq i64 %11, 0
  %13 = zext i1 %12 to i64
  %14 = icmp ne i64 %13, 0
  br i1 %14, label %then0, label %else1
then0:
  %15 = load i64, i64* %cap.10
  %16 = sub i64 %15, 1
  br label %merge2
else1:
  %17 = load i64, i64* %head.4
  %18 = sub i64 %17, 1
  br label %merge2
merge2:
  %19 = phi i64 [ %16, %then0 ], [ %18, %else1 ]
  %new_head.20 = alloca i64
  store i64 %19, i64* %new_head.20
  %21 = add i64 %dq, 8
  %22 = load i64, i64* %new_head.20
  call void @__store_i64(i64 %21, i64 %22)
  %23 = load i64, i64* %data.1
  %24 = load i64, i64* %new_head.20
  %25 = mul i64 %24, 8
  %26 = add i64 %23, %25
  call void @__store_i64(i64 %26, i64 %value)
  %27 = add i64 %dq, 24
  %28 = load i64, i64* %len.7
  %29 = add i64 %28, 1
  call void @__store_i64(i64 %27, i64 %29)
  %30 = load i64, i64* %len.7
  %31 = add i64 %30, 1
  ret i64 %31
}

define i64 @deque_pop_back(i64 %dq) {
entry:
  %0 = add i64 %dq, 24
  %1 = call i64 @__load_i64(i64 %0)
  %len.2 = alloca i64
  store i64 %1, i64* %len.2
  %3 = load i64, i64* %len.2
  %4 = icmp eq i64 %3, 0
  %5 = zext i1 %4 to i64
  %6 = icmp ne i64 %5, 0
  br i1 %6, label %then0, label %else1
then0:
  br label %merge2
else1:
  %7 = call i64 @__load_i64(i64 %dq)
  %data.8 = alloca i64
  store i64 %7, i64* %data.8
  %9 = add i64 %dq, 16
  %10 = call i64 @__load_i64(i64 %9)
  %tail.11 = alloca i64
  store i64 %10, i64* %tail.11
  %12 = add i64 %dq, 32
  %13 = call i64 @__load_i64(i64 %12)
  %cap.14 = alloca i64
  store i64 %13, i64* %cap.14
  %15 = load i64, i64* %tail.11
  %16 = icmp eq i64 %15, 0
  %17 = zext i1 %16 to i64
  %18 = icmp ne i64 %17, 0
  br i1 %18, label %then3, label %else4
then3:
  %19 = load i64, i64* %cap.14
  %20 = sub i64 %19, 1
  br label %merge5
else4:
  %21 = load i64, i64* %tail.11
  %22 = sub i64 %21, 1
  br label %merge5
merge5:
  %23 = phi i64 [ %20, %then3 ], [ %22, %else4 ]
  %new_tail.24 = alloca i64
  store i64 %23, i64* %new_tail.24
  %25 = add i64 %dq, 16
  %26 = load i64, i64* %new_tail.24
  call void @__store_i64(i64 %25, i64 %26)
  %27 = load i64, i64* %data.8
  %28 = load i64, i64* %new_tail.24
  %29 = mul i64 %28, 8
  %30 = add i64 %27, %29
  %31 = call i64 @__load_i64(i64 %30)
  %value.32 = alloca i64
  store i64 %31, i64* %value.32
  %33 = add i64 %dq, 24
  %34 = load i64, i64* %len.2
  %35 = sub i64 %34, 1
  call void @__store_i64(i64 %33, i64 %35)
  %36 = load i64, i64* %value.32
  br label %merge2
merge2:
  %37 = phi i64 [ 0, %then0 ], [ %36, %else1 ]
  ret i64 %37
}

define i64 @deque_pop_front(i64 %dq) {
entry:
  %0 = add i64 %dq, 24
  %1 = call i64 @__load_i64(i64 %0)
  %len.2 = alloca i64
  store i64 %1, i64* %len.2
  %3 = load i64, i64* %len.2
  %4 = icmp eq i64 %3, 0
  %5 = zext i1 %4 to i64
  %6 = icmp ne i64 %5, 0
  br i1 %6, label %then0, label %else1
then0:
  br label %merge2
else1:
  %7 = call i64 @__load_i64(i64 %dq)
  %data.8 = alloca i64
  store i64 %7, i64* %data.8
  %9 = add i64 %dq, 8
  %10 = call i64 @__load_i64(i64 %9)
  %head.11 = alloca i64
  store i64 %10, i64* %head.11
  %12 = add i64 %dq, 32
  %13 = call i64 @__load_i64(i64 %12)
  %cap.14 = alloca i64
  store i64 %13, i64* %cap.14
  %15 = load i64, i64* %data.8
  %16 = load i64, i64* %head.11
  %17 = mul i64 %16, 8
  %18 = add i64 %15, %17
  %19 = call i64 @__load_i64(i64 %18)
  %value.20 = alloca i64
  store i64 %19, i64* %value.20
  %21 = load i64, i64* %head.11
  %22 = add i64 %21, 1
  %new_head.23 = alloca i64
  store i64 %22, i64* %new_head.23
  %24 = load i64, i64* %new_head.23
  %25 = load i64, i64* %cap.14
  %26 = icmp sge i64 %24, %25
  %27 = zext i1 %26 to i64
  %28 = icmp ne i64 %27, 0
  br i1 %28, label %then3, label %else4
then3:
  br label %merge5
else4:
  %29 = load i64, i64* %new_head.23
  br label %merge5
merge5:
  %30 = phi i64 [ 0, %then3 ], [ %29, %else4 ]
  %wrapped_head.31 = alloca i64
  store i64 %30, i64* %wrapped_head.31
  %32 = add i64 %dq, 8
  %33 = load i64, i64* %wrapped_head.31
  call void @__store_i64(i64 %32, i64 %33)
  %34 = add i64 %dq, 24
  %35 = load i64, i64* %len.2
  %36 = sub i64 %35, 1
  call void @__store_i64(i64 %34, i64 %36)
  %37 = load i64, i64* %value.20
  br label %merge2
merge2:
  %38 = phi i64 [ 0, %then0 ], [ %37, %else1 ]
  ret i64 %38
}

define i64 @deque_front(i64 %dq) {
entry:
  %0 = add i64 %dq, 24
  %1 = call i64 @__load_i64(i64 %0)
  %len.2 = alloca i64
  store i64 %1, i64* %len.2
  %3 = load i64, i64* %len.2
  %4 = icmp eq i64 %3, 0
  %5 = zext i1 %4 to i64
  %6 = icmp ne i64 %5, 0
  br i1 %6, label %then0, label %else1
then0:
  br label %merge2
else1:
  %7 = call i64 @__load_i64(i64 %dq)
  %data.8 = alloca i64
  store i64 %7, i64* %data.8
  %9 = add i64 %dq, 8
  %10 = call i64 @__load_i64(i64 %9)
  %head.11 = alloca i64
  store i64 %10, i64* %head.11
  %12 = load i64, i64* %data.8
  %13 = load i64, i64* %head.11
  %14 = mul i64 %13, 8
  %15 = add i64 %12, %14
  %16 = call i64 @__load_i64(i64 %15)
  br label %merge2
merge2:
  %17 = phi i64 [ 0, %then0 ], [ %16, %else1 ]
  ret i64 %17
}

define i64 @deque_back(i64 %dq) {
entry:
  %0 = add i64 %dq, 24
  %1 = call i64 @__load_i64(i64 %0)
  %len.2 = alloca i64
  store i64 %1, i64* %len.2
  %3 = load i64, i64* %len.2
  %4 = icmp eq i64 %3, 0
  %5 = zext i1 %4 to i64
  %6 = icmp ne i64 %5, 0
  br i1 %6, label %then0, label %else1
then0:
  br label %merge2
else1:
  %7 = call i64 @__load_i64(i64 %dq)
  %data.8 = alloca i64
  store i64 %7, i64* %data.8
  %9 = add i64 %dq, 16
  %10 = call i64 @__load_i64(i64 %9)
  %tail.11 = alloca i64
  store i64 %10, i64* %tail.11
  %12 = add i64 %dq, 32
  %13 = call i64 @__load_i64(i64 %12)
  %cap.14 = alloca i64
  store i64 %13, i64* %cap.14
  %15 = load i64, i64* %tail.11
  %16 = icmp eq i64 %15, 0
  %17 = zext i1 %16 to i64
  %18 = icmp ne i64 %17, 0
  br i1 %18, label %then3, label %else4
then3:
  %19 = load i64, i64* %cap.14
  %20 = sub i64 %19, 1
  br label %merge5
else4:
  %21 = load i64, i64* %tail.11
  %22 = sub i64 %21, 1
  br label %merge5
merge5:
  %23 = phi i64 [ %20, %then3 ], [ %22, %else4 ]
  %back_idx.24 = alloca i64
  store i64 %23, i64* %back_idx.24
  %25 = load i64, i64* %data.8
  %26 = load i64, i64* %back_idx.24
  %27 = mul i64 %26, 8
  %28 = add i64 %25, %27
  %29 = call i64 @__load_i64(i64 %28)
  br label %merge2
merge2:
  %30 = phi i64 [ 0, %then0 ], [ %29, %else1 ]
  ret i64 %30
}

define i64 @test_push_back() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([29 x i8], [29 x i8]* @.str.0, i64 0, i64 0))
  %1 = call i64 @deque_create(i64 8)
  %dq.2 = alloca i64
  store i64 %1, i64* %dq.2
  %3 = load i64, i64* %dq.2
  %4 = call i64 @deque_push_back(i64 %3, i64 10)
  %5 = load i64, i64* %dq.2
  %6 = call i64 @deque_push_back(i64 %5, i64 20)
  %7 = load i64, i64* %dq.2
  %8 = call i64 @deque_push_back(i64 %7, i64 30)
  %9 = load i64, i64* %dq.2
  %10 = call i64 @deque_len(i64 %9)
  %len.11 = alloca i64
  store i64 %10, i64* %len.11
  %12 = load i64, i64* %dq.2
  %13 = call i64 @deque_front(i64 %12)
  %front.14 = alloca i64
  store i64 %13, i64* %front.14
  %15 = load i64, i64* %dq.2
  %16 = call i64 @deque_back(i64 %15)
  %back.17 = alloca i64
  store i64 %16, i64* %back.17
  %18 = call i32 @puts(i8* getelementptr ([20 x i8], [20 x i8]* @.str.1, i64 0, i64 0))
  %19 = call i32 @puts(i8* getelementptr ([11 x i8], [11 x i8]* @.str.2, i64 0, i64 0))
  %20 = load i64, i64* %len.11
  %21 = call i64 @print_num(i64 %20)
  %22 = trunc i64 10 to i32
  %23 = call i32 @putchar(i32 %22)
  %24 = call i32 @puts(i8* getelementptr ([10 x i8], [10 x i8]* @.str.3, i64 0, i64 0))
  %25 = load i64, i64* %front.14
  %26 = call i64 @print_num(i64 %25)
  %27 = trunc i64 10 to i32
  %28 = call i32 @putchar(i32 %27)
  %29 = call i32 @puts(i8* getelementptr ([9 x i8], [9 x i8]* @.str.4, i64 0, i64 0))
  %30 = load i64, i64* %back.17
  %31 = call i64 @print_num(i64 %30)
  %32 = trunc i64 10 to i32
  %33 = call i32 @putchar(i32 %32)
  %34 = load i64, i64* %len.11
  %35 = icmp eq i64 %34, 3
  %36 = zext i1 %35 to i64
  %37 = icmp ne i64 %36, 0
  br i1 %37, label %then0, label %else1
then0:
  %38 = load i64, i64* %front.14
  %39 = icmp eq i64 %38, 10
  %40 = zext i1 %39 to i64
  %41 = icmp ne i64 %40, 0
  br i1 %41, label %then3, label %else4
then3:
  %42 = load i64, i64* %back.17
  %43 = icmp eq i64 %42, 30
  %44 = zext i1 %43 to i64
  %45 = icmp ne i64 %44, 0
  br i1 %45, label %then6, label %else7
then6:
  %46 = call i32 @puts(i8* getelementptr ([9 x i8], [9 x i8]* @.str.5, i64 0, i64 0))
  %47 = trunc i64 10 to i32
  %48 = call i32 @putchar(i32 %47)
  br label %merge8
else7:
  br label %merge8
merge8:
  %49 = phi i64 [ 1, %then6 ], [ 0, %else7 ]
  br label %merge5
else4:
  br label %merge5
merge5:
  %50 = phi i64 [ %49, %then3 ], [ 0, %else4 ]
  br label %merge2
else1:
  br label %merge2
merge2:
  %51 = phi i64 [ %50, %then0 ], [ 0, %else1 ]
  ret i64 %51
}

define i64 @test_push_front() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([30 x i8], [30 x i8]* @.str.6, i64 0, i64 0))
  %1 = call i64 @deque_create(i64 8)
  %dq.2 = alloca i64
  store i64 %1, i64* %dq.2
  %3 = load i64, i64* %dq.2
  %4 = call i64 @deque_push_front(i64 %3, i64 10)
  %5 = load i64, i64* %dq.2
  %6 = call i64 @deque_push_front(i64 %5, i64 20)
  %7 = load i64, i64* %dq.2
  %8 = call i64 @deque_push_front(i64 %7, i64 30)
  %9 = load i64, i64* %dq.2
  %10 = call i64 @deque_len(i64 %9)
  %len.11 = alloca i64
  store i64 %10, i64* %len.11
  %12 = load i64, i64* %dq.2
  %13 = call i64 @deque_front(i64 %12)
  %front.14 = alloca i64
  store i64 %13, i64* %front.14
  %15 = load i64, i64* %dq.2
  %16 = call i64 @deque_back(i64 %15)
  %back.17 = alloca i64
  store i64 %16, i64* %back.17
  %18 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.7, i64 0, i64 0))
  %19 = call i32 @puts(i8* getelementptr ([11 x i8], [11 x i8]* @.str.8, i64 0, i64 0))
  %20 = load i64, i64* %len.11
  %21 = call i64 @print_num(i64 %20)
  %22 = trunc i64 10 to i32
  %23 = call i32 @putchar(i32 %22)
  %24 = call i32 @puts(i8* getelementptr ([25 x i8], [25 x i8]* @.str.9, i64 0, i64 0))
  %25 = load i64, i64* %front.14
  %26 = call i64 @print_num(i64 %25)
  %27 = trunc i64 10 to i32
  %28 = call i32 @putchar(i32 %27)
  %29 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.10, i64 0, i64 0))
  %30 = load i64, i64* %back.17
  %31 = call i64 @print_num(i64 %30)
  %32 = trunc i64 10 to i32
  %33 = call i32 @putchar(i32 %32)
  %34 = load i64, i64* %len.11
  %35 = icmp eq i64 %34, 3
  %36 = zext i1 %35 to i64
  %37 = icmp ne i64 %36, 0
  br i1 %37, label %then0, label %else1
then0:
  %38 = load i64, i64* %front.14
  %39 = icmp eq i64 %38, 30
  %40 = zext i1 %39 to i64
  %41 = icmp ne i64 %40, 0
  br i1 %41, label %then3, label %else4
then3:
  %42 = load i64, i64* %back.17
  %43 = icmp eq i64 %42, 10
  %44 = zext i1 %43 to i64
  %45 = icmp ne i64 %44, 0
  br i1 %45, label %then6, label %else7
then6:
  %46 = call i32 @puts(i8* getelementptr ([9 x i8], [9 x i8]* @.str.11, i64 0, i64 0))
  %47 = trunc i64 10 to i32
  %48 = call i32 @putchar(i32 %47)
  br label %merge8
else7:
  br label %merge8
merge8:
  %49 = phi i64 [ 1, %then6 ], [ 0, %else7 ]
  br label %merge5
else4:
  br label %merge5
merge5:
  %50 = phi i64 [ %49, %then3 ], [ 0, %else4 ]
  br label %merge2
else1:
  br label %merge2
merge2:
  %51 = phi i64 [ %50, %then0 ], [ 0, %else1 ]
  ret i64 %51
}

define i64 @test_pop_operations() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([23 x i8], [23 x i8]* @.str.12, i64 0, i64 0))
  %1 = call i64 @deque_create(i64 8)
  %dq.2 = alloca i64
  store i64 %1, i64* %dq.2
  %3 = load i64, i64* %dq.2
  %4 = call i64 @deque_push_back(i64 %3, i64 1)
  %5 = load i64, i64* %dq.2
  %6 = call i64 @deque_push_back(i64 %5, i64 2)
  %7 = load i64, i64* %dq.2
  %8 = call i64 @deque_push_back(i64 %7, i64 3)
  %9 = load i64, i64* %dq.2
  %10 = call i64 @deque_push_back(i64 %9, i64 4)
  %11 = load i64, i64* %dq.2
  %12 = call i64 @deque_pop_front(i64 %11)
  %pop_front.13 = alloca i64
  store i64 %12, i64* %pop_front.13
  %14 = load i64, i64* %dq.2
  %15 = call i64 @deque_pop_back(i64 %14)
  %pop_back.16 = alloca i64
  store i64 %15, i64* %pop_back.16
  %17 = call i32 @puts(i8* getelementptr ([17 x i8], [17 x i8]* @.str.13, i64 0, i64 0))
  %18 = call i32 @puts(i8* getelementptr ([14 x i8], [14 x i8]* @.str.14, i64 0, i64 0))
  %19 = load i64, i64* %pop_front.13
  %20 = call i64 @print_num(i64 %19)
  %21 = trunc i64 10 to i32
  %22 = call i32 @putchar(i32 %21)
  %23 = call i32 @puts(i8* getelementptr ([13 x i8], [13 x i8]* @.str.15, i64 0, i64 0))
  %24 = load i64, i64* %pop_back.16
  %25 = call i64 @print_num(i64 %24)
  %26 = trunc i64 10 to i32
  %27 = call i32 @putchar(i32 %26)
  %28 = load i64, i64* %dq.2
  %29 = call i64 @deque_len(i64 %28)
  %len.30 = alloca i64
  store i64 %29, i64* %len.30
  %31 = call i32 @puts(i8* getelementptr ([21 x i8], [21 x i8]* @.str.16, i64 0, i64 0))
  %32 = load i64, i64* %len.30
  %33 = call i64 @print_num(i64 %32)
  %34 = trunc i64 10 to i32
  %35 = call i32 @putchar(i32 %34)
  %36 = load i64, i64* %pop_front.13
  %37 = icmp eq i64 %36, 1
  %38 = zext i1 %37 to i64
  %39 = icmp ne i64 %38, 0
  br i1 %39, label %then0, label %else1
then0:
  %40 = load i64, i64* %pop_back.16
  %41 = icmp eq i64 %40, 4
  %42 = zext i1 %41 to i64
  %43 = icmp ne i64 %42, 0
  br i1 %43, label %then3, label %else4
then3:
  %44 = load i64, i64* %len.30
  %45 = icmp eq i64 %44, 2
  %46 = zext i1 %45 to i64
  %47 = icmp ne i64 %46, 0
  br i1 %47, label %then6, label %else7
then6:
  %48 = call i32 @puts(i8* getelementptr ([9 x i8], [9 x i8]* @.str.17, i64 0, i64 0))
  %49 = trunc i64 10 to i32
  %50 = call i32 @putchar(i32 %49)
  br label %merge8
else7:
  br label %merge8
merge8:
  %51 = phi i64 [ 1, %then6 ], [ 0, %else7 ]
  br label %merge5
else4:
  br label %merge5
merge5:
  %52 = phi i64 [ %51, %then3 ], [ 0, %else4 ]
  br label %merge2
else1:
  br label %merge2
merge2:
  %53 = phi i64 [ %52, %then0 ], [ 0, %else1 ]
  ret i64 %53
}

define i64 @test_wrap_around() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([29 x i8], [29 x i8]* @.str.18, i64 0, i64 0))
  %1 = call i64 @deque_create(i64 4)
  %dq.2 = alloca i64
  store i64 %1, i64* %dq.2
  %3 = load i64, i64* %dq.2
  %4 = call i64 @deque_push_back(i64 %3, i64 1)
  %5 = load i64, i64* %dq.2
  %6 = call i64 @deque_push_back(i64 %5, i64 2)
  %7 = load i64, i64* %dq.2
  %8 = call i64 @deque_pop_front(i64 %7)
  %9 = load i64, i64* %dq.2
  %10 = call i64 @deque_pop_front(i64 %9)
  %11 = load i64, i64* %dq.2
  %12 = call i64 @deque_push_back(i64 %11, i64 10)
  %13 = load i64, i64* %dq.2
  %14 = call i64 @deque_push_back(i64 %13, i64 20)
  %15 = load i64, i64* %dq.2
  %16 = call i64 @deque_push_back(i64 %15, i64 30)
  %17 = load i64, i64* %dq.2
  %18 = call i64 @deque_front(i64 %17)
  %front.19 = alloca i64
  store i64 %18, i64* %front.19
  %20 = load i64, i64* %dq.2
  %21 = call i64 @deque_back(i64 %20)
  %back.22 = alloca i64
  store i64 %21, i64* %back.22
  %23 = load i64, i64* %dq.2
  %24 = call i64 @deque_len(i64 %23)
  %len.25 = alloca i64
  store i64 %24, i64* %len.25
  %26 = call i32 @puts(i8* getelementptr ([31 x i8], [31 x i8]* @.str.19, i64 0, i64 0))
  %27 = call i32 @puts(i8* getelementptr ([11 x i8], [11 x i8]* @.str.20, i64 0, i64 0))
  %28 = load i64, i64* %len.25
  %29 = call i64 @print_num(i64 %28)
  %30 = trunc i64 10 to i32
  %31 = call i32 @putchar(i32 %30)
  %32 = call i32 @puts(i8* getelementptr ([10 x i8], [10 x i8]* @.str.21, i64 0, i64 0))
  %33 = load i64, i64* %front.19
  %34 = call i64 @print_num(i64 %33)
  %35 = trunc i64 10 to i32
  %36 = call i32 @putchar(i32 %35)
  %37 = call i32 @puts(i8* getelementptr ([9 x i8], [9 x i8]* @.str.22, i64 0, i64 0))
  %38 = load i64, i64* %back.22
  %39 = call i64 @print_num(i64 %38)
  %40 = trunc i64 10 to i32
  %41 = call i32 @putchar(i32 %40)
  %42 = load i64, i64* %len.25
  %43 = icmp eq i64 %42, 3
  %44 = zext i1 %43 to i64
  %45 = icmp ne i64 %44, 0
  br i1 %45, label %then0, label %else1
then0:
  %46 = load i64, i64* %front.19
  %47 = icmp eq i64 %46, 10
  %48 = zext i1 %47 to i64
  %49 = icmp ne i64 %48, 0
  br i1 %49, label %then3, label %else4
then3:
  %50 = load i64, i64* %back.22
  %51 = icmp eq i64 %50, 30
  %52 = zext i1 %51 to i64
  %53 = icmp ne i64 %52, 0
  br i1 %53, label %then6, label %else7
then6:
  %54 = call i32 @puts(i8* getelementptr ([9 x i8], [9 x i8]* @.str.23, i64 0, i64 0))
  %55 = trunc i64 10 to i32
  %56 = call i32 @putchar(i32 %55)
  br label %merge8
else7:
  br label %merge8
merge8:
  %57 = phi i64 [ 1, %then6 ], [ 0, %else7 ]
  br label %merge5
else4:
  br label %merge5
merge5:
  %58 = phi i64 [ %57, %then3 ], [ 0, %else4 ]
  br label %merge2
else1:
  br label %merge2
merge2:
  %59 = phi i64 [ %58, %then0 ], [ 0, %else1 ]
  ret i64 %59
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([30 x i8], [30 x i8]* @.str.24, i64 0, i64 0))
  %1 = trunc i64 10 to i32
  %2 = call i32 @putchar(i32 %1)
  %3 = call i64 @test_push_back()
  %r1.4 = alloca i64
  store i64 %3, i64* %r1.4
  %5 = call i64 @test_push_front()
  %r2.6 = alloca i64
  store i64 %5, i64* %r2.6
  %7 = call i64 @test_pop_operations()
  %r3.8 = alloca i64
  store i64 %7, i64* %r3.8
  %9 = call i64 @test_wrap_around()
  %r4.10 = alloca i64
  store i64 %9, i64* %r4.10
  %11 = trunc i64 10 to i32
  %12 = call i32 @putchar(i32 %11)
  %13 = load i64, i64* %r1.4
  %14 = icmp eq i64 %13, 1
  %15 = zext i1 %14 to i64
  %16 = icmp ne i64 %15, 0
  br i1 %16, label %then0, label %else1
then0:
  %17 = load i64, i64* %r2.6
  %18 = icmp eq i64 %17, 1
  %19 = zext i1 %18 to i64
  %20 = icmp ne i64 %19, 0
  br i1 %20, label %then3, label %else4
then3:
  %21 = load i64, i64* %r3.8
  %22 = icmp eq i64 %21, 1
  %23 = zext i1 %22 to i64
  %24 = icmp ne i64 %23, 0
  br i1 %24, label %then6, label %else7
then6:
  %25 = load i64, i64* %r4.10
  %26 = icmp eq i64 %25, 1
  %27 = zext i1 %26 to i64
  %28 = icmp ne i64 %27, 0
  br i1 %28, label %then9, label %else10
then9:
  %29 = call i32 @puts(i8* getelementptr ([31 x i8], [31 x i8]* @.str.25, i64 0, i64 0))
  br label %merge11
else10:
  %30 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.26, i64 0, i64 0))
  br label %merge11
merge11:
  %31 = phi i64 [ 0, %then9 ], [ 1, %else10 ]
  br label %merge8
else7:
  %32 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.27, i64 0, i64 0))
  br label %merge8
merge8:
  %33 = phi i64 [ %31, %then6 ], [ 1, %else7 ]
  br label %merge5
else4:
  %34 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.28, i64 0, i64 0))
  br label %merge5
merge5:
  %35 = phi i64 [ %33, %then3 ], [ 1, %else4 ]
  br label %merge2
else1:
  %36 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.29, i64 0, i64 0))
  br label %merge2
merge2:
  %37 = phi i64 [ %35, %then0 ], [ 1, %else1 ]
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

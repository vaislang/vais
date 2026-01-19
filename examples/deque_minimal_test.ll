; ModuleID = 'deque_minimal_test'
source_filename = "<vais>"

%Deque = type { i64, i64, i64, i64, i64 }
declare i64 @fgetc(i64)
declare i64 @fopen(i8*, i8*)
declare i32 @puts(i8*)
declare i64 @fseek(i64, i64, i64)
declare i64 @ftell(i64)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @fflush(i64)
declare i32 @putchar(i32)
declare i64 @feof(i64)
declare i64 @malloc(i64)
declare i32 @printf(i8*)
declare i64 @fwrite(i64, i64, i64, i64)
declare i32 @sched_yield()
declare i64 @fputs(i8*, i64)
declare i64 @strlen(i64)
declare void @free(i64)
declare i64 @fputc(i64, i64)
declare i32 @strcmp(i8*, i8*)
declare i32 @strncmp(i8*, i8*, i64)
declare void @exit(i32)
declare i64 @fgets(i64, i64, i64)
declare i32 @fclose(i64)
declare i64 @memcpy(i64, i64, i64)
declare i32 @usleep(i64)
@.str.0 = private unnamed_addr constant [18 x i8] c"Creating deque...\00"
@.str.1 = private unnamed_addr constant [16 x i8] c"Pushing back 10\00"
@.str.2 = private unnamed_addr constant [19 x i8] c"Length should be 1\00"
@.str.3 = private unnamed_addr constant [19 x i8] c"Front should be 10\00"
@.str.4 = private unnamed_addr constant [6 x i8] c"Done!\00"

define i64 @deque_copy_element(i64 %src_data, i64 %dst_data, i64 %head, i64 %cap, i64 %src_idx, i64 %dst_idx, i64 %remaining) {
entry:
  %0 = icmp sle i64 %remaining, 0
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %then0, label %else1
then0:
  br label %merge2
else1:
  %3 = add i64 %head, %src_idx
  %actual_src_temp.4 = alloca i64
  store i64 %3, i64* %actual_src_temp.4
  %5 = load i64, i64* %actual_src_temp.4
  %6 = icmp sge i64 %5, %cap
  %7 = zext i1 %6 to i64
  %8 = icmp ne i64 %7, 0
  br i1 %8, label %then3, label %else4
then3:
  %9 = load i64, i64* %actual_src_temp.4
  %10 = sub i64 %9, %cap
  br label %merge5
else4:
  %11 = load i64, i64* %actual_src_temp.4
  br label %merge5
merge5:
  %12 = phi i64 [ %10, %then3 ], [ %11, %else4 ]
  %actual_src.13 = alloca i64
  store i64 %12, i64* %actual_src.13
  %14 = load i64, i64* %actual_src.13
  %15 = mul i64 %14, 8
  %16 = add i64 %src_data, %15
  %src_ptr.17 = alloca i64
  store i64 %16, i64* %src_ptr.17
  %18 = mul i64 %dst_idx, 8
  %19 = add i64 %dst_data, %18
  %dst_ptr.20 = alloca i64
  store i64 %19, i64* %dst_ptr.20
  %21 = load i64, i64* %src_ptr.17
  %22 = call i64 @__load_i64(i64 %21)
  %value.23 = alloca i64
  store i64 %22, i64* %value.23
  %24 = load i64, i64* %dst_ptr.20
  %25 = load i64, i64* %value.23
  call void @__store_i64(i64 %24, i64 %25)
  %26 = add i64 %src_idx, 1
  %27 = add i64 %dst_idx, 1
  %28 = sub i64 %remaining, 1
  %29 = call i64 @deque_copy_element(i64 %src_data, i64 %dst_data, i64 %head, i64 %cap, i64 %26, i64 %27, i64 %28)
  br label %merge2
merge2:
  %30 = phi i64 [ 0, %then0 ], [ %29, %else1 ]
  ret i64 %30
}

define %Deque @Deque_with_capacity(i64 %capacity) {
entry:
  %0 = mul i64 %capacity, 8
  %1 = call i8* @malloc(i64 %0)
  %2 = ptrtoint i8* %1 to i64
  %data.3 = alloca i64
  store i64 %2, i64* %data.3
  %4 = alloca %Deque
  %5 = load i64, i64* %data.3
  %6 = getelementptr %Deque, %Deque* %4, i32 0, i32 0
  store i64 %5, i64* %6
  %7 = getelementptr %Deque, %Deque* %4, i32 0, i32 1
  store i64 0, i64* %7
  %8 = getelementptr %Deque, %Deque* %4, i32 0, i32 2
  store i64 0, i64* %8
  %9 = getelementptr %Deque, %Deque* %4, i32 0, i32 3
  store i64 0, i64* %9
  %10 = getelementptr %Deque, %Deque* %4, i32 0, i32 4
  store i64 %capacity, i64* %10
  %ret.11 = load %Deque, %Deque* %4
  ret %Deque %ret.11
}

define i64 @Deque_len(%Deque* %self) {
entry:
  %0 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %1 = load i64, i64* %0
  ret i64 %1
}

define i64 @Deque_capacity(%Deque* %self) {
entry:
  %0 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %1 = load i64, i64* %0
  ret i64 %1
}

define i64 @Deque_is_empty(%Deque* %self) {
entry:
  %0 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %1 = load i64, i64* %0
  %2 = icmp eq i64 %1, 0
  %3 = zext i1 %2 to i64
  %4 = icmp ne i64 %3, 0
  br i1 %4, label %then0, label %else1
then0:
  br label %merge2
else1:
  br label %merge2
merge2:
  %5 = phi i64 [ 1, %then0 ], [ 0, %else1 ]
  ret i64 %5
}

define i64 @Deque_push_front(%Deque* %self, i64 %value) {
entry:
  %0 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %1 = load i64, i64* %0
  %2 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %3 = load i64, i64* %2
  %4 = icmp sge i64 %1, %3
  %5 = zext i1 %4 to i64
  %6 = icmp ne i64 %5, 0
  br i1 %6, label %then0, label %else1
then0:
  %7 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %8 = load i64, i64* %7
  %9 = mul i64 %8, 2
  %doubled.10 = alloca i64
  store i64 %9, i64* %doubled.10
  %11 = load i64, i64* %doubled.10
  %12 = icmp slt i64 %11, 8
  %13 = zext i1 %12 to i64
  %14 = icmp ne i64 %13, 0
  br i1 %14, label %then3, label %else4
then3:
  br label %merge5
else4:
  %15 = load i64, i64* %doubled.10
  br label %merge5
merge5:
  %16 = phi i64 [ 8, %then3 ], [ %15, %else4 ]
  %new_cap.17 = alloca i64
  store i64 %16, i64* %new_cap.17
  %18 = load i64, i64* %new_cap.17
  %19 = mul i64 %18, 8
  %20 = call i8* @malloc(i64 %19)
  %21 = ptrtoint i8* %20 to i64
  %new_data.22 = alloca i64
  store i64 %21, i64* %new_data.22
  %23 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %24 = load i64, i64* %23
  %25 = load i64, i64* %new_data.22
  %26 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  %27 = load i64, i64* %26
  %28 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %29 = load i64, i64* %28
  %30 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %31 = load i64, i64* %30
  %32 = call i64 @deque_copy_element(i64 %24, i64 %25, i64 %27, i64 %29, i64 0, i64 0, i64 %31)
  %33 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %34 = load i64, i64* %33
  %35 = inttoptr i64 %34 to i8*
  call void @free(i8* %35)
  %36 = load i64, i64* %new_data.22
  %37 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  store i64 %36, i64* %37
  %38 = load i64, i64* %new_cap.17
  %39 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  store i64 %38, i64* %39
  %40 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  store i64 0, i64* %40
  %41 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %42 = load i64, i64* %41
  %43 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  store i64 %42, i64* %43
  br label %merge2
else1:
  br label %merge2
merge2:
  %44 = add i64 0, 0
  %45 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  %46 = load i64, i64* %45
  %47 = icmp eq i64 %46, 0
  %48 = zext i1 %47 to i64
  %49 = icmp ne i64 %48, 0
  br i1 %49, label %then6, label %else7
then6:
  %50 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %51 = load i64, i64* %50
  %52 = sub i64 %51, 1
  %53 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  store i64 %52, i64* %53
  br label %merge8
else7:
  %54 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  %55 = load i64, i64* %54
  %56 = sub i64 %55, 1
  %57 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  store i64 %56, i64* %57
  br label %merge8
merge8:
  %58 = phi i64 [ %52, %then6 ], [ %56, %else7 ]
  %59 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %60 = load i64, i64* %59
  %61 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  %62 = load i64, i64* %61
  %63 = mul i64 %62, 8
  %64 = add i64 %60, %63
  %ptr.65 = alloca i64
  store i64 %64, i64* %ptr.65
  %66 = load i64, i64* %ptr.65
  call void @__store_i64(i64 %66, i64 %value)
  %67 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %68 = load i64, i64* %67
  %69 = add i64 %68, 1
  %70 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  store i64 %69, i64* %70
  %71 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %72 = load i64, i64* %71
  ret i64 %72
}

define i64 @Deque_push_back(%Deque* %self, i64 %value) {
entry:
  %0 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %1 = load i64, i64* %0
  %2 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %3 = load i64, i64* %2
  %4 = icmp sge i64 %1, %3
  %5 = zext i1 %4 to i64
  %6 = icmp ne i64 %5, 0
  br i1 %6, label %then0, label %else1
then0:
  %7 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %8 = load i64, i64* %7
  %9 = mul i64 %8, 2
  %doubled.10 = alloca i64
  store i64 %9, i64* %doubled.10
  %11 = load i64, i64* %doubled.10
  %12 = icmp slt i64 %11, 8
  %13 = zext i1 %12 to i64
  %14 = icmp ne i64 %13, 0
  br i1 %14, label %then3, label %else4
then3:
  br label %merge5
else4:
  %15 = load i64, i64* %doubled.10
  br label %merge5
merge5:
  %16 = phi i64 [ 8, %then3 ], [ %15, %else4 ]
  %new_cap.17 = alloca i64
  store i64 %16, i64* %new_cap.17
  %18 = load i64, i64* %new_cap.17
  %19 = mul i64 %18, 8
  %20 = call i8* @malloc(i64 %19)
  %21 = ptrtoint i8* %20 to i64
  %new_data.22 = alloca i64
  store i64 %21, i64* %new_data.22
  %23 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %24 = load i64, i64* %23
  %25 = load i64, i64* %new_data.22
  %26 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  %27 = load i64, i64* %26
  %28 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %29 = load i64, i64* %28
  %30 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %31 = load i64, i64* %30
  %32 = call i64 @deque_copy_element(i64 %24, i64 %25, i64 %27, i64 %29, i64 0, i64 0, i64 %31)
  %33 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %34 = load i64, i64* %33
  %35 = inttoptr i64 %34 to i8*
  call void @free(i8* %35)
  %36 = load i64, i64* %new_data.22
  %37 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  store i64 %36, i64* %37
  %38 = load i64, i64* %new_cap.17
  %39 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  store i64 %38, i64* %39
  %40 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  store i64 0, i64* %40
  %41 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %42 = load i64, i64* %41
  %43 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  store i64 %42, i64* %43
  br label %merge2
else1:
  br label %merge2
merge2:
  %44 = add i64 0, 0
  %45 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %46 = load i64, i64* %45
  %47 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  %48 = load i64, i64* %47
  %49 = mul i64 %48, 8
  %50 = add i64 %46, %49
  %ptr.51 = alloca i64
  store i64 %50, i64* %ptr.51
  %52 = load i64, i64* %ptr.51
  call void @__store_i64(i64 %52, i64 %value)
  %53 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  %54 = load i64, i64* %53
  %55 = add i64 %54, 1
  %56 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  store i64 %55, i64* %56
  %57 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  %58 = load i64, i64* %57
  %59 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %60 = load i64, i64* %59
  %61 = icmp sge i64 %58, %60
  %62 = zext i1 %61 to i64
  %63 = icmp ne i64 %62, 0
  br i1 %63, label %then6, label %else7
then6:
  %64 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  store i64 0, i64* %64
  br label %merge8
else7:
  br label %merge8
merge8:
  %65 = add i64 0, 0
  %66 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %67 = load i64, i64* %66
  %68 = add i64 %67, 1
  %69 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  store i64 %68, i64* %69
  %70 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %71 = load i64, i64* %70
  ret i64 %71
}

define i64 @Deque_pop_front(%Deque* %self) {
entry:
  %0 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %1 = load i64, i64* %0
  %2 = icmp eq i64 %1, 0
  %3 = zext i1 %2 to i64
  %4 = icmp ne i64 %3, 0
  br i1 %4, label %then0, label %else1
then0:
  br label %merge2
else1:
  %5 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %6 = load i64, i64* %5
  %7 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  %8 = load i64, i64* %7
  %9 = mul i64 %8, 8
  %10 = add i64 %6, %9
  %ptr.11 = alloca i64
  store i64 %10, i64* %ptr.11
  %12 = load i64, i64* %ptr.11
  %13 = call i64 @__load_i64(i64 %12)
  %value.14 = alloca i64
  store i64 %13, i64* %value.14
  %15 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  %16 = load i64, i64* %15
  %17 = add i64 %16, 1
  %18 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  store i64 %17, i64* %18
  %19 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  %20 = load i64, i64* %19
  %21 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %22 = load i64, i64* %21
  %23 = icmp sge i64 %20, %22
  %24 = zext i1 %23 to i64
  %25 = icmp ne i64 %24, 0
  br i1 %25, label %then3, label %else4
then3:
  %26 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  store i64 0, i64* %26
  br label %merge5
else4:
  br label %merge5
merge5:
  %27 = add i64 0, 0
  %28 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %29 = load i64, i64* %28
  %30 = sub i64 %29, 1
  %31 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  store i64 %30, i64* %31
  %32 = load i64, i64* %value.14
  br label %merge2
merge2:
  %33 = phi i64 [ 0, %then0 ], [ %32, %else1 ]
  ret i64 %33
}

define i64 @Deque_pop_back(%Deque* %self) {
entry:
  %0 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %1 = load i64, i64* %0
  %2 = icmp eq i64 %1, 0
  %3 = zext i1 %2 to i64
  %4 = icmp ne i64 %3, 0
  br i1 %4, label %then0, label %else1
then0:
  br label %merge2
else1:
  %5 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  %6 = load i64, i64* %5
  %7 = icmp eq i64 %6, 0
  %8 = zext i1 %7 to i64
  %9 = icmp ne i64 %8, 0
  br i1 %9, label %then3, label %else4
then3:
  %10 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %11 = load i64, i64* %10
  %12 = sub i64 %11, 1
  %13 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  store i64 %12, i64* %13
  br label %merge5
else4:
  %14 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  %15 = load i64, i64* %14
  %16 = sub i64 %15, 1
  %17 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  store i64 %16, i64* %17
  br label %merge5
merge5:
  %18 = phi i64 [ %12, %then3 ], [ %16, %else4 ]
  %19 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %20 = load i64, i64* %19
  %21 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  %22 = load i64, i64* %21
  %23 = mul i64 %22, 8
  %24 = add i64 %20, %23
  %ptr.25 = alloca i64
  store i64 %24, i64* %ptr.25
  %26 = load i64, i64* %ptr.25
  %27 = call i64 @__load_i64(i64 %26)
  %value.28 = alloca i64
  store i64 %27, i64* %value.28
  %29 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %30 = load i64, i64* %29
  %31 = sub i64 %30, 1
  %32 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  store i64 %31, i64* %32
  %33 = load i64, i64* %value.28
  br label %merge2
merge2:
  %34 = phi i64 [ 0, %then0 ], [ %33, %else1 ]
  ret i64 %34
}

define i64 @Deque_front(%Deque* %self) {
entry:
  %0 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %1 = load i64, i64* %0
  %2 = icmp eq i64 %1, 0
  %3 = zext i1 %2 to i64
  %4 = icmp ne i64 %3, 0
  br i1 %4, label %then0, label %else1
then0:
  br label %merge2
else1:
  %5 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %6 = load i64, i64* %5
  %7 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  %8 = load i64, i64* %7
  %9 = mul i64 %8, 8
  %10 = add i64 %6, %9
  %ptr.11 = alloca i64
  store i64 %10, i64* %ptr.11
  %12 = load i64, i64* %ptr.11
  %13 = call i64 @__load_i64(i64 %12)
  br label %merge2
merge2:
  %14 = phi i64 [ 0, %then0 ], [ %13, %else1 ]
  ret i64 %14
}

define i64 @Deque_back(%Deque* %self) {
entry:
  %0 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %1 = load i64, i64* %0
  %2 = icmp eq i64 %1, 0
  %3 = zext i1 %2 to i64
  %4 = icmp ne i64 %3, 0
  br i1 %4, label %then0, label %else1
then0:
  br label %merge2
else1:
  %5 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  %6 = load i64, i64* %5
  %7 = sub i64 %6, 1
  %back_idx_temp.8 = alloca i64
  store i64 %7, i64* %back_idx_temp.8
  %9 = load i64, i64* %back_idx_temp.8
  %10 = icmp slt i64 %9, 0
  %11 = zext i1 %10 to i64
  %12 = icmp ne i64 %11, 0
  br i1 %12, label %then3, label %else4
then3:
  %13 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %14 = load i64, i64* %13
  %15 = sub i64 %14, 1
  br label %merge5
else4:
  %16 = load i64, i64* %back_idx_temp.8
  br label %merge5
merge5:
  %17 = phi i64 [ %15, %then3 ], [ %16, %else4 ]
  %back_idx.18 = alloca i64
  store i64 %17, i64* %back_idx.18
  %19 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %20 = load i64, i64* %19
  %21 = load i64, i64* %back_idx.18
  %22 = mul i64 %21, 8
  %23 = add i64 %20, %22
  %ptr.24 = alloca i64
  store i64 %23, i64* %ptr.24
  %25 = load i64, i64* %ptr.24
  %26 = call i64 @__load_i64(i64 %25)
  br label %merge2
merge2:
  %27 = phi i64 [ 0, %then0 ], [ %26, %else1 ]
  ret i64 %27
}

define i64 @Deque_get(%Deque* %self, i64 %index) {
entry:
  %0 = icmp slt i64 %index, 0
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %then0, label %else1
then0:
  br label %merge2
else1:
  %3 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %4 = load i64, i64* %3
  %5 = icmp sge i64 %index, %4
  %6 = zext i1 %5 to i64
  %7 = icmp ne i64 %6, 0
  br i1 %7, label %elseif.then3, label %elseif.else4
elseif.then3:
  br label %elseif.merge5
elseif.else4:
  %8 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  %9 = load i64, i64* %8
  %10 = add i64 %9, %index
  %actual_idx_temp.11 = alloca i64
  store i64 %10, i64* %actual_idx_temp.11
  %12 = load i64, i64* %actual_idx_temp.11
  %13 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %14 = load i64, i64* %13
  %15 = icmp sge i64 %12, %14
  %16 = zext i1 %15 to i64
  %17 = icmp ne i64 %16, 0
  br i1 %17, label %then6, label %else7
then6:
  %18 = load i64, i64* %actual_idx_temp.11
  %19 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %20 = load i64, i64* %19
  %21 = sub i64 %18, %20
  br label %merge8
else7:
  %22 = load i64, i64* %actual_idx_temp.11
  br label %merge8
merge8:
  %23 = phi i64 [ %21, %then6 ], [ %22, %else7 ]
  %actual_idx.24 = alloca i64
  store i64 %23, i64* %actual_idx.24
  %25 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %26 = load i64, i64* %25
  %27 = load i64, i64* %actual_idx.24
  %28 = mul i64 %27, 8
  %29 = add i64 %26, %28
  %ptr.30 = alloca i64
  store i64 %29, i64* %ptr.30
  %31 = load i64, i64* %ptr.30
  %32 = call i64 @__load_i64(i64 %31)
  br label %elseif.merge5
elseif.merge5:
  %33 = phi i64 [ 0, %elseif.then3 ], [ %32, %elseif.else4 ]
  br label %merge2
merge2:
  %34 = phi i64 [ 0, %then0 ], [ %33, %else1 ]
  ret i64 %34
}

define i64 @Deque_set(%Deque* %self, i64 %index, i64 %value) {
entry:
  %0 = icmp slt i64 %index, 0
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %then0, label %else1
then0:
  br label %merge2
else1:
  %3 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %4 = load i64, i64* %3
  %5 = icmp sge i64 %index, %4
  %6 = zext i1 %5 to i64
  %7 = icmp ne i64 %6, 0
  br i1 %7, label %elseif.then3, label %elseif.else4
elseif.then3:
  br label %elseif.merge5
elseif.else4:
  %8 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  %9 = load i64, i64* %8
  %10 = add i64 %9, %index
  %actual_idx_temp.11 = alloca i64
  store i64 %10, i64* %actual_idx_temp.11
  %12 = load i64, i64* %actual_idx_temp.11
  %13 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %14 = load i64, i64* %13
  %15 = icmp sge i64 %12, %14
  %16 = zext i1 %15 to i64
  %17 = icmp ne i64 %16, 0
  br i1 %17, label %then6, label %else7
then6:
  %18 = load i64, i64* %actual_idx_temp.11
  %19 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %20 = load i64, i64* %19
  %21 = sub i64 %18, %20
  br label %merge8
else7:
  %22 = load i64, i64* %actual_idx_temp.11
  br label %merge8
merge8:
  %23 = phi i64 [ %21, %then6 ], [ %22, %else7 ]
  %actual_idx.24 = alloca i64
  store i64 %23, i64* %actual_idx.24
  %25 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %26 = load i64, i64* %25
  %27 = load i64, i64* %actual_idx.24
  %28 = mul i64 %27, 8
  %29 = add i64 %26, %28
  %ptr.30 = alloca i64
  store i64 %29, i64* %ptr.30
  %31 = load i64, i64* %ptr.30
  call void @__store_i64(i64 %31, i64 %value)
  br label %elseif.merge5
elseif.merge5:
  %32 = phi i64 [ 0, %elseif.then3 ], [ 1, %elseif.else4 ]
  br label %merge2
merge2:
  %33 = phi i64 [ 0, %then0 ], [ %32, %else1 ]
  ret i64 %33
}

define i64 @Deque_clear(%Deque* %self) {
entry:
  %0 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  store i64 0, i64* %0
  %1 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  store i64 0, i64* %1
  %2 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  store i64 0, i64* %2
  ret i64 0
}

define i64 @Deque_drop(%Deque* %self) {
entry:
  %0 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = inttoptr i64 %1 to i8*
  call void @free(i8* %2)
  %3 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  store i64 0, i64* %3
  %4 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  store i64 0, i64* %4
  %5 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  store i64 0, i64* %5
  %6 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  store i64 0, i64* %6
  %7 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  store i64 0, i64* %7
  ret i64 0
}

define %Deque @deque_new() {
entry:
  %0 = call %Deque @Deque_with_capacity(i64 8)
  %ret.1 = load %Deque, %Deque* %0
  ret %Deque %ret.1
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([18 x i8], [18 x i8]* @.str.0, i64 0, i64 0))
  %1 = call %Deque @Deque_with_capacity(i64 8)
  %dq.2.struct = alloca %Deque
  store %Deque %1, %Deque* %dq.2.struct
  %dq.2 = alloca %Deque*
  store %Deque* %dq.2.struct, %Deque** %dq.2
  %3 = call i32 @puts(i8* getelementptr ([16 x i8], [16 x i8]* @.str.1, i64 0, i64 0))
  %4 = load %Deque*, %Deque** %dq.2
  %5 = call i64 @Deque_push_back(%Deque* %4, i64 10)
  %6 = load %Deque*, %Deque** %dq.2
  %7 = call i64 @Deque_len(%Deque* %6)
  %len.8 = alloca i64
  store i64 %7, i64* %len.8
  %9 = call i32 @puts(i8* getelementptr ([19 x i8], [19 x i8]* @.str.2, i64 0, i64 0))
  %10 = load %Deque*, %Deque** %dq.2
  %11 = call i64 @Deque_front(%Deque* %10)
  %front.12 = alloca i64
  store i64 %11, i64* %front.12
  %13 = call i32 @puts(i8* getelementptr ([19 x i8], [19 x i8]* @.str.3, i64 0, i64 0))
  %14 = load %Deque*, %Deque** %dq.2
  %15 = call i64 @Deque_drop(%Deque* %14)
  %16 = call i32 @puts(i8* getelementptr ([6 x i8], [6 x i8]* @.str.4, i64 0, i64 0))
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

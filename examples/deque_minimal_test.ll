; ModuleID = 'deque_minimal_test'
source_filename = "<vais>"

%Deque = type { i64, i64, i64, i64, i64 }
%Option = type { i32, { i64 } }
declare i64 @memcpy(i64, i64, i64)
declare void @free(i64)
declare i64 @memcpy_str(i64, i8*, i64)
declare i64 @atol(i64)
declare i64 @fflush(i64)
declare i64 @fputs(i8*, i64)
declare i64 @strcat(i64, i8*)
declare i32 @printf(i8*, ...)
declare i64 @malloc(i64)
declare i64 @feof(i64)
declare i64 @vais_gc_print_stats()
declare i64 @vais_gc_add_root(i64)
declare i64 @vais_gc_collections()
declare i32 @putchar(i32)
declare void @srand(i32)
declare i64 @fgetc(i64)
declare i64 @fgets(i64, i64, i64)
declare i32 @strcmp(i8*, i8*)
declare i64 @vais_gc_set_threshold(i64)
declare i32 @rand()
declare i64 @fopen(i8*, i8*)
declare i32 @strncmp(i8*, i8*, i64)
declare void @exit(i32)
declare i32 @isdigit(i32)
declare i64 @vais_gc_collect()
declare i64 @labs(i64)
declare i32 @puts(i64)
declare i32 @tolower(i32)
declare i32 @sched_yield()
declare double @atof(i8*)
declare i32 @fclose(i64)
declare i64 @strcpy(i64, i8*)
declare i32 @usleep(i64)
declare i64 @fseek(i64, i64, i64)
declare i64 @vais_gc_objects_count()
declare i64 @fputc(i64, i64)
declare i32 @toupper(i32)
declare i64 @strlen(i8*)
declare i32 @atoi(i8*)
declare i64 @vais_gc_remove_root(i64)
declare i64 @vais_gc_alloc(i64, i32)
declare double @sqrt(double)
declare i64 @fwrite(i64, i64, i64, i64)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare i32 @isalpha(i32)
declare i64 @vais_gc_bytes_allocated()
declare i64 @fread(i64, i64, i64, i64)
declare i64 @vais_gc_init()
declare double @fabs(double)
declare i64 @ftell(i64)
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

@.str.0 = private unnamed_addr constant [18 x i8] c"Creating deque...\00"
@.str.1 = private unnamed_addr constant [16 x i8] c"Pushing back 10\00"
@.str.2 = private unnamed_addr constant [19 x i8] c"Length should be 1\00"
@.str.3 = private unnamed_addr constant [19 x i8] c"Front should be 10\00"
@.str.4 = private unnamed_addr constant [6 x i8] c"Done!\00"

define i64 @Option_is_some(%Option* %self) {
entry:
  br label %match.check1
match.check1:
  %0 = getelementptr { i32 }, { i32 }* %self, i32 0, i32 0
  %1 = load i32, i32* %0
  %2 = icmp eq i32 %1, 1
  br i1 %2, label %match.arm3, label %match.check2
match.arm3:
  %3 = getelementptr { i32, i64 }, { i32, i64 }* %self, i32 0, i32 1
  %4 = load i64, i64* %3
  br label %match.merge0
match.check2:
  %5 = getelementptr { i32 }, { i32 }* %self, i32 0, i32 0
  %6 = load i32, i32* %5
  %7 = icmp eq i32 %6, 0
  br i1 %7, label %match.arm4, label %match.merge0
match.arm4:
  br label %match.merge0
match.merge0:
  %8 = phi i64 [ 1, %match.arm3 ], [ 0, %match.arm4 ]
  ret i64 %8
}

define i64 @Option_is_none(%Option* %self) {
entry:
  br label %match.check1
match.check1:
  %0 = getelementptr { i32 }, { i32 }* %self, i32 0, i32 0
  %1 = load i32, i32* %0
  %2 = icmp eq i32 %1, 1
  br i1 %2, label %match.arm3, label %match.check2
match.arm3:
  %3 = getelementptr { i32, i64 }, { i32, i64 }* %self, i32 0, i32 1
  %4 = load i64, i64* %3
  br label %match.merge0
match.check2:
  %5 = getelementptr { i32 }, { i32 }* %self, i32 0, i32 0
  %6 = load i32, i32* %5
  %7 = icmp eq i32 %6, 0
  br i1 %7, label %match.arm4, label %match.merge0
match.arm4:
  br label %match.merge0
match.merge0:
  %8 = phi i64 [ 0, %match.arm3 ], [ 1, %match.arm4 ]
  ret i64 %8
}

define i64 @Option_unwrap_or(%Option* %self, i64 %default) {
entry:
  br label %match.check1
match.check1:
  %0 = getelementptr { i32 }, { i32 }* %self, i32 0, i32 0
  %1 = load i32, i32* %0
  %2 = icmp eq i32 %1, 1
  br i1 %2, label %match.arm3, label %match.check2
match.arm3:
  %3 = getelementptr { i32, i64 }, { i32, i64 }* %self, i32 0, i32 1
  %4 = load i64, i64* %3
  br label %match.merge0
match.check2:
  %6 = getelementptr { i32 }, { i32 }* %self, i32 0, i32 0
  %7 = load i32, i32* %6
  %8 = icmp eq i32 %7, 0
  br i1 %8, label %match.arm4, label %match.merge0
match.arm4:
  br label %match.merge0
match.merge0:
  %9 = phi i64 [ %4, %match.arm3 ], [ %default, %match.arm4 ]
  ret i64 %9
}

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
  %4 = icmp sge i64 %3, %cap
  %5 = zext i1 %4 to i64
  %6 = icmp ne i64 %5, 0
  br i1 %6, label %then3, label %else4
then3:
  %7 = sub i64 %3, %cap
  br label %merge5
else4:
  br label %merge5
merge5:
  %8 = phi i64 [ %7, %then3 ], [ %3, %else4 ]
  %9 = mul i64 %8, 8
  %10 = add i64 %src_data, %9
  %11 = mul i64 %dst_idx, 8
  %12 = add i64 %dst_data, %11
  %13 = call i64 @__load_i64(i64 %10)
  call void @__store_i64(i64 %12, i64 %13)
  %14 = add i64 %src_idx, 1
  %15 = add i64 %dst_idx, 1
  %16 = sub i64 %remaining, 1
  %17 = call i64 @deque_copy_element(i64 %src_data, i64 %dst_data, i64 %head, i64 %cap, i64 %14, i64 %15, i64 %16)
  br label %merge2
merge2:
  %18 = phi i64 [ 0, %then0 ], [ %17, %merge5 ]
  ret i64 %18
}

define %Deque @Deque_with_capacity(i64 %capacity) {
entry:
  %0 = mul i64 %capacity, 8
  %1 = call i8* @malloc(i64 %0)
  %2 = ptrtoint i8* %1 to i64
  %3 = alloca %Deque
  %4 = getelementptr %Deque, %Deque* %3, i32 0, i32 0
  store i64 %2, i64* %4
  %5 = getelementptr %Deque, %Deque* %3, i32 0, i32 1
  store i64 0, i64* %5
  %6 = getelementptr %Deque, %Deque* %3, i32 0, i32 2
  store i64 0, i64* %6
  %7 = getelementptr %Deque, %Deque* %3, i32 0, i32 3
  store i64 0, i64* %7
  %8 = getelementptr %Deque, %Deque* %3, i32 0, i32 4
  store i64 %capacity, i64* %8
  %ret.9 = load %Deque, %Deque* %3
  ret %Deque %ret.9
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
  %10 = icmp slt i64 %9, 8
  %11 = zext i1 %10 to i64
  %12 = icmp ne i64 %11, 0
  br i1 %12, label %then3, label %else4
then3:
  br label %merge5
else4:
  br label %merge5
merge5:
  %13 = phi i64 [ 8, %then3 ], [ %9, %else4 ]
  %14 = mul i64 %13, 8
  %15 = call i8* @malloc(i64 %14)
  %16 = ptrtoint i8* %15 to i64
  %17 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %18 = load i64, i64* %17
  %19 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  %20 = load i64, i64* %19
  %21 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %22 = load i64, i64* %21
  %23 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %24 = load i64, i64* %23
  %25 = call i64 @deque_copy_element(i64 %18, i64 %16, i64 %20, i64 %22, i64 0, i64 0, i64 %24)
  %26 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %27 = load i64, i64* %26
  %28 = inttoptr i64 %27 to i8*
  call void @free(i8* %28)
  %29 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  store i64 %16, i64* %29
  %30 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  store i64 %13, i64* %30
  %31 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  store i64 0, i64* %31
  %32 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %33 = load i64, i64* %32
  %34 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  store i64 %33, i64* %34
  br label %merge2
else1:
  br label %merge2
merge2:
  %35 = add i64 0, 0
  %36 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  %37 = load i64, i64* %36
  %38 = icmp eq i64 %37, 0
  %39 = zext i1 %38 to i64
  %40 = icmp ne i64 %39, 0
  br i1 %40, label %then6, label %else7
then6:
  %41 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %42 = load i64, i64* %41
  %43 = sub i64 %42, 1
  %44 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  store i64 %43, i64* %44
  br label %merge8
else7:
  %45 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  %46 = load i64, i64* %45
  %47 = sub i64 %46, 1
  %48 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  store i64 %47, i64* %48
  br label %merge8
merge8:
  %49 = phi i64 [ %43, %then6 ], [ %47, %else7 ]
  %50 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %51 = load i64, i64* %50
  %52 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  %53 = load i64, i64* %52
  %54 = mul i64 %53, 8
  %55 = add i64 %51, %54
  call void @__store_i64(i64 %55, i64 %value)
  %56 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %57 = load i64, i64* %56
  %58 = add i64 %57, 1
  %59 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  store i64 %58, i64* %59
  %60 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %61 = load i64, i64* %60
  ret i64 %61
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
  %10 = icmp slt i64 %9, 8
  %11 = zext i1 %10 to i64
  %12 = icmp ne i64 %11, 0
  br i1 %12, label %then3, label %else4
then3:
  br label %merge5
else4:
  br label %merge5
merge5:
  %13 = phi i64 [ 8, %then3 ], [ %9, %else4 ]
  %14 = mul i64 %13, 8
  %15 = call i8* @malloc(i64 %14)
  %16 = ptrtoint i8* %15 to i64
  %17 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %18 = load i64, i64* %17
  %19 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  %20 = load i64, i64* %19
  %21 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %22 = load i64, i64* %21
  %23 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %24 = load i64, i64* %23
  %25 = call i64 @deque_copy_element(i64 %18, i64 %16, i64 %20, i64 %22, i64 0, i64 0, i64 %24)
  %26 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %27 = load i64, i64* %26
  %28 = inttoptr i64 %27 to i8*
  call void @free(i8* %28)
  %29 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  store i64 %16, i64* %29
  %30 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  store i64 %13, i64* %30
  %31 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  store i64 0, i64* %31
  %32 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %33 = load i64, i64* %32
  %34 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  store i64 %33, i64* %34
  br label %merge2
else1:
  br label %merge2
merge2:
  %35 = add i64 0, 0
  %36 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %37 = load i64, i64* %36
  %38 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  %39 = load i64, i64* %38
  %40 = mul i64 %39, 8
  %41 = add i64 %37, %40
  call void @__store_i64(i64 %41, i64 %value)
  %42 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  %43 = load i64, i64* %42
  %44 = add i64 %43, 1
  %45 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  store i64 %44, i64* %45
  %46 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  %47 = load i64, i64* %46
  %48 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %49 = load i64, i64* %48
  %50 = icmp sge i64 %47, %49
  %51 = zext i1 %50 to i64
  %52 = icmp ne i64 %51, 0
  br i1 %52, label %then6, label %else7
then6:
  %53 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  store i64 0, i64* %53
  br label %merge8
else7:
  br label %merge8
merge8:
  %54 = add i64 0, 0
  %55 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %56 = load i64, i64* %55
  %57 = add i64 %56, 1
  %58 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  store i64 %57, i64* %58
  %59 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %60 = load i64, i64* %59
  ret i64 %60
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
  %11 = call i64 @__load_i64(i64 %10)
  %12 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  %13 = load i64, i64* %12
  %14 = add i64 %13, 1
  %15 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  store i64 %14, i64* %15
  %16 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  %17 = load i64, i64* %16
  %18 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %19 = load i64, i64* %18
  %20 = icmp sge i64 %17, %19
  %21 = zext i1 %20 to i64
  %22 = icmp ne i64 %21, 0
  br i1 %22, label %then3, label %else4
then3:
  %23 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  store i64 0, i64* %23
  br label %merge5
else4:
  br label %merge5
merge5:
  %24 = add i64 0, 0
  %25 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %26 = load i64, i64* %25
  %27 = sub i64 %26, 1
  %28 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  store i64 %27, i64* %28
  br label %merge2
merge2:
  %29 = phi i64 [ 0, %then0 ], [ %11, %merge5 ]
  ret i64 %29
}

define %Option @Deque_pop_front_opt(%Deque* %self) {
entry:
  %0 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %1 = load i64, i64* %0
  %2 = icmp eq i64 %1, 0
  %3 = zext i1 %2 to i64
  %4 = icmp ne i64 %3, 0
  br i1 %4, label %then0, label %else1
then0:
  %5 = alloca %Option
  %6 = getelementptr %Option, %Option* %5, i32 0, i32 0
  store i32 0, i32* %6
  %7 = load %Option, %Option* %5
  br label %merge2
else1:
  %8 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %9 = load i64, i64* %8
  %10 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  %11 = load i64, i64* %10
  %12 = mul i64 %11, 8
  %13 = add i64 %9, %12
  %14 = call i64 @__load_i64(i64 %13)
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
  %32 = alloca %Option
  %33 = getelementptr %Option, %Option* %32, i32 0, i32 0
  store i32 1, i32* %33
  %34 = getelementptr %Option, %Option* %32, i32 0, i32 1
  store i64 %14, i64* %34
  %35 = load %Option, %Option* %32
  br label %merge2
merge2:
  %36 = phi %Option [ %7, %then0 ], [ %35, %merge5 ]
  ret %Option %36
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
  %25 = call i64 @__load_i64(i64 %24)
  %26 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %27 = load i64, i64* %26
  %28 = sub i64 %27, 1
  %29 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  store i64 %28, i64* %29
  br label %merge2
merge2:
  %30 = phi i64 [ 0, %then0 ], [ %25, %merge5 ]
  ret i64 %30
}

define %Option @Deque_pop_back_opt(%Deque* %self) {
entry:
  %0 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %1 = load i64, i64* %0
  %2 = icmp eq i64 %1, 0
  %3 = zext i1 %2 to i64
  %4 = icmp ne i64 %3, 0
  br i1 %4, label %then0, label %else1
then0:
  %5 = alloca %Option
  %6 = getelementptr %Option, %Option* %5, i32 0, i32 0
  store i32 0, i32* %6
  %7 = load %Option, %Option* %5
  br label %merge2
else1:
  %8 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  %9 = load i64, i64* %8
  %10 = icmp eq i64 %9, 0
  %11 = zext i1 %10 to i64
  %12 = icmp ne i64 %11, 0
  br i1 %12, label %then3, label %else4
then3:
  %13 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %14 = load i64, i64* %13
  %15 = sub i64 %14, 1
  %16 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  store i64 %15, i64* %16
  br label %merge5
else4:
  %17 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  %18 = load i64, i64* %17
  %19 = sub i64 %18, 1
  %20 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  store i64 %19, i64* %20
  br label %merge5
merge5:
  %21 = phi i64 [ %15, %then3 ], [ %19, %else4 ]
  %22 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %23 = load i64, i64* %22
  %24 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  %25 = load i64, i64* %24
  %26 = mul i64 %25, 8
  %27 = add i64 %23, %26
  %28 = call i64 @__load_i64(i64 %27)
  %29 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %30 = load i64, i64* %29
  %31 = sub i64 %30, 1
  %32 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  store i64 %31, i64* %32
  %33 = alloca %Option
  %34 = getelementptr %Option, %Option* %33, i32 0, i32 0
  store i32 1, i32* %34
  %35 = getelementptr %Option, %Option* %33, i32 0, i32 1
  store i64 %28, i64* %35
  %36 = load %Option, %Option* %33
  br label %merge2
merge2:
  %37 = phi %Option [ %7, %then0 ], [ %36, %merge5 ]
  ret %Option %37
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
  %11 = call i64 @__load_i64(i64 %10)
  br label %merge2
merge2:
  %12 = phi i64 [ 0, %then0 ], [ %11, %else1 ]
  ret i64 %12
}

define %Option @Deque_front_opt(%Deque* %self) {
entry:
  %0 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %1 = load i64, i64* %0
  %2 = icmp eq i64 %1, 0
  %3 = zext i1 %2 to i64
  %4 = icmp ne i64 %3, 0
  br i1 %4, label %then0, label %else1
then0:
  %5 = alloca %Option
  %6 = getelementptr %Option, %Option* %5, i32 0, i32 0
  store i32 0, i32* %6
  %7 = load %Option, %Option* %5
  br label %merge2
else1:
  %8 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %9 = load i64, i64* %8
  %10 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  %11 = load i64, i64* %10
  %12 = mul i64 %11, 8
  %13 = add i64 %9, %12
  %14 = call i64 @__load_i64(i64 %13)
  %15 = alloca %Option
  %16 = getelementptr %Option, %Option* %15, i32 0, i32 0
  store i32 1, i32* %16
  %17 = getelementptr %Option, %Option* %15, i32 0, i32 1
  store i64 %14, i64* %17
  %18 = load %Option, %Option* %15
  br label %merge2
merge2:
  %19 = phi %Option [ %7, %then0 ], [ %18, %else1 ]
  ret %Option %19
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
  %8 = icmp slt i64 %7, 0
  %9 = zext i1 %8 to i64
  %10 = icmp ne i64 %9, 0
  br i1 %10, label %then3, label %else4
then3:
  %11 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %12 = load i64, i64* %11
  %13 = sub i64 %12, 1
  br label %merge5
else4:
  br label %merge5
merge5:
  %14 = phi i64 [ %13, %then3 ], [ %7, %else4 ]
  %15 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %16 = load i64, i64* %15
  %17 = mul i64 %14, 8
  %18 = add i64 %16, %17
  %19 = call i64 @__load_i64(i64 %18)
  br label %merge2
merge2:
  %20 = phi i64 [ 0, %then0 ], [ %19, %merge5 ]
  ret i64 %20
}

define %Option @Deque_back_opt(%Deque* %self) {
entry:
  %0 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %1 = load i64, i64* %0
  %2 = icmp eq i64 %1, 0
  %3 = zext i1 %2 to i64
  %4 = icmp ne i64 %3, 0
  br i1 %4, label %then0, label %else1
then0:
  %5 = alloca %Option
  %6 = getelementptr %Option, %Option* %5, i32 0, i32 0
  store i32 0, i32* %6
  %7 = load %Option, %Option* %5
  br label %merge2
else1:
  %8 = getelementptr %Deque, %Deque* %self, i32 0, i32 2
  %9 = load i64, i64* %8
  %10 = sub i64 %9, 1
  %11 = icmp slt i64 %10, 0
  %12 = zext i1 %11 to i64
  %13 = icmp ne i64 %12, 0
  br i1 %13, label %then3, label %else4
then3:
  %14 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %15 = load i64, i64* %14
  %16 = sub i64 %15, 1
  br label %merge5
else4:
  br label %merge5
merge5:
  %17 = phi i64 [ %16, %then3 ], [ %10, %else4 ]
  %18 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %19 = load i64, i64* %18
  %20 = mul i64 %17, 8
  %21 = add i64 %19, %20
  %22 = call i64 @__load_i64(i64 %21)
  %23 = alloca %Option
  %24 = getelementptr %Option, %Option* %23, i32 0, i32 0
  store i32 1, i32* %24
  %25 = getelementptr %Option, %Option* %23, i32 0, i32 1
  store i64 %22, i64* %25
  %26 = load %Option, %Option* %23
  br label %merge2
merge2:
  %27 = phi %Option [ %7, %then0 ], [ %26, %merge5 ]
  ret %Option %27
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
  %11 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %12 = load i64, i64* %11
  %13 = icmp sge i64 %10, %12
  %14 = zext i1 %13 to i64
  %15 = icmp ne i64 %14, 0
  br i1 %15, label %then6, label %else7
then6:
  %16 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %17 = load i64, i64* %16
  %18 = sub i64 %10, %17
  br label %merge8
else7:
  br label %merge8
merge8:
  %19 = phi i64 [ %18, %then6 ], [ %10, %else7 ]
  %20 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %21 = load i64, i64* %20
  %22 = mul i64 %19, 8
  %23 = add i64 %21, %22
  %24 = call i64 @__load_i64(i64 %23)
  br label %elseif.merge5
elseif.merge5:
  %25 = phi i64 [ 0, %elseif.then3 ], [ %24, %merge8 ]
  br label %merge2
merge2:
  %26 = phi i64 [ 0, %then0 ], [ %25, %elseif.merge5 ]
  ret i64 %26
}

define %Option @Deque_get_opt(%Deque* %self, i64 %index) {
entry:
  %0 = icmp slt i64 %index, 0
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %then0, label %else1
then0:
  %3 = alloca %Option
  %4 = getelementptr %Option, %Option* %3, i32 0, i32 0
  store i32 0, i32* %4
  %5 = load %Option, %Option* %3
  br label %merge2
else1:
  %6 = getelementptr %Deque, %Deque* %self, i32 0, i32 3
  %7 = load i64, i64* %6
  %8 = icmp sge i64 %index, %7
  %9 = zext i1 %8 to i64
  %10 = icmp ne i64 %9, 0
  br i1 %10, label %elseif.then3, label %elseif.else4
elseif.then3:
  %11 = alloca %Option
  %12 = getelementptr %Option, %Option* %11, i32 0, i32 0
  store i32 0, i32* %12
  %13 = load %Option, %Option* %11
  br label %elseif.merge5
elseif.else4:
  %14 = getelementptr %Deque, %Deque* %self, i32 0, i32 1
  %15 = load i64, i64* %14
  %16 = add i64 %15, %index
  %17 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %18 = load i64, i64* %17
  %19 = icmp sge i64 %16, %18
  %20 = zext i1 %19 to i64
  %21 = icmp ne i64 %20, 0
  br i1 %21, label %then6, label %else7
then6:
  %22 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %23 = load i64, i64* %22
  %24 = sub i64 %16, %23
  br label %merge8
else7:
  br label %merge8
merge8:
  %25 = phi i64 [ %24, %then6 ], [ %16, %else7 ]
  %26 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %27 = load i64, i64* %26
  %28 = mul i64 %25, 8
  %29 = add i64 %27, %28
  %30 = call i64 @__load_i64(i64 %29)
  %31 = alloca %Option
  %32 = getelementptr %Option, %Option* %31, i32 0, i32 0
  store i32 1, i32* %32
  %33 = getelementptr %Option, %Option* %31, i32 0, i32 1
  store i64 %30, i64* %33
  %34 = load %Option, %Option* %31
  br label %elseif.merge5
elseif.merge5:
  %35 = phi %Option [ %13, %elseif.then3 ], [ %34, %merge8 ]
  br label %merge2
merge2:
  %36 = phi %Option [ %5, %then0 ], [ %35, %elseif.merge5 ]
  ret %Option %36
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
  %11 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %12 = load i64, i64* %11
  %13 = icmp sge i64 %10, %12
  %14 = zext i1 %13 to i64
  %15 = icmp ne i64 %14, 0
  br i1 %15, label %then6, label %else7
then6:
  %16 = getelementptr %Deque, %Deque* %self, i32 0, i32 4
  %17 = load i64, i64* %16
  %18 = sub i64 %10, %17
  br label %merge8
else7:
  br label %merge8
merge8:
  %19 = phi i64 [ %18, %then6 ], [ %10, %else7 ]
  %20 = getelementptr %Deque, %Deque* %self, i32 0, i32 0
  %21 = load i64, i64* %20
  %22 = mul i64 %19, 8
  %23 = add i64 %21, %22
  call void @__store_i64(i64 %23, i64 %value)
  br label %elseif.merge5
elseif.merge5:
  %24 = phi i64 [ 0, %elseif.then3 ], [ 1, %merge8 ]
  br label %merge2
merge2:
  %25 = phi i64 [ 0, %then0 ], [ %24, %elseif.merge5 ]
  ret i64 %25
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
  ret %Deque %0
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([18 x i8], [18 x i8]* @.str.0, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = call %Deque @Deque_with_capacity(i64 8)
  %dq.3.struct = alloca %Deque
  store %Deque %2, %Deque* %dq.3.struct
  %dq.3 = alloca %Deque*
  store %Deque* %dq.3.struct, %Deque** %dq.3
  %4 = call i32 @puts(i8* getelementptr ([16 x i8], [16 x i8]* @.str.1, i64 0, i64 0))
  %5 = sext i32 %4 to i64
  %6 = load %Deque*, %Deque** %dq.3
  %7 = call i64 @Deque_push_back(%Deque* %6, i64 10)
  %8 = load %Deque*, %Deque** %dq.3
  %9 = call i64 @Deque_len(%Deque* %8)
  %10 = call i32 @puts(i8* getelementptr ([19 x i8], [19 x i8]* @.str.2, i64 0, i64 0))
  %11 = sext i32 %10 to i64
  %12 = load %Deque*, %Deque** %dq.3
  %13 = call i64 @Deque_front(%Deque* %12)
  %14 = call i32 @puts(i8* getelementptr ([19 x i8], [19 x i8]* @.str.3, i64 0, i64 0))
  %15 = sext i32 %14 to i64
  %16 = load %Deque*, %Deque** %dq.3
  %17 = call i64 @Deque_drop(%Deque* %16)
  %18 = call i32 @puts(i8* getelementptr ([6 x i8], [6 x i8]* @.str.4, i64 0, i64 0))
  %19 = sext i32 %18 to i64
  ret i64 0
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

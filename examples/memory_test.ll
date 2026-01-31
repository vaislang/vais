; ModuleID = 'memory_test'
source_filename = "<vais>"

declare i32 @rand()
declare i64 @memcpy(i64, i64, i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare void @exit(i32)
declare i64 @malloc(i64)
declare i64 @vais_gc_bytes_allocated()
declare i64 @vais_gc_collections()
declare i64 @strlen(i8*)
declare i32 @strcmp(i8*, i8*)
declare i64 @vais_gc_print_stats()
declare i64 @vais_gc_objects_count()
declare i64 @vais_gc_remove_root(i64)
declare i32 @tolower(i32)
declare double @atof(i8*)
declare i64 @fseek(i64, i64, i64)
declare i32 @puts(i64)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @vais_gc_set_threshold(i64)
declare void @free(i64)
declare i64 @vais_gc_add_root(i64)
declare i32 @printf(i8*, ...)
declare i64 @fflush(i64)
declare i64 @strcpy(i64, i8*)
declare i32 @toupper(i32)
declare i32 @putchar(i32)
declare i32 @atoi(i8*)
declare i64 @fputs(i8*, i64)
declare i64 @labs(i64)
declare i64 @vais_gc_collect()
declare i32 @strncmp(i8*, i8*, i64)
declare i32 @usleep(i64)
declare i64 @atol(i8*)
declare i32 @fclose(i64)
declare i64 @fgetc(i64)
declare double @fabs(double)
declare i64 @strcat(i64, i8*)
declare i32 @sched_yield()
declare i32 @isdigit(i32)
declare i64 @feof(i64)
declare i64 @vais_gc_init()
declare i64 @vais_gc_alloc(i64, i32)
declare i64 @ftell(i64)
declare i64 @fgets(i64, i64, i64)
declare i64 @memcpy_str(i64, i8*, i64)
declare void @srand(i32)
declare i64 @fopen(i8*, i8*)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare i64 @fputc(i64, i64)
declare double @sqrt(double)
declare i32 @isalpha(i32)
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

@.str.0 = private unnamed_addr constant [31 x i8] c"=== Memory Management Test ===\00"
@.str.1 = private unnamed_addr constant [27 x i8] c"Part 1: Direct malloc/free\00"
@.str.2 = private unnamed_addr constant [21 x i8] c"  Allocated 32 bytes\00"
@.str.3 = private unnamed_addr constant [22 x i8] c"  Stored and loaded: \00"
@.str.4 = private unnamed_addr constant [15 x i8] c"  Freed memory\00"
@.str.5 = private unnamed_addr constant [27 x i8] c"Part 2: Reference Counting\00"
@.str.6 = private unnamed_addr constant [41 x i8] c"  Created Rc with value 100, ref_count=1\00"
@.str.7 = private unnamed_addr constant [21 x i8] c"  Cloned: ref_count=\00"
@.str.8 = private unnamed_addr constant [24 x i8] c"  Release 1: ref_count=\00"
@.str.9 = private unnamed_addr constant [30 x i8] c"  Release 2: Freed (last ref)\00"
@.str.10 = private unnamed_addr constant [31 x i8] c"Part 3: Arena-style allocation\00"
@.str.11 = private unnamed_addr constant [32 x i8] c"  Created arena with 1024 bytes\00"
@.str.12 = private unnamed_addr constant [24 x i8] c"  Allocated 3 objects: \00"
@.str.13 = private unnamed_addr constant [3 x i8] c", \00"
@.str.14 = private unnamed_addr constant [3 x i8] c", \00"
@.str.15 = private unnamed_addr constant [23 x i8] c"  Arena used: 24 bytes\00"
@.str.16 = private unnamed_addr constant [21 x i8] c"  Freed entire arena\00"
@.str.17 = private unnamed_addr constant [29 x i8] c"Part 4: Box single ownership\00"
@.str.18 = private unnamed_addr constant [18 x i8] c"  Created Box(42)\00"
@.str.19 = private unnamed_addr constant [16 x i8] c"  Moved to box2\00"
@.str.20 = private unnamed_addr constant [18 x i8] c"  Value in box2: \00"
@.str.21 = private unnamed_addr constant [15 x i8] c"  Dropped box2\00"
@.str.22 = private unnamed_addr constant [25 x i8] c"=== All Tests Passed ===\00"

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([31 x i8], [31 x i8]* @.str.0, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = trunc i64 10 to i32
  %3 = call i32 @putchar(i32 %2)
  %4 = sext i32 %3 to i64
  %5 = call i32 @puts(i8* getelementptr ([27 x i8], [27 x i8]* @.str.1, i64 0, i64 0))
  %6 = sext i32 %5 to i64
  %7 = call i8* @malloc(i64 32)
  %8 = ptrtoint i8* %7 to i64
  %9 = call i32 @puts(i8* getelementptr ([21 x i8], [21 x i8]* @.str.2, i64 0, i64 0))
  %10 = sext i32 %9 to i64
  call void @__store_i64(i64 %8, i64 12345)
  %11 = call i64 @__load_i64(i64 %8)
  %12 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.3, i64 0, i64 0))
  %13 = sext i32 %12 to i64
  %14 = call i64 @print_num(i64 %11)
  %15 = trunc i64 10 to i32
  %16 = call i32 @putchar(i32 %15)
  %17 = sext i32 %16 to i64
  %18 = inttoptr i64 %8 to i8*
  call void @free(i8* %18)
  %19 = call i32 @puts(i8* getelementptr ([15 x i8], [15 x i8]* @.str.4, i64 0, i64 0))
  %20 = sext i32 %19 to i64
  %21 = trunc i64 10 to i32
  %22 = call i32 @putchar(i32 %21)
  %23 = sext i32 %22 to i64
  %24 = call i32 @puts(i8* getelementptr ([27 x i8], [27 x i8]* @.str.5, i64 0, i64 0))
  %25 = sext i32 %24 to i64
  %26 = call i8* @malloc(i64 16)
  %27 = ptrtoint i8* %26 to i64
  call void @__store_i64(i64 %27, i64 1)
  %28 = add i64 %27, 8
  call void @__store_i64(i64 %28, i64 100)
  %29 = call i32 @puts(i8* getelementptr ([41 x i8], [41 x i8]* @.str.6, i64 0, i64 0))
  %30 = sext i32 %29 to i64
  %31 = call i64 @__load_i64(i64 %27)
  %32 = add i64 %31, 1
  call void @__store_i64(i64 %27, i64 %32)
  %33 = call i32 @puts(i8* getelementptr ([21 x i8], [21 x i8]* @.str.7, i64 0, i64 0))
  %34 = sext i32 %33 to i64
  %35 = call i64 @__load_i64(i64 %27)
  %36 = call i64 @print_num(i64 %35)
  %37 = trunc i64 10 to i32
  %38 = call i32 @putchar(i32 %37)
  %39 = sext i32 %38 to i64
  %40 = call i64 @__load_i64(i64 %27)
  %41 = sub i64 %40, 1
  call void @__store_i64(i64 %27, i64 %41)
  %42 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.8, i64 0, i64 0))
  %43 = sext i32 %42 to i64
  %44 = call i64 @__load_i64(i64 %27)
  %45 = call i64 @print_num(i64 %44)
  %46 = trunc i64 10 to i32
  %47 = call i32 @putchar(i32 %46)
  %48 = sext i32 %47 to i64
  %49 = call i64 @__load_i64(i64 %27)
  %50 = icmp sle i64 %49, 1
  %51 = zext i1 %50 to i64
  %52 = icmp ne i64 %51, 0
  br i1 %52, label %then0, label %else1
then0:
  %53 = inttoptr i64 %27 to i8*
  call void @free(i8* %53)
  %54 = call i32 @puts(i8* getelementptr ([30 x i8], [30 x i8]* @.str.9, i64 0, i64 0))
  %55 = sext i32 %54 to i64
  br label %merge2
else1:
  %56 = sub i64 %49, 1
  call void @__store_i64(i64 %27, i64 %56)
  br label %merge2
merge2:
  %57 = phi i64 [ 0, %then0 ], [ 0, %else1 ]
  %58 = trunc i64 10 to i32
  %59 = call i32 @putchar(i32 %58)
  %60 = sext i32 %59 to i64
  %61 = call i32 @puts(i8* getelementptr ([31 x i8], [31 x i8]* @.str.10, i64 0, i64 0))
  %62 = sext i32 %61 to i64
  %63 = call i8* @malloc(i64 1024)
  %64 = ptrtoint i8* %63 to i64
  %65 = call i32 @puts(i8* getelementptr ([32 x i8], [32 x i8]* @.str.11, i64 0, i64 0))
  %66 = sext i32 %65 to i64
  %67 = add i64 %64, 0
  call void @__store_i64(i64 %67, i64 111)
  %68 = add i64 %64, 8
  call void @__store_i64(i64 %68, i64 222)
  %69 = add i64 %64, 16
  call void @__store_i64(i64 %69, i64 333)
  %70 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.12, i64 0, i64 0))
  %71 = sext i32 %70 to i64
  %72 = call i64 @__load_i64(i64 %67)
  %73 = call i64 @__load_i64(i64 %68)
  %74 = call i64 @__load_i64(i64 %69)
  %75 = call i64 @print_num(i64 %72)
  %76 = call i32 @puts(i8* getelementptr ([3 x i8], [3 x i8]* @.str.13, i64 0, i64 0))
  %77 = sext i32 %76 to i64
  %78 = call i64 @print_num(i64 %73)
  %79 = call i32 @puts(i8* getelementptr ([3 x i8], [3 x i8]* @.str.14, i64 0, i64 0))
  %80 = sext i32 %79 to i64
  %81 = call i64 @print_num(i64 %74)
  %82 = trunc i64 10 to i32
  %83 = call i32 @putchar(i32 %82)
  %84 = sext i32 %83 to i64
  %85 = call i32 @puts(i8* getelementptr ([23 x i8], [23 x i8]* @.str.15, i64 0, i64 0))
  %86 = sext i32 %85 to i64
  %87 = trunc i64 10 to i32
  %88 = call i32 @putchar(i32 %87)
  %89 = sext i32 %88 to i64
  %90 = inttoptr i64 %64 to i8*
  call void @free(i8* %90)
  %91 = call i32 @puts(i8* getelementptr ([21 x i8], [21 x i8]* @.str.16, i64 0, i64 0))
  %92 = sext i32 %91 to i64
  %93 = trunc i64 10 to i32
  %94 = call i32 @putchar(i32 %93)
  %95 = sext i32 %94 to i64
  %96 = call i32 @puts(i8* getelementptr ([29 x i8], [29 x i8]* @.str.17, i64 0, i64 0))
  %97 = sext i32 %96 to i64
  %98 = call i8* @malloc(i64 8)
  %99 = ptrtoint i8* %98 to i64
  call void @__store_i64(i64 %99, i64 42)
  %100 = call i32 @puts(i8* getelementptr ([18 x i8], [18 x i8]* @.str.18, i64 0, i64 0))
  %101 = sext i32 %100 to i64
  %102 = call i32 @puts(i8* getelementptr ([16 x i8], [16 x i8]* @.str.19, i64 0, i64 0))
  %103 = sext i32 %102 to i64
  %104 = call i64 @__load_i64(i64 %99)
  %105 = call i32 @puts(i8* getelementptr ([18 x i8], [18 x i8]* @.str.20, i64 0, i64 0))
  %106 = sext i32 %105 to i64
  %107 = call i64 @print_num(i64 %104)
  %108 = trunc i64 10 to i32
  %109 = call i32 @putchar(i32 %108)
  %110 = sext i32 %109 to i64
  %111 = inttoptr i64 %99 to i8*
  call void @free(i8* %111)
  %112 = call i32 @puts(i8* getelementptr ([15 x i8], [15 x i8]* @.str.21, i64 0, i64 0))
  %113 = sext i32 %112 to i64
  %114 = trunc i64 10 to i32
  %115 = call i32 @putchar(i32 %114)
  %116 = sext i32 %115 to i64
  %117 = call i32 @puts(i8* getelementptr ([25 x i8], [25 x i8]* @.str.22, i64 0, i64 0))
  %118 = sext i32 %117 to i64
  ret i64 0
}

define i64 @print_num(i64 %n) {
entry:
  %0 = icmp sge i64 %n, 10000
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %then0, label %else1
then0:
  %3 = sdiv i64 %n, 10000
  %4 = add i64 %3, 48
  %5 = trunc i64 %4 to i32
  %6 = call i32 @putchar(i32 %5)
  %7 = sext i32 %6 to i64
  br label %merge2
else1:
  br label %merge2
merge2:
  %8 = add i64 0, 0
  %9 = icmp sge i64 %n, 1000
  %10 = zext i1 %9 to i64
  %11 = icmp ne i64 %10, 0
  br i1 %11, label %then3, label %else4
then3:
  %12 = sdiv i64 %n, 1000
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
  %19 = icmp sge i64 %n, 100
  %20 = zext i1 %19 to i64
  %21 = icmp ne i64 %20, 0
  br i1 %21, label %then6, label %else7
then6:
  %22 = sdiv i64 %n, 100
  %23 = srem i64 %22, 10
  %24 = add i64 %23, 48
  %25 = trunc i64 %24 to i32
  %26 = call i32 @putchar(i32 %25)
  %27 = sext i32 %26 to i64
  br label %merge8
else7:
  br label %merge8
merge8:
  %28 = add i64 0, 0
  %29 = icmp sge i64 %n, 10
  %30 = zext i1 %29 to i64
  %31 = icmp ne i64 %30, 0
  br i1 %31, label %then9, label %else10
then9:
  %32 = sdiv i64 %n, 10
  %33 = srem i64 %32, 10
  %34 = add i64 %33, 48
  %35 = trunc i64 %34 to i32
  %36 = call i32 @putchar(i32 %35)
  %37 = sext i32 %36 to i64
  br label %merge11
else10:
  br label %merge11
merge11:
  %38 = add i64 0, 0
  %39 = srem i64 %n, 10
  %40 = add i64 %39, 48
  %41 = trunc i64 %40 to i32
  %42 = call i32 @putchar(i32 %41)
  %43 = sext i32 %42 to i64
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

; ModuleID = 'memory_test'
source_filename = "<vais>"

declare void @exit(i32)
declare i64 @fputs(i8*, i64)
declare i32 @fclose(i64)
declare i32 @usleep(i64)
declare i64 @strlen(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @fputc(i64, i64)
declare i32 @strncmp(i8*, i8*, i64)
declare void @free(i64)
declare i64 @memcpy(i64, i64, i64)
declare i64 @fflush(i64)
declare i64 @feof(i64)
declare i32 @sched_yield()
declare i64 @fseek(i64, i64, i64)
declare i64 @ftell(i64)
declare i32 @putchar(i32)
declare i32 @puts(i8*)
declare i64 @malloc(i64)
declare i64 @fgetc(i64)
declare i64 @fgets(i64, i64, i64)
declare i32 @strcmp(i8*, i8*)
declare i64 @fopen(i8*, i8*)
declare i64 @fread(i64, i64, i64, i64)
declare i32 @printf(i8*)
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
  %1 = trunc i64 10 to i32
  %2 = call i32 @putchar(i32 %1)
  %3 = call i32 @puts(i8* getelementptr ([27 x i8], [27 x i8]* @.str.1, i64 0, i64 0))
  %4 = call i8* @malloc(i64 32)
  %5 = ptrtoint i8* %4 to i64
  %ptr1.6 = alloca i64
  store i64 %5, i64* %ptr1.6
  %7 = call i32 @puts(i8* getelementptr ([21 x i8], [21 x i8]* @.str.2, i64 0, i64 0))
  %8 = load i64, i64* %ptr1.6
  call void @__store_i64(i64 %8, i64 12345)
  %9 = load i64, i64* %ptr1.6
  %10 = call i64 @__load_i64(i64 %9)
  %val1.11 = alloca i64
  store i64 %10, i64* %val1.11
  %12 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.3, i64 0, i64 0))
  %13 = load i64, i64* %val1.11
  %14 = call i64 @print_num(i64 %13)
  %15 = trunc i64 10 to i32
  %16 = call i32 @putchar(i32 %15)
  %17 = load i64, i64* %ptr1.6
  %18 = inttoptr i64 %17 to i8*
  call void @free(i8* %18)
  %19 = call i32 @puts(i8* getelementptr ([15 x i8], [15 x i8]* @.str.4, i64 0, i64 0))
  %20 = trunc i64 10 to i32
  %21 = call i32 @putchar(i32 %20)
  %22 = call i32 @puts(i8* getelementptr ([27 x i8], [27 x i8]* @.str.5, i64 0, i64 0))
  %23 = call i8* @malloc(i64 16)
  %24 = ptrtoint i8* %23 to i64
  %rc.25 = alloca i64
  store i64 %24, i64* %rc.25
  %26 = load i64, i64* %rc.25
  call void @__store_i64(i64 %26, i64 1)
  %27 = load i64, i64* %rc.25
  %28 = add i64 %27, 8
  call void @__store_i64(i64 %28, i64 100)
  %29 = call i32 @puts(i8* getelementptr ([41 x i8], [41 x i8]* @.str.6, i64 0, i64 0))
  %30 = load i64, i64* %rc.25
  %31 = call i64 @__load_i64(i64 %30)
  %count1.32 = alloca i64
  store i64 %31, i64* %count1.32
  %33 = load i64, i64* %rc.25
  %34 = load i64, i64* %count1.32
  %35 = add i64 %34, 1
  call void @__store_i64(i64 %33, i64 %35)
  %36 = call i32 @puts(i8* getelementptr ([21 x i8], [21 x i8]* @.str.7, i64 0, i64 0))
  %37 = load i64, i64* %rc.25
  %38 = call i64 @__load_i64(i64 %37)
  %count2.39 = alloca i64
  store i64 %38, i64* %count2.39
  %40 = load i64, i64* %count2.39
  %41 = call i64 @print_num(i64 %40)
  %42 = trunc i64 10 to i32
  %43 = call i32 @putchar(i32 %42)
  %44 = load i64, i64* %rc.25
  %45 = call i64 @__load_i64(i64 %44)
  %count3.46 = alloca i64
  store i64 %45, i64* %count3.46
  %47 = load i64, i64* %rc.25
  %48 = load i64, i64* %count3.46
  %49 = sub i64 %48, 1
  call void @__store_i64(i64 %47, i64 %49)
  %50 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.8, i64 0, i64 0))
  %51 = load i64, i64* %rc.25
  %52 = call i64 @__load_i64(i64 %51)
  %count4.53 = alloca i64
  store i64 %52, i64* %count4.53
  %54 = load i64, i64* %count4.53
  %55 = call i64 @print_num(i64 %54)
  %56 = trunc i64 10 to i32
  %57 = call i32 @putchar(i32 %56)
  %58 = load i64, i64* %rc.25
  %59 = call i64 @__load_i64(i64 %58)
  %count5.60 = alloca i64
  store i64 %59, i64* %count5.60
  %61 = load i64, i64* %count5.60
  %62 = icmp sle i64 %61, 1
  %63 = zext i1 %62 to i64
  %64 = icmp ne i64 %63, 0
  br i1 %64, label %then0, label %else1
then0:
  %65 = load i64, i64* %rc.25
  %66 = inttoptr i64 %65 to i8*
  call void @free(i8* %66)
  %67 = call i32 @puts(i8* getelementptr ([30 x i8], [30 x i8]* @.str.9, i64 0, i64 0))
  br label %merge2
else1:
  %68 = load i64, i64* %rc.25
  %69 = load i64, i64* %count5.60
  %70 = sub i64 %69, 1
  call void @__store_i64(i64 %68, i64 %70)
  br label %merge2
merge2:
  %71 = phi i64 [ 0, %then0 ], [ 0, %else1 ]
  %72 = trunc i64 10 to i32
  %73 = call i32 @putchar(i32 %72)
  %74 = call i32 @puts(i8* getelementptr ([31 x i8], [31 x i8]* @.str.10, i64 0, i64 0))
  %arena_size.75 = alloca i64
  store i64 1024, i64* %arena_size.75
  %76 = load i64, i64* %arena_size.75
  %77 = call i8* @malloc(i64 %76)
  %78 = ptrtoint i8* %77 to i64
  %arena.79 = alloca i64
  store i64 %78, i64* %arena.79
  %80 = call i32 @puts(i8* getelementptr ([32 x i8], [32 x i8]* @.str.11, i64 0, i64 0))
  %81 = load i64, i64* %arena.79
  %82 = add i64 %81, 0
  %obj1.83 = alloca i64
  store i64 %82, i64* %obj1.83
  %84 = load i64, i64* %obj1.83
  call void @__store_i64(i64 %84, i64 111)
  %85 = load i64, i64* %arena.79
  %86 = add i64 %85, 8
  %obj2.87 = alloca i64
  store i64 %86, i64* %obj2.87
  %88 = load i64, i64* %obj2.87
  call void @__store_i64(i64 %88, i64 222)
  %89 = load i64, i64* %arena.79
  %90 = add i64 %89, 16
  %obj3.91 = alloca i64
  store i64 %90, i64* %obj3.91
  %92 = load i64, i64* %obj3.91
  call void @__store_i64(i64 %92, i64 333)
  %93 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.12, i64 0, i64 0))
  %94 = load i64, i64* %obj1.83
  %95 = call i64 @__load_i64(i64 %94)
  %v1.96 = alloca i64
  store i64 %95, i64* %v1.96
  %97 = load i64, i64* %obj2.87
  %98 = call i64 @__load_i64(i64 %97)
  %v2.99 = alloca i64
  store i64 %98, i64* %v2.99
  %100 = load i64, i64* %obj3.91
  %101 = call i64 @__load_i64(i64 %100)
  %v3.102 = alloca i64
  store i64 %101, i64* %v3.102
  %103 = load i64, i64* %v1.96
  %104 = call i64 @print_num(i64 %103)
  %105 = call i32 @puts(i8* getelementptr ([3 x i8], [3 x i8]* @.str.13, i64 0, i64 0))
  %106 = load i64, i64* %v2.99
  %107 = call i64 @print_num(i64 %106)
  %108 = call i32 @puts(i8* getelementptr ([3 x i8], [3 x i8]* @.str.14, i64 0, i64 0))
  %109 = load i64, i64* %v3.102
  %110 = call i64 @print_num(i64 %109)
  %111 = trunc i64 10 to i32
  %112 = call i32 @putchar(i32 %111)
  %113 = call i32 @puts(i8* getelementptr ([23 x i8], [23 x i8]* @.str.15, i64 0, i64 0))
  %114 = trunc i64 10 to i32
  %115 = call i32 @putchar(i32 %114)
  %116 = load i64, i64* %arena.79
  %117 = inttoptr i64 %116 to i8*
  call void @free(i8* %117)
  %118 = call i32 @puts(i8* getelementptr ([21 x i8], [21 x i8]* @.str.16, i64 0, i64 0))
  %119 = trunc i64 10 to i32
  %120 = call i32 @putchar(i32 %119)
  %121 = call i32 @puts(i8* getelementptr ([29 x i8], [29 x i8]* @.str.17, i64 0, i64 0))
  %122 = call i8* @malloc(i64 8)
  %123 = ptrtoint i8* %122 to i64
  %box1.124 = alloca i64
  store i64 %123, i64* %box1.124
  %125 = load i64, i64* %box1.124
  call void @__store_i64(i64 %125, i64 42)
  %126 = call i32 @puts(i8* getelementptr ([18 x i8], [18 x i8]* @.str.18, i64 0, i64 0))
  %127 = load i64, i64* %box1.124
  %box2.128 = alloca i64
  store i64 %127, i64* %box2.128
  %129 = call i32 @puts(i8* getelementptr ([16 x i8], [16 x i8]* @.str.19, i64 0, i64 0))
  %130 = load i64, i64* %box2.128
  %131 = call i64 @__load_i64(i64 %130)
  %val.132 = alloca i64
  store i64 %131, i64* %val.132
  %133 = call i32 @puts(i8* getelementptr ([18 x i8], [18 x i8]* @.str.20, i64 0, i64 0))
  %134 = load i64, i64* %val.132
  %135 = call i64 @print_num(i64 %134)
  %136 = trunc i64 10 to i32
  %137 = call i32 @putchar(i32 %136)
  %138 = load i64, i64* %box2.128
  %139 = inttoptr i64 %138 to i8*
  call void @free(i8* %139)
  %140 = call i32 @puts(i8* getelementptr ([15 x i8], [15 x i8]* @.str.21, i64 0, i64 0))
  %141 = trunc i64 10 to i32
  %142 = call i32 @putchar(i32 %141)
  %143 = call i32 @puts(i8* getelementptr ([25 x i8], [25 x i8]* @.str.22, i64 0, i64 0))
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
  %10 = icmp sge i64 %n, 1000
  %11 = zext i1 %10 to i64
  %12 = icmp ne i64 %11, 0
  br i1 %12, label %then3, label %else4
then3:
  %13 = sdiv i64 %n, 1000
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
  %21 = icmp sge i64 %n, 100
  %22 = zext i1 %21 to i64
  %23 = icmp ne i64 %22, 0
  br i1 %23, label %then6, label %else7
then6:
  %24 = sdiv i64 %n, 100
  %25 = srem i64 %24, 10
  %d3.26 = alloca i64
  store i64 %25, i64* %d3.26
  %27 = load i64, i64* %d3.26
  %28 = add i64 %27, 48
  %29 = trunc i64 %28 to i32
  %30 = call i32 @putchar(i32 %29)
  br label %merge8
else7:
  br label %merge8
merge8:
  %31 = add i64 0, 0
  %32 = icmp sge i64 %n, 10
  %33 = zext i1 %32 to i64
  %34 = icmp ne i64 %33, 0
  br i1 %34, label %then9, label %else10
then9:
  %35 = sdiv i64 %n, 10
  %36 = srem i64 %35, 10
  %d4.37 = alloca i64
  store i64 %36, i64* %d4.37
  %38 = load i64, i64* %d4.37
  %39 = add i64 %38, 48
  %40 = trunc i64 %39 to i32
  %41 = call i32 @putchar(i32 %40)
  br label %merge11
else10:
  br label %merge11
merge11:
  %42 = add i64 0, 0
  %43 = srem i64 %n, 10
  %d5.44 = alloca i64
  store i64 %43, i64* %d5.44
  %45 = load i64, i64* %d5.44
  %46 = add i64 %45, 48
  %47 = trunc i64 %46 to i32
  %48 = call i32 @putchar(i32 %47)
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

; ModuleID = 'rc_test'
source_filename = "<vais>"

declare i32 @puts(i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @ftell(i64)
declare i64 @fopen(i8*, i8*)
declare i32 @usleep(i64)
declare void @exit(i32)
declare i64 @fgets(i64, i64, i64)
declare i64 @malloc(i64)
declare i64 @fputs(i8*, i64)
declare i64 @strlen(i64)
declare i64 @memcpy(i64, i64, i64)
declare i64 @fflush(i64)
declare i32 @sched_yield()
declare i64 @feof(i64)
declare i64 @fputc(i64, i64)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @fseek(i64, i64, i64)
declare i32 @strcmp(i8*, i8*)
declare void @free(i64)
declare i64 @fgetc(i64)
declare i32 @printf(i8*)
declare i32 @fclose(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i32 @putchar(i32)
@.str.0 = private unnamed_addr constant [34 x i8] c"=== RC Memory Management Test ===\00"
@.str.1 = private unnamed_addr constant [32 x i8] c"Test 1: Create Rc with value 42\00"
@.str.2 = private unnamed_addr constant [14 x i8] c"  ref_count: \00"
@.str.3 = private unnamed_addr constant [10 x i8] c"  value: \00"
@.str.4 = private unnamed_addr constant [39 x i8] c"Test 2: Clone Rc (increment ref_count)\00"
@.str.5 = private unnamed_addr constant [26 x i8] c"  ref_count after clone: \00"
@.str.6 = private unnamed_addr constant [30 x i8] c"Test 3: Release one reference\00"
@.str.7 = private unnamed_addr constant [28 x i8] c"  ref_count after release: \00"
@.str.8 = private unnamed_addr constant [31 x i8] c"Test 4: Release last reference\00"
@.str.9 = private unnamed_addr constant [34 x i8] c"  Freeing memory (last reference)\00"
@.str.10 = private unnamed_addr constant [29 x i8] c"Test 5: Box single ownership\00"
@.str.11 = private unnamed_addr constant [14 x i8] c"  Box value: \00"
@.str.12 = private unnamed_addr constant [14 x i8] c"  Freeing Box\00"
@.str.13 = private unnamed_addr constant [22 x i8] c"=== Test Complete ===\00"

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([34 x i8], [34 x i8]* @.str.0, i64 0, i64 0))
  %1 = trunc i64 10 to i32
  %2 = call i32 @putchar(i32 %1)
  %3 = call i32 @puts(i8* getelementptr ([32 x i8], [32 x i8]* @.str.1, i64 0, i64 0))
  %4 = call i8* @malloc(i64 16)
  %5 = ptrtoint i8* %4 to i64
  %rc_ptr.6 = alloca i64
  store i64 %5, i64* %rc_ptr.6
  %7 = load i64, i64* %rc_ptr.6
  call void @__store_i64(i64 %7, i64 1)
  %8 = load i64, i64* %rc_ptr.6
  %9 = add i64 %8, 8
  call void @__store_i64(i64 %9, i64 42)
  %10 = call i32 @puts(i8* getelementptr ([14 x i8], [14 x i8]* @.str.2, i64 0, i64 0))
  %11 = load i64, i64* %rc_ptr.6
  %12 = call i64 @__load_i64(i64 %11)
  %count1.13 = alloca i64
  store i64 %12, i64* %count1.13
  %14 = load i64, i64* %count1.13
  %15 = call i64 @print_num(i64 %14)
  %16 = trunc i64 10 to i32
  %17 = call i32 @putchar(i32 %16)
  %18 = call i32 @puts(i8* getelementptr ([10 x i8], [10 x i8]* @.str.3, i64 0, i64 0))
  %19 = load i64, i64* %rc_ptr.6
  %20 = add i64 %19, 8
  %21 = call i64 @__load_i64(i64 %20)
  %val.22 = alloca i64
  store i64 %21, i64* %val.22
  %23 = load i64, i64* %val.22
  %24 = call i64 @print_num(i64 %23)
  %25 = trunc i64 10 to i32
  %26 = call i32 @putchar(i32 %25)
  %27 = call i32 @puts(i8* getelementptr ([39 x i8], [39 x i8]* @.str.4, i64 0, i64 0))
  %28 = load i64, i64* %rc_ptr.6
  %29 = call i64 @__load_i64(i64 %28)
  %count2.30 = alloca i64
  store i64 %29, i64* %count2.30
  %31 = load i64, i64* %rc_ptr.6
  %32 = load i64, i64* %count2.30
  %33 = add i64 %32, 1
  call void @__store_i64(i64 %31, i64 %33)
  %34 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.5, i64 0, i64 0))
  %35 = load i64, i64* %rc_ptr.6
  %36 = call i64 @__load_i64(i64 %35)
  %count3.37 = alloca i64
  store i64 %36, i64* %count3.37
  %38 = load i64, i64* %count3.37
  %39 = call i64 @print_num(i64 %38)
  %40 = trunc i64 10 to i32
  %41 = call i32 @putchar(i32 %40)
  %42 = call i32 @puts(i8* getelementptr ([30 x i8], [30 x i8]* @.str.6, i64 0, i64 0))
  %43 = load i64, i64* %rc_ptr.6
  %44 = call i64 @__load_i64(i64 %43)
  %count4.45 = alloca i64
  store i64 %44, i64* %count4.45
  %46 = load i64, i64* %rc_ptr.6
  %47 = load i64, i64* %count4.45
  %48 = sub i64 %47, 1
  call void @__store_i64(i64 %46, i64 %48)
  %49 = call i32 @puts(i8* getelementptr ([28 x i8], [28 x i8]* @.str.7, i64 0, i64 0))
  %50 = load i64, i64* %rc_ptr.6
  %51 = call i64 @__load_i64(i64 %50)
  %count5.52 = alloca i64
  store i64 %51, i64* %count5.52
  %53 = load i64, i64* %count5.52
  %54 = call i64 @print_num(i64 %53)
  %55 = trunc i64 10 to i32
  %56 = call i32 @putchar(i32 %55)
  %57 = call i32 @puts(i8* getelementptr ([31 x i8], [31 x i8]* @.str.8, i64 0, i64 0))
  %58 = load i64, i64* %rc_ptr.6
  %59 = call i64 @__load_i64(i64 %58)
  %count6.60 = alloca i64
  store i64 %59, i64* %count6.60
  %61 = load i64, i64* %count6.60
  %62 = icmp sle i64 %61, 1
  %63 = zext i1 %62 to i64
  %64 = icmp ne i64 %63, 0
  br i1 %64, label %then0, label %else1
then0:
  %65 = call i32 @puts(i8* getelementptr ([34 x i8], [34 x i8]* @.str.9, i64 0, i64 0))
  %66 = load i64, i64* %rc_ptr.6
  %67 = inttoptr i64 %66 to i8*
  call void @free(i8* %67)
  br label %merge2
else1:
  %68 = load i64, i64* %rc_ptr.6
  %69 = load i64, i64* %count6.60
  %70 = sub i64 %69, 1
  call void @__store_i64(i64 %68, i64 %70)
  br label %merge2
merge2:
  %71 = phi i64 [ 0, %then0 ], [ 0, %else1 ]
  %72 = trunc i64 10 to i32
  %73 = call i32 @putchar(i32 %72)
  %74 = call i32 @puts(i8* getelementptr ([29 x i8], [29 x i8]* @.str.10, i64 0, i64 0))
  %75 = call i8* @malloc(i64 8)
  %76 = ptrtoint i8* %75 to i64
  %box_ptr.77 = alloca i64
  store i64 %76, i64* %box_ptr.77
  %78 = load i64, i64* %box_ptr.77
  call void @__store_i64(i64 %78, i64 100)
  %79 = call i32 @puts(i8* getelementptr ([14 x i8], [14 x i8]* @.str.11, i64 0, i64 0))
  %80 = load i64, i64* %box_ptr.77
  %81 = call i64 @__load_i64(i64 %80)
  %box_val.82 = alloca i64
  store i64 %81, i64* %box_val.82
  %83 = load i64, i64* %box_val.82
  %84 = call i64 @print_num(i64 %83)
  %85 = trunc i64 10 to i32
  %86 = call i32 @putchar(i32 %85)
  %87 = call i32 @puts(i8* getelementptr ([14 x i8], [14 x i8]* @.str.12, i64 0, i64 0))
  %88 = load i64, i64* %box_ptr.77
  %89 = inttoptr i64 %88 to i8*
  call void @free(i8* %89)
  %90 = trunc i64 10 to i32
  %91 = call i32 @putchar(i32 %90)
  %92 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.13, i64 0, i64 0))
  ret i64 0
}

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

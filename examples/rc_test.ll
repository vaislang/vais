; ModuleID = 'rc_test'
source_filename = "<vais>"

declare i64 @labs(i64)
declare double @sqrt(double)
declare i64 @memcpy_str(i64, i8*, i64)
declare i32 @usleep(i64)
declare void @exit(i32)
declare i64 @fopen(i8*, i8*)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @strcat(i64, i8*)
declare i32 @isdigit(i32)
declare i64 @vais_gc_add_root(i64)
declare i64 @vais_gc_objects_count()
declare i32 @toupper(i32)
declare i64 @vais_gc_collections()
declare i64 @fgetc(i64)
declare i64 @fgets(i64, i64, i64)
declare i64 @feof(i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @atol(i8*)
declare i32 @tolower(i32)
declare i32 @isalpha(i32)
declare i64 @vais_gc_remove_root(i64)
declare i32 @printf(i8*, ...)
declare i64 @fseek(i64, i64, i64)
declare double @fabs(double)
declare i64 @strcpy(i64, i8*)
declare i64 @vais_gc_alloc(i64, i32)
declare i64 @vais_gc_collect()
declare i32 @strcmp(i8*, i8*)
declare i64 @fwrite(i64, i64, i64, i64)
declare void @srand(i32)
declare i64 @malloc(i64)
declare i64 @vais_gc_print_stats()
declare void @free(i64)
declare i64 @memcpy(i64, i64, i64)
declare i32 @fclose(i64)
declare i64 @ftell(i64)
declare i32 @rand()
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare i64 @strlen(i8*)
declare i64 @fputc(i64, i64)
declare double @atof(i8*)
declare i32 @sched_yield()
declare i32 @putchar(i32)
declare i64 @fflush(i64)
declare i32 @atoi(i8*)
declare i64 @fputs(i8*, i64)
declare i64 @vais_gc_init()
declare i32 @puts(i64)
declare i64 @vais_gc_bytes_allocated()
declare i64 @vais_gc_set_threshold(i64)
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

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
  %1 = sext i32 %0 to i64
  %2 = trunc i64 10 to i32
  %3 = call i32 @putchar(i32 %2)
  %4 = sext i32 %3 to i64
  %5 = call i32 @puts(i8* getelementptr ([32 x i8], [32 x i8]* @.str.1, i64 0, i64 0))
  %6 = sext i32 %5 to i64
  %7 = call i8* @malloc(i64 16)
  %8 = ptrtoint i8* %7 to i64
  call void @__store_i64(i64 %8, i64 1)
  %9 = add i64 %8, 8
  call void @__store_i64(i64 %9, i64 42)
  %10 = call i32 @puts(i8* getelementptr ([14 x i8], [14 x i8]* @.str.2, i64 0, i64 0))
  %11 = sext i32 %10 to i64
  %12 = call i64 @__load_i64(i64 %8)
  %13 = call i64 @print_num(i64 %12)
  %14 = trunc i64 10 to i32
  %15 = call i32 @putchar(i32 %14)
  %16 = sext i32 %15 to i64
  %17 = call i32 @puts(i8* getelementptr ([10 x i8], [10 x i8]* @.str.3, i64 0, i64 0))
  %18 = sext i32 %17 to i64
  %19 = add i64 %8, 8
  %20 = call i64 @__load_i64(i64 %19)
  %21 = call i64 @print_num(i64 %20)
  %22 = trunc i64 10 to i32
  %23 = call i32 @putchar(i32 %22)
  %24 = sext i32 %23 to i64
  %25 = call i32 @puts(i8* getelementptr ([39 x i8], [39 x i8]* @.str.4, i64 0, i64 0))
  %26 = sext i32 %25 to i64
  %27 = call i64 @__load_i64(i64 %8)
  %28 = add i64 %27, 1
  call void @__store_i64(i64 %8, i64 %28)
  %29 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.5, i64 0, i64 0))
  %30 = sext i32 %29 to i64
  %31 = call i64 @__load_i64(i64 %8)
  %32 = call i64 @print_num(i64 %31)
  %33 = trunc i64 10 to i32
  %34 = call i32 @putchar(i32 %33)
  %35 = sext i32 %34 to i64
  %36 = call i32 @puts(i8* getelementptr ([30 x i8], [30 x i8]* @.str.6, i64 0, i64 0))
  %37 = sext i32 %36 to i64
  %38 = call i64 @__load_i64(i64 %8)
  %39 = sub i64 %38, 1
  call void @__store_i64(i64 %8, i64 %39)
  %40 = call i32 @puts(i8* getelementptr ([28 x i8], [28 x i8]* @.str.7, i64 0, i64 0))
  %41 = sext i32 %40 to i64
  %42 = call i64 @__load_i64(i64 %8)
  %43 = call i64 @print_num(i64 %42)
  %44 = trunc i64 10 to i32
  %45 = call i32 @putchar(i32 %44)
  %46 = sext i32 %45 to i64
  %47 = call i32 @puts(i8* getelementptr ([31 x i8], [31 x i8]* @.str.8, i64 0, i64 0))
  %48 = sext i32 %47 to i64
  %49 = call i64 @__load_i64(i64 %8)
  %50 = icmp sle i64 %49, 1
  %51 = zext i1 %50 to i64
  %52 = icmp ne i64 %51, 0
  br i1 %52, label %then0, label %else1
then0:
  %53 = call i32 @puts(i8* getelementptr ([34 x i8], [34 x i8]* @.str.9, i64 0, i64 0))
  %54 = sext i32 %53 to i64
  %55 = inttoptr i64 %8 to i8*
  call void @free(i8* %55)
  br label %merge2
else1:
  %56 = sub i64 %49, 1
  call void @__store_i64(i64 %8, i64 %56)
  br label %merge2
merge2:
  %57 = phi i64 [ 0, %then0 ], [ 0, %else1 ]
  %58 = trunc i64 10 to i32
  %59 = call i32 @putchar(i32 %58)
  %60 = sext i32 %59 to i64
  %61 = call i32 @puts(i8* getelementptr ([29 x i8], [29 x i8]* @.str.10, i64 0, i64 0))
  %62 = sext i32 %61 to i64
  %63 = call i8* @malloc(i64 8)
  %64 = ptrtoint i8* %63 to i64
  call void @__store_i64(i64 %64, i64 100)
  %65 = call i32 @puts(i8* getelementptr ([14 x i8], [14 x i8]* @.str.11, i64 0, i64 0))
  %66 = sext i32 %65 to i64
  %67 = call i64 @__load_i64(i64 %64)
  %68 = call i64 @print_num(i64 %67)
  %69 = trunc i64 10 to i32
  %70 = call i32 @putchar(i32 %69)
  %71 = sext i32 %70 to i64
  %72 = call i32 @puts(i8* getelementptr ([14 x i8], [14 x i8]* @.str.12, i64 0, i64 0))
  %73 = sext i32 %72 to i64
  %74 = inttoptr i64 %64 to i8*
  call void @free(i8* %74)
  %75 = trunc i64 10 to i32
  %76 = call i32 @putchar(i32 %75)
  %77 = sext i32 %76 to i64
  %78 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.13, i64 0, i64 0))
  %79 = sext i32 %78 to i64
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

; ModuleID = 'pattern_match_test'
source_filename = "<vais>"

declare i64 @fputs(i8*, i64)
declare i64 @atol(i64)
declare void @srand(i32)
declare i64 @fopen(i8*, i8*)
declare double @sqrt(double)
declare i32 @toupper(i32)
declare i64 @vais_gc_init()
declare i64 @labs(i64)
declare i32 @putchar(i32)
declare i64 @vais_gc_collect()
declare i64 @vais_gc_collections()
declare i64 @vais_gc_print_stats()
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @vais_gc_set_threshold(i64)
declare i32 @atoi(i8*)
declare i64 @fgetc(i64)
declare i32 @tolower(i32)
declare i64 @vais_gc_bytes_allocated()
declare i64 @feof(i64)
declare i32 @strcmp(i8*, i8*)
declare i64 @vais_gc_add_root(i64)
declare double @atof(i8*)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare i64 @malloc(i64)
declare i64 @vais_gc_alloc(i64, i32)
declare i64 @fwrite(i64, i64, i64, i64)
declare i32 @rand()
declare i32 @isalpha(i32)
declare i32 @puts(i64)
declare void @free(i64)
declare i64 @strcpy(i64, i8*)
declare i64 @fseek(i64, i64, i64)
declare i64 @fflush(i64)
declare i64 @ftell(i64)
declare i32 @sched_yield()
declare double @fabs(double)
declare i64 @vais_gc_objects_count()
declare i64 @memcpy(i64, i64, i64)
declare i32 @printf(i8*, ...)
declare void @exit(i32)
declare i64 @memcpy_str(i64, i8*, i64)
declare i32 @isdigit(i32)
declare i64 @vais_gc_remove_root(i64)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @fgets(i64, i64, i64)
declare i64 @fputc(i64, i64)
declare i64 @strcat(i64, i8*)
declare i32 @usleep(i64)
declare i64 @strlen(i8*)
declare i32 @fclose(i64)
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

@.str.0 = private unnamed_addr constant [26 x i8] c"Testing pattern matching:\00"
@.str.1 = private unnamed_addr constant [15 x i8] c"color_code(0):\00"
@.str.2 = private unnamed_addr constant [13 x i8] c"describe(0):\00"
@.str.3 = private unnamed_addr constant [14 x i8] c"describe(41):\00"

define i64 @color_code(i64 %c) {
entry:
  switch i64 %c, label %match.default1 [
    i64 0, label %match.arm2
    i64 1, label %match.arm3
  ]
match.arm2:
  br label %match.merge0
match.arm3:
  br label %match.merge0
match.default1:
  br label %match.merge0
match.merge0:
  %0 = phi i64 [ 255, %match.arm2 ], [ 65280, %match.arm3 ], [ 0, %match.default1 ]
  ret i64 %0
}

define i64 @describe(i64 %n) {
entry:
  br label %match.check1
match.check1:
  %0 = icmp eq i64 %n, 0
  br i1 %0, label %match.arm3, label %match.check2
match.arm3:
  br label %match.merge0
match.check2:
  br i1 1, label %match.arm4, label %match.merge0
match.arm4:
  %2 = add i64 %n, 1
  br label %match.merge0
match.merge0:
  %3 = phi i64 [ 100, %match.arm3 ], [ %2, %match.arm4 ]
  ret i64 %3
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.0, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = call i64 @color_code(i64 0)
  %3 = call i32 @puts(i8* getelementptr ([15 x i8], [15 x i8]* @.str.1, i64 0, i64 0))
  %4 = sext i32 %3 to i64
  %5 = sdiv i64 %2, 100
  %6 = add i64 %5, 48
  %7 = trunc i64 %6 to i32
  %8 = call i32 @putchar(i32 %7)
  %9 = sext i32 %8 to i64
  %10 = sdiv i64 %2, 10
  %11 = srem i64 %10, 10
  %12 = add i64 %11, 48
  %13 = trunc i64 %12 to i32
  %14 = call i32 @putchar(i32 %13)
  %15 = sext i32 %14 to i64
  %16 = srem i64 %2, 10
  %17 = add i64 %16, 48
  %18 = trunc i64 %17 to i32
  %19 = call i32 @putchar(i32 %18)
  %20 = sext i32 %19 to i64
  %21 = trunc i64 10 to i32
  %22 = call i32 @putchar(i32 %21)
  %23 = sext i32 %22 to i64
  %24 = call i64 @describe(i64 0)
  %25 = call i32 @puts(i8* getelementptr ([13 x i8], [13 x i8]* @.str.2, i64 0, i64 0))
  %26 = sext i32 %25 to i64
  %27 = sdiv i64 %24, 100
  %28 = add i64 %27, 48
  %29 = trunc i64 %28 to i32
  %30 = call i32 @putchar(i32 %29)
  %31 = sext i32 %30 to i64
  %32 = sdiv i64 %24, 10
  %33 = srem i64 %32, 10
  %34 = add i64 %33, 48
  %35 = trunc i64 %34 to i32
  %36 = call i32 @putchar(i32 %35)
  %37 = sext i32 %36 to i64
  %38 = srem i64 %24, 10
  %39 = add i64 %38, 48
  %40 = trunc i64 %39 to i32
  %41 = call i32 @putchar(i32 %40)
  %42 = sext i32 %41 to i64
  %43 = trunc i64 10 to i32
  %44 = call i32 @putchar(i32 %43)
  %45 = sext i32 %44 to i64
  %46 = call i64 @describe(i64 41)
  %47 = call i32 @puts(i8* getelementptr ([14 x i8], [14 x i8]* @.str.3, i64 0, i64 0))
  %48 = sext i32 %47 to i64
  %49 = sdiv i64 %46, 10
  %50 = add i64 %49, 48
  %51 = trunc i64 %50 to i32
  %52 = call i32 @putchar(i32 %51)
  %53 = sext i32 %52 to i64
  %54 = srem i64 %46, 10
  %55 = add i64 %54, 48
  %56 = trunc i64 %55 to i32
  %57 = call i32 @putchar(i32 %56)
  %58 = sext i32 %57 to i64
  %59 = trunc i64 10 to i32
  %60 = call i32 @putchar(i32 %59)
  %61 = sext i32 %60 to i64
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

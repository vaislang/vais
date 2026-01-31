; ModuleID = 'range_test'
source_filename = "<vais>"

declare i32 @strcmp(i8*, i8*)
declare i32 @tolower(i32)
declare i64 @vais_gc_alloc(i64, i32)
declare i32 @printf(i8*, ...)
declare void @exit(i32)
declare i32 @putchar(i32)
declare i64 @strlen(i8*)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare i64 @vais_gc_remove_root(i64)
declare i64 @vais_gc_bytes_allocated()
declare i64 @vais_gc_init()
declare i64 @fseek(i64, i64, i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i32 @atoi(i8*)
declare double @fabs(double)
declare i32 @isdigit(i32)
declare i32 @toupper(i32)
declare i64 @vais_gc_set_threshold(i64)
declare i64 @labs(i64)
declare i64 @strcpy(i64, i8*)
declare i64 @fflush(i64)
declare i64 @fputc(i64, i64)
declare i64 @fgetc(i64)
declare i32 @sched_yield()
declare i32 @rand()
declare i64 @vais_gc_collections()
declare i64 @vais_gc_print_stats()
declare i64 @feof(i64)
declare i64 @malloc(i64)
declare i32 @fclose(i64)
declare i32 @puts(i8*)
declare i64 @vais_gc_add_root(i64)
declare i64 @vais_gc_objects_count()
declare i64 @memcpy(i64, i64, i64)
declare double @atof(i8*)
declare i64 @memcpy_str(i64, i8*, i64)
declare double @sqrt(double)
declare i64 @strcat(i64, i8*)
declare i64 @fgets(i64, i64, i64)
declare i64 @atol(i8*)
declare i64 @fwrite(i64, i64, i64, i64)
declare i32 @usleep(i64)
declare void @srand(i32)
declare i64 @fopen(i8*, i8*)
declare i64 @fread(i64, i64, i64, i64)
declare i32 @isalpha(i32)
declare i64 @ftell(i64)
declare void @free(i64)
declare i64 @fputs(i8*, i64)
declare i64 @vais_gc_collect()
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

@.str.0 = private unnamed_addr constant [25 x i8] c"Testing range iteration:\00"
@.str.1 = private unnamed_addr constant [10 x i8] c"Sum 0..5:\00"

define i64 @test_range() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([25 x i8], [25 x i8]* @.str.0, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %sum.2 = alloca i64
  store i64 0, i64* %sum.2
  %loop_counter.0 = alloca i64
  store i64 0, i64* %loop_counter.0
  %i.for = alloca i64
  br label %for.cond1
for.cond1:
  %3 = load i64, i64* %loop_counter.0
  %4 = icmp slt i64 %3, 5
  br i1 %4, label %for.body2, label %for.end4
for.body2:
  %5 = load i64, i64* %loop_counter.0
  store i64 %5, i64* %i.for
  %6 = load i64, i64* %sum.2
  %7 = load i64, i64* %i.for
  %8 = add i64 %6, %7
  store i64 %8, i64* %sum.2
  %9 = load i64, i64* %i.for
  %10 = add i64 %9, 48
  %11 = trunc i64 %10 to i32
  %12 = call i32 @putchar(i32 %11)
  %13 = sext i32 %12 to i64
  %14 = trunc i64 32 to i32
  %15 = call i32 @putchar(i32 %14)
  %16 = sext i32 %15 to i64
  br label %for.inc3
for.inc3:
  %17 = load i64, i64* %loop_counter.0
  %18 = add i64 %17, 1
  store i64 %18, i64* %loop_counter.0
  br label %for.cond1
for.end4:
  %19 = trunc i64 10 to i32
  %20 = call i32 @putchar(i32 %19)
  %21 = sext i32 %20 to i64
  %22 = call i32 @puts(i8* getelementptr ([10 x i8], [10 x i8]* @.str.1, i64 0, i64 0))
  %23 = sext i32 %22 to i64
  %24 = load i64, i64* %sum.2
  %25 = call i64 @print_num(i64 %24)
  %26 = trunc i64 10 to i32
  %27 = call i32 @putchar(i32 %26)
  %28 = sext i32 %27 to i64
  %29 = load i64, i64* %sum.2
  ret i64 %29
}

define i64 @print_num(i64 %n) {
entry:
  %0 = icmp sge i64 %n, 10
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %then0, label %else1
then0:
  %3 = sdiv i64 %n, 10
  %4 = add i64 %3, 48
  %5 = trunc i64 %4 to i32
  %6 = call i32 @putchar(i32 %5)
  %7 = sext i32 %6 to i64
  br label %merge2
else1:
  br label %merge2
merge2:
  %8 = phi i64 [ 0, %then0 ], [ 0, %else1 ]
  %9 = srem i64 %n, 10
  %10 = add i64 %9, 48
  %11 = trunc i64 %10 to i32
  %12 = call i32 @putchar(i32 %11)
  %13 = sext i32 %12 to i64
  ret i64 0
}

define i64 @main() {
entry:
  %0 = call i64 @test_range()
  ret i64 %0
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

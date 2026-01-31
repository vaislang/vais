; ModuleID = 'closure_test'
source_filename = "<vais>"

declare i32 @puts(i8*)
declare void @free(i64)
declare i64 @malloc(i64)
declare i64 @vais_gc_print_stats()
declare i64 @fseek(i64, i64, i64)
declare double @fabs(double)
declare i64 @fgets(i64, i64, i64)
declare i64 @vais_gc_init()
declare i64 @vais_gc_remove_root(i64)
declare i64 @fgetc(i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @vais_gc_objects_count()
declare double @atof(i8*)
declare double @sqrt(double)
declare i64 @ftell(i64)
declare i32 @strcmp(i8*, i8*)
declare i32 @toupper(i32)
declare i64 @fputs(i8*, i64)
declare i32 @isalpha(i32)
declare i64 @strcat(i64, i8*)
declare i64 @fputc(i64, i64)
declare i64 @atol(i8*)
declare i64 @vais_gc_add_root(i64)
declare void @srand(i32)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare i32 @tolower(i32)
declare i64 @vais_gc_bytes_allocated()
declare i32 @atoi(i8*)
declare i64 @feof(i64)
declare i64 @fflush(i64)
declare void @exit(i32)
declare i32 @fclose(i64)
declare i64 @vais_gc_collect()
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @vais_gc_collections()
declare i64 @strlen(i8*)
declare i64 @vais_gc_alloc(i64, i32)
declare i64 @memcpy_str(i64, i8*, i64)
declare i32 @rand()
declare i64 @vais_gc_set_threshold(i64)
declare i64 @memcpy(i64, i64, i64)
declare i32 @sched_yield()
declare i64 @fread(i64, i64, i64, i64)
declare i32 @printf(i8*, ...)
declare i32 @usleep(i64)
declare i64 @strcpy(i64, i8*)
declare i64 @labs(i64)
declare i32 @isdigit(i32)
declare i64 @fopen(i8*, i8*)
declare i32 @putchar(i32)
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

@.str.0 = private unnamed_addr constant [21 x i8] c"=== Closure Test ===\00"
@.str.1 = private unnamed_addr constant [23 x i8] c"Test 1: Simple capture\00"
@.str.2 = private unnamed_addr constant [29 x i8] c"scale(5) with multiplier=10:\00"
@.str.3 = private unnamed_addr constant [26 x i8] c"Test 2: Multiple captures\00"
@.str.4 = private unnamed_addr constant [36 x i8] c"compute(3) with base=100, offset=7:\00"
@.str.5 = private unnamed_addr constant [27 x i8] c"Test 3: Nested value usage\00"
@.str.6 = private unnamed_addr constant [25 x i8] c"triple(2) with factor=5:\00"
@.str.7 = private unnamed_addr constant [13 x i8] c"=== Done ===\00"

define i64 @print_num(i64 %n) {
entry:
  %0 = sdiv i64 %n, 100
  %1 = icmp sgt i64 %0, 0
  %2 = zext i1 %1 to i64
  %3 = icmp ne i64 %2, 0
  br i1 %3, label %then0, label %else1
then0:
  %4 = add i64 %0, 48
  %5 = trunc i64 %4 to i32
  %6 = call i32 @putchar(i32 %5)
  %7 = sext i32 %6 to i64
  ret i64 0
else1:
  br label %merge2
merge2:
  %8 = phi i64 [ 0, %else1 ]
  %9 = sdiv i64 %n, 10
  %10 = srem i64 %9, 10
  %11 = icmp sge i64 %n, 10
  %12 = zext i1 %11 to i64
  %13 = icmp ne i64 %12, 0
  br i1 %13, label %then3, label %else4
then3:
  %14 = add i64 %10, 48
  %15 = trunc i64 %14 to i32
  %16 = call i32 @putchar(i32 %15)
  %17 = sext i32 %16 to i64
  ret i64 0
else4:
  br label %merge5
merge5:
  %18 = phi i64 [ 0, %else4 ]
  %19 = srem i64 %n, 10
  %20 = add i64 %19, 48
  %21 = trunc i64 %20 to i32
  %22 = call i32 @putchar(i32 %21)
  %23 = sext i32 %22 to i64
  ret i64 0
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([21 x i8], [21 x i8]* @.str.0, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = call i32 @puts(i8* getelementptr ([23 x i8], [23 x i8]* @.str.1, i64 0, i64 0))
  %3 = sext i32 %2 to i64
  %4 = ptrtoint i64 (i64, i64)* @__lambda_0 to i64
  %5 = call i64 @__lambda_0(i64 10, i64 5)
  %6 = call i32 @puts(i8* getelementptr ([29 x i8], [29 x i8]* @.str.2, i64 0, i64 0))
  %7 = sext i32 %6 to i64
  %8 = call i64 @print_num(i64 %5)
  %9 = trunc i64 10 to i32
  %10 = call i32 @putchar(i32 %9)
  %11 = sext i32 %10 to i64
  %12 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.3, i64 0, i64 0))
  %13 = sext i32 %12 to i64
  %14 = ptrtoint i64 (i64, i64, i64)* @__lambda_1 to i64
  %15 = call i64 @__lambda_1(i64 100, i64 7, i64 3)
  %16 = call i32 @puts(i8* getelementptr ([36 x i8], [36 x i8]* @.str.4, i64 0, i64 0))
  %17 = sext i32 %16 to i64
  %18 = call i64 @print_num(i64 %15)
  %19 = trunc i64 10 to i32
  %20 = call i32 @putchar(i32 %19)
  %21 = sext i32 %20 to i64
  %22 = call i32 @puts(i8* getelementptr ([27 x i8], [27 x i8]* @.str.5, i64 0, i64 0))
  %23 = sext i32 %22 to i64
  %24 = ptrtoint i64 (i64, i64)* @__lambda_2 to i64
  %25 = call i64 @__lambda_2(i64 5, i64 2)
  %26 = call i32 @puts(i8* getelementptr ([25 x i8], [25 x i8]* @.str.6, i64 0, i64 0))
  %27 = sext i32 %26 to i64
  %28 = call i64 @print_num(i64 %25)
  %29 = trunc i64 10 to i32
  %30 = call i32 @putchar(i32 %29)
  %31 = sext i32 %30 to i64
  %32 = call i32 @puts(i8* getelementptr ([13 x i8], [13 x i8]* @.str.7, i64 0, i64 0))
  %33 = sext i32 %32 to i64
  ret i64 0
}


define i64 @__lambda_0(i64 %__cap_multiplier, i64 %x) {
entry:
  %0 = mul i64 %x, %__cap_multiplier
  ret i64 %0
}

define i64 @__lambda_1(i64 %__cap_base, i64 %__cap_offset, i64 %x) {
entry:
  %0 = mul i64 %x, %__cap_offset
  %1 = add i64 %__cap_base, %0
  ret i64 %1
}

define i64 @__lambda_2(i64 %__cap_factor, i64 %n) {
entry:
  %0 = add i64 %n, %n
  %1 = add i64 %0, %n
  %2 = add i64 %1, %__cap_factor
  ret i64 %2
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

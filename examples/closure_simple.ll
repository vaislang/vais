; ModuleID = 'closure_simple'
source_filename = "<vais>"

declare i64 @strlen(i8*)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @fputs(i8*, i64)
declare i64 @feof(i64)
declare i64 @atol(i64)
declare i32 @isalpha(i32)
declare i64 @memcpy_str(i64, i8*, i64)
declare i64 @vais_gc_collections()
declare i64 @vais_gc_add_root(i64)
declare i64 @memcpy(i64, i64, i64)
declare void @free(i64)
declare i32 @fclose(i64)
declare i32 @atoi(i8*)
declare i32 @strncmp(i8*, i8*, i64)
declare void @exit(i32)
declare i64 @vais_gc_remove_root(i64)
declare i64 @vais_gc_bytes_allocated()
declare i64 @labs(i64)
declare i64 @vais_gc_init()
declare i64 @vais_gc_print_stats()
declare i64 @fflush(i64)
declare i32 @strcmp(i8*, i8*)
declare i64 @malloc(i64)
declare i64 @fgets(i64, i64, i64)
declare void @srand(i32)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare i64 @fgetc(i64)
declare double @atof(i64)
declare i32 @printf(i8*, ...)
declare i64 @strcpy(i64, i8*)
declare i64 @fseek(i64, i64, i64)
declare i32 @sched_yield()
declare i64 @vais_gc_collect()
declare double @fabs(double)
declare i32 @puts(i8*)
declare i32 @toupper(i32)
declare i64 @strcat(i64, i8*)
declare double @sqrt(double)
declare i64 @fopen(i8*, i8*)
declare i64 @vais_gc_objects_count()
declare i64 @vais_gc_alloc(i64, i32)
declare i64 @ftell(i64)
declare i32 @isdigit(i32)
declare i64 @fputc(i64, i64)
declare i32 @rand()
declare i64 @vais_gc_set_threshold(i64)
declare i32 @putchar(i32)
declare i64 @fread(i64, i64, i64, i64)
declare i32 @usleep(i64)
declare i32 @tolower(i32)
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

@.str.0 = private unnamed_addr constant [21 x i8] c"=== Closure Test ===\00"
@.str.1 = private unnamed_addr constant [23 x i8] c"Test 1: Simple capture\00"
@.str.2 = private unnamed_addr constant [26 x i8] c"Test 2: Multiple captures\00"
@.str.3 = private unnamed_addr constant [13 x i8] c"=== Done ===\00"

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([21 x i8], [21 x i8]* @.str.0, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = call i32 @puts(i8* getelementptr ([23 x i8], [23 x i8]* @.str.1, i64 0, i64 0))
  %3 = sext i32 %2 to i64
  %4 = ptrtoint i64 (i64, i64)* @__lambda_0 to i64
  %5 = call i64 @__lambda_0(i64 10, i64 5)
  %6 = sdiv i64 %5, 10
  %7 = add i64 %6, 48
  %8 = trunc i64 %7 to i32
  %9 = call i32 @putchar(i32 %8)
  %10 = sext i32 %9 to i64
  %11 = srem i64 %5, 10
  %12 = add i64 %11, 48
  %13 = trunc i64 %12 to i32
  %14 = call i32 @putchar(i32 %13)
  %15 = sext i32 %14 to i64
  %16 = trunc i64 10 to i32
  %17 = call i32 @putchar(i32 %16)
  %18 = sext i32 %17 to i64
  %19 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.2, i64 0, i64 0))
  %20 = sext i32 %19 to i64
  %21 = ptrtoint i64 (i64, i64, i64)* @__lambda_1 to i64
  %22 = call i64 @__lambda_1(i64 20, i64 3, i64 7)
  %23 = sdiv i64 %22, 10
  %24 = add i64 %23, 48
  %25 = trunc i64 %24 to i32
  %26 = call i32 @putchar(i32 %25)
  %27 = sext i32 %26 to i64
  %28 = srem i64 %22, 10
  %29 = add i64 %28, 48
  %30 = trunc i64 %29 to i32
  %31 = call i32 @putchar(i32 %30)
  %32 = sext i32 %31 to i64
  %33 = trunc i64 10 to i32
  %34 = call i32 @putchar(i32 %33)
  %35 = sext i32 %34 to i64
  %36 = call i32 @puts(i8* getelementptr ([13 x i8], [13 x i8]* @.str.3, i64 0, i64 0))
  %37 = sext i32 %36 to i64
  ret i64 0
}


define i64 @__lambda_0(i64 %__cap_multiplier, i64 %x) {
entry:
  %0 = mul i64 %x, %__cap_multiplier
  ret i64 %0
}

define i64 @__lambda_1(i64 %__cap_base, i64 %__cap_offset, i64 %x) {
entry:
  %0 = add i64 %__cap_base, %x
  %1 = add i64 %0, %__cap_offset
  ret i64 %1
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

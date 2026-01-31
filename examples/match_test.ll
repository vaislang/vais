; ModuleID = 'match_test'
source_filename = "<vais>"

declare i32 @rand()
declare i64 @vais_gc_set_threshold(i64)
declare double @sqrt(double)
declare i64 @memcpy_str(i64, i8*, i64)
declare i64 @fflush(i64)
declare i64 @strlen(i8*)
declare i32 @tolower(i32)
declare i64 @fgets(i64, i64, i64)
declare i64 @ftell(i64)
declare i64 @fopen(i8*, i8*)
declare i64 @atol(i64)
declare i32 @isdigit(i32)
declare i64 @fread(i64, i64, i64, i64)
declare i32 @putchar(i32)
declare i64 @strcpy(i64, i8*)
declare i64 @vais_gc_init()
declare i64 @vais_gc_collect()
declare i64 @malloc(i64)
declare void @exit(i32)
declare i64 @fwrite(i64, i64, i64, i64)
declare i32 @printf(i8*, ...)
declare i64 @vais_gc_print_stats()
declare i64 @vais_gc_add_root(i64)
declare i64 @fseek(i64, i64, i64)
declare double @atof(i8*)
declare i64 @vais_gc_objects_count()
declare i64 @strcat(i64, i8*)
declare i64 @labs(i64)
declare i32 @puts(i8*)
declare i32 @toupper(i32)
declare i32 @sched_yield()
declare i32 @strcmp(i8*, i8*)
declare i64 @feof(i64)
declare i32 @usleep(i64)
declare i64 @fgetc(i64)
declare i64 @fputc(i64, i64)
declare i32 @atoi(i8*)
declare i64 @fputs(i8*, i64)
declare i32 @isalpha(i32)
declare double @fabs(double)
declare i64 @vais_gc_alloc(i64, i32)
declare void @srand(i32)
declare void @free(i64)
declare i64 @vais_gc_remove_root(i64)
declare i64 @vais_gc_collections()
declare i64 @vais_gc_bytes_allocated()
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare i32 @fclose(i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @memcpy(i64, i64, i64)
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

@.str.0 = private unnamed_addr constant [22 x i8] c"Testing match with 0:\00"
@.str.1 = private unnamed_addr constant [22 x i8] c"Testing match with 1:\00"
@.str.2 = private unnamed_addr constant [22 x i8] c"Testing match with 5:\00"

define i64 @describe(i64 %n) {
entry:
  switch i64 %n, label %match.default1 [
    i64 0, label %match.arm2
    i64 1, label %match.arm3
    i64 2, label %match.arm4
  ]
match.arm2:
  br label %match.merge0
match.arm3:
  br label %match.merge0
match.arm4:
  br label %match.merge0
match.default1:
  br label %match.merge0
match.merge0:
  %0 = phi i64 [ 0, %match.arm2 ], [ 1, %match.arm3 ], [ 2, %match.arm4 ], [ %n, %match.default1 ]
  ret i64 %0
}

define i64 @main() {
entry:
  %0 = call i64 @describe(i64 0)
  %r.1 = alloca i64
  store i64 %0, i64* %r.1
  %2 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.0, i64 0, i64 0))
  %3 = sext i32 %2 to i64
  %4 = load i64, i64* %r.1
  %5 = add i64 %4, 48
  %6 = trunc i64 %5 to i32
  %7 = call i32 @putchar(i32 %6)
  %8 = sext i32 %7 to i64
  %9 = trunc i64 10 to i32
  %10 = call i32 @putchar(i32 %9)
  %11 = sext i32 %10 to i64
  %12 = call i64 @describe(i64 1)
  store i64 %12, i64* %r.1
  %13 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.1, i64 0, i64 0))
  %14 = sext i32 %13 to i64
  %15 = load i64, i64* %r.1
  %16 = add i64 %15, 48
  %17 = trunc i64 %16 to i32
  %18 = call i32 @putchar(i32 %17)
  %19 = sext i32 %18 to i64
  %20 = trunc i64 10 to i32
  %21 = call i32 @putchar(i32 %20)
  %22 = sext i32 %21 to i64
  %23 = call i64 @describe(i64 5)
  store i64 %23, i64* %r.1
  %24 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.2, i64 0, i64 0))
  %25 = sext i32 %24 to i64
  %26 = load i64, i64* %r.1
  %27 = add i64 %26, 48
  %28 = trunc i64 %27 to i32
  %29 = call i32 @putchar(i32 %28)
  %30 = sext i32 %29 to i64
  %31 = trunc i64 10 to i32
  %32 = call i32 @putchar(i32 %31)
  %33 = sext i32 %32 to i64
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

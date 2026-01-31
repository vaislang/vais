; ModuleID = 'match_binding'
source_filename = "<vais>"

declare i64 @vais_gc_collect()
declare i32 @sched_yield()
declare i64 @strcat(i64, i8*)
declare i64 @feof(i64)
declare i32 @isalpha(i32)
declare i64 @fgetc(i64)
declare i64 @vais_gc_set_threshold(i64)
declare double @atof(i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @memcpy(i64, i64, i64)
declare i64 @ftell(i64)
declare i64 @fgets(i64, i64, i64)
declare i64 @vais_gc_init()
declare i32 @tolower(i32)
declare i64 @fputs(i8*, i64)
declare i64 @fopen(i8*, i8*)
declare i32 @rand()
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare i32 @puts(i8*)
declare i64 @fseek(i64, i64, i64)
declare i64 @vais_gc_bytes_allocated()
declare i32 @toupper(i32)
declare double @fabs(double)
declare i64 @fwrite(i64, i64, i64, i64)
declare i32 @printf(i8*, ...)
declare i32 @isdigit(i32)
declare i64 @fflush(i64)
declare void @free(i64)
declare i64 @labs(i64)
declare double @exp(double)
declare double @log(double)
declare double @cos(double)
declare i32 @fclose(i64)
declare i64 @memcpy_str(i64, i8*, i64)
declare i64 @vais_gc_print_stats()
declare i32 @strcmp(i8*, i8*)
declare void @srand(i32)
declare i32 @usleep(i64)
declare i32 @atoi(i8*)
declare double @sin(double)
declare i64 @vais_gc_objects_count()
declare i64 @fread(i64, i64, i64, i64)
declare i64 @strcpy(i64, i8*)
declare i64 @vais_gc_collections()
declare i64 @vais_gc_remove_root(i64)
declare i64 @malloc(i64)
declare void @exit(i32)
declare i32 @putchar(i32)
declare i64 @strlen(i8*)
declare i64 @vais_gc_add_root(i64)
declare i64 @fputc(i64, i64)
declare double @sqrt(double)
declare i64 @atol(i64)
declare i64 @vais_gc_alloc(i64, i32)
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

@.str.0 = private unnamed_addr constant [10 x i8] c"check(0):\00"
@.str.1 = private unnamed_addr constant [10 x i8] c"check(1):\00"
@.str.2 = private unnamed_addr constant [25 x i8] c"check(5) - should be 50:\00"

define i64 @check(i64 %n) {
entry:
  br label %match.check1
match.check1:
  %0 = icmp eq i64 %n, 0
  br i1 %0, label %match.arm3, label %match.check2
match.arm3:
  br label %match.merge0
match.check2:
  %1 = icmp eq i64 %n, 1
  br i1 %1, label %match.arm5, label %match.check4
match.arm5:
  br label %match.merge0
match.check4:
  br i1 1, label %match.arm6, label %match.merge0
match.arm6:
  %3 = mul i64 %n, 10
  br label %match.merge0
match.merge0:
  %4 = phi i64 [ 100, %match.arm3 ], [ 200, %match.arm5 ], [ %3, %match.arm6 ]
  ret i64 %4
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([10 x i8], [10 x i8]* @.str.0, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = call i64 @check(i64 0)
  %3 = sdiv i64 %2, 100
  %4 = add i64 %3, 48
  %5 = trunc i64 %4 to i32
  %6 = call i32 @putchar(i32 %5)
  %7 = sext i32 %6 to i64
  %8 = trunc i64 10 to i32
  %9 = call i32 @putchar(i32 %8)
  %10 = sext i32 %9 to i64
  %11 = call i32 @puts(i8* getelementptr ([10 x i8], [10 x i8]* @.str.1, i64 0, i64 0))
  %12 = sext i32 %11 to i64
  %13 = call i64 @check(i64 1)
  %14 = sdiv i64 %13, 100
  %15 = add i64 %14, 48
  %16 = trunc i64 %15 to i32
  %17 = call i32 @putchar(i32 %16)
  %18 = sext i32 %17 to i64
  %19 = trunc i64 10 to i32
  %20 = call i32 @putchar(i32 %19)
  %21 = sext i32 %20 to i64
  %22 = call i32 @puts(i8* getelementptr ([25 x i8], [25 x i8]* @.str.2, i64 0, i64 0))
  %23 = sext i32 %22 to i64
  %24 = call i64 @check(i64 5)
  %25 = sdiv i64 %24, 10
  %26 = add i64 %25, 48
  %27 = trunc i64 %26 to i32
  %28 = call i32 @putchar(i32 %27)
  %29 = sext i32 %28 to i64
  %30 = srem i64 %24, 10
  %31 = add i64 %30, 48
  %32 = trunc i64 %31 to i32
  %33 = call i32 @putchar(i32 %32)
  %34 = sext i32 %33 to i64
  %35 = trunc i64 10 to i32
  %36 = call i32 @putchar(i32 %35)
  %37 = sext i32 %36 to i64
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

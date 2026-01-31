; ModuleID = 'file_test'
source_filename = "<vais>"

declare i64 @vais_gc_bytes_allocated()
declare i64 @vais_gc_collections()
declare i64 @fseek(i64, i64, i64)
declare double @fabs(double)
declare i64 @vais_gc_init()
declare i64 @feof(i64)
declare i32 @puts(i8*)
declare i32 @isalpha(i32)
declare i64 @vais_gc_remove_root(i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i32 @tolower(i32)
declare i64 @vais_gc_alloc(i64, i32)
declare i64 @vais_gc_set_threshold(i64)
declare i64 @fputc(i64, i64)
declare i64 @fflush(i64)
declare double @atof(i8*)
declare i64 @fgets(i64, i64, i64)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare i64 @fwrite(i64, i64, i64, i64)
declare void @free(i64)
declare double @sqrt(double)
declare i32 @strcmp(i8*, i8*)
declare i64 @labs(i64)
declare i32 @printf(i8*, ...)
declare i64 @memcpy_str(i64, i8*, i64)
declare i64 @strcat(i64, i8*)
declare void @srand(i32)
declare i32 @fclose(i64)
declare i64 @fopen(i8*, i8*)
declare i32 @isdigit(i32)
declare i32 @usleep(i64)
declare i32 @atoi(i8*)
declare void @exit(i32)
declare i64 @fputs(i8*, i64)
declare i64 @vais_gc_print_stats()
declare i64 @vais_gc_add_root(i64)
declare i64 @malloc(i64)
declare i64 @vais_gc_collect()
declare i64 @fgetc(i64)
declare i32 @sched_yield()
declare i64 @fread(i64, i64, i64, i64)
declare i64 @ftell(i64)
declare i64 @strlen(i8*)
declare i32 @rand()
declare i64 @memcpy(i64, i64, i64)
declare i64 @atol(i8*)
declare i32 @toupper(i32)
declare i64 @strcpy(i64, i8*)
declare i64 @vais_gc_objects_count()
declare i32 @putchar(i32)
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

@.str.0 = private unnamed_addr constant [22 x i8] c"=== File I/O Test ===\00"
@.str.1 = private unnamed_addr constant [23 x i8] c"Writing to test.txt...\00"
@.str.2 = private unnamed_addr constant [9 x i8] c"test.txt\00"
@.str.3 = private unnamed_addr constant [2 x i8] c"w\00"
@.str.4 = private unnamed_addr constant [17 x i8] c"Hello from VAIS!\00"
@.str.5 = private unnamed_addr constant [10 x i8] c"Write OK!\00"
@.str.6 = private unnamed_addr constant [14 x i8] c"Write FAILED!\00"
@.str.7 = private unnamed_addr constant [25 x i8] c"Reading from test.txt...\00"
@.str.8 = private unnamed_addr constant [9 x i8] c"test.txt\00"
@.str.9 = private unnamed_addr constant [2 x i8] c"r\00"
@.str.10 = private unnamed_addr constant [12 x i8] c"First char:\00"
@.str.11 = private unnamed_addr constant [9 x i8] c"Read OK!\00"
@.str.12 = private unnamed_addr constant [13 x i8] c"Read FAILED!\00"
@.str.13 = private unnamed_addr constant [13 x i8] c"=== Done ===\00"

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.0, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = call i32 @puts(i8* getelementptr ([23 x i8], [23 x i8]* @.str.1, i64 0, i64 0))
  %3 = sext i32 %2 to i64
  %4 = call i64 @fopen(i8* getelementptr ([9 x i8], [9 x i8]* @.str.2, i64 0, i64 0), i8* getelementptr ([2 x i8], [2 x i8]* @.str.3, i64 0, i64 0))
  %5 = icmp ne i64 %4, 0
  %6 = zext i1 %5 to i64
  %7 = icmp ne i64 %6, 0
  br i1 %7, label %then0, label %else1
then0:
  %8 = call i64 @fputs(i8* getelementptr ([17 x i8], [17 x i8]* @.str.4, i64 0, i64 0), i64 %4)
  %9 = call i32 @fclose(i64 %4)
  %10 = sext i32 %9 to i64
  %11 = call i32 @puts(i8* getelementptr ([10 x i8], [10 x i8]* @.str.5, i64 0, i64 0))
  %12 = sext i32 %11 to i64
  br label %merge2
else1:
  %13 = call i32 @puts(i8* getelementptr ([14 x i8], [14 x i8]* @.str.6, i64 0, i64 0))
  %14 = sext i32 %13 to i64
  br label %merge2
merge2:
  %15 = phi i64 [ 0, %then0 ], [ 0, %else1 ]
  %16 = call i32 @puts(i8* getelementptr ([25 x i8], [25 x i8]* @.str.7, i64 0, i64 0))
  %17 = sext i32 %16 to i64
  %18 = call i64 @fopen(i8* getelementptr ([9 x i8], [9 x i8]* @.str.8, i64 0, i64 0), i8* getelementptr ([2 x i8], [2 x i8]* @.str.9, i64 0, i64 0))
  %19 = icmp ne i64 %18, 0
  %20 = zext i1 %19 to i64
  %21 = icmp ne i64 %20, 0
  br i1 %21, label %then3, label %else4
then3:
  %22 = call i64 @fgetc(i64 %18)
  %23 = icmp sge i64 %22, 0
  %24 = zext i1 %23 to i64
  %25 = icmp ne i64 %24, 0
  br i1 %25, label %then6, label %else7
then6:
  %26 = call i32 @puts(i8* getelementptr ([12 x i8], [12 x i8]* @.str.10, i64 0, i64 0))
  %27 = sext i32 %26 to i64
  %28 = trunc i64 %22 to i32
  %29 = call i32 @putchar(i32 %28)
  %30 = sext i32 %29 to i64
  %31 = trunc i64 10 to i32
  %32 = call i32 @putchar(i32 %31)
  %33 = sext i32 %32 to i64
  br label %merge8
else7:
  br label %merge8
merge8:
  %34 = phi i64 [ 0, %then6 ], [ 0, %else7 ]
  %35 = call i32 @fclose(i64 %18)
  %36 = sext i32 %35 to i64
  %37 = call i32 @puts(i8* getelementptr ([9 x i8], [9 x i8]* @.str.11, i64 0, i64 0))
  %38 = sext i32 %37 to i64
  br label %merge5
else4:
  %39 = call i32 @puts(i8* getelementptr ([13 x i8], [13 x i8]* @.str.12, i64 0, i64 0))
  %40 = sext i32 %39 to i64
  br label %merge5
merge5:
  %41 = phi i64 [ 0, %merge8 ], [ 0, %else4 ]
  %42 = call i32 @puts(i8* getelementptr ([13 x i8], [13 x i8]* @.str.13, i64 0, i64 0))
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

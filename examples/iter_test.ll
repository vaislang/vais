; ModuleID = 'iter_test'
source_filename = "<vais>"

%Range = type { i64, i64, i64 }
declare i32 @putchar(i32)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @fputc(i64, i64)
declare i32 @strcmp(i8*, i8*)
declare double @atof(i8*)
declare i64 @fgets(i64, i64, i64)
declare i64 @strcat(i64, i8*)
declare i64 @vais_gc_bytes_allocated()
declare i64 @fread(i64, i64, i64, i64)
declare i64 @memcpy(i64, i64, i64)
declare void @free(i64)
declare i64 @labs(i64)
declare i64 @fseek(i64, i64, i64)
declare i64 @atol(i8*)
declare i32 @puts(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @fflush(i64)
declare double @fabs(double)
declare i32 @isdigit(i32)
declare i32 @sched_yield()
declare i32 @usleep(i64)
declare i64 @strlen(i8*)
declare i32 @atoi(i8*)
declare i64 @vais_gc_objects_count()
declare void @exit(i32)
declare i64 @vais_gc_print_stats()
declare i64 @vais_gc_set_threshold(i64)
declare i32 @tolower(i32)
declare i64 @vais_gc_collections()
declare i64 @strcpy(i64, i8*)
declare i64 @vais_gc_add_root(i64)
declare void @srand(i32)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare i64 @memcpy_str(i64, i8*, i64)
declare double @sqrt(double)
declare i32 @rand()
declare i64 @fgetc(i64)
declare i64 @vais_gc_alloc(i64, i32)
declare i64 @vais_gc_collect()
declare i64 @fputs(i8*, i64)
declare i32 @toupper(i32)
declare i64 @feof(i64)
declare i64 @vais_gc_remove_root(i64)
declare i64 @vais_gc_init()
declare i32 @printf(i8*, ...)
declare i64 @malloc(i64)
declare i64 @fopen(i8*, i8*)
declare i32 @fclose(i64)
declare i32 @isalpha(i32)
declare i64 @ftell(i64)
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

@.str.0 = private unnamed_addr constant [22 x i8] c"=== Iterator Test ===\00"
@.str.1 = private unnamed_addr constant [14 x i8] c"Range 0 to 5:\00"
@.str.2 = private unnamed_addr constant [16 x i8] c"Sum of 1 to 10:\00"
@.str.3 = private unnamed_addr constant [33 x i8] c"Count elements in range 5 to 15:\00"
@.str.4 = private unnamed_addr constant [13 x i8] c"=== Done ===\00"

define i64 @Range_has_next(%Range* %self) {
entry:
  %0 = getelementptr %Range, %Range* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = getelementptr %Range, %Range* %self, i32 0, i32 1
  %3 = load i64, i64* %2
  %4 = icmp slt i64 %1, %3
  %5 = zext i1 %4 to i64
  %6 = icmp ne i64 %5, 0
  br i1 %6, label %then0, label %else1
then0:
  br label %merge2
else1:
  br label %merge2
merge2:
  %7 = phi i64 [ 1, %then0 ], [ 0, %else1 ]
  ret i64 %7
}

define i64 @Range_next(%Range* %self) {
entry:
  %0 = getelementptr %Range, %Range* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = getelementptr %Range, %Range* %self, i32 0, i32 1
  %3 = load i64, i64* %2
  %4 = icmp slt i64 %1, %3
  %5 = zext i1 %4 to i64
  %6 = icmp ne i64 %5, 0
  br i1 %6, label %then0, label %else1
then0:
  %7 = getelementptr %Range, %Range* %self, i32 0, i32 0
  %8 = load i64, i64* %7
  %9 = getelementptr %Range, %Range* %self, i32 0, i32 0
  %10 = load i64, i64* %9
  %11 = getelementptr %Range, %Range* %self, i32 0, i32 2
  %12 = load i64, i64* %11
  %13 = add i64 %10, %12
  %14 = getelementptr %Range, %Range* %self, i32 0, i32 0
  store i64 %13, i64* %14
  br label %merge2
else1:
  %15 = sub i64 0, 1
  br label %merge2
merge2:
  %16 = phi i64 [ %8, %then0 ], [ %15, %else1 ]
  ret i64 %16
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
  %0 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.0, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = call i32 @puts(i8* getelementptr ([14 x i8], [14 x i8]* @.str.1, i64 0, i64 0))
  %3 = sext i32 %2 to i64
  %4 = alloca %Range
  %5 = getelementptr %Range, %Range* %4, i32 0, i32 0
  store i64 0, i64* %5
  %6 = getelementptr %Range, %Range* %4, i32 0, i32 1
  store i64 5, i64* %6
  %7 = getelementptr %Range, %Range* %4, i32 0, i32 2
  store i64 1, i64* %7
  %r.8 = alloca %Range*
  store %Range* %4, %Range** %r.8
  br label %loop.start0
loop.start0:
  %9 = load %Range*, %Range** %r.8
  %10 = call i64 @Range_next(%Range* %9)
  %11 = icmp slt i64 %10, 0
  %12 = zext i1 %11 to i64
  %13 = icmp ne i64 %12, 0
  br i1 %13, label %then3, label %else4
then3:
  br label %loop.end2
else4:
  br label %merge5
merge5:
  %14 = add i64 0, 0
  %15 = call i64 @print_num(i64 %10)
  %16 = trunc i64 32 to i32
  %17 = call i32 @putchar(i32 %16)
  %18 = sext i32 %17 to i64
  br label %loop.start0
loop.end2:
  %19 = trunc i64 10 to i32
  %20 = call i32 @putchar(i32 %19)
  %21 = sext i32 %20 to i64
  %22 = call i32 @puts(i8* getelementptr ([16 x i8], [16 x i8]* @.str.2, i64 0, i64 0))
  %23 = sext i32 %22 to i64
  %24 = alloca %Range
  %25 = getelementptr %Range, %Range* %24, i32 0, i32 0
  store i64 1, i64* %25
  %26 = getelementptr %Range, %Range* %24, i32 0, i32 1
  store i64 11, i64* %26
  %27 = getelementptr %Range, %Range* %24, i32 0, i32 2
  store i64 1, i64* %27
  %r2.28 = alloca %Range*
  store %Range* %24, %Range** %r2.28
  %sum.29 = alloca i64
  store i64 0, i64* %sum.29
  br label %loop.start6
loop.start6:
  %30 = load %Range*, %Range** %r2.28
  %31 = call i64 @Range_next(%Range* %30)
  %32 = icmp slt i64 %31, 0
  %33 = zext i1 %32 to i64
  %34 = icmp ne i64 %33, 0
  br i1 %34, label %then9, label %else10
then9:
  br label %loop.end8
else10:
  br label %merge11
merge11:
  %35 = add i64 0, 0
  %36 = load i64, i64* %sum.29
  %37 = add i64 %36, %31
  store i64 %37, i64* %sum.29
  br label %loop.start6
loop.end8:
  %38 = load i64, i64* %sum.29
  %39 = call i64 @print_num(i64 %38)
  %40 = trunc i64 10 to i32
  %41 = call i32 @putchar(i32 %40)
  %42 = sext i32 %41 to i64
  %43 = call i32 @puts(i8* getelementptr ([33 x i8], [33 x i8]* @.str.3, i64 0, i64 0))
  %44 = sext i32 %43 to i64
  %45 = alloca %Range
  %46 = getelementptr %Range, %Range* %45, i32 0, i32 0
  store i64 5, i64* %46
  %47 = getelementptr %Range, %Range* %45, i32 0, i32 1
  store i64 15, i64* %47
  %48 = getelementptr %Range, %Range* %45, i32 0, i32 2
  store i64 1, i64* %48
  %r3.49 = alloca %Range*
  store %Range* %45, %Range** %r3.49
  %count.50 = alloca i64
  store i64 0, i64* %count.50
  br label %loop.start12
loop.start12:
  %51 = load %Range*, %Range** %r3.49
  %52 = call i64 @Range_next(%Range* %51)
  %53 = icmp slt i64 %52, 0
  %54 = zext i1 %53 to i64
  %55 = icmp ne i64 %54, 0
  br i1 %55, label %then15, label %else16
then15:
  br label %loop.end14
else16:
  br label %merge17
merge17:
  %56 = add i64 0, 0
  %57 = load i64, i64* %count.50
  %58 = add i64 %57, 1
  store i64 %58, i64* %count.50
  br label %loop.start12
loop.end14:
  %59 = load i64, i64* %count.50
  %60 = call i64 @print_num(i64 %59)
  %61 = trunc i64 10 to i32
  %62 = call i32 @putchar(i32 %61)
  %63 = sext i32 %62 to i64
  %64 = call i32 @puts(i8* getelementptr ([13 x i8], [13 x i8]* @.str.4, i64 0, i64 0))
  %65 = sext i32 %64 to i64
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

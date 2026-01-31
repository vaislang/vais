; ModuleID = 'iterator_type_inference_test'
source_filename = "<vais>"

%Counter = type { i64, i64 }
declare i64 @feof(i64)
declare double @atof(i8*)
declare i32 @isalpha(i32)
declare i64 @memcpy_str(i64, i8*, i64)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @fgets(i64, i64, i64)
declare i32 @usleep(i64)
declare double @exp(double)
declare i32 @sched_yield()
declare i64 @strlen(i8*)
declare i64 @labs(i64)
declare double @log(double)
declare i32 @tolower(i32)
declare i64 @ftell(i64)
declare i64 @vais_gc_add_root(i64)
declare i64 @atol(i8*)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare i64 @vais_gc_collect()
declare i64 @vais_gc_collections()
declare i64 @strcpy(i64, i8*)
declare i32 @rand()
declare i32 @atoi(i8*)
declare double @cos(double)
declare i32 @toupper(i32)
declare i64 @vais_gc_objects_count()
declare i64 @fputc(i64, i64)
declare void @srand(i32)
declare i32 @puts(i8*)
declare i64 @vais_gc_print_stats()
declare i64 @fflush(i64)
declare i64 @fgetc(i64)
declare void @free(i64)
declare i64 @fopen(i8*, i8*)
declare i32 @printf(i8*, ...)
declare i32 @strcmp(i8*, i8*)
declare i64 @malloc(i64)
declare i32 @putchar(i32)
declare i32 @isdigit(i32)
declare double @fabs(double)
declare void @exit(i32)
declare i64 @fseek(i64, i64, i64)
declare double @sin(double)
declare double @sqrt(double)
declare i64 @strcat(i64, i8*)
declare i64 @vais_gc_bytes_allocated()
declare i64 @vais_gc_remove_root(i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i32 @fclose(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @memcpy(i64, i64, i64)
declare i64 @fputs(i8*, i64)
declare i64 @vais_gc_init()
declare i64 @vais_gc_set_threshold(i64)
declare i64 @vais_gc_alloc(i64, i32)
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

@.str.0 = private unnamed_addr constant [33 x i8] c"Testing iterator type inference:\00"
@.str.1 = private unnamed_addr constant [17 x i8] c"Range iteration:\00"
@.str.2 = private unnamed_addr constant [17 x i8] c"Custom iterator:\00"
@.str.3 = private unnamed_addr constant [27 x i8] c"Type inference successful!\00"

define i64 @Counter_next(%Counter* %self) {
entry:
  %0 = getelementptr %Counter, %Counter* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = getelementptr %Counter, %Counter* %self, i32 0, i32 1
  %3 = load i64, i64* %2
  %4 = icmp slt i64 %1, %3
  %5 = zext i1 %4 to i64
  %6 = icmp ne i64 %5, 0
  br i1 %6, label %then0, label %else1
then0:
  %7 = getelementptr %Counter, %Counter* %self, i32 0, i32 0
  %8 = load i64, i64* %7
  %9 = getelementptr %Counter, %Counter* %self, i32 0, i32 0
  %10 = load i64, i64* %9
  %11 = add i64 %10, 1
  %12 = getelementptr %Counter, %Counter* %self, i32 0, i32 0
  store i64 %11, i64* %12
  br label %merge2
else1:
  %13 = sub i64 0, 1
  br label %merge2
merge2:
  %14 = phi i64 [ %8, %then0 ], [ %13, %else1 ]
  ret i64 %14
}

define i64 @test_iteration() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([33 x i8], [33 x i8]* @.str.0, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = call i32 @puts(i8* getelementptr ([17 x i8], [17 x i8]* @.str.1, i64 0, i64 0))
  %3 = sext i32 %2 to i64
  %sum1.4 = alloca i64
  store i64 0, i64* %sum1.4
  %loop_counter.0 = alloca i64
  store i64 0, i64* %loop_counter.0
  %i.for = alloca i64
  br label %for.cond1
for.cond1:
  %5 = load i64, i64* %loop_counter.0
  %6 = icmp slt i64 %5, 5
  br i1 %6, label %for.body2, label %for.end4
for.body2:
  %7 = load i64, i64* %loop_counter.0
  store i64 %7, i64* %i.for
  %8 = load i64, i64* %i.for
  %9 = add i64 %8, 48
  %10 = trunc i64 %9 to i32
  %11 = call i32 @putchar(i32 %10)
  %12 = sext i32 %11 to i64
  %13 = trunc i64 32 to i32
  %14 = call i32 @putchar(i32 %13)
  %15 = sext i32 %14 to i64
  %16 = load i64, i64* %sum1.4
  %17 = load i64, i64* %i.for
  %18 = add i64 %16, %17
  store i64 %18, i64* %sum1.4
  br label %for.inc3
for.inc3:
  %19 = load i64, i64* %loop_counter.0
  %20 = add i64 %19, 1
  store i64 %20, i64* %loop_counter.0
  br label %for.cond1
for.end4:
  %21 = trunc i64 10 to i32
  %22 = call i32 @putchar(i32 %21)
  %23 = sext i32 %22 to i64
  %24 = call i32 @puts(i8* getelementptr ([17 x i8], [17 x i8]* @.str.2, i64 0, i64 0))
  %25 = sext i32 %24 to i64
  %26 = alloca %Counter
  %27 = getelementptr %Counter, %Counter* %26, i32 0, i32 0
  store i64 0, i64* %27
  %28 = getelementptr %Counter, %Counter* %26, i32 0, i32 1
  store i64 5, i64* %28
  %counter.29 = alloca %Counter*
  store %Counter* %26, %Counter** %counter.29
  %sum2.30 = alloca i64
  store i64 0, i64* %sum2.30
  br label %loop.start5
loop.start5:
  %31 = load %Counter*, %Counter** %counter.29
  %32 = call i64 @Counter_next(%Counter* %31)
  %33 = icmp slt i64 %32, 0
  %34 = zext i1 %33 to i64
  %35 = icmp ne i64 %34, 0
  br i1 %35, label %then8, label %else9
then8:
  br label %loop.end7
else9:
  br label %merge10
merge10:
  %36 = add i64 0, 0
  %37 = load i64, i64* %sum2.30
  %38 = add i64 %37, %32
  store i64 %38, i64* %sum2.30
  %39 = add i64 %32, 48
  %40 = trunc i64 %39 to i32
  %41 = call i32 @putchar(i32 %40)
  %42 = sext i32 %41 to i64
  %43 = trunc i64 32 to i32
  %44 = call i32 @putchar(i32 %43)
  %45 = sext i32 %44 to i64
  br label %loop.start5
loop.end7:
  %46 = trunc i64 10 to i32
  %47 = call i32 @putchar(i32 %46)
  %48 = sext i32 %47 to i64
  %49 = call i32 @puts(i8* getelementptr ([27 x i8], [27 x i8]* @.str.3, i64 0, i64 0))
  %50 = sext i32 %49 to i64
  ret i64 0
}

define i64 @main() {
entry:
  %0 = call i64 @test_iteration()
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

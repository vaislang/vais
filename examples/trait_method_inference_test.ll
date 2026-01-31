; ModuleID = 'trait_method_inference_test'
source_filename = "<vais>"

%SimpleCounter = type { i64 }
declare double @exp(double)
declare i64 @fgets(i64, i64, i64)
declare i64 @fflush(i64)
declare i64 @atol(i64)
declare i64 @vais_gc_bytes_allocated()
declare i64 @vais_gc_print_stats()
declare i64 @fread(i64, i64, i64, i64)
declare i32 @puts(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @labs(i64)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare void @exit(i32)
declare i64 @fseek(i64, i64, i64)
declare i32 @usleep(i64)
declare i64 @strlen(i8*)
declare i64 @vais_gc_init()
declare i32 @fclose(i64)
declare i64 @fgetc(i64)
declare i32 @atoi(i8*)
declare i32 @rand()
declare i64 @memcpy(i64, i64, i64)
declare double @atof(i64)
declare i32 @printf(i8*, ...)
declare i64 @strcat(i64, i8*)
declare double @log(double)
declare i64 @vais_gc_objects_count()
declare i64 @feof(i64)
declare i64 @strcpy(i64, i8*)
declare void @srand(i32)
declare i64 @vais_gc_alloc(i64, i32)
declare i32 @strncmp(i8*, i8*, i64)
declare i32 @tolower(i32)
declare double @cos(double)
declare i64 @vais_gc_add_root(i64)
declare i64 @vais_gc_collections()
declare i64 @fopen(i8*, i8*)
declare i32 @isdigit(i32)
declare i64 @ftell(i64)
declare i32 @strcmp(i8*, i8*)
declare double @sqrt(double)
declare i32 @putchar(i32)
declare i64 @vais_gc_remove_root(i64)
declare i64 @fputs(i8*, i64)
declare i32 @toupper(i32)
declare i64 @malloc(i64)
declare i64 @fputc(i64, i64)
declare i64 @memcpy_str(i64, i8*, i64)
declare i64 @vais_gc_set_threshold(i64)
declare i32 @isalpha(i32)
declare void @free(i64)
declare double @sin(double)
declare i64 @vais_gc_collect()
declare i32 @sched_yield()
declare double @fabs(double)
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

@.str.0 = private unnamed_addr constant [33 x i8] c"Testing trait method resolution:\00"
@.str.1 = private unnamed_addr constant [23 x i8] c"Calling trait methods:\00"
@.str.2 = private unnamed_addr constant [35 x i8] c"Trait method inference successful!\00"

define %SimpleCounter @SimpleCounter_new(i64 %v) {
entry:
  %0 = alloca %SimpleCounter
  %1 = getelementptr %SimpleCounter, %SimpleCounter* %0, i32 0, i32 0
  store i64 %v, i64* %1
  %ret.2 = load %SimpleCounter, %SimpleCounter* %0
  ret %SimpleCounter %ret.2
}

define i64 @SimpleCounter_next(%SimpleCounter* %self) {
entry:
  %0 = getelementptr %SimpleCounter, %SimpleCounter* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = getelementptr %SimpleCounter, %SimpleCounter* %self, i32 0, i32 0
  %3 = load i64, i64* %2
  %4 = add i64 %3, 1
  %5 = getelementptr %SimpleCounter, %SimpleCounter* %self, i32 0, i32 0
  store i64 %4, i64* %5
  ret i64 %1
}

define i64 @SimpleCounter_has_next(%SimpleCounter* %self) {
entry:
  %0 = getelementptr %SimpleCounter, %SimpleCounter* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = icmp slt i64 %1, 10
  %3 = zext i1 %2 to i64
  %4 = icmp ne i64 %3, 0
  br i1 %4, label %then0, label %else1
then0:
  br label %merge2
else1:
  br label %merge2
merge2:
  %5 = phi i64 [ 1, %then0 ], [ 0, %else1 ]
  ret i64 %5
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([33 x i8], [33 x i8]* @.str.0, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = call %SimpleCounter @SimpleCounter_new(i64 0)
  %counter.3.struct = alloca %SimpleCounter
  store %SimpleCounter %2, %SimpleCounter* %counter.3.struct
  %counter.3 = alloca %SimpleCounter*
  store %SimpleCounter* %counter.3.struct, %SimpleCounter** %counter.3
  %4 = call i32 @puts(i8* getelementptr ([23 x i8], [23 x i8]* @.str.1, i64 0, i64 0))
  %5 = sext i32 %4 to i64
  %6 = load %SimpleCounter*, %SimpleCounter** %counter.3
  %7 = call i64 @SimpleCounter_has_next(%SimpleCounter* %6)
  %8 = add i64 %7, 48
  %9 = trunc i64 %8 to i32
  %10 = call i32 @putchar(i32 %9)
  %11 = sext i32 %10 to i64
  %12 = trunc i64 10 to i32
  %13 = call i32 @putchar(i32 %12)
  %14 = sext i32 %13 to i64
  %15 = load %SimpleCounter*, %SimpleCounter** %counter.3
  %16 = call i64 @SimpleCounter_next(%SimpleCounter* %15)
  %17 = add i64 %16, 48
  %18 = trunc i64 %17 to i32
  %19 = call i32 @putchar(i32 %18)
  %20 = sext i32 %19 to i64
  %21 = trunc i64 10 to i32
  %22 = call i32 @putchar(i32 %21)
  %23 = sext i32 %22 to i64
  %24 = load %SimpleCounter*, %SimpleCounter** %counter.3
  %25 = call i64 @SimpleCounter_next(%SimpleCounter* %24)
  %26 = add i64 %25, 48
  %27 = trunc i64 %26 to i32
  %28 = call i32 @putchar(i32 %27)
  %29 = sext i32 %28 to i64
  %30 = trunc i64 10 to i32
  %31 = call i32 @putchar(i32 %30)
  %32 = sext i32 %31 to i64
  %33 = call i32 @puts(i8* getelementptr ([35 x i8], [35 x i8]* @.str.2, i64 0, i64 0))
  %34 = sext i32 %33 to i64
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

; ModuleID = 'trait_test'
source_filename = "<vais>"

%Counter = type { i64 }
declare i64 @fgets(i64, i64, i64)
declare i64 @fflush(i64)
declare i64 @ftell(i64)
declare i32 @puts(i8*)
declare i32 @fclose(i64)
declare i64 @memcpy_str(i64, i8*, i64)
declare i64 @fopen(i8*, i8*)
declare i64 @fgetc(i64)
declare double @sqrt(double)
declare void @srand(i32)
declare i32 @rand()
declare i32 @tolower(i32)
declare i64 @feof(i64)
declare i32 @toupper(i32)
declare i32 @atoi(i8*)
declare void @free(i64)
declare i64 @atol(i64)
declare i64 @strlen(i8*)
declare i32 @putchar(i32)
declare i64 @vais_gc_add_root(i64)
declare i64 @vais_gc_collect()
declare i64 @fseek(i64, i64, i64)
declare i64 @vais_gc_bytes_allocated()
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @vais_gc_set_threshold(i64)
declare i64 @vais_gc_print_stats()
declare double @fabs(double)
declare i32 @isalpha(i32)
declare i32 @sched_yield()
declare i32 @isdigit(i32)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @vais_gc_alloc(i64, i32)
declare i64 @vais_gc_remove_root(i64)
declare i64 @vais_gc_objects_count()
declare i64 @vais_gc_init()
declare i64 @fread(i64, i64, i64, i64)
declare i64 @malloc(i64)
declare i64 @fputs(i8*, i64)
declare i64 @labs(i64)
declare i64 @vais_gc_collections()
declare i32 @strcmp(i8*, i8*)
declare i64 @memcpy(i64, i64, i64)
declare i64 @fputc(i64, i64)
declare double @atof(i8*)
declare i64 @strcpy(i64, i8*)
declare i32 @printf(i8*, ...)
declare void @exit(i32)
declare i32 @usleep(i64)
declare i64 @strcat(i64, i8*)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

@.str.0 = private unnamed_addr constant [15 x i8] c"Counter value:\00"
@.str.1 = private unnamed_addr constant [25 x i8] c"Testing traits and impl:\00"
@.str.2 = private unnamed_addr constant [14 x i8] c"increment() =\00"
@.str.3 = private unnamed_addr constant [11 x i8] c"double() =\00"

define i64 @Counter_print(%Counter* %self) {
entry:
  %0 = call i32 @puts(i8* getelementptr ([15 x i8], [15 x i8]* @.str.0, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = getelementptr %Counter, %Counter* %self, i32 0, i32 0
  %3 = load i64, i64* %2
  %4 = sdiv i64 %3, 10
  %5 = add i64 %4, 48
  %6 = trunc i64 %5 to i32
  %7 = call i32 @putchar(i32 %6)
  %8 = sext i32 %7 to i64
  %9 = getelementptr %Counter, %Counter* %self, i32 0, i32 0
  %10 = load i64, i64* %9
  %11 = srem i64 %10, 10
  %12 = add i64 %11, 48
  %13 = trunc i64 %12 to i32
  %14 = call i32 @putchar(i32 %13)
  %15 = sext i32 %14 to i64
  %16 = trunc i64 10 to i32
  %17 = call i32 @putchar(i32 %16)
  %18 = sext i32 %17 to i64
  ret i64 0
}

define i64 @Counter_increment(%Counter* %self) {
entry:
  %0 = getelementptr %Counter, %Counter* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = add i64 %1, 1
  ret i64 %2
}

define i64 @Counter_double(%Counter* %self) {
entry:
  %0 = getelementptr %Counter, %Counter* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = mul i64 %1, 2
  ret i64 %2
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([25 x i8], [25 x i8]* @.str.1, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = alloca %Counter
  %3 = getelementptr %Counter, %Counter* %2, i32 0, i32 0
  store i64 42, i64* %3
  %c.4 = alloca %Counter*
  store %Counter* %2, %Counter** %c.4
  %5 = load %Counter*, %Counter** %c.4
  %6 = call i64 @Counter_print(%Counter* %5)
  %7 = load %Counter*, %Counter** %c.4
  %8 = call i64 @Counter_increment(%Counter* %7)
  %9 = call i32 @puts(i8* getelementptr ([14 x i8], [14 x i8]* @.str.2, i64 0, i64 0))
  %10 = sext i32 %9 to i64
  %11 = sdiv i64 %8, 10
  %12 = add i64 %11, 48
  %13 = trunc i64 %12 to i32
  %14 = call i32 @putchar(i32 %13)
  %15 = sext i32 %14 to i64
  %16 = srem i64 %8, 10
  %17 = add i64 %16, 48
  %18 = trunc i64 %17 to i32
  %19 = call i32 @putchar(i32 %18)
  %20 = sext i32 %19 to i64
  %21 = trunc i64 10 to i32
  %22 = call i32 @putchar(i32 %21)
  %23 = sext i32 %22 to i64
  %24 = load %Counter*, %Counter** %c.4
  %25 = call i64 @Counter_double(%Counter* %24)
  %26 = call i32 @puts(i8* getelementptr ([11 x i8], [11 x i8]* @.str.3, i64 0, i64 0))
  %27 = sext i32 %26 to i64
  %28 = sdiv i64 %25, 10
  %29 = add i64 %28, 48
  %30 = trunc i64 %29 to i32
  %31 = call i32 @putchar(i32 %30)
  %32 = sext i32 %31 to i64
  %33 = srem i64 %25, 10
  %34 = add i64 %33, 48
  %35 = trunc i64 %34 to i32
  %36 = call i32 @putchar(i32 %35)
  %37 = sext i32 %36 to i64
  %38 = trunc i64 10 to i32
  %39 = call i32 @putchar(i32 %38)
  %40 = sext i32 %39 to i64
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

; ModuleID = 'enum_struct_variant_test'
source_filename = "<vais>"

%Message = type { i32, { i64, i64, i64 } }
%Color = type { i32, { i64, i64, i64 } }
declare i32 @rand()
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare double @atof(i8*)
declare i64 @fflush(i64)
declare i64 @vais_gc_init()
declare i64 @fgets(i64, i64, i64)
declare i64 @memcpy_str(i64, i8*, i64)
declare i32 @isalpha(i32)
declare i64 @vais_gc_alloc(i64, i32)
declare i32 @atoi(i8*)
declare i64 @atol(i64)
declare i64 @strcpy(i64, i8*)
declare i32 @tolower(i32)
declare i64 @vais_gc_print_stats()
declare i64 @feof(i64)
declare double @fabs(double)
declare i64 @fgetc(i64)
declare i64 @strlen(i8*)
declare void @srand(i32)
declare i64 @memcpy(i64, i64, i64)
declare i32 @puts(i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i32 @fclose(i64)
declare i64 @vais_gc_set_threshold(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @vais_gc_collect()
declare double @sqrt(double)
declare i32 @toupper(i32)
declare i64 @vais_gc_add_root(i64)
declare i64 @strcat(i64, i8*)
declare i32 @usleep(i64)
declare i64 @labs(i64)
declare i32 @putchar(i32)
declare i64 @fread(i64, i64, i64, i64)
declare void @exit(i32)
declare void @free(i64)
declare i64 @fputc(i64, i64)
declare i64 @vais_gc_bytes_allocated()
declare i64 @fopen(i8*, i8*)
declare i64 @vais_gc_objects_count()
declare i64 @ftell(i64)
declare i64 @malloc(i64)
declare i32 @strcmp(i8*, i8*)
declare i64 @vais_gc_collections()
declare i32 @printf(i8*, ...)
declare i64 @vais_gc_remove_root(i64)
declare i32 @sched_yield()
declare i64 @fputs(i8*, i64)
declare i64 @fseek(i64, i64, i64)
declare i32 @isdigit(i32)
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

@.str.0 = private unnamed_addr constant [44 x i8] c"=== Enum Struct Variant Type Check Test ===\00"
@.str.1 = private unnamed_addr constant [42 x i8] c"Test 1: Enum with struct variants defined\00"
@.str.2 = private unnamed_addr constant [48 x i8] c"Color enum has: Red, Green, Blue, Custom{r,g,b}\00"
@.str.3 = private unnamed_addr constant [66 x i8] c"Message enum has: Quit, Move{x,y}, Write(i64), ChangeColor{r,g,b}\00"
@.str.4 = private unnamed_addr constant [33 x i8] c"Test 2: Type checking successful\00"
@.str.5 = private unnamed_addr constant [8 x i8] c"Result:\00"
@.str.6 = private unnamed_addr constant [49 x i8] c"=== Struct variant registration test passed! ===\00"

define i64 @get_red() {
entry:
  ret i64 0
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([44 x i8], [44 x i8]* @.str.0, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = trunc i64 10 to i32
  %3 = call i32 @putchar(i32 %2)
  %4 = sext i32 %3 to i64
  %5 = call i32 @puts(i8* getelementptr ([42 x i8], [42 x i8]* @.str.1, i64 0, i64 0))
  %6 = sext i32 %5 to i64
  %7 = call i32 @puts(i8* getelementptr ([48 x i8], [48 x i8]* @.str.2, i64 0, i64 0))
  %8 = sext i32 %7 to i64
  %9 = call i32 @puts(i8* getelementptr ([66 x i8], [66 x i8]* @.str.3, i64 0, i64 0))
  %10 = sext i32 %9 to i64
  %11 = trunc i64 10 to i32
  %12 = call i32 @putchar(i32 %11)
  %13 = sext i32 %12 to i64
  %14 = call i32 @puts(i8* getelementptr ([33 x i8], [33 x i8]* @.str.4, i64 0, i64 0))
  %15 = sext i32 %14 to i64
  %16 = call i64 @get_red()
  %17 = call i32 @puts(i8* getelementptr ([8 x i8], [8 x i8]* @.str.5, i64 0, i64 0))
  %18 = sext i32 %17 to i64
  %19 = add i64 %16, 48
  %20 = trunc i64 %19 to i32
  %21 = call i32 @putchar(i32 %20)
  %22 = sext i32 %21 to i64
  %23 = trunc i64 10 to i32
  %24 = call i32 @putchar(i32 %23)
  %25 = sext i32 %24 to i64
  %26 = call i32 @puts(i8* getelementptr ([49 x i8], [49 x i8]* @.str.6, i64 0, i64 0))
  %27 = sext i32 %26 to i64
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

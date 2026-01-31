; ModuleID = 'string_test'
source_filename = "<vais>"

%String = type { i64, i64, i64 }
declare i64 @malloc(i64)
declare double @atof(i8*)
declare double @fabs(double)
declare i64 @vais_gc_alloc(i64, i32)
declare i64 @feof(i64)
declare i32 @puts(i8*)
declare i64 @labs(i64)
declare i64 @fgets(i64, i64, i64)
declare i64 @memcpy_str(i64, i8*, i64)
declare i32 @isdigit(i32)
declare void @free(i64)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare i64 @strlen(i8*)
declare i32 @putchar(i32)
declare i64 @fgetc(i64)
declare i64 @atol(i8*)
declare i64 @fseek(i64, i64, i64)
declare i64 @vais_gc_remove_root(i64)
declare i32 @usleep(i64)
declare i64 @fflush(i64)
declare i64 @vais_gc_collect()
declare i32 @printf(i8*, ...)
declare i64 @strcpy(i64, i8*)
declare i64 @fputc(i64, i64)
declare i32 @fclose(i64)
declare i64 @strcat(i64, i8*)
declare i64 @vais_gc_bytes_allocated()
declare i64 @vais_gc_objects_count()
declare i64 @fwrite(i64, i64, i64, i64)
declare double @sqrt(double)
declare i64 @ftell(i64)
declare i64 @fread(i64, i64, i64, i64)
declare i32 @strncmp(i8*, i8*, i64)
declare void @srand(i32)
declare i32 @strcmp(i8*, i8*)
declare i32 @tolower(i32)
declare i64 @vais_gc_add_root(i64)
declare i32 @rand()
declare i32 @sched_yield()
declare i64 @vais_gc_init()
declare i64 @vais_gc_print_stats()
declare i32 @toupper(i32)
declare void @exit(i32)
declare i64 @fopen(i8*, i8*)
declare i64 @fputs(i8*, i64)
declare i32 @atoi(i8*)
declare i64 @vais_gc_set_threshold(i64)
declare i32 @isalpha(i32)
declare i64 @memcpy(i64, i64, i64)
declare i64 @vais_gc_collections()
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

@.str.0 = private unnamed_addr constant [21 x i8] c"Testing String type:\00"
@.str.1 = private unnamed_addr constant [33 x i8] c"Created string, pushing chars...\00"
@.str.2 = private unnamed_addr constant [16 x i8] c"String content:\00"
@.str.3 = private unnamed_addr constant [15 x i8] c"String length:\00"
@.str.4 = private unnamed_addr constant [6 x i8] c"Done!\00"

define i64 @String_len(%String* %self) {
entry:
  %0 = getelementptr %String, %String* %self, i32 0, i32 1
  %1 = load i64, i64* %0
  ret i64 %1
}

define i64 @String_push_char(%String* %self, i64 %c) {
entry:
  %0 = getelementptr %String, %String* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = getelementptr %String, %String* %self, i32 0, i32 1
  %3 = load i64, i64* %2
  %4 = add i64 %1, %3
  call void @__store_byte(i64 %4, i64 %c)
  %5 = getelementptr %String, %String* %self, i32 0, i32 1
  %6 = load i64, i64* %5
  %7 = add i64 %6, 1
  %8 = getelementptr %String, %String* %self, i32 0, i32 1
  store i64 %7, i64* %8
  %9 = getelementptr %String, %String* %self, i32 0, i32 0
  %10 = load i64, i64* %9
  %11 = getelementptr %String, %String* %self, i32 0, i32 1
  %12 = load i64, i64* %11
  %13 = add i64 %10, %12
  call void @__store_byte(i64 %13, i64 0)
  %14 = getelementptr %String, %String* %self, i32 0, i32 1
  %15 = load i64, i64* %14
  ret i64 %15
}

define i64 @String_print(%String* %self) {
entry:
  %0 = getelementptr %String, %String* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = inttoptr i64 %1 to i8*
  %3 = call i32 @puts(i8* %2)
  %4 = sext i32 %3 to i64
  ret i64 0
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([21 x i8], [21 x i8]* @.str.0, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = call i8* @malloc(i64 64)
  %3 = ptrtoint i8* %2 to i64
  call void @__store_byte(i64 %3, i64 0)
  %4 = alloca %String
  %5 = getelementptr %String, %String* %4, i32 0, i32 0
  store i64 %3, i64* %5
  %6 = getelementptr %String, %String* %4, i32 0, i32 1
  store i64 0, i64* %6
  %7 = getelementptr %String, %String* %4, i32 0, i32 2
  store i64 64, i64* %7
  %s.8 = alloca %String*
  store %String* %4, %String** %s.8
  %9 = call i32 @puts(i8* getelementptr ([33 x i8], [33 x i8]* @.str.1, i64 0, i64 0))
  %10 = sext i32 %9 to i64
  %11 = load %String*, %String** %s.8
  %12 = call i64 @String_push_char(%String* %11, i64 72)
  %13 = load %String*, %String** %s.8
  %14 = call i64 @String_push_char(%String* %13, i64 101)
  %15 = load %String*, %String** %s.8
  %16 = call i64 @String_push_char(%String* %15, i64 108)
  %17 = load %String*, %String** %s.8
  %18 = call i64 @String_push_char(%String* %17, i64 108)
  %19 = load %String*, %String** %s.8
  %20 = call i64 @String_push_char(%String* %19, i64 111)
  %21 = call i32 @puts(i8* getelementptr ([16 x i8], [16 x i8]* @.str.2, i64 0, i64 0))
  %22 = sext i32 %21 to i64
  %23 = load %String*, %String** %s.8
  %24 = call i64 @String_print(%String* %23)
  %25 = call i32 @puts(i8* getelementptr ([15 x i8], [15 x i8]* @.str.3, i64 0, i64 0))
  %26 = sext i32 %25 to i64
  %27 = load %String*, %String** %s.8
  %28 = call i64 @String_len(%String* %27)
  %29 = add i64 %28, 48
  %30 = trunc i64 %29 to i32
  %31 = call i32 @putchar(i32 %30)
  %32 = sext i32 %31 to i64
  %33 = trunc i64 10 to i32
  %34 = call i32 @putchar(i32 %33)
  %35 = sext i32 %34 to i64
  %36 = inttoptr i64 %3 to i8*
  call void @free(i8* %36)
  %37 = call i32 @puts(i8* getelementptr ([6 x i8], [6 x i8]* @.str.4, i64 0, i64 0))
  %38 = sext i32 %37 to i64
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

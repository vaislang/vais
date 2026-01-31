; ModuleID = 'test_import'
source_filename = "<vais>"

declare i32 @atoi(i8*)
declare i64 @vais_gc_bytes_allocated()
declare void @exit(i32)
declare i32 @puts(i8*)
declare i64 @fgets(i64, i64, i64)
declare double @sqrt(double)
declare i32 @isdigit(i32)
declare void @srand(i32)
declare i64 @fgetc(i64)
declare i64 @malloc(i64)
declare i32 @tolower(i32)
declare i64 @strlen(i8*)
declare i64 @strcat(i64, i8*)
declare i64 @vais_gc_add_root(i64)
declare void @free(i64)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare i64 @vais_gc_print_stats()
declare i32 @sched_yield()
declare i32 @rand()
declare i64 @fopen(i8*, i8*)
declare i32 @putchar(i32)
declare i64 @fseek(i64, i64, i64)
declare i64 @ftell(i64)
declare i64 @strcpy(i64, i8*)
declare i32 @strcmp(i8*, i8*)
declare i64 @vais_gc_objects_count()
declare i64 @memcpy(i64, i64, i64)
declare i32 @fclose(i64)
declare i64 @vais_gc_init()
declare i32 @printf(i8*, ...)
declare double @fabs(double)
declare i64 @vais_gc_collections()
declare i64 @labs(i64)
declare i64 @fputs(i8*, i64)
declare i64 @vais_gc_alloc(i64, i32)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i32 @usleep(i64)
declare i64 @fflush(i64)
declare i64 @memcpy_str(i64, i8*, i64)
declare i64 @vais_gc_collect()
declare i64 @vais_gc_set_threshold(i64)
declare i64 @fputc(i64, i64)
declare double @atof(i8*)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @vais_gc_remove_root(i64)
declare i64 @atol(i8*)
declare i32 @isalpha(i32)
declare i64 @feof(i64)
declare i32 @toupper(i32)
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

define i64 @test_func(i64 %a, i64 %b) {
entry:
  %0 = add i64 %a, %b
  ret i64 %0
}

define i64 @main() {
entry:
  %0 = call i64 @test_func(i64 10, i64 20)
  %1 = sdiv i64 %0, 10
  %2 = add i64 %1, 48
  %3 = trunc i64 %2 to i32
  %4 = call i32 @putchar(i32 %3)
  %5 = sext i32 %4 to i64
  %6 = srem i64 %0, 10
  %7 = add i64 %6, 48
  %8 = trunc i64 %7 to i32
  %9 = call i32 @putchar(i32 %8)
  %10 = sext i32 %9 to i64
  %11 = trunc i64 10 to i32
  %12 = call i32 @putchar(i32 %11)
  %13 = sext i32 %12 to i64
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

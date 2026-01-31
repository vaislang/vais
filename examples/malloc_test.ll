; ModuleID = 'malloc_test'
source_filename = "<vais>"

declare i32 @atoi(i8*)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare i64 @fputc(i64, i64)
declare i64 @vais_gc_objects_count()
declare double @sqrt(double)
declare i64 @strcpy(i64, i8*)
declare i64 @vais_gc_collections()
declare void @srand(i32)
declare i64 @labs(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @vais_gc_collect()
declare i32 @toupper(i32)
declare i32 @tolower(i32)
declare void @exit(i32)
declare i64 @feof(i64)
declare i64 @fgets(i64, i64, i64)
declare i32 @printf(i8*, ...)
declare double @fabs(double)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @vais_gc_set_threshold(i64)
declare i64 @malloc(i64)
declare i32 @rand()
declare i32 @fclose(i64)
declare i64 @memcpy(i64, i64, i64)
declare i64 @memcpy_str(i64, i8*, i64)
declare i64 @vais_gc_init()
declare i64 @vais_gc_remove_root(i64)
declare void @free(i64)
declare i64 @fflush(i64)
declare i64 @fgetc(i64)
declare i64 @fseek(i64, i64, i64)
declare i32 @usleep(i64)
declare i64 @strlen(i8*)
declare i32 @sched_yield()
declare i64 @fputs(i8*, i64)
declare double @atof(i8*)
declare i32 @isdigit(i32)
declare i32 @strcmp(i8*, i8*)
declare i32 @putchar(i32)
declare i64 @vais_gc_bytes_allocated()
declare i64 @vais_gc_alloc(i64, i32)
declare i32 @isalpha(i32)
declare i32 @puts(i8*)
declare i64 @atol(i8*)
declare i64 @vais_gc_add_root(i64)
declare i64 @ftell(i64)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @vais_gc_print_stats()
declare i64 @strcat(i64, i8*)
declare i64 @fopen(i8*, i8*)
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

@.str.0 = private unnamed_addr constant [18 x i8] c"Memory allocated!\00"
@.str.1 = private unnamed_addr constant [14 x i8] c"Memory freed!\00"

define i64 @main() {
entry:
  %0 = call i8* @malloc(i64 100)
  %1 = ptrtoint i8* %0 to i64
  %2 = call i32 @puts(i8* getelementptr ([18 x i8], [18 x i8]* @.str.0, i64 0, i64 0))
  %3 = sext i32 %2 to i64
  %4 = inttoptr i64 %1 to i8*
  call void @free(i8* %4)
  %5 = call i32 @puts(i8* getelementptr ([14 x i8], [14 x i8]* @.str.1, i64 0, i64 0))
  %6 = sext i32 %5 to i64
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

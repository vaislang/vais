; ModuleID = 'simd_test'
source_filename = "<vais>"

declare void @srand(i32)
declare i32 @printf(i8*, ...)
declare i32 @usleep(i64)
declare double @exp(double)
declare void @exit(i32)
declare i64 @vais_gc_alloc(i64, i32)
declare i64 @vais_gc_collect()
declare double @cos(double)
declare i64 @vais_gc_print_stats()
declare i64 @memcpy_str(i64, i8*, i64)
declare i64 @vais_gc_add_root(i64)
declare i64 @vais_gc_bytes_allocated()
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @feof(i64)
declare i64 @vais_gc_remove_root(i64)
declare i64 @atol(i8*)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare void @free(i64)
declare i64 @ftell(i64)
declare i32 @isdigit(i32)
declare i32 @tolower(i32)
declare i64 @fgets(i64, i64, i64)
declare double @sqrt(double)
declare i64 @strlen(i8*)
declare i64 @fseek(i64, i64, i64)
declare i64 @fopen(i8*, i8*)
declare double @sin(double)
declare double @fabs(double)
declare i64 @vais_gc_init()
declare i64 @fputc(i64, i64)
declare double @atof(i64)
declare i32 @isalpha(i32)
declare i64 @strcpy(i64, i8*)
declare i64 @memcpy(i64, i64, i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i32 @strcmp(i8*, i8*)
declare i32 @fclose(i64)
declare i32 @putchar(i32)
declare i32 @rand()
declare double @log(double)
declare i64 @vais_gc_objects_count()
declare i64 @labs(i64)
declare i32 @puts(i8*)
declare i32 @toupper(i32)
declare i64 @vais_gc_collections()
declare i64 @vais_gc_set_threshold(i64)
declare i64 @fflush(i64)
declare i64 @fgetc(i64)
declare i32 @atoi(i8*)
declare i64 @fputs(i8*, i64)
declare i64 @strcat(i64, i8*)
declare i64 @malloc(i64)
declare i64 @fread(i64, i64, i64, i64)
declare i32 @sched_yield()
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

define float @dot_product(<4 x float> %a, <4 x float> %b) {
entry:
  %0 = fmul <4 x float> %a, %b
  %product.1 = alloca <4 x float>
  store <4 x float> %0, <4 x float>* %product.1
  %2 = load <4 x float>, <4 x float>* %product.1
  %3 = call float @llvm.vector.reduce.fadd.v4f32(float 0.0, <4 x float> %2)
  ret float %3
}

define <4 x i32> @vec_add_test(<4 x i32> %a, <4 x i32> %b) {
entry:
  %0 = add <4 x i32> %a, %b
  ret <4 x i32> %0
}

define <4 x i32> @vec_mul_test(<4 x i32> %a, <4 x i32> %b) {
entry:
  %0 = mul <4 x i32> %a, %b
  ret <4 x i32> %0
}

define i32 @main() {
entry:
  %0 = insertelement <4 x i32> undef, i32 1, i32 0
  %1 = insertelement <4 x i32> %0, i32 2, i32 1
  %2 = insertelement <4 x i32> %1, i32 3, i32 2
  %3 = insertelement <4 x i32> %2, i32 4, i32 3
  %v1.4 = alloca <4 x i32>
  store <4 x i32> %3, <4 x i32>* %v1.4
  %5 = insertelement <4 x i32> undef, i32 5, i32 0
  %6 = insertelement <4 x i32> %5, i32 6, i32 1
  %7 = insertelement <4 x i32> %6, i32 7, i32 2
  %8 = insertelement <4 x i32> %7, i32 8, i32 3
  %v2.9 = alloca <4 x i32>
  store <4 x i32> %8, <4 x i32>* %v2.9
  %10 = load <4 x i32>, <4 x i32>* %v1.4
  %11 = load <4 x i32>, <4 x i32>* %v2.9
  %12 = add <4 x i32> %10, %11
  %sum.13 = alloca <4 x i32>
  store <4 x i32> %12, <4 x i32>* %sum.13
  %14 = load <4 x i32>, <4 x i32>* %v1.4
  %15 = load <4 x i32>, <4 x i32>* %v2.9
  %16 = mul <4 x i32> %14, %15
  %product.17 = alloca <4 x i32>
  store <4 x i32> %16, <4 x i32>* %product.17
  %18 = load <4 x i32>, <4 x i32>* %sum.13
  %19 = call i32 @llvm.vector.reduce.add.v4i32(<4 x i32> %18)
  %20 = insertelement <2 x i64> undef, i64 100, i32 0
  %21 = insertelement <2 x i64> %20, i64 200, i32 1
  %v3.22 = alloca <2 x i64>
  store <2 x i64> %21, <2 x i64>* %v3.22
  %23 = insertelement <2 x i64> undef, i64 1, i32 0
  %24 = insertelement <2 x i64> %23, i64 2, i32 1
  %v4.25 = alloca <2 x i64>
  store <2 x i64> %24, <2 x i64>* %v4.25
  %26 = load <2 x i64>, <2 x i64>* %v3.22
  %27 = load <2 x i64>, <2 x i64>* %v4.25
  %28 = add <2 x i64> %26, %27
  %sum64.29 = alloca <2 x i64>
  store <2 x i64> %28, <2 x i64>* %sum64.29
  %30 = load <2 x i64>, <2 x i64>* %sum64.29
  %31 = call i64 @llvm.vector.reduce.add.v2i64(<2 x i64> %30)
  ret i32 0
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

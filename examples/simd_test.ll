; ModuleID = 'simd_test'
source_filename = "<vais>"

declare i64 @fgetc(i64)
declare i32 @putchar(i32)
declare i64 @malloc(i64)
declare i64 @memcpy(i64, i64, i64)
declare void @free(i64)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @fputc(i64, i64)
declare i32 @usleep(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i32 @sched_yield()
declare i32 @printf(i8*)
declare i32 @puts(i8*)
declare i64 @strlen(i8*)
declare i64 @feof(i64)
declare i64 @fflush(i64)
declare void @exit(i32)
declare i64 @fputs(i8*, i64)
declare i64 @fopen(i8*, i8*)
declare i64 @fseek(i64, i64, i64)
declare i32 @fclose(i64)
declare i64 @ftell(i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i32 @strcmp(i8*, i8*)
declare i64 @fgets(i64, i64, i64)
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
  %total.20 = alloca i64
  store i64 %19, i64* %total.20
  %21 = insertelement <2 x i64> undef, i64 100, i32 0
  %22 = insertelement <2 x i64> %21, i64 200, i32 1
  %v3.23 = alloca <2 x i64>
  store <2 x i64> %22, <2 x i64>* %v3.23
  %24 = insertelement <2 x i64> undef, i64 1, i32 0
  %25 = insertelement <2 x i64> %24, i64 2, i32 1
  %v4.26 = alloca <2 x i64>
  store <2 x i64> %25, <2 x i64>* %v4.26
  %27 = load <2 x i64>, <2 x i64>* %v3.23
  %28 = load <2 x i64>, <2 x i64>* %v4.26
  %29 = add <2 x i64> %27, %28
  %sum64.30 = alloca <2 x i64>
  store <2 x i64> %29, <2 x i64>* %sum64.30
  %31 = load <2 x i64>, <2 x i64>* %sum64.30
  %32 = call i64 @llvm.vector.reduce.add.v2i64(<2 x i64> %31)
  %total64.33 = alloca i64
  store i64 %32, i64* %total64.33
  ret i32 0
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

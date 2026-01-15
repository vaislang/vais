; ModuleID = 'generic_struct_test'
source_filename = "<vais>"

%Pair = type { i64, i64 }
declare i64 @feof(i64)
declare i32 @putchar(i32)
declare void @free(i64)
declare i32 @puts(i64)
declare i64 @fread(i64, i64, i64, i64)
declare i32 @fclose(i64)
declare i64 @fflush(i64)
declare i32 @printf(i8*)
declare i64 @memcpy(i64, i64, i64)
declare i64 @fgets(i64, i64, i64)
declare i64 @malloc(i64)
declare i64 @fputs(i8*, i64)
declare i64 @fseek(i64, i64, i64)
declare i64 @fputc(i64, i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare void @exit(i32)
declare i64 @strlen(i64)
declare i64 @fopen(i8*, i8*)
declare i64 @fgetc(i64)
declare i64 @ftell(i64)
@.str.0 = private unnamed_addr constant [24 x i8] c"Testing generic struct:\00"
@.str.1 = private unnamed_addr constant [21 x i8] c"Pair{10, 20}.sum() =\00"

define i64 @Pair_sum(%Pair* %self) {
entry:
  %0 = getelementptr %Pair, %Pair* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = getelementptr %Pair, %Pair* %self, i32 0, i32 1
  %3 = load i64, i64* %2
  %4 = add i64 %1, %3
  ret i64 %4
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.0, i64 0, i64 0))
  %1 = alloca %Pair
  %2 = getelementptr %Pair, %Pair* %1, i32 0, i32 0
  store i64 10, i64* %2
  %3 = getelementptr %Pair, %Pair* %1, i32 0, i32 1
  store i64 20, i64* %3
  %p.4 = alloca %Pair*
  store %Pair* %1, %Pair** %p.4
  %5 = load %Pair*, %Pair** %p.4
  %6 = call i64 @Pair_sum(%Pair* %5)
  %s.7 = alloca i64
  store i64 %6, i64* %s.7
  %8 = call i32 @puts(i8* getelementptr ([21 x i8], [21 x i8]* @.str.1, i64 0, i64 0))
  %9 = load i64, i64* %s.7
  %10 = sdiv i64 %9, 10
  %11 = add i64 %10, 48
  %12 = trunc i64 %11 to i32
  %13 = call i32 @putchar(i32 %12)
  %14 = load i64, i64* %s.7
  %15 = srem i64 %14, 10
  %16 = add i64 %15, 48
  %17 = trunc i64 %16 to i32
  %18 = call i32 @putchar(i32 %17)
  %19 = trunc i64 10 to i32
  %20 = call i32 @putchar(i32 %19)
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

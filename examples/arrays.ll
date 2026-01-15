; ModuleID = 'arrays'
source_filename = "<vais>"

declare i64 @fseek(i64, i64, i64)
declare i32 @putchar(i32)
declare i64 @malloc(i64)
declare i32 @puts(i64)
declare i64 @fopen(i8*, i8*)
declare i32 @printf(i8*)
declare i64 @fwrite(i64, i64, i64, i64)
declare i32 @fclose(i64)
declare i64 @fgetc(i64)
declare void @exit(i32)
declare i64 @fputs(i8*, i64)
declare i64 @ftell(i64)
declare void @free(i64)
declare i64 @fgets(i64, i64, i64)
declare i64 @memcpy(i64, i64, i64)
declare i64 @strlen(i64)
declare i64 @fputc(i64, i64)
declare i64 @fflush(i64)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @feof(i64)
define i64 @get_elem(i64* %arr, i64 %idx) {
entry:
  %0 = getelementptr i64, i64* %arr, i64 %idx
  %1 = load i64, i64* %0
  ret i64 %1
}

define i64 @main() {
entry:
  %0 = alloca [4  x i64]
  %1 = getelementptr [4  x i64], [4  x i64]* %0, i64 0, i64 0
  store i64 10, i64* %1
  %2 = getelementptr [4  x i64], [4  x i64]* %0, i64 0, i64 1
  store i64 20, i64* %2
  %3 = getelementptr [4  x i64], [4  x i64]* %0, i64 0, i64 2
  store i64 30, i64* %3
  %4 = getelementptr [4  x i64], [4  x i64]* %0, i64 0, i64 3
  store i64 40, i64* %4
  %5 = getelementptr [4  x i64], [4  x i64]* %0, i64 0, i64 0
  %arr.6 = alloca i64*
  store i64* %5, i64** %arr.6
  %7 = load i64*, i64** %arr.6
  %8 = call i64 @get_elem(i64* %7, i64 2)
  ret i64 %8
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

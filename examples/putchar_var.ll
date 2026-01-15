; ModuleID = 'putchar_var'
source_filename = "<vais>"

declare i64 @fputc(i64, i64)
declare void @exit(i32)
declare i64 @fgets(i64, i64, i64)
declare i32 @fclose(i64)
declare i64 @memcpy(i64, i64, i64)
declare i64 @fflush(i64)
declare i64 @strlen(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare void @free(i64)
declare i64 @fputs(i8*, i64)
declare i32 @puts(i8*)
declare i64 @feof(i64)
declare i64 @fopen(i8*, i8*)
declare i64 @fgetc(i64)
declare i32 @putchar(i32)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @fseek(i64, i64, i64)
declare i32 @printf(i8*)
declare i64 @ftell(i64)
declare i64 @malloc(i64)
define i64 @main() {
entry:
  %ch.0 = alloca i64
  store i64 65, i64* %ch.0
  %1 = load i64, i64* %ch.0
  %2 = trunc i64 %1 to i32
  %3 = call i32 @putchar(i32 %2)
  %4 = trunc i64 10 to i32
  %5 = call i32 @putchar(i32 %4)
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

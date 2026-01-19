; ModuleID = 'range_type_test'
source_filename = "<vais>"

declare i64 @strlen(i64)
declare i64 @fopen(i8*, i8*)
declare i32 @puts(i64)
declare i32 @sched_yield()
declare void @exit(i32)
declare i64 @fread(i64, i64, i64, i64)
declare i32 @strcmp(i8*, i8*)
declare i64 @memcpy(i64, i64, i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @feof(i64)
declare i64 @fgetc(i64)
declare i64 @fflush(i64)
declare i64 @fputc(i64, i64)
declare i32 @usleep(i64)
declare i32 @printf(i8*)
declare i32 @putchar(i32)
declare i64 @malloc(i64)
declare void @free(i64)
declare i64 @fputs(i8*, i64)
declare i32 @fclose(i64)
declare i64 @fgets(i64, i64, i64)
declare i64 @ftell(i64)
declare i64 @fseek(i64, i64, i64)
define i64 @test_range_types() {
entry:
  %r1.0 = alloca i64
  store i64 0, i64* %r1.0
  %r2.1 = alloca i64
  store i64 1, i64* %r2.1
  %r3.2 = alloca i64
  store i64 0, i64* %r3.2
  %r4.3 = alloca i64
  store i64 0, i64* %r4.3
  %r5.4 = alloca i64
  store i64 5, i64* %r5.4
  ret i64 0
}

define i64 @main() {
entry:
  %0 = call i64 @test_range_types()
  ret i64 %0
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

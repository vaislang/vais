; ModuleID = 'control_flow'
source_filename = "<vais>"

declare i64 @fputs(i8*, i64)
declare i64 @feof(i64)
declare i64 @fgets(i64, i64, i64)
declare i64 @ftell(i64)
declare i64 @fflush(i64)
declare i32 @putchar(i32)
declare void @free(i64)
declare i64 @malloc(i64)
declare i64 @fgetc(i64)
declare i64 @strlen(i64)
declare i64 @fseek(i64, i64, i64)
declare i64 @fopen(i8*, i8*)
declare i64 @memcpy(i64, i64, i64)
declare i64 @fread(i64, i64, i64, i64)
declare i32 @printf(i8*)
declare i32 @fclose(i64)
declare i32 @puts(i8*)
declare void @exit(i32)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @fputc(i64, i64)
define i64 @max(i64 %a, i64 %b) {
entry:
  %0 = icmp sgt i64 %a, %b
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %ternary.then0, label %ternary.else1
ternary.then0:
  br label %ternary.merge2
ternary.else1:
  br label %ternary.merge2
ternary.merge2:
  %3 = phi i64 [ %a, %ternary.then0 ], [ %b, %ternary.else1 ]
  ret i64 %3
}

define i64 @countdown(i64 %n) {
entry:
  %0 = icmp slt i64 %n, 1
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %ternary.then0, label %ternary.else1
ternary.then0:
  br label %ternary.merge2
ternary.else1:
  %3 = sub i64 %n, 1
  %4 = call i64 @countdown(i64 %3)
  br label %ternary.merge2
ternary.merge2:
  %5 = phi i64 [ 0, %ternary.then0 ], [ %4, %ternary.else1 ]
  ret i64 %5
}

define i64 @factorial(i64 %n) {
entry:
  %0 = icmp slt i64 %n, 2
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %ternary.then0, label %ternary.else1
ternary.then0:
  br label %ternary.merge2
ternary.else1:
  %3 = sub i64 %n, 1
  %4 = call i64 @factorial(i64 %3)
  %5 = mul i64 %n, %4
  br label %ternary.merge2
ternary.merge2:
  %6 = phi i64 [ 1, %ternary.then0 ], [ %5, %ternary.else1 ]
  ret i64 %6
}

define i64 @main() {
entry:
  %0 = call i64 @factorial(i64 5)
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

; ModuleID = 'trait_iter_test'
source_filename = "<vais>"

%Range = type { i64, i64, i64 }
declare i32 @puts(i8*)
declare i64 @fseek(i64, i64, i64)
declare i64 @ftell(i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i32 @putchar(i32)
declare i64 @fputc(i64, i64)
declare i64 @fputs(i8*, i64)
declare i32 @sched_yield()
declare i64 @fgetc(i64)
declare i32 @strcmp(i8*, i8*)
declare i64 @fflush(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @malloc(i64)
declare i64 @memcpy(i64, i64, i64)
declare i64 @fopen(i8*, i8*)
declare i64 @fgets(i64, i64, i64)
declare i64 @feof(i64)
declare void @free(i64)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @strlen(i64)
declare i32 @usleep(i64)
declare i32 @printf(i8*)
declare i32 @fclose(i64)
declare void @exit(i32)
@.str.0 = private unnamed_addr constant [24 x i8] c"Testing iterator trait:\00"
@.str.1 = private unnamed_addr constant [6 x i8] c"Done!\00"

define i64 @Range_has_next(%Range* %self) {
entry:
  %0 = getelementptr %Range, %Range* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = getelementptr %Range, %Range* %self, i32 0, i32 1
  %3 = load i64, i64* %2
  %4 = icmp slt i64 %1, %3
  %5 = zext i1 %4 to i64
  %6 = icmp ne i64 %5, 0
  br i1 %6, label %then0, label %else1
then0:
  br label %merge2
else1:
  br label %merge2
merge2:
  %7 = phi i64 [ 1, %then0 ], [ 0, %else1 ]
  ret i64 %7
}

define i64 @Range_next(%Range* %self) {
entry:
  %0 = getelementptr %Range, %Range* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = getelementptr %Range, %Range* %self, i32 0, i32 1
  %3 = load i64, i64* %2
  %4 = icmp slt i64 %1, %3
  %5 = zext i1 %4 to i64
  %6 = icmp ne i64 %5, 0
  br i1 %6, label %then0, label %else1
then0:
  %7 = getelementptr %Range, %Range* %self, i32 0, i32 0
  %8 = load i64, i64* %7
  %value.9 = alloca i64
  store i64 %8, i64* %value.9
  %10 = getelementptr %Range, %Range* %self, i32 0, i32 0
  %11 = load i64, i64* %10
  %12 = getelementptr %Range, %Range* %self, i32 0, i32 2
  %13 = load i64, i64* %12
  %14 = add i64 %11, %13
  %15 = getelementptr %Range, %Range* %self, i32 0, i32 0
  store i64 %14, i64* %15
  %16 = load i64, i64* %value.9
  br label %merge2
else1:
  %17 = sub i64 0, 1
  br label %merge2
merge2:
  %18 = phi i64 [ %16, %then0 ], [ %17, %else1 ]
  ret i64 %18
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.0, i64 0, i64 0))
  %1 = alloca %Range
  %2 = getelementptr %Range, %Range* %1, i32 0, i32 0
  store i64 0, i64* %2
  %3 = getelementptr %Range, %Range* %1, i32 0, i32 1
  store i64 5, i64* %3
  %4 = getelementptr %Range, %Range* %1, i32 0, i32 2
  store i64 1, i64* %4
  %r.5 = alloca %Range*
  store %Range* %1, %Range** %r.5
  br label %loop.start0
loop.start0:
  %6 = load %Range*, %Range** %r.5
  %7 = call i64 @Range_next(%Range* %6)
  %v.8 = alloca i64
  store i64 %7, i64* %v.8
  %9 = load i64, i64* %v.8
  %10 = icmp slt i64 %9, 0
  %11 = zext i1 %10 to i64
  %12 = icmp ne i64 %11, 0
  br i1 %12, label %then3, label %else4
then3:
  br label %loop.end2
else4:
  br label %merge5
merge5:
  %13 = add i64 0, 0
  %14 = load i64, i64* %v.8
  %15 = add i64 %14, 48
  %16 = trunc i64 %15 to i32
  %17 = call i32 @putchar(i32 %16)
  %18 = trunc i64 32 to i32
  %19 = call i32 @putchar(i32 %18)
  br label %loop.start0
loop.end2:
  %20 = trunc i64 10 to i32
  %21 = call i32 @putchar(i32 %20)
  %22 = call i32 @puts(i8* getelementptr ([6 x i8], [6 x i8]* @.str.1, i64 0, i64 0))
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

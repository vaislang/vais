; ModuleID = 'iter_test'
source_filename = "<vais>"

%Range = type { i64, i64, i64 }
declare i32 @usleep(i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @ftell(i64)
declare i32 @puts(i64)
declare i64 @fputs(i8*, i64)
declare i64 @memcpy(i64, i64, i64)
declare void @exit(i32)
declare i64 @strlen(i64)
declare i64 @fputc(i64, i64)
declare i32 @putchar(i32)
declare i64 @malloc(i64)
declare i32 @printf(i8*)
declare i64 @fgets(i64, i64, i64)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @fflush(i64)
declare i64 @fseek(i64, i64, i64)
declare i64 @fgetc(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @feof(i64)
declare void @free(i64)
declare i64 @fopen(i8*, i8*)
declare i32 @fclose(i64)
declare i32 @sched_yield()
declare i32 @strcmp(i8*, i8*)
@.str.0 = private unnamed_addr constant [22 x i8] c"=== Iterator Test ===\00"
@.str.1 = private unnamed_addr constant [14 x i8] c"Range 0 to 5:\00"
@.str.2 = private unnamed_addr constant [16 x i8] c"Sum of 1 to 10:\00"
@.str.3 = private unnamed_addr constant [33 x i8] c"Count elements in range 5 to 15:\00"
@.str.4 = private unnamed_addr constant [13 x i8] c"=== Done ===\00"

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

define i64 @print_num(i64 %n) {
entry:
  %0 = icmp sge i64 %n, 10
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %then0, label %else1
then0:
  %3 = sdiv i64 %n, 10
  %4 = add i64 %3, 48
  %5 = trunc i64 %4 to i32
  %6 = call i32 @putchar(i32 %5)
  br label %merge2
else1:
  br label %merge2
merge2:
  %7 = phi i64 [ 0, %then0 ], [ 0, %else1 ]
  %8 = srem i64 %n, 10
  %9 = add i64 %8, 48
  %10 = trunc i64 %9 to i32
  %11 = call i32 @putchar(i32 %10)
  ret i64 0
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.0, i64 0, i64 0))
  %1 = call i32 @puts(i8* getelementptr ([14 x i8], [14 x i8]* @.str.1, i64 0, i64 0))
  %2 = alloca %Range
  %3 = getelementptr %Range, %Range* %2, i32 0, i32 0
  store i64 0, i64* %3
  %4 = getelementptr %Range, %Range* %2, i32 0, i32 1
  store i64 5, i64* %4
  %5 = getelementptr %Range, %Range* %2, i32 0, i32 2
  store i64 1, i64* %5
  %r.6 = alloca %Range*
  store %Range* %2, %Range** %r.6
  br label %loop.start0
loop.start0:
  %7 = load %Range*, %Range** %r.6
  %8 = call i64 @Range_next(%Range* %7)
  %v.9 = alloca i64
  store i64 %8, i64* %v.9
  %10 = load i64, i64* %v.9
  %11 = icmp slt i64 %10, 0
  %12 = zext i1 %11 to i64
  %13 = icmp ne i64 %12, 0
  br i1 %13, label %then3, label %else4
then3:
  br label %loop.end2
else4:
  br label %merge5
merge5:
  %14 = add i64 0, 0
  %15 = load i64, i64* %v.9
  %16 = call i64 @print_num(i64 %15)
  %17 = trunc i64 32 to i32
  %18 = call i32 @putchar(i32 %17)
  br label %loop.start0
loop.end2:
  %19 = trunc i64 10 to i32
  %20 = call i32 @putchar(i32 %19)
  %21 = call i32 @puts(i8* getelementptr ([16 x i8], [16 x i8]* @.str.2, i64 0, i64 0))
  %22 = alloca %Range
  %23 = getelementptr %Range, %Range* %22, i32 0, i32 0
  store i64 1, i64* %23
  %24 = getelementptr %Range, %Range* %22, i32 0, i32 1
  store i64 11, i64* %24
  %25 = getelementptr %Range, %Range* %22, i32 0, i32 2
  store i64 1, i64* %25
  %r2.26 = alloca %Range*
  store %Range* %22, %Range** %r2.26
  %sum.27 = alloca i64
  store i64 0, i64* %sum.27
  br label %loop.start6
loop.start6:
  %28 = load %Range*, %Range** %r2.26
  %29 = call i64 @Range_next(%Range* %28)
  %v.30 = alloca i64
  store i64 %29, i64* %v.30
  %31 = load i64, i64* %v.30
  %32 = icmp slt i64 %31, 0
  %33 = zext i1 %32 to i64
  %34 = icmp ne i64 %33, 0
  br i1 %34, label %then9, label %else10
then9:
  br label %loop.end8
else10:
  br label %merge11
merge11:
  %35 = add i64 0, 0
  %36 = load i64, i64* %sum.27
  %37 = load i64, i64* %v.30
  %38 = add i64 %36, %37
  store i64 %38, i64* %sum.27
  br label %loop.start6
loop.end8:
  %39 = load i64, i64* %sum.27
  %40 = call i64 @print_num(i64 %39)
  %41 = trunc i64 10 to i32
  %42 = call i32 @putchar(i32 %41)
  %43 = call i32 @puts(i8* getelementptr ([33 x i8], [33 x i8]* @.str.3, i64 0, i64 0))
  %44 = alloca %Range
  %45 = getelementptr %Range, %Range* %44, i32 0, i32 0
  store i64 5, i64* %45
  %46 = getelementptr %Range, %Range* %44, i32 0, i32 1
  store i64 15, i64* %46
  %47 = getelementptr %Range, %Range* %44, i32 0, i32 2
  store i64 1, i64* %47
  %r3.48 = alloca %Range*
  store %Range* %44, %Range** %r3.48
  %count.49 = alloca i64
  store i64 0, i64* %count.49
  br label %loop.start12
loop.start12:
  %50 = load %Range*, %Range** %r3.48
  %51 = call i64 @Range_next(%Range* %50)
  %v.52 = alloca i64
  store i64 %51, i64* %v.52
  %53 = load i64, i64* %v.52
  %54 = icmp slt i64 %53, 0
  %55 = zext i1 %54 to i64
  %56 = icmp ne i64 %55, 0
  br i1 %56, label %then15, label %else16
then15:
  br label %loop.end14
else16:
  br label %merge17
merge17:
  %57 = add i64 0, 0
  %58 = load i64, i64* %count.49
  %59 = add i64 %58, 1
  store i64 %59, i64* %count.49
  br label %loop.start12
loop.end14:
  %60 = load i64, i64* %count.49
  %61 = call i64 @print_num(i64 %60)
  %62 = trunc i64 10 to i32
  %63 = call i32 @putchar(i32 %62)
  %64 = call i32 @puts(i8* getelementptr ([13 x i8], [13 x i8]* @.str.4, i64 0, i64 0))
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

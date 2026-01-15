; ModuleID = 'std_import_test'
source_filename = "<vais>"

%VecIter = type { i64, i64, i64 }
%Take = type { i64, i64 }
%Range = type { i64, i64, i64 }
%SliceIter = type { i64, i64, i64 }
declare i32 @puts(i64)
declare i64 @malloc(i64)
declare i32 @putchar(i32)
declare i64 @fputs(i8*, i64)
declare i64 @fputc(i64, i64)
declare i64 @strlen(i64)
declare i32 @fclose(i64)
declare void @free(i64)
declare i32 @printf(i8*)
declare i64 @fseek(i64, i64, i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @fgetc(i64)
declare i64 @fopen(i8*, i8*)
declare i64 @fgets(i64, i64, i64)
declare void @exit(i32)
declare i64 @ftell(i64)
declare i64 @fflush(i64)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @memcpy(i64, i64, i64)
declare i64 @feof(i64)
@.str.0 = private unnamed_addr constant [21 x i8] c"Testing std imports:\00"
@.str.1 = private unnamed_addr constant [20 x i8] c"iter_count(0, 10) =\00"
@.str.2 = private unnamed_addr constant [18 x i8] c"iter_sum(1, 11) =\00"
@.str.3 = private unnamed_addr constant [31 x i8] c"iter_contains(0, 10, 5) = true\00"
@.str.4 = private unnamed_addr constant [33 x i8] c"iter_contains(0, 10, 15) = false\00"
@.str.5 = private unnamed_addr constant [6 x i8] c"Done!\00"

define i64 @Range_has_next(%Range* %self) {
entry:
  %0 = getelementptr %Range, %Range* %self, i32 0, i32 2
  %1 = load i64, i64* %0
  %2 = icmp sgt i64 %1, 0
  %3 = zext i1 %2 to i64
  %4 = icmp ne i64 %3, 0
  br i1 %4, label %then0, label %else1
then0:
  %5 = getelementptr %Range, %Range* %self, i32 0, i32 0
  %6 = load i64, i64* %5
  %7 = getelementptr %Range, %Range* %self, i32 0, i32 1
  %8 = load i64, i64* %7
  %9 = icmp slt i64 %6, %8
  %10 = zext i1 %9 to i64
  %11 = icmp ne i64 %10, 0
  br i1 %11, label %then3, label %else4
then3:
  br label %merge5
else4:
  br label %merge5
merge5:
  %12 = phi i64 [ 1, %then3 ], [ 0, %else4 ]
  br label %merge2
else1:
  %13 = getelementptr %Range, %Range* %self, i32 0, i32 0
  %14 = load i64, i64* %13
  %15 = getelementptr %Range, %Range* %self, i32 0, i32 1
  %16 = load i64, i64* %15
  %17 = icmp sgt i64 %14, %16
  %18 = zext i1 %17 to i64
  %19 = icmp ne i64 %18, 0
  br i1 %19, label %then6, label %else7
then6:
  br label %merge8
else7:
  br label %merge8
merge8:
  %20 = phi i64 [ 1, %then6 ], [ 0, %else7 ]
  br label %merge2
merge2:
  %21 = phi i64 [ %12, %then0 ], [ %20, %else1 ]
  ret i64 %21
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

define i64 @VecIter_has_next(%VecIter* %self) {
entry:
  %0 = getelementptr %VecIter, %VecIter* %self, i32 0, i32 2
  %1 = load i64, i64* %0
  %2 = getelementptr %VecIter, %VecIter* %self, i32 0, i32 1
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

define i64 @VecIter_peek(%VecIter* %self) {
entry:
  %0 = getelementptr %VecIter, %VecIter* %self, i32 0, i32 2
  %1 = load i64, i64* %0
  %2 = getelementptr %VecIter, %VecIter* %self, i32 0, i32 1
  %3 = load i64, i64* %2
  %4 = icmp slt i64 %1, %3
  %5 = zext i1 %4 to i64
  %6 = icmp ne i64 %5, 0
  br i1 %6, label %then0, label %else1
then0:
  %7 = getelementptr %VecIter, %VecIter* %self, i32 0, i32 0
  %8 = load i64, i64* %7
  %9 = getelementptr %VecIter, %VecIter* %self, i32 0, i32 2
  %10 = load i64, i64* %9
  %11 = mul i64 %10, 8
  %12 = add i64 %8, %11
  %13 = call i64 @__load_i64(i64 %12)
  br label %merge2
else1:
  %14 = sub i64 0, 1
  br label %merge2
merge2:
  %15 = phi i64 [ %13, %then0 ], [ %14, %else1 ]
  ret i64 %15
}

define i64 @VecIter_next(%VecIter* %self) {
entry:
  %0 = getelementptr %VecIter, %VecIter* %self, i32 0, i32 2
  %1 = load i64, i64* %0
  %2 = getelementptr %VecIter, %VecIter* %self, i32 0, i32 1
  %3 = load i64, i64* %2
  %4 = icmp slt i64 %1, %3
  %5 = zext i1 %4 to i64
  %6 = icmp ne i64 %5, 0
  br i1 %6, label %then0, label %else1
then0:
  %7 = getelementptr %VecIter, %VecIter* %self, i32 0, i32 0
  %8 = load i64, i64* %7
  %9 = getelementptr %VecIter, %VecIter* %self, i32 0, i32 2
  %10 = load i64, i64* %9
  %11 = mul i64 %10, 8
  %12 = add i64 %8, %11
  %13 = call i64 @__load_i64(i64 %12)
  %value.14 = alloca i64
  store i64 %13, i64* %value.14
  %15 = getelementptr %VecIter, %VecIter* %self, i32 0, i32 2
  %16 = load i64, i64* %15
  %17 = add i64 %16, 1
  %18 = getelementptr %VecIter, %VecIter* %self, i32 0, i32 2
  store i64 %17, i64* %18
  %19 = load i64, i64* %value.14
  br label %merge2
else1:
  %20 = sub i64 0, 1
  br label %merge2
merge2:
  %21 = phi i64 [ %19, %then0 ], [ %20, %else1 ]
  ret i64 %21
}

define i64 @SliceIter_has_next(%SliceIter* %self) {
entry:
  %0 = getelementptr %SliceIter, %SliceIter* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = getelementptr %SliceIter, %SliceIter* %self, i32 0, i32 1
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

define i64 @SliceIter_next(%SliceIter* %self) {
entry:
  %0 = getelementptr %SliceIter, %SliceIter* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = getelementptr %SliceIter, %SliceIter* %self, i32 0, i32 1
  %3 = load i64, i64* %2
  %4 = icmp slt i64 %1, %3
  %5 = zext i1 %4 to i64
  %6 = icmp ne i64 %5, 0
  br i1 %6, label %then0, label %else1
then0:
  %7 = getelementptr %SliceIter, %SliceIter* %self, i32 0, i32 0
  %8 = load i64, i64* %7
  %9 = call i64 @__load_i64(i64 %8)
  %value.10 = alloca i64
  store i64 %9, i64* %value.10
  %11 = getelementptr %SliceIter, %SliceIter* %self, i32 0, i32 0
  %12 = load i64, i64* %11
  %13 = getelementptr %SliceIter, %SliceIter* %self, i32 0, i32 2
  %14 = load i64, i64* %13
  %15 = add i64 %12, %14
  %16 = getelementptr %SliceIter, %SliceIter* %self, i32 0, i32 0
  store i64 %15, i64* %16
  %17 = load i64, i64* %value.10
  br label %merge2
else1:
  %18 = sub i64 0, 1
  br label %merge2
merge2:
  %19 = phi i64 [ %17, %then0 ], [ %18, %else1 ]
  ret i64 %19
}

define i64 @Take_next(%Take* %self) {
entry:
  %0 = getelementptr %Take, %Take* %self, i32 0, i32 1
  %1 = load i64, i64* %0
  %2 = icmp sgt i64 %1, 0
  %3 = zext i1 %2 to i64
  %4 = icmp ne i64 %3, 0
  br i1 %4, label %then0, label %else1
then0:
  %5 = getelementptr %Take, %Take* %self, i32 0, i32 1
  %6 = load i64, i64* %5
  %7 = sub i64 %6, 1
  %8 = getelementptr %Take, %Take* %self, i32 0, i32 1
  store i64 %7, i64* %8
  br label %merge2
else1:
  %9 = sub i64 0, 1
  br label %merge2
merge2:
  %10 = phi i64 [ 0, %then0 ], [ %9, %else1 ]
  ret i64 %10
}

define i64 @iter_count(i64 %start, i64 %end) {
entry:
  %0 = icmp sgt i64 %end, %start
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %then0, label %else1
then0:
  %3 = sub i64 %end, %start
  br label %merge2
else1:
  br label %merge2
merge2:
  %4 = phi i64 [ %3, %then0 ], [ 0, %else1 ]
  ret i64 %4
}

define i64 @iter_sum(i64 %start, i64 %end) {
entry:
  %0 = icmp sle i64 %end, %start
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %then0, label %else1
then0:
  br label %merge2
else1:
  %3 = sub i64 %end, 1
  %n.4 = alloca i64
  store i64 %3, i64* %n.4
  %k.5 = alloca i64
  store i64 %start, i64* %k.5
  %6 = load i64, i64* %n.4
  %7 = load i64, i64* %n.4
  %8 = add i64 %7, 1
  %9 = mul i64 %6, %8
  %10 = sdiv i64 %9, 2
  %n_sum.11 = alloca i64
  store i64 %10, i64* %n_sum.11
  %12 = load i64, i64* %k.5
  %13 = load i64, i64* %k.5
  %14 = sub i64 %13, 1
  %15 = mul i64 %12, %14
  %16 = sdiv i64 %15, 2
  %k_sum.17 = alloca i64
  store i64 %16, i64* %k_sum.17
  %18 = load i64, i64* %n_sum.11
  %19 = load i64, i64* %k_sum.17
  %20 = sub i64 %18, %19
  br label %merge2
merge2:
  %21 = phi i64 [ 0, %then0 ], [ %20, %else1 ]
  ret i64 %21
}

define i64 @iter_product(i64 %start, i64 %end) {
entry:
  %0 = icmp sle i64 %end, %start
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %then0, label %else1
then0:
  br label %merge2
else1:
  %result.3 = alloca i64
  store i64 1, i64* %result.3
  %4 = sub i64 %end, %start
  %5 = icmp sge i64 %4, 1
  %6 = zext i1 %5 to i64
  %7 = icmp ne i64 %6, 0
  br i1 %7, label %then3, label %else4
then3:
  %8 = load i64, i64* %result.3
  %9 = mul i64 %8, %start
  store i64 %9, i64* %result.3
  br label %merge5
else4:
  br label %merge5
merge5:
  %10 = phi i64 [ %9, %then3 ], [ 0, %else4 ]
  %11 = sub i64 %end, %start
  %12 = icmp sge i64 %11, 2
  %13 = zext i1 %12 to i64
  %14 = icmp ne i64 %13, 0
  br i1 %14, label %then6, label %else7
then6:
  %15 = load i64, i64* %result.3
  %16 = add i64 %start, 1
  %17 = mul i64 %15, %16
  store i64 %17, i64* %result.3
  br label %merge8
else7:
  br label %merge8
merge8:
  %18 = phi i64 [ %17, %then6 ], [ 0, %else7 ]
  %19 = sub i64 %end, %start
  %20 = icmp sge i64 %19, 3
  %21 = zext i1 %20 to i64
  %22 = icmp ne i64 %21, 0
  br i1 %22, label %then9, label %else10
then9:
  %23 = load i64, i64* %result.3
  %24 = add i64 %start, 2
  %25 = mul i64 %23, %24
  store i64 %25, i64* %result.3
  br label %merge11
else10:
  br label %merge11
merge11:
  %26 = phi i64 [ %25, %then9 ], [ 0, %else10 ]
  %27 = sub i64 %end, %start
  %28 = icmp sge i64 %27, 4
  %29 = zext i1 %28 to i64
  %30 = icmp ne i64 %29, 0
  br i1 %30, label %then12, label %else13
then12:
  %31 = load i64, i64* %result.3
  %32 = add i64 %start, 3
  %33 = mul i64 %31, %32
  store i64 %33, i64* %result.3
  br label %merge14
else13:
  br label %merge14
merge14:
  %34 = phi i64 [ %33, %then12 ], [ 0, %else13 ]
  %35 = sub i64 %end, %start
  %36 = icmp sge i64 %35, 5
  %37 = zext i1 %36 to i64
  %38 = icmp ne i64 %37, 0
  br i1 %38, label %then15, label %else16
then15:
  %39 = load i64, i64* %result.3
  %40 = add i64 %start, 4
  %41 = mul i64 %39, %40
  store i64 %41, i64* %result.3
  br label %merge17
else16:
  br label %merge17
merge17:
  %42 = phi i64 [ %41, %then15 ], [ 0, %else16 ]
  %43 = load i64, i64* %result.3
  br label %merge2
merge2:
  %44 = phi i64 [ 1, %then0 ], [ %43, %else1 ]
  ret i64 %44
}

define i64 @iter_min(i64 %start, i64 %end) {
entry:
  %0 = icmp sle i64 %end, %start
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %then0, label %else1
then0:
  br label %merge2
else1:
  br label %merge2
merge2:
  %3 = phi i64 [ 0, %then0 ], [ %start, %else1 ]
  ret i64 %3
}

define i64 @iter_max(i64 %start, i64 %end) {
entry:
  %0 = icmp sle i64 %end, %start
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %then0, label %else1
then0:
  br label %merge2
else1:
  %3 = sub i64 %end, 1
  br label %merge2
merge2:
  %4 = phi i64 [ 0, %then0 ], [ %3, %else1 ]
  ret i64 %4
}

define i64 @iter_contains(i64 %start, i64 %end, i64 %value) {
entry:
  %0 = icmp sge i64 %value, %start
  %1 = zext i1 %0 to i64
  %2 = icmp slt i64 %value, %end
  %3 = zext i1 %2 to i64
  %4 = icmp ne i64 %1, 0
  %5 = icmp ne i64 %3, 0
  %6 = and i1 %4, %5
  %7 = zext i1 %6 to i64
  %8 = icmp ne i64 %7, 0
  br i1 %8, label %then0, label %else1
then0:
  br label %merge2
else1:
  br label %merge2
merge2:
  %9 = phi i64 [ 1, %then0 ], [ 0, %else1 ]
  ret i64 %9
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([21 x i8], [21 x i8]* @.str.0, i64 0, i64 0))
  %1 = call i64 @iter_count(i64 0, i64 10)
  %count.2 = alloca i64
  store i64 %1, i64* %count.2
  %3 = call i32 @puts(i8* getelementptr ([20 x i8], [20 x i8]* @.str.1, i64 0, i64 0))
  %4 = load i64, i64* %count.2
  %5 = sdiv i64 %4, 10
  %6 = add i64 %5, 48
  %7 = trunc i64 %6 to i32
  %8 = call i32 @putchar(i32 %7)
  %9 = load i64, i64* %count.2
  %10 = srem i64 %9, 10
  %11 = add i64 %10, 48
  %12 = trunc i64 %11 to i32
  %13 = call i32 @putchar(i32 %12)
  %14 = trunc i64 10 to i32
  %15 = call i32 @putchar(i32 %14)
  %16 = call i64 @iter_sum(i64 1, i64 11)
  %sum.17 = alloca i64
  store i64 %16, i64* %sum.17
  %18 = call i32 @puts(i8* getelementptr ([18 x i8], [18 x i8]* @.str.2, i64 0, i64 0))
  %19 = load i64, i64* %sum.17
  %20 = sdiv i64 %19, 10
  %21 = add i64 %20, 48
  %22 = trunc i64 %21 to i32
  %23 = call i32 @putchar(i32 %22)
  %24 = load i64, i64* %sum.17
  %25 = srem i64 %24, 10
  %26 = add i64 %25, 48
  %27 = trunc i64 %26 to i32
  %28 = call i32 @putchar(i32 %27)
  %29 = trunc i64 10 to i32
  %30 = call i32 @putchar(i32 %29)
  %31 = call i64 @iter_contains(i64 0, i64 10, i64 5)
  %contains.32 = alloca i64
  store i64 %31, i64* %contains.32
  %33 = call i32 @puts(i8* getelementptr ([31 x i8], [31 x i8]* @.str.3, i64 0, i64 0))
  %34 = call i64 @iter_contains(i64 0, i64 10, i64 15)
  %not_contains.35 = alloca i64
  store i64 %34, i64* %not_contains.35
  %36 = call i32 @puts(i8* getelementptr ([33 x i8], [33 x i8]* @.str.4, i64 0, i64 0))
  %37 = call i32 @puts(i8* getelementptr ([6 x i8], [6 x i8]* @.str.5, i64 0, i64 0))
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

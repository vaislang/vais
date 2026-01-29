; ModuleID = 'std_import_test'
source_filename = "<vais>"

%VecIter = type { i64, i64, i64 }
%SliceIter = type { i64, i64, i64 }
%Range = type { i64, i64, i64 }
%Take = type { i64, i64 }
declare i64 @memcpy(i64, i64, i64)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @vais_gc_set_threshold(i64)
declare i64 @vais_gc_collections()
declare i32 @printf(i8*)
declare i64 @vais_gc_remove_root(i64)
declare i64 @vais_gc_alloc(i64, i32)
declare i32 @puts(i64)
declare i64 @fflush(i64)
declare i64 @vais_gc_print_stats()
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @fseek(i64, i64, i64)
declare i64 @fgets(i64, i64, i64)
declare i64 @vais_gc_add_root(i64)
declare i64 @feof(i64)
declare i32 @putchar(i32)
declare i64 @fopen(i8*, i8*)
declare i32 @fclose(i64)
declare i64 @memcpy_str(i64, i8*, i64)
declare i64 @vais_gc_init()
declare i32 @sched_yield()
declare i32 @usleep(i64)
declare void @exit(i32)
declare void @free(i64)
declare i64 @vais_gc_objects_count()
declare i64 @malloc(i64)
declare i64 @fputs(i8*, i64)
declare i32 @strcmp(i8*, i8*)
declare i64 @fgetc(i64)
declare i64 @fputc(i64, i64)
declare i64 @ftell(i64)
declare i64 @strlen(i8*)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @vais_gc_bytes_allocated()
declare i64 @vais_gc_collect()
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
  %21 = phi i64 [ %12, %merge5 ], [ %20, %merge8 ]
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
  %9 = getelementptr %Range, %Range* %self, i32 0, i32 0
  %10 = load i64, i64* %9
  %11 = getelementptr %Range, %Range* %self, i32 0, i32 2
  %12 = load i64, i64* %11
  %13 = add i64 %10, %12
  %14 = getelementptr %Range, %Range* %self, i32 0, i32 0
  store i64 %13, i64* %14
  br label %merge2
else1:
  %15 = sub i64 0, 1
  br label %merge2
merge2:
  %16 = phi i64 [ %8, %then0 ], [ %15, %else1 ]
  ret i64 %16
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
  %14 = getelementptr %VecIter, %VecIter* %self, i32 0, i32 2
  %15 = load i64, i64* %14
  %16 = add i64 %15, 1
  %17 = getelementptr %VecIter, %VecIter* %self, i32 0, i32 2
  store i64 %16, i64* %17
  br label %merge2
else1:
  %18 = sub i64 0, 1
  br label %merge2
merge2:
  %19 = phi i64 [ %13, %then0 ], [ %18, %else1 ]
  ret i64 %19
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
  %10 = getelementptr %SliceIter, %SliceIter* %self, i32 0, i32 0
  %11 = load i64, i64* %10
  %12 = getelementptr %SliceIter, %SliceIter* %self, i32 0, i32 2
  %13 = load i64, i64* %12
  %14 = add i64 %11, %13
  %15 = getelementptr %SliceIter, %SliceIter* %self, i32 0, i32 0
  store i64 %14, i64* %15
  br label %merge2
else1:
  %16 = sub i64 0, 1
  br label %merge2
merge2:
  %17 = phi i64 [ %9, %then0 ], [ %16, %else1 ]
  ret i64 %17
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
  %4 = add i64 %3, 1
  %5 = mul i64 %3, %4
  %6 = sdiv i64 %5, 2
  %7 = sub i64 %start, 1
  %8 = mul i64 %start, %7
  %9 = sdiv i64 %8, 2
  %10 = sub i64 %6, %9
  br label %merge2
merge2:
  %11 = phi i64 [ 0, %then0 ], [ %10, %else1 ]
  ret i64 %11
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
  %10 = add i64 0, 0
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
  %18 = add i64 0, 0
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
  %26 = add i64 0, 0
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
  %34 = add i64 0, 0
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
  %42 = add i64 0, 0
  %43 = load i64, i64* %result.3
  br label %merge2
merge2:
  %44 = phi i64 [ 1, %then0 ], [ %43, %merge17 ]
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
  %1 = sext i32 %0 to i64
  %2 = call i64 @iter_count(i64 0, i64 10)
  %3 = call i32 @puts(i8* getelementptr ([20 x i8], [20 x i8]* @.str.1, i64 0, i64 0))
  %4 = sext i32 %3 to i64
  %5 = sdiv i64 %2, 10
  %6 = add i64 %5, 48
  %7 = trunc i64 %6 to i32
  %8 = call i32 @putchar(i32 %7)
  %9 = sext i32 %8 to i64
  %10 = srem i64 %2, 10
  %11 = add i64 %10, 48
  %12 = trunc i64 %11 to i32
  %13 = call i32 @putchar(i32 %12)
  %14 = sext i32 %13 to i64
  %15 = trunc i64 10 to i32
  %16 = call i32 @putchar(i32 %15)
  %17 = sext i32 %16 to i64
  %18 = call i64 @iter_sum(i64 1, i64 11)
  %19 = call i32 @puts(i8* getelementptr ([18 x i8], [18 x i8]* @.str.2, i64 0, i64 0))
  %20 = sext i32 %19 to i64
  %21 = sdiv i64 %18, 10
  %22 = add i64 %21, 48
  %23 = trunc i64 %22 to i32
  %24 = call i32 @putchar(i32 %23)
  %25 = sext i32 %24 to i64
  %26 = srem i64 %18, 10
  %27 = add i64 %26, 48
  %28 = trunc i64 %27 to i32
  %29 = call i32 @putchar(i32 %28)
  %30 = sext i32 %29 to i64
  %31 = trunc i64 10 to i32
  %32 = call i32 @putchar(i32 %31)
  %33 = sext i32 %32 to i64
  %34 = call i64 @iter_contains(i64 0, i64 10, i64 5)
  %35 = call i32 @puts(i8* getelementptr ([31 x i8], [31 x i8]* @.str.3, i64 0, i64 0))
  %36 = sext i32 %35 to i64
  %37 = call i64 @iter_contains(i64 0, i64 10, i64 15)
  %38 = call i32 @puts(i8* getelementptr ([33 x i8], [33 x i8]* @.str.4, i64 0, i64 0))
  %39 = sext i32 %38 to i64
  %40 = call i32 @puts(i8* getelementptr ([6 x i8], [6 x i8]* @.str.5, i64 0, i64 0))
  %41 = sext i32 %40 to i64
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

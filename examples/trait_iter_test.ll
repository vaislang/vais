; ModuleID = 'trait_iter_test'
source_filename = "<vais>"

%Range = type { i64, i64, i64 }
declare i64 @feof(i64)
declare i64 @malloc(i64)
declare void @free(i64)
declare i32 @tolower(i32)
declare i64 @fopen(i8*, i8*)
declare i64 @fputs(i8*, i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @memcpy(i64, i64, i64)
declare i64 @atol(i8*)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare double @atof(i8*)
declare i64 @vais_gc_print_stats()
declare i32 @puts(i64)
declare i32 @rand()
declare double @sqrt(double)
declare i64 @strcat(i64, i8*)
declare i64 @fgets(i64, i64, i64)
declare i64 @fputc(i64, i64)
declare double @fabs(double)
declare i32 @toupper(i32)
declare i64 @fseek(i64, i64, i64)
declare void @exit(i32)
declare i32 @isalpha(i32)
declare i64 @vais_gc_bytes_allocated()
declare i64 @vais_gc_objects_count()
declare i64 @vais_gc_collections()
declare i32 @fclose(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i32 @usleep(i64)
declare i32 @isdigit(i32)
declare i32 @strcmp(i8*, i8*)
declare i64 @vais_gc_init()
declare i64 @vais_gc_add_root(i64)
declare i64 @vais_gc_remove_root(i64)
declare void @srand(i32)
declare i64 @strcpy(i64, i8*)
declare i32 @printf(i8*, ...)
declare i32 @putchar(i32)
declare i32 @sched_yield()
declare i32 @atoi(i8*)
declare i64 @strlen(i8*)
declare i64 @labs(i64)
declare i64 @fflush(i64)
declare i64 @fgetc(i64)
declare i64 @memcpy_str(i64, i8*, i64)
declare i64 @vais_gc_alloc(i64, i32)
declare i64 @vais_gc_collect()
declare i64 @vais_gc_set_threshold(i64)
declare i64 @ftell(i64)
declare i64 @fread(i64, i64, i64, i64)
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

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

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.0, i64 0, i64 0))
  %1 = sext i32 %0 to i64
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
  %9 = icmp slt i64 %8, 0
  %10 = zext i1 %9 to i64
  %11 = icmp ne i64 %10, 0
  br i1 %11, label %then3, label %else4
then3:
  br label %loop.end2
else4:
  br label %merge5
merge5:
  %12 = add i64 0, 0
  %13 = add i64 %8, 48
  %14 = trunc i64 %13 to i32
  %15 = call i32 @putchar(i32 %14)
  %16 = sext i32 %15 to i64
  %17 = trunc i64 32 to i32
  %18 = call i32 @putchar(i32 %17)
  %19 = sext i32 %18 to i64
  br label %loop.start0
loop.end2:
  %20 = trunc i64 10 to i32
  %21 = call i32 @putchar(i32 %20)
  %22 = sext i32 %21 to i64
  %23 = call i32 @puts(i8* getelementptr ([6 x i8], [6 x i8]* @.str.1, i64 0, i64 0))
  %24 = sext i32 %23 to i64
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

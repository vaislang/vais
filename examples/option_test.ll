; ModuleID = 'option_test'
source_filename = "<vais>"

%Option = type { i32, { i64 } }
declare i64 @vais_gc_print_stats()
declare i64 @vais_gc_alloc(i64, i32)
declare i64 @atol(i8*)
declare i64 @vais_gc_bytes_allocated()
declare i64 @fseek(i64, i64, i64)
declare i64 @vais_gc_collect()
declare i32 @sched_yield()
declare void @free(i64)
declare i64 @fputc(i64, i64)
declare i64 @fflush(i64)
declare i64 @memcpy(i64, i64, i64)
declare i64 @feof(i64)
declare i32 @puts(i8*)
declare i64 @fputs(i8*, i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i32 @fclose(i64)
declare i64 @labs(i64)
declare i32 @isalpha(i32)
declare void @exit(i32)
declare i64 @vais_gc_collections()
declare i32 @toupper(i32)
declare i64 @vais_gc_remove_root(i64)
declare i64 @vais_gc_add_root(i64)
declare i32 @putchar(i32)
declare i32 @strcmp(i8*, i8*)
declare i32 @rand()
declare i64 @fread(i64, i64, i64, i64)
declare i64 @strcpy(i64, i8*)
declare i64 @vais_gc_objects_count()
declare i64 @fwrite(i64, i64, i64, i64)
declare i32 @tolower(i32)
declare i64 @fgetc(i64)
declare double @sqrt(double)
declare i64 @strcat(i64, i8*)
declare i64 @memcpy_str(i64, i8*, i64)
declare i64 @vais_gc_set_threshold(i64)
declare void @srand(i32)
declare i32 @printf(i8*, ...)
declare i64 @fgets(i64, i64, i64)
declare i64 @strlen(i8*)
declare double @fabs(double)
declare i32 @isdigit(i32)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare i64 @vais_gc_init()
declare i64 @ftell(i64)
declare i32 @atoi(i8*)
declare double @atof(i8*)
declare i64 @malloc(i64)
declare i32 @usleep(i64)
declare i64 @fopen(i8*, i8*)
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

define i64 @Option_is_some(%Option* %self) {
entry:
  %0 = alloca %Option
  store %Option %self, %Option* %0
  br label %match.check1
match.check1:
  %1 = getelementptr { i32 }, { i32 }* %0, i32 0, i32 0
  %2 = load i32, i32* %1
  %3 = icmp eq i32 %2, 1
  br i1 %3, label %match.arm3, label %match.check2
match.arm3:
  %4 = getelementptr { i32, i64 }, { i32, i64 }* %0, i32 0, i32 1
  %5 = load i64, i64* %4
  br label %match.merge0
match.check2:
  %6 = getelementptr { i32 }, { i32 }* %0, i32 0, i32 0
  %7 = load i32, i32* %6
  %8 = icmp eq i32 %7, 0
  br i1 %8, label %match.arm4, label %match.merge0
match.arm4:
  br label %match.merge0
match.merge0:
  %9 = phi i64 [ 1, %match.arm3 ], [ 0, %match.arm4 ]
  ret i64 %9
}

define i64 @Option_is_none(%Option* %self) {
entry:
  %0 = alloca %Option
  store %Option %self, %Option* %0
  br label %match.check1
match.check1:
  %1 = getelementptr { i32 }, { i32 }* %0, i32 0, i32 0
  %2 = load i32, i32* %1
  %3 = icmp eq i32 %2, 1
  br i1 %3, label %match.arm3, label %match.check2
match.arm3:
  %4 = getelementptr { i32, i64 }, { i32, i64 }* %0, i32 0, i32 1
  %5 = load i64, i64* %4
  br label %match.merge0
match.check2:
  %6 = getelementptr { i32 }, { i32 }* %0, i32 0, i32 0
  %7 = load i32, i32* %6
  %8 = icmp eq i32 %7, 0
  br i1 %8, label %match.arm4, label %match.merge0
match.arm4:
  br label %match.merge0
match.merge0:
  %9 = phi i64 [ 0, %match.arm3 ], [ 1, %match.arm4 ]
  ret i64 %9
}

define i64 @Option_unwrap_or(%Option* %self, i64 %default) {
entry:
  %0 = alloca %Option
  store %Option %self, %Option* %0
  br label %match.check1
match.check1:
  %1 = getelementptr { i32 }, { i32 }* %0, i32 0, i32 0
  %2 = load i32, i32* %1
  %3 = icmp eq i32 %2, 1
  br i1 %3, label %match.arm3, label %match.check2
match.arm3:
  %4 = getelementptr { i32, i64 }, { i32, i64 }* %0, i32 0, i32 1
  %5 = load i64, i64* %4
  br label %match.merge0
match.check2:
  %7 = getelementptr { i32 }, { i32 }* %0, i32 0, i32 0
  %8 = load i32, i32* %7
  %9 = icmp eq i32 %8, 0
  br i1 %9, label %match.arm4, label %match.merge0
match.arm4:
  br label %match.merge0
match.merge0:
  %10 = phi i64 [ %5, %match.arm3 ], [ %default, %match.arm4 ]
  ret i64 %10
}

define i64 @main() {
entry:
  %0 = alloca %Option
  %1 = getelementptr %Option, %Option* %0, i32 0, i32 0
  store i32 1, i32* %1
  %2 = getelementptr %Option, %Option* %0, i32 0, i32 1
  store i64 42, i64* %2
  %x.3 = alloca %Option*
  store %Option* %0, %Option** %x.3
  %4 = load %Option*, %Option** %x.3
  br label %match.check1
match.check1:
  %5 = getelementptr { i32 }, { i32 }* %4, i32 0, i32 0
  %6 = load i32, i32* %5
  %7 = icmp eq i32 %6, 1
  br i1 %7, label %match.arm3, label %match.check2
match.arm3:
  %8 = getelementptr { i32, i64 }, { i32, i64 }* %4, i32 0, i32 1
  %9 = load i64, i64* %8
  br label %match.merge0
match.check2:
  %11 = getelementptr { i32 }, { i32 }* %4, i32 0, i32 0
  %12 = load i32, i32* %11
  %13 = icmp eq i32 %12, 0
  br i1 %13, label %match.arm4, label %match.merge0
match.arm4:
  br label %match.merge0
match.merge0:
  %14 = phi i64 [ %9, %match.arm3 ], [ 0, %match.arm4 ]
  ret i64 %14
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

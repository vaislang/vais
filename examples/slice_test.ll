; ModuleID = 'slice_test'
source_filename = "<vais>"

declare i64 @fopen(i8*, i8*)
declare i64 @feof(i64)
declare void @exit(i32)
declare i64 @fputc(i64, i64)
declare double @cos(double)
declare i64 @vais_gc_collections()
declare i64 @malloc(i64)
declare i32 @toupper(i32)
declare i64 @vais_gc_add_root(i64)
declare i32 @isalpha(i32)
declare i64 @vais_gc_set_threshold(i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @strcpy(i64, i8*)
declare i32 @putchar(i32)
declare i64 @vais_gc_alloc(i64, i32)
declare i64 @vais_gc_collect()
declare void @srand(i32)
declare i64 @strcat(i64, i8*)
declare i64 @atol(i64)
declare i64 @labs(i64)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare i64 @vais_gc_init()
declare i64 @strlen(i8*)
declare i64 @fgetc(i64)
declare i64 @vais_gc_print_stats()
declare i64 @fwrite(i64, i64, i64, i64)
declare i32 @puts(i8*)
declare double @sqrt(double)
declare double @exp(double)
declare i64 @memcpy_str(i64, i8*, i64)
declare double @atof(i8*)
declare i64 @ftell(i64)
declare i64 @vais_gc_objects_count()
declare i32 @strcmp(i8*, i8*)
declare i32 @usleep(i64)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @fflush(i64)
declare i32 @rand()
declare void @free(i64)
declare i32 @printf(i8*, ...)
declare i32 @tolower(i32)
declare i32 @atoi(i8*)
declare double @fabs(double)
declare double @log(double)
declare i32 @fclose(i64)
declare i64 @fseek(i64, i64, i64)
declare i64 @fgets(i64, i64, i64)
declare i32 @isdigit(i32)
declare i64 @memcpy(i64, i64, i64)
declare double @sin(double)
declare i64 @vais_gc_remove_root(i64)
declare i64 @fputs(i8*, i64)
declare i64 @vais_gc_bytes_allocated()
declare i32 @sched_yield()
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

@.str.0 = private unnamed_addr constant [23 x i8] c"Testing array slicing:\00"
@.str.1 = private unnamed_addr constant [11 x i8] c"arr[1..3]:\00"
@.str.2 = private unnamed_addr constant [12 x i8] c"arr[1..=3]:\00"
@.str.3 = private unnamed_addr constant [11 x i8] c"arr[0..2]:\00"

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([23 x i8], [23 x i8]* @.str.0, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = alloca [5  x i64]
  %3 = getelementptr [5  x i64], [5  x i64]* %2, i64 0, i64 0
  store i64 10, i64* %3
  %4 = getelementptr [5  x i64], [5  x i64]* %2, i64 0, i64 1
  store i64 20, i64* %4
  %5 = getelementptr [5  x i64], [5  x i64]* %2, i64 0, i64 2
  store i64 30, i64* %5
  %6 = getelementptr [5  x i64], [5  x i64]* %2, i64 0, i64 3
  store i64 40, i64* %6
  %7 = getelementptr [5  x i64], [5  x i64]* %2, i64 0, i64 4
  store i64 50, i64* %7
  %8 = getelementptr [5  x i64], [5  x i64]* %2, i64 0, i64 0
  %9 = sub i64 3, 1
  %10 = mul i64 %9, 8
  %11 = call i8* @malloc(i64 %10)
  %12 = bitcast i8* %11 to i64*
  %13 = alloca i64
  store i64 0, i64* %13
  br label %slice_loop0
slice_loop0:
  %14 = load i64, i64* %13
  %15 = icmp slt i64 %14, %9
  br i1 %15, label %slice_body1, label %slice_end2
slice_body1:
  %16 = add i64 1, %14
  %17 = getelementptr i64, i64* %8, i64 %16
  %18 = load i64, i64* %17
  %19 = getelementptr i64, i64* %12, i64 %14
  store i64 %18, i64* %19
  %20 = add i64 %14, 1
  store i64 %20, i64* %13
  br label %slice_loop0
slice_end2:
  %21 = call i32 @puts(i8* getelementptr ([11 x i8], [11 x i8]* @.str.1, i64 0, i64 0))
  %22 = sext i32 %21 to i64
  %23 = getelementptr i64, i64* %12, i64 0
  %24 = load i64, i64* %23
  %25 = sdiv i64 %24, 10
  %26 = add i64 %25, 48
  %27 = trunc i64 %26 to i32
  %28 = call i32 @putchar(i32 %27)
  %29 = sext i32 %28 to i64
  %30 = getelementptr i64, i64* %12, i64 0
  %31 = load i64, i64* %30
  %32 = srem i64 %31, 10
  %33 = add i64 %32, 48
  %34 = trunc i64 %33 to i32
  %35 = call i32 @putchar(i32 %34)
  %36 = sext i32 %35 to i64
  %37 = trunc i64 32 to i32
  %38 = call i32 @putchar(i32 %37)
  %39 = sext i32 %38 to i64
  %40 = getelementptr i64, i64* %12, i64 1
  %41 = load i64, i64* %40
  %42 = sdiv i64 %41, 10
  %43 = add i64 %42, 48
  %44 = trunc i64 %43 to i32
  %45 = call i32 @putchar(i32 %44)
  %46 = sext i32 %45 to i64
  %47 = getelementptr i64, i64* %12, i64 1
  %48 = load i64, i64* %47
  %49 = srem i64 %48, 10
  %50 = add i64 %49, 48
  %51 = trunc i64 %50 to i32
  %52 = call i32 @putchar(i32 %51)
  %53 = sext i32 %52 to i64
  %54 = trunc i64 10 to i32
  %55 = call i32 @putchar(i32 %54)
  %56 = sext i32 %55 to i64
  %57 = add i64 3, 1
  %58 = sub i64 %57, 1
  %59 = mul i64 %58, 8
  %60 = call i8* @malloc(i64 %59)
  %61 = bitcast i8* %60 to i64*
  %62 = alloca i64
  store i64 0, i64* %62
  br label %slice_loop3
slice_loop3:
  %63 = load i64, i64* %62
  %64 = icmp slt i64 %63, %58
  br i1 %64, label %slice_body4, label %slice_end5
slice_body4:
  %65 = add i64 1, %63
  %66 = getelementptr i64, i64* %8, i64 %65
  %67 = load i64, i64* %66
  %68 = getelementptr i64, i64* %61, i64 %63
  store i64 %67, i64* %68
  %69 = add i64 %63, 1
  store i64 %69, i64* %62
  br label %slice_loop3
slice_end5:
  %70 = call i32 @puts(i8* getelementptr ([12 x i8], [12 x i8]* @.str.2, i64 0, i64 0))
  %71 = sext i32 %70 to i64
  %72 = getelementptr i64, i64* %61, i64 0
  %73 = load i64, i64* %72
  %74 = sdiv i64 %73, 10
  %75 = add i64 %74, 48
  %76 = trunc i64 %75 to i32
  %77 = call i32 @putchar(i32 %76)
  %78 = sext i32 %77 to i64
  %79 = getelementptr i64, i64* %61, i64 0
  %80 = load i64, i64* %79
  %81 = srem i64 %80, 10
  %82 = add i64 %81, 48
  %83 = trunc i64 %82 to i32
  %84 = call i32 @putchar(i32 %83)
  %85 = sext i32 %84 to i64
  %86 = trunc i64 32 to i32
  %87 = call i32 @putchar(i32 %86)
  %88 = sext i32 %87 to i64
  %89 = getelementptr i64, i64* %61, i64 1
  %90 = load i64, i64* %89
  %91 = sdiv i64 %90, 10
  %92 = add i64 %91, 48
  %93 = trunc i64 %92 to i32
  %94 = call i32 @putchar(i32 %93)
  %95 = sext i32 %94 to i64
  %96 = getelementptr i64, i64* %61, i64 1
  %97 = load i64, i64* %96
  %98 = srem i64 %97, 10
  %99 = add i64 %98, 48
  %100 = trunc i64 %99 to i32
  %101 = call i32 @putchar(i32 %100)
  %102 = sext i32 %101 to i64
  %103 = trunc i64 32 to i32
  %104 = call i32 @putchar(i32 %103)
  %105 = sext i32 %104 to i64
  %106 = getelementptr i64, i64* %61, i64 2
  %107 = load i64, i64* %106
  %108 = sdiv i64 %107, 10
  %109 = add i64 %108, 48
  %110 = trunc i64 %109 to i32
  %111 = call i32 @putchar(i32 %110)
  %112 = sext i32 %111 to i64
  %113 = getelementptr i64, i64* %61, i64 2
  %114 = load i64, i64* %113
  %115 = srem i64 %114, 10
  %116 = add i64 %115, 48
  %117 = trunc i64 %116 to i32
  %118 = call i32 @putchar(i32 %117)
  %119 = sext i32 %118 to i64
  %120 = trunc i64 10 to i32
  %121 = call i32 @putchar(i32 %120)
  %122 = sext i32 %121 to i64
  %123 = sub i64 2, 0
  %124 = mul i64 %123, 8
  %125 = call i8* @malloc(i64 %124)
  %126 = bitcast i8* %125 to i64*
  %127 = alloca i64
  store i64 0, i64* %127
  br label %slice_loop6
slice_loop6:
  %128 = load i64, i64* %127
  %129 = icmp slt i64 %128, %123
  br i1 %129, label %slice_body7, label %slice_end8
slice_body7:
  %130 = add i64 0, %128
  %131 = getelementptr i64, i64* %8, i64 %130
  %132 = load i64, i64* %131
  %133 = getelementptr i64, i64* %126, i64 %128
  store i64 %132, i64* %133
  %134 = add i64 %128, 1
  store i64 %134, i64* %127
  br label %slice_loop6
slice_end8:
  %135 = call i32 @puts(i8* getelementptr ([11 x i8], [11 x i8]* @.str.3, i64 0, i64 0))
  %136 = sext i32 %135 to i64
  %137 = getelementptr i64, i64* %126, i64 0
  %138 = load i64, i64* %137
  %139 = sdiv i64 %138, 10
  %140 = add i64 %139, 48
  %141 = trunc i64 %140 to i32
  %142 = call i32 @putchar(i32 %141)
  %143 = sext i32 %142 to i64
  %144 = getelementptr i64, i64* %126, i64 0
  %145 = load i64, i64* %144
  %146 = srem i64 %145, 10
  %147 = add i64 %146, 48
  %148 = trunc i64 %147 to i32
  %149 = call i32 @putchar(i32 %148)
  %150 = sext i32 %149 to i64
  %151 = trunc i64 32 to i32
  %152 = call i32 @putchar(i32 %151)
  %153 = sext i32 %152 to i64
  %154 = getelementptr i64, i64* %126, i64 1
  %155 = load i64, i64* %154
  %156 = sdiv i64 %155, 10
  %157 = add i64 %156, 48
  %158 = trunc i64 %157 to i32
  %159 = call i32 @putchar(i32 %158)
  %160 = sext i32 %159 to i64
  %161 = getelementptr i64, i64* %126, i64 1
  %162 = load i64, i64* %161
  %163 = srem i64 %162, 10
  %164 = add i64 %163, 48
  %165 = trunc i64 %164 to i32
  %166 = call i32 @putchar(i32 %165)
  %167 = sext i32 %166 to i64
  %168 = trunc i64 10 to i32
  %169 = call i32 @putchar(i32 %168)
  %170 = sext i32 %169 to i64
  ret i64 0
}


; C library function declarations
declare i64 @write(i32, i8*, i64)

; Global constants for runtime functions
@.panic_newline = private unnamed_addr constant [2 x i8] c"\0A\00"

; Runtime panic function (used by assert)
define i64 @__panic(i8* %msg) {
entry:
  ; Calculate message length
  %len = call i64 @strlen(i8* %msg)
  ; Write message to stderr (fd=2)
  %0 = call i64 @write(i32 2, i8* %msg, i64 %len)
  ; Write newline
  %1 = call i64 @write(i32 2, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @.panic_newline, i64 0, i64 0), i64 1)
  call void @exit(i32 1)
  unreachable
}

; Runtime contract failure function
define i64 @__contract_fail(i64 %kind, i8* %condition, i8* %file, i64 %line, i8* %func) {
entry:
  ; Calculate message length
  %len = call i64 @strlen(i8* %condition)
  ; Write contract failure message to stderr (fd=2)
  %0 = call i64 @write(i32 2, i8* %condition, i64 %len)
  ; Write newline
  %1 = call i64 @write(i32 2, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @.panic_newline, i64 0, i64 0), i64 1)
  call void @exit(i32 1)
  unreachable
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

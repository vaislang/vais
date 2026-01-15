; ModuleID = 'slice_test'
source_filename = "<vais>"

declare i64 @memcpy(i64, i64, i64)
declare i32 @fclose(i64)
declare i32 @puts(i8*)
declare i64 @fseek(i64, i64, i64)
declare i64 @fputc(i64, i64)
declare i64 @feof(i64)
declare i64 @ftell(i64)
declare i64 @fopen(i8*, i8*)
declare i64 @strlen(i64)
declare i64 @fread(i64, i64, i64, i64)
declare i32 @putchar(i32)
declare void @free(i64)
declare i64 @fflush(i64)
declare i64 @malloc(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @fgetc(i64)
declare i64 @fgets(i64, i64, i64)
declare i64 @fputs(i8*, i64)
declare void @exit(i32)
declare i32 @printf(i8*)
@.str.0 = private unnamed_addr constant [23 x i8] c"Testing array slicing:\00"
@.str.1 = private unnamed_addr constant [11 x i8] c"arr[1..3]:\00"
@.str.2 = private unnamed_addr constant [12 x i8] c"arr[1..=3]:\00"
@.str.3 = private unnamed_addr constant [11 x i8] c"arr[0..2]:\00"

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([23 x i8], [23 x i8]* @.str.0, i64 0, i64 0))
  %1 = alloca [5  x i64]
  %2 = getelementptr [5  x i64], [5  x i64]* %1, i64 0, i64 0
  store i64 10, i64* %2
  %3 = getelementptr [5  x i64], [5  x i64]* %1, i64 0, i64 1
  store i64 20, i64* %3
  %4 = getelementptr [5  x i64], [5  x i64]* %1, i64 0, i64 2
  store i64 30, i64* %4
  %5 = getelementptr [5  x i64], [5  x i64]* %1, i64 0, i64 3
  store i64 40, i64* %5
  %6 = getelementptr [5  x i64], [5  x i64]* %1, i64 0, i64 4
  store i64 50, i64* %6
  %7 = getelementptr [5  x i64], [5  x i64]* %1, i64 0, i64 0
  %arr.8 = alloca i64*
  store i64* %7, i64** %arr.8
  %9 = load i64*, i64** %arr.8
  %10 = sub i64 3, 1
  %11 = mul i64 %10, 8
  %12 = call i8* @malloc(i64 %11)
  %13 = bitcast i8* %12 to i64*
  %14 = alloca i64
  store i64 0, i64* %14
  br label %slice_loop0
slice_loop0:
  %15 = load i64, i64* %14
  %16 = icmp slt i64 %15, %10
  br i1 %16, label %slice_body1, label %slice_end2
slice_body1:
  %17 = add i64 1, %15
  %18 = getelementptr i64, i64* %9, i64 %17
  %19 = load i64, i64* %18
  %20 = getelementptr i64, i64* %13, i64 %15
  store i64 %19, i64* %20
  %21 = add i64 %15, 1
  store i64 %21, i64* %14
  br label %slice_loop0
slice_end2:
  %slice1.22 = alloca i64*
  store i64* %13, i64** %slice1.22
  %23 = call i32 @puts(i8* getelementptr ([11 x i8], [11 x i8]* @.str.1, i64 0, i64 0))
  %24 = load i64*, i64** %slice1.22
  %25 = getelementptr i64, i64* %24, i64 0
  %26 = load i64, i64* %25
  %27 = sdiv i64 %26, 10
  %28 = add i64 %27, 48
  %29 = trunc i64 %28 to i32
  %30 = call i32 @putchar(i32 %29)
  %31 = load i64*, i64** %slice1.22
  %32 = getelementptr i64, i64* %31, i64 0
  %33 = load i64, i64* %32
  %34 = srem i64 %33, 10
  %35 = add i64 %34, 48
  %36 = trunc i64 %35 to i32
  %37 = call i32 @putchar(i32 %36)
  %38 = trunc i64 32 to i32
  %39 = call i32 @putchar(i32 %38)
  %40 = load i64*, i64** %slice1.22
  %41 = getelementptr i64, i64* %40, i64 1
  %42 = load i64, i64* %41
  %43 = sdiv i64 %42, 10
  %44 = add i64 %43, 48
  %45 = trunc i64 %44 to i32
  %46 = call i32 @putchar(i32 %45)
  %47 = load i64*, i64** %slice1.22
  %48 = getelementptr i64, i64* %47, i64 1
  %49 = load i64, i64* %48
  %50 = srem i64 %49, 10
  %51 = add i64 %50, 48
  %52 = trunc i64 %51 to i32
  %53 = call i32 @putchar(i32 %52)
  %54 = trunc i64 10 to i32
  %55 = call i32 @putchar(i32 %54)
  %56 = load i64*, i64** %arr.8
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
  %66 = getelementptr i64, i64* %56, i64 %65
  %67 = load i64, i64* %66
  %68 = getelementptr i64, i64* %61, i64 %63
  store i64 %67, i64* %68
  %69 = add i64 %63, 1
  store i64 %69, i64* %62
  br label %slice_loop3
slice_end5:
  %slice2.70 = alloca i64*
  store i64* %61, i64** %slice2.70
  %71 = call i32 @puts(i8* getelementptr ([12 x i8], [12 x i8]* @.str.2, i64 0, i64 0))
  %72 = load i64*, i64** %slice2.70
  %73 = getelementptr i64, i64* %72, i64 0
  %74 = load i64, i64* %73
  %75 = sdiv i64 %74, 10
  %76 = add i64 %75, 48
  %77 = trunc i64 %76 to i32
  %78 = call i32 @putchar(i32 %77)
  %79 = load i64*, i64** %slice2.70
  %80 = getelementptr i64, i64* %79, i64 0
  %81 = load i64, i64* %80
  %82 = srem i64 %81, 10
  %83 = add i64 %82, 48
  %84 = trunc i64 %83 to i32
  %85 = call i32 @putchar(i32 %84)
  %86 = trunc i64 32 to i32
  %87 = call i32 @putchar(i32 %86)
  %88 = load i64*, i64** %slice2.70
  %89 = getelementptr i64, i64* %88, i64 1
  %90 = load i64, i64* %89
  %91 = sdiv i64 %90, 10
  %92 = add i64 %91, 48
  %93 = trunc i64 %92 to i32
  %94 = call i32 @putchar(i32 %93)
  %95 = load i64*, i64** %slice2.70
  %96 = getelementptr i64, i64* %95, i64 1
  %97 = load i64, i64* %96
  %98 = srem i64 %97, 10
  %99 = add i64 %98, 48
  %100 = trunc i64 %99 to i32
  %101 = call i32 @putchar(i32 %100)
  %102 = trunc i64 32 to i32
  %103 = call i32 @putchar(i32 %102)
  %104 = load i64*, i64** %slice2.70
  %105 = getelementptr i64, i64* %104, i64 2
  %106 = load i64, i64* %105
  %107 = sdiv i64 %106, 10
  %108 = add i64 %107, 48
  %109 = trunc i64 %108 to i32
  %110 = call i32 @putchar(i32 %109)
  %111 = load i64*, i64** %slice2.70
  %112 = getelementptr i64, i64* %111, i64 2
  %113 = load i64, i64* %112
  %114 = srem i64 %113, 10
  %115 = add i64 %114, 48
  %116 = trunc i64 %115 to i32
  %117 = call i32 @putchar(i32 %116)
  %118 = trunc i64 10 to i32
  %119 = call i32 @putchar(i32 %118)
  %120 = load i64*, i64** %arr.8
  %121 = sub i64 2, 0
  %122 = mul i64 %121, 8
  %123 = call i8* @malloc(i64 %122)
  %124 = bitcast i8* %123 to i64*
  %125 = alloca i64
  store i64 0, i64* %125
  br label %slice_loop6
slice_loop6:
  %126 = load i64, i64* %125
  %127 = icmp slt i64 %126, %121
  br i1 %127, label %slice_body7, label %slice_end8
slice_body7:
  %128 = add i64 0, %126
  %129 = getelementptr i64, i64* %120, i64 %128
  %130 = load i64, i64* %129
  %131 = getelementptr i64, i64* %124, i64 %126
  store i64 %130, i64* %131
  %132 = add i64 %126, 1
  store i64 %132, i64* %125
  br label %slice_loop6
slice_end8:
  %slice3.133 = alloca i64*
  store i64* %124, i64** %slice3.133
  %134 = call i32 @puts(i8* getelementptr ([11 x i8], [11 x i8]* @.str.3, i64 0, i64 0))
  %135 = load i64*, i64** %slice3.133
  %136 = getelementptr i64, i64* %135, i64 0
  %137 = load i64, i64* %136
  %138 = sdiv i64 %137, 10
  %139 = add i64 %138, 48
  %140 = trunc i64 %139 to i32
  %141 = call i32 @putchar(i32 %140)
  %142 = load i64*, i64** %slice3.133
  %143 = getelementptr i64, i64* %142, i64 0
  %144 = load i64, i64* %143
  %145 = srem i64 %144, 10
  %146 = add i64 %145, 48
  %147 = trunc i64 %146 to i32
  %148 = call i32 @putchar(i32 %147)
  %149 = trunc i64 32 to i32
  %150 = call i32 @putchar(i32 %149)
  %151 = load i64*, i64** %slice3.133
  %152 = getelementptr i64, i64* %151, i64 1
  %153 = load i64, i64* %152
  %154 = sdiv i64 %153, 10
  %155 = add i64 %154, 48
  %156 = trunc i64 %155 to i32
  %157 = call i32 @putchar(i32 %156)
  %158 = load i64*, i64** %slice3.133
  %159 = getelementptr i64, i64* %158, i64 1
  %160 = load i64, i64* %159
  %161 = srem i64 %160, 10
  %162 = add i64 %161, 48
  %163 = trunc i64 %162 to i32
  %164 = call i32 @putchar(i32 %163)
  %165 = trunc i64 10 to i32
  %166 = call i32 @putchar(i32 %165)
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

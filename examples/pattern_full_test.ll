; ModuleID = 'pattern_full_test'
source_filename = "<vais>"

declare i64 @fgets(i64, i64, i64)
declare i32 @printf(i8*)
declare i64 @fseek(i64, i64, i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @strlen(i64)
declare i32 @strcmp(i8*, i8*)
declare i64 @fputc(i64, i64)
declare i32 @fclose(i64)
declare i64 @ftell(i64)
declare i64 @malloc(i64)
declare i64 @feof(i64)
declare i64 @fflush(i64)
declare void @exit(i32)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @memcpy(i64, i64, i64)
declare i32 @puts(i64)
declare i64 @fopen(i8*, i8*)
declare i64 @fgetc(i64)
declare i32 @putchar(i32)
declare void @free(i64)
declare i64 @fputs(i8*, i64)
declare i64 @fread(i64, i64, i64, i64)
@.str.0 = private unnamed_addr constant [31 x i8] c"=== Pattern Matching Tests ===\00"
@.str.1 = private unnamed_addr constant [18 x i8] c"Integer patterns:\00"
@.str.2 = private unnamed_addr constant [15 x i8] c"match_int(0) =\00"
@.str.3 = private unnamed_addr constant [15 x i8] c"match_int(2) =\00"
@.str.4 = private unnamed_addr constant [16 x i8] c"match_int(99) =\00"
@.str.5 = private unnamed_addr constant [18 x i8] c"Boolean patterns:\00"
@.str.6 = private unnamed_addr constant [22 x i8] c"match_bool(0/false) =\00"
@.str.7 = private unnamed_addr constant [21 x i8] c"match_bool(1/true) =\00"
@.str.8 = private unnamed_addr constant [16 x i8] c"Range patterns:\00"
@.str.9 = private unnamed_addr constant [24 x i8] c"match_range(3) [0..5] =\00"
@.str.10 = private unnamed_addr constant [25 x i8] c"match_range(7) [5..10] =\00"
@.str.11 = private unnamed_addr constant [28 x i8] c"match_range(15) [10..=20] =\00"
@.str.12 = private unnamed_addr constant [28 x i8] c"match_range(50) [default] =\00"
@.str.13 = private unnamed_addr constant [13 x i8] c"Or patterns:\00"
@.str.14 = private unnamed_addr constant [22 x i8] c"match_or(2) [1|2|3] =\00"
@.str.15 = private unnamed_addr constant [20 x i8] c"match_or(5) [4|5] =\00"
@.str.16 = private unnamed_addr constant [16 x i8] c"Guard patterns:\00"
@.str.17 = private unnamed_addr constant [24 x i8] c"match_guard(60) [>50] =\00"
@.str.18 = private unnamed_addr constant [24 x i8] c"match_guard(30) [>20] =\00"
@.str.19 = private unnamed_addr constant [25 x i8] c"match_guard(10) [else] =\00"
@.str.20 = private unnamed_addr constant [18 x i8] c"Variable binding:\00"
@.str.21 = private unnamed_addr constant [16 x i8] c"match_bind(0) =\00"
@.str.22 = private unnamed_addr constant [18 x i8] c"match_bind(5)*2 =\00"
@.str.23 = private unnamed_addr constant [26 x i8] c"=== All tests passed! ===\00"

define i64 @match_int(i64 %n) {
entry:
  switch i64 %n, label %match.default1 [
    i64 0, label %match.arm2
    i64 1, label %match.arm3
    i64 2, label %match.arm4
    i64 3, label %match.arm5
  ]
match.arm2:
  br label %match.merge0
match.arm3:
  br label %match.merge0
match.arm4:
  br label %match.merge0
match.arm5:
  br label %match.merge0
match.default1:
  br label %match.merge0
match.merge0:
  %0 = phi i64 [ 0, %match.arm2 ], [ 10, %match.arm3 ], [ 20, %match.arm4 ], [ 30, %match.arm5 ], [ 99, %match.default1 ]
  ret i64 %0
}

define i64 @match_bool(i64 %b) {
entry:
  switch i64 %b, label %match.default1 [
    i64 0, label %match.arm2
    i64 1, label %match.arm3
  ]
match.arm2:
  br label %match.merge0
match.arm3:
  br label %match.merge0
match.default1:
  br label %match.merge0
match.merge0:
  %0 = phi i64 [ 100, %match.arm2 ], [ 200, %match.arm3 ], [ 300, %match.default1 ]
  ret i64 %0
}

define i64 @match_range(i64 %n) {
entry:
  br label %match.check1
match.check1:
  %0 = icmp sge i64 %n, 0
  %1 = icmp slt i64 %n, 5
  %2 = and i1 %0, %1
  br i1 %2, label %match.arm3, label %match.check2
match.arm3:
  br label %match.merge0
match.check2:
  %3 = icmp sge i64 %n, 5
  %4 = icmp slt i64 %n, 10
  %5 = and i1 %3, %4
  br i1 %5, label %match.arm5, label %match.check4
match.arm5:
  br label %match.merge0
match.check4:
  %6 = icmp sge i64 %n, 10
  %7 = icmp sle i64 %n, 20
  %8 = and i1 %6, %7
  br i1 %8, label %match.arm7, label %match.check6
match.arm7:
  br label %match.merge0
match.check6:
  br i1 1, label %match.arm8, label %match.merge0
match.arm8:
  br label %match.merge0
match.merge0:
  %9 = phi i64 [ 1, %match.arm3 ], [ 2, %match.arm5 ], [ 3, %match.arm7 ], [ 0, %match.arm8 ]
  ret i64 %9
}

define i64 @match_or(i64 %n) {
entry:
  br label %match.check1
match.check1:
  %0 = icmp eq i64 %n, 1
  %1 = icmp eq i64 %n, 2
  %2 = icmp eq i64 %n, 3
  %3 = or i1 %0, %1
  %4 = or i1 %3, %2
  br i1 %4, label %match.arm3, label %match.check2
match.arm3:
  br label %match.merge0
match.check2:
  %5 = icmp eq i64 %n, 4
  %6 = icmp eq i64 %n, 5
  %7 = or i1 %5, %6
  br i1 %7, label %match.arm5, label %match.check4
match.arm5:
  br label %match.merge0
match.check4:
  br i1 1, label %match.arm6, label %match.merge0
match.arm6:
  br label %match.merge0
match.merge0:
  %8 = phi i64 [ 10, %match.arm3 ], [ 20, %match.arm5 ], [ 0, %match.arm6 ]
  ret i64 %8
}

define i64 @match_guard(i64 %n) {
entry:
  br label %match.check1
match.check1:
  br i1 1, label %match.guard.bind4, label %match.check2
match.guard.bind4:
  %x.0 = alloca i64
  store i64 %n, i64* %x.0
  br label %match.guard.check5
match.guard.check5:
  %1 = load i64, i64* %x.0
  %2 = icmp sgt i64 %1, 50
  %3 = zext i1 %2 to i64
  %4 = icmp ne i64 %3, 0
  br i1 %4, label %match.arm3, label %match.check2
match.arm3:
  br label %match.merge0
match.check2:
  br i1 1, label %match.guard.bind8, label %match.check6
match.guard.bind8:
  %x.5 = alloca i64
  store i64 %n, i64* %x.5
  br label %match.guard.check9
match.guard.check9:
  %6 = load i64, i64* %x.5
  %7 = icmp sgt i64 %6, 20
  %8 = zext i1 %7 to i64
  %9 = icmp ne i64 %8, 0
  br i1 %9, label %match.arm7, label %match.check6
match.arm7:
  br label %match.merge0
match.check6:
  br i1 1, label %match.arm10, label %match.merge0
match.arm10:
  %x.10 = alloca i64
  store i64 %n, i64* %x.10
  %11 = load i64, i64* %x.10
  br label %match.merge0
match.merge0:
  %12 = phi i64 [ 100, %match.arm3 ], [ 50, %match.arm7 ], [ %11, %match.arm10 ]
  ret i64 %12
}

define i64 @match_bind(i64 %n) {
entry:
  br label %match.check1
match.check1:
  %0 = icmp eq i64 %n, 0
  br i1 %0, label %match.arm3, label %match.check2
match.arm3:
  br label %match.merge0
match.check2:
  br i1 1, label %match.arm4, label %match.merge0
match.arm4:
  %x.1 = alloca i64
  store i64 %n, i64* %x.1
  %2 = load i64, i64* %x.1
  %3 = mul i64 %2, 2
  br label %match.merge0
match.merge0:
  %4 = phi i64 [ 0, %match.arm3 ], [ %3, %match.arm4 ]
  ret i64 %4
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([31 x i8], [31 x i8]* @.str.0, i64 0, i64 0))
  %1 = trunc i64 10 to i32
  %2 = call i32 @putchar(i32 %1)
  %3 = call i32 @puts(i8* getelementptr ([18 x i8], [18 x i8]* @.str.1, i64 0, i64 0))
  %4 = call i32 @puts(i8* getelementptr ([15 x i8], [15 x i8]* @.str.2, i64 0, i64 0))
  %5 = call i64 @match_int(i64 0)
  %6 = add i64 %5, 48
  %7 = trunc i64 %6 to i32
  %8 = call i32 @putchar(i32 %7)
  %9 = trunc i64 10 to i32
  %10 = call i32 @putchar(i32 %9)
  %11 = call i32 @puts(i8* getelementptr ([15 x i8], [15 x i8]* @.str.3, i64 0, i64 0))
  %12 = call i64 @match_int(i64 2)
  %13 = sdiv i64 %12, 10
  %14 = add i64 %13, 48
  %15 = trunc i64 %14 to i32
  %16 = call i32 @putchar(i32 %15)
  %17 = call i64 @match_int(i64 2)
  %18 = srem i64 %17, 10
  %19 = add i64 %18, 48
  %20 = trunc i64 %19 to i32
  %21 = call i32 @putchar(i32 %20)
  %22 = trunc i64 10 to i32
  %23 = call i32 @putchar(i32 %22)
  %24 = call i32 @puts(i8* getelementptr ([16 x i8], [16 x i8]* @.str.4, i64 0, i64 0))
  %25 = call i64 @match_int(i64 99)
  %26 = sdiv i64 %25, 10
  %27 = add i64 %26, 48
  %28 = trunc i64 %27 to i32
  %29 = call i32 @putchar(i32 %28)
  %30 = call i64 @match_int(i64 99)
  %31 = srem i64 %30, 10
  %32 = add i64 %31, 48
  %33 = trunc i64 %32 to i32
  %34 = call i32 @putchar(i32 %33)
  %35 = trunc i64 10 to i32
  %36 = call i32 @putchar(i32 %35)
  %37 = trunc i64 10 to i32
  %38 = call i32 @putchar(i32 %37)
  %39 = call i32 @puts(i8* getelementptr ([18 x i8], [18 x i8]* @.str.5, i64 0, i64 0))
  %40 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.6, i64 0, i64 0))
  %41 = call i64 @match_bool(i64 0)
  %r.42 = alloca i64
  store i64 %41, i64* %r.42
  %43 = load i64, i64* %r.42
  %44 = sdiv i64 %43, 100
  %45 = add i64 %44, 48
  %46 = trunc i64 %45 to i32
  %47 = call i32 @putchar(i32 %46)
  %48 = load i64, i64* %r.42
  %49 = sdiv i64 %48, 10
  %50 = srem i64 %49, 10
  %51 = add i64 %50, 48
  %52 = trunc i64 %51 to i32
  %53 = call i32 @putchar(i32 %52)
  %54 = load i64, i64* %r.42
  %55 = srem i64 %54, 10
  %56 = add i64 %55, 48
  %57 = trunc i64 %56 to i32
  %58 = call i32 @putchar(i32 %57)
  %59 = trunc i64 10 to i32
  %60 = call i32 @putchar(i32 %59)
  %61 = call i32 @puts(i8* getelementptr ([21 x i8], [21 x i8]* @.str.7, i64 0, i64 0))
  %62 = call i64 @match_bool(i64 1)
  %r2.63 = alloca i64
  store i64 %62, i64* %r2.63
  %64 = load i64, i64* %r2.63
  %65 = sdiv i64 %64, 100
  %66 = add i64 %65, 48
  %67 = trunc i64 %66 to i32
  %68 = call i32 @putchar(i32 %67)
  %69 = load i64, i64* %r2.63
  %70 = sdiv i64 %69, 10
  %71 = srem i64 %70, 10
  %72 = add i64 %71, 48
  %73 = trunc i64 %72 to i32
  %74 = call i32 @putchar(i32 %73)
  %75 = load i64, i64* %r2.63
  %76 = srem i64 %75, 10
  %77 = add i64 %76, 48
  %78 = trunc i64 %77 to i32
  %79 = call i32 @putchar(i32 %78)
  %80 = trunc i64 10 to i32
  %81 = call i32 @putchar(i32 %80)
  %82 = trunc i64 10 to i32
  %83 = call i32 @putchar(i32 %82)
  %84 = call i32 @puts(i8* getelementptr ([16 x i8], [16 x i8]* @.str.8, i64 0, i64 0))
  %85 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.9, i64 0, i64 0))
  %86 = call i64 @match_range(i64 3)
  %87 = add i64 %86, 48
  %88 = trunc i64 %87 to i32
  %89 = call i32 @putchar(i32 %88)
  %90 = trunc i64 10 to i32
  %91 = call i32 @putchar(i32 %90)
  %92 = call i32 @puts(i8* getelementptr ([25 x i8], [25 x i8]* @.str.10, i64 0, i64 0))
  %93 = call i64 @match_range(i64 7)
  %94 = add i64 %93, 48
  %95 = trunc i64 %94 to i32
  %96 = call i32 @putchar(i32 %95)
  %97 = trunc i64 10 to i32
  %98 = call i32 @putchar(i32 %97)
  %99 = call i32 @puts(i8* getelementptr ([28 x i8], [28 x i8]* @.str.11, i64 0, i64 0))
  %100 = call i64 @match_range(i64 15)
  %101 = add i64 %100, 48
  %102 = trunc i64 %101 to i32
  %103 = call i32 @putchar(i32 %102)
  %104 = trunc i64 10 to i32
  %105 = call i32 @putchar(i32 %104)
  %106 = call i32 @puts(i8* getelementptr ([28 x i8], [28 x i8]* @.str.12, i64 0, i64 0))
  %107 = call i64 @match_range(i64 50)
  %108 = add i64 %107, 48
  %109 = trunc i64 %108 to i32
  %110 = call i32 @putchar(i32 %109)
  %111 = trunc i64 10 to i32
  %112 = call i32 @putchar(i32 %111)
  %113 = trunc i64 10 to i32
  %114 = call i32 @putchar(i32 %113)
  %115 = call i32 @puts(i8* getelementptr ([13 x i8], [13 x i8]* @.str.13, i64 0, i64 0))
  %116 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.14, i64 0, i64 0))
  %117 = call i64 @match_or(i64 2)
  %118 = sdiv i64 %117, 10
  %119 = add i64 %118, 48
  %120 = trunc i64 %119 to i32
  %121 = call i32 @putchar(i32 %120)
  %122 = call i64 @match_or(i64 2)
  %123 = srem i64 %122, 10
  %124 = add i64 %123, 48
  %125 = trunc i64 %124 to i32
  %126 = call i32 @putchar(i32 %125)
  %127 = trunc i64 10 to i32
  %128 = call i32 @putchar(i32 %127)
  %129 = call i32 @puts(i8* getelementptr ([20 x i8], [20 x i8]* @.str.15, i64 0, i64 0))
  %130 = call i64 @match_or(i64 5)
  %131 = sdiv i64 %130, 10
  %132 = add i64 %131, 48
  %133 = trunc i64 %132 to i32
  %134 = call i32 @putchar(i32 %133)
  %135 = call i64 @match_or(i64 5)
  %136 = srem i64 %135, 10
  %137 = add i64 %136, 48
  %138 = trunc i64 %137 to i32
  %139 = call i32 @putchar(i32 %138)
  %140 = trunc i64 10 to i32
  %141 = call i32 @putchar(i32 %140)
  %142 = trunc i64 10 to i32
  %143 = call i32 @putchar(i32 %142)
  %144 = call i32 @puts(i8* getelementptr ([16 x i8], [16 x i8]* @.str.16, i64 0, i64 0))
  %145 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.17, i64 0, i64 0))
  %146 = call i64 @match_guard(i64 60)
  %r3.147 = alloca i64
  store i64 %146, i64* %r3.147
  %148 = load i64, i64* %r3.147
  %149 = sdiv i64 %148, 100
  %150 = add i64 %149, 48
  %151 = trunc i64 %150 to i32
  %152 = call i32 @putchar(i32 %151)
  %153 = load i64, i64* %r3.147
  %154 = sdiv i64 %153, 10
  %155 = srem i64 %154, 10
  %156 = add i64 %155, 48
  %157 = trunc i64 %156 to i32
  %158 = call i32 @putchar(i32 %157)
  %159 = load i64, i64* %r3.147
  %160 = srem i64 %159, 10
  %161 = add i64 %160, 48
  %162 = trunc i64 %161 to i32
  %163 = call i32 @putchar(i32 %162)
  %164 = trunc i64 10 to i32
  %165 = call i32 @putchar(i32 %164)
  %166 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.18, i64 0, i64 0))
  %167 = call i64 @match_guard(i64 30)
  %r4.168 = alloca i64
  store i64 %167, i64* %r4.168
  %169 = load i64, i64* %r4.168
  %170 = sdiv i64 %169, 10
  %171 = add i64 %170, 48
  %172 = trunc i64 %171 to i32
  %173 = call i32 @putchar(i32 %172)
  %174 = load i64, i64* %r4.168
  %175 = srem i64 %174, 10
  %176 = add i64 %175, 48
  %177 = trunc i64 %176 to i32
  %178 = call i32 @putchar(i32 %177)
  %179 = trunc i64 10 to i32
  %180 = call i32 @putchar(i32 %179)
  %181 = call i32 @puts(i8* getelementptr ([25 x i8], [25 x i8]* @.str.19, i64 0, i64 0))
  %182 = call i64 @match_guard(i64 10)
  %183 = sdiv i64 %182, 10
  %184 = add i64 %183, 48
  %185 = trunc i64 %184 to i32
  %186 = call i32 @putchar(i32 %185)
  %187 = call i64 @match_guard(i64 10)
  %188 = srem i64 %187, 10
  %189 = add i64 %188, 48
  %190 = trunc i64 %189 to i32
  %191 = call i32 @putchar(i32 %190)
  %192 = trunc i64 10 to i32
  %193 = call i32 @putchar(i32 %192)
  %194 = trunc i64 10 to i32
  %195 = call i32 @putchar(i32 %194)
  %196 = call i32 @puts(i8* getelementptr ([18 x i8], [18 x i8]* @.str.20, i64 0, i64 0))
  %197 = call i32 @puts(i8* getelementptr ([16 x i8], [16 x i8]* @.str.21, i64 0, i64 0))
  %198 = call i64 @match_bind(i64 0)
  %199 = add i64 %198, 48
  %200 = trunc i64 %199 to i32
  %201 = call i32 @putchar(i32 %200)
  %202 = trunc i64 10 to i32
  %203 = call i32 @putchar(i32 %202)
  %204 = call i32 @puts(i8* getelementptr ([18 x i8], [18 x i8]* @.str.22, i64 0, i64 0))
  %205 = call i64 @match_bind(i64 5)
  %206 = sdiv i64 %205, 10
  %207 = add i64 %206, 48
  %208 = trunc i64 %207 to i32
  %209 = call i32 @putchar(i32 %208)
  %210 = call i64 @match_bind(i64 5)
  %211 = srem i64 %210, 10
  %212 = add i64 %211, 48
  %213 = trunc i64 %212 to i32
  %214 = call i32 @putchar(i32 %213)
  %215 = trunc i64 10 to i32
  %216 = call i32 @putchar(i32 %215)
  %217 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.23, i64 0, i64 0))
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

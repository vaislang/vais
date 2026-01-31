; ModuleID = 'pattern_full_test'
source_filename = "<vais>"

declare i64 @fputs(i8*, i64)
declare double @atof(i8*)
declare i32 @atoi(i8*)
declare i64 @vais_gc_alloc(i64, i32)
declare i64 @atol(i64)
declare i64 @vais_gc_remove_root(i64)
declare i64 @vais_gc_print_stats()
declare i64 @vais_gc_collections()
declare i32 @puts(i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @vais_gc_add_root(i64)
declare i64 @fopen(i8*, i8*)
declare i64 @fgets(i64, i64, i64)
declare i64 @vais_gc_objects_count()
declare void @exit(i32)
declare i32 @tolower(i32)
declare i64 @strcpy(i64, i8*)
declare i64 @fseek(i64, i64, i64)
declare i32 @strcmp(i8*, i8*)
declare i64 @fflush(i64)
declare i32 @isdigit(i32)
declare i32 @sched_yield()
declare void @free(i64)
declare double @sqrt(double)
declare i64 @vais_gc_collect()
declare i32 @fclose(i64)
declare i32 @toupper(i32)
declare i32 @usleep(i64)
declare double @fabs(double)
declare i32 @putchar(i32)
declare i64 @memcpy(i64, i64, i64)
declare i64 @fgetc(i64)
declare i64 @vais_gc_bytes_allocated()
declare i64 @labs(i64)
declare i32 @printf(i8*, ...)
declare i32 @rand()
declare i64 @fputc(i64, i64)
declare i64 @malloc(i64)
declare void @srand(i32)
declare i64 @strlen(i8*)
declare i32 @isalpha(i32)
declare i64 @fwrite(i64, i64, i64, i64)
define i64 @fopen_ptr(i64 %path, i8* %mode) {
entry:
  %0 = call i64 @fopen(i64 %path, i8* %mode)
  ret i64 %0
}
declare i64 @ftell(i64)
declare i64 @memcpy_str(i64, i8*, i64)
declare i64 @strcat(i64, i8*)
declare i64 @vais_gc_set_threshold(i64)
declare i64 @feof(i64)
declare i64 @vais_gc_init()
declare i64 @fread(i64, i64, i64, i64)
@__vais_abi_version = constant [6 x i8] c"1.0.0\00"

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
  br label %match.guard.check5
match.guard.check5:
  %1 = icmp sgt i64 %n, 50
  %2 = zext i1 %1 to i64
  %3 = icmp ne i64 %2, 0
  br i1 %3, label %match.arm3, label %match.check2
match.arm3:
  br label %match.merge0
match.check2:
  br i1 1, label %match.guard.bind8, label %match.check6
match.guard.bind8:
  br label %match.guard.check9
match.guard.check9:
  %5 = icmp sgt i64 %n, 20
  %6 = zext i1 %5 to i64
  %7 = icmp ne i64 %6, 0
  br i1 %7, label %match.arm7, label %match.check6
match.arm7:
  br label %match.merge0
match.check6:
  br i1 1, label %match.arm10, label %match.merge0
match.arm10:
  br label %match.merge0
match.merge0:
  %9 = phi i64 [ 100, %match.arm3 ], [ 50, %match.arm7 ], [ %n, %match.arm10 ]
  ret i64 %9
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
  %2 = mul i64 %n, 2
  br label %match.merge0
match.merge0:
  %3 = phi i64 [ 0, %match.arm3 ], [ %2, %match.arm4 ]
  ret i64 %3
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([31 x i8], [31 x i8]* @.str.0, i64 0, i64 0))
  %1 = sext i32 %0 to i64
  %2 = trunc i64 10 to i32
  %3 = call i32 @putchar(i32 %2)
  %4 = sext i32 %3 to i64
  %5 = call i32 @puts(i8* getelementptr ([18 x i8], [18 x i8]* @.str.1, i64 0, i64 0))
  %6 = sext i32 %5 to i64
  %7 = call i32 @puts(i8* getelementptr ([15 x i8], [15 x i8]* @.str.2, i64 0, i64 0))
  %8 = sext i32 %7 to i64
  %9 = call i64 @match_int(i64 0)
  %10 = add i64 %9, 48
  %11 = trunc i64 %10 to i32
  %12 = call i32 @putchar(i32 %11)
  %13 = sext i32 %12 to i64
  %14 = trunc i64 10 to i32
  %15 = call i32 @putchar(i32 %14)
  %16 = sext i32 %15 to i64
  %17 = call i32 @puts(i8* getelementptr ([15 x i8], [15 x i8]* @.str.3, i64 0, i64 0))
  %18 = sext i32 %17 to i64
  %19 = call i64 @match_int(i64 2)
  %20 = sdiv i64 %19, 10
  %21 = add i64 %20, 48
  %22 = trunc i64 %21 to i32
  %23 = call i32 @putchar(i32 %22)
  %24 = sext i32 %23 to i64
  %25 = call i64 @match_int(i64 2)
  %26 = srem i64 %25, 10
  %27 = add i64 %26, 48
  %28 = trunc i64 %27 to i32
  %29 = call i32 @putchar(i32 %28)
  %30 = sext i32 %29 to i64
  %31 = trunc i64 10 to i32
  %32 = call i32 @putchar(i32 %31)
  %33 = sext i32 %32 to i64
  %34 = call i32 @puts(i8* getelementptr ([16 x i8], [16 x i8]* @.str.4, i64 0, i64 0))
  %35 = sext i32 %34 to i64
  %36 = call i64 @match_int(i64 99)
  %37 = sdiv i64 %36, 10
  %38 = add i64 %37, 48
  %39 = trunc i64 %38 to i32
  %40 = call i32 @putchar(i32 %39)
  %41 = sext i32 %40 to i64
  %42 = call i64 @match_int(i64 99)
  %43 = srem i64 %42, 10
  %44 = add i64 %43, 48
  %45 = trunc i64 %44 to i32
  %46 = call i32 @putchar(i32 %45)
  %47 = sext i32 %46 to i64
  %48 = trunc i64 10 to i32
  %49 = call i32 @putchar(i32 %48)
  %50 = sext i32 %49 to i64
  %51 = trunc i64 10 to i32
  %52 = call i32 @putchar(i32 %51)
  %53 = sext i32 %52 to i64
  %54 = call i32 @puts(i8* getelementptr ([18 x i8], [18 x i8]* @.str.5, i64 0, i64 0))
  %55 = sext i32 %54 to i64
  %56 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.6, i64 0, i64 0))
  %57 = sext i32 %56 to i64
  %58 = call i64 @match_bool(i64 0)
  %59 = sdiv i64 %58, 100
  %60 = add i64 %59, 48
  %61 = trunc i64 %60 to i32
  %62 = call i32 @putchar(i32 %61)
  %63 = sext i32 %62 to i64
  %64 = sdiv i64 %58, 10
  %65 = srem i64 %64, 10
  %66 = add i64 %65, 48
  %67 = trunc i64 %66 to i32
  %68 = call i32 @putchar(i32 %67)
  %69 = sext i32 %68 to i64
  %70 = srem i64 %58, 10
  %71 = add i64 %70, 48
  %72 = trunc i64 %71 to i32
  %73 = call i32 @putchar(i32 %72)
  %74 = sext i32 %73 to i64
  %75 = trunc i64 10 to i32
  %76 = call i32 @putchar(i32 %75)
  %77 = sext i32 %76 to i64
  %78 = call i32 @puts(i8* getelementptr ([21 x i8], [21 x i8]* @.str.7, i64 0, i64 0))
  %79 = sext i32 %78 to i64
  %80 = call i64 @match_bool(i64 1)
  %81 = sdiv i64 %80, 100
  %82 = add i64 %81, 48
  %83 = trunc i64 %82 to i32
  %84 = call i32 @putchar(i32 %83)
  %85 = sext i32 %84 to i64
  %86 = sdiv i64 %80, 10
  %87 = srem i64 %86, 10
  %88 = add i64 %87, 48
  %89 = trunc i64 %88 to i32
  %90 = call i32 @putchar(i32 %89)
  %91 = sext i32 %90 to i64
  %92 = srem i64 %80, 10
  %93 = add i64 %92, 48
  %94 = trunc i64 %93 to i32
  %95 = call i32 @putchar(i32 %94)
  %96 = sext i32 %95 to i64
  %97 = trunc i64 10 to i32
  %98 = call i32 @putchar(i32 %97)
  %99 = sext i32 %98 to i64
  %100 = trunc i64 10 to i32
  %101 = call i32 @putchar(i32 %100)
  %102 = sext i32 %101 to i64
  %103 = call i32 @puts(i8* getelementptr ([16 x i8], [16 x i8]* @.str.8, i64 0, i64 0))
  %104 = sext i32 %103 to i64
  %105 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.9, i64 0, i64 0))
  %106 = sext i32 %105 to i64
  %107 = call i64 @match_range(i64 3)
  %108 = add i64 %107, 48
  %109 = trunc i64 %108 to i32
  %110 = call i32 @putchar(i32 %109)
  %111 = sext i32 %110 to i64
  %112 = trunc i64 10 to i32
  %113 = call i32 @putchar(i32 %112)
  %114 = sext i32 %113 to i64
  %115 = call i32 @puts(i8* getelementptr ([25 x i8], [25 x i8]* @.str.10, i64 0, i64 0))
  %116 = sext i32 %115 to i64
  %117 = call i64 @match_range(i64 7)
  %118 = add i64 %117, 48
  %119 = trunc i64 %118 to i32
  %120 = call i32 @putchar(i32 %119)
  %121 = sext i32 %120 to i64
  %122 = trunc i64 10 to i32
  %123 = call i32 @putchar(i32 %122)
  %124 = sext i32 %123 to i64
  %125 = call i32 @puts(i8* getelementptr ([28 x i8], [28 x i8]* @.str.11, i64 0, i64 0))
  %126 = sext i32 %125 to i64
  %127 = call i64 @match_range(i64 15)
  %128 = add i64 %127, 48
  %129 = trunc i64 %128 to i32
  %130 = call i32 @putchar(i32 %129)
  %131 = sext i32 %130 to i64
  %132 = trunc i64 10 to i32
  %133 = call i32 @putchar(i32 %132)
  %134 = sext i32 %133 to i64
  %135 = call i32 @puts(i8* getelementptr ([28 x i8], [28 x i8]* @.str.12, i64 0, i64 0))
  %136 = sext i32 %135 to i64
  %137 = call i64 @match_range(i64 50)
  %138 = add i64 %137, 48
  %139 = trunc i64 %138 to i32
  %140 = call i32 @putchar(i32 %139)
  %141 = sext i32 %140 to i64
  %142 = trunc i64 10 to i32
  %143 = call i32 @putchar(i32 %142)
  %144 = sext i32 %143 to i64
  %145 = trunc i64 10 to i32
  %146 = call i32 @putchar(i32 %145)
  %147 = sext i32 %146 to i64
  %148 = call i32 @puts(i8* getelementptr ([13 x i8], [13 x i8]* @.str.13, i64 0, i64 0))
  %149 = sext i32 %148 to i64
  %150 = call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.str.14, i64 0, i64 0))
  %151 = sext i32 %150 to i64
  %152 = call i64 @match_or(i64 2)
  %153 = sdiv i64 %152, 10
  %154 = add i64 %153, 48
  %155 = trunc i64 %154 to i32
  %156 = call i32 @putchar(i32 %155)
  %157 = sext i32 %156 to i64
  %158 = call i64 @match_or(i64 2)
  %159 = srem i64 %158, 10
  %160 = add i64 %159, 48
  %161 = trunc i64 %160 to i32
  %162 = call i32 @putchar(i32 %161)
  %163 = sext i32 %162 to i64
  %164 = trunc i64 10 to i32
  %165 = call i32 @putchar(i32 %164)
  %166 = sext i32 %165 to i64
  %167 = call i32 @puts(i8* getelementptr ([20 x i8], [20 x i8]* @.str.15, i64 0, i64 0))
  %168 = sext i32 %167 to i64
  %169 = call i64 @match_or(i64 5)
  %170 = sdiv i64 %169, 10
  %171 = add i64 %170, 48
  %172 = trunc i64 %171 to i32
  %173 = call i32 @putchar(i32 %172)
  %174 = sext i32 %173 to i64
  %175 = call i64 @match_or(i64 5)
  %176 = srem i64 %175, 10
  %177 = add i64 %176, 48
  %178 = trunc i64 %177 to i32
  %179 = call i32 @putchar(i32 %178)
  %180 = sext i32 %179 to i64
  %181 = trunc i64 10 to i32
  %182 = call i32 @putchar(i32 %181)
  %183 = sext i32 %182 to i64
  %184 = trunc i64 10 to i32
  %185 = call i32 @putchar(i32 %184)
  %186 = sext i32 %185 to i64
  %187 = call i32 @puts(i8* getelementptr ([16 x i8], [16 x i8]* @.str.16, i64 0, i64 0))
  %188 = sext i32 %187 to i64
  %189 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.17, i64 0, i64 0))
  %190 = sext i32 %189 to i64
  %191 = call i64 @match_guard(i64 60)
  %192 = sdiv i64 %191, 100
  %193 = add i64 %192, 48
  %194 = trunc i64 %193 to i32
  %195 = call i32 @putchar(i32 %194)
  %196 = sext i32 %195 to i64
  %197 = sdiv i64 %191, 10
  %198 = srem i64 %197, 10
  %199 = add i64 %198, 48
  %200 = trunc i64 %199 to i32
  %201 = call i32 @putchar(i32 %200)
  %202 = sext i32 %201 to i64
  %203 = srem i64 %191, 10
  %204 = add i64 %203, 48
  %205 = trunc i64 %204 to i32
  %206 = call i32 @putchar(i32 %205)
  %207 = sext i32 %206 to i64
  %208 = trunc i64 10 to i32
  %209 = call i32 @putchar(i32 %208)
  %210 = sext i32 %209 to i64
  %211 = call i32 @puts(i8* getelementptr ([24 x i8], [24 x i8]* @.str.18, i64 0, i64 0))
  %212 = sext i32 %211 to i64
  %213 = call i64 @match_guard(i64 30)
  %214 = sdiv i64 %213, 10
  %215 = add i64 %214, 48
  %216 = trunc i64 %215 to i32
  %217 = call i32 @putchar(i32 %216)
  %218 = sext i32 %217 to i64
  %219 = srem i64 %213, 10
  %220 = add i64 %219, 48
  %221 = trunc i64 %220 to i32
  %222 = call i32 @putchar(i32 %221)
  %223 = sext i32 %222 to i64
  %224 = trunc i64 10 to i32
  %225 = call i32 @putchar(i32 %224)
  %226 = sext i32 %225 to i64
  %227 = call i32 @puts(i8* getelementptr ([25 x i8], [25 x i8]* @.str.19, i64 0, i64 0))
  %228 = sext i32 %227 to i64
  %229 = call i64 @match_guard(i64 10)
  %230 = sdiv i64 %229, 10
  %231 = add i64 %230, 48
  %232 = trunc i64 %231 to i32
  %233 = call i32 @putchar(i32 %232)
  %234 = sext i32 %233 to i64
  %235 = call i64 @match_guard(i64 10)
  %236 = srem i64 %235, 10
  %237 = add i64 %236, 48
  %238 = trunc i64 %237 to i32
  %239 = call i32 @putchar(i32 %238)
  %240 = sext i32 %239 to i64
  %241 = trunc i64 10 to i32
  %242 = call i32 @putchar(i32 %241)
  %243 = sext i32 %242 to i64
  %244 = trunc i64 10 to i32
  %245 = call i32 @putchar(i32 %244)
  %246 = sext i32 %245 to i64
  %247 = call i32 @puts(i8* getelementptr ([18 x i8], [18 x i8]* @.str.20, i64 0, i64 0))
  %248 = sext i32 %247 to i64
  %249 = call i32 @puts(i8* getelementptr ([16 x i8], [16 x i8]* @.str.21, i64 0, i64 0))
  %250 = sext i32 %249 to i64
  %251 = call i64 @match_bind(i64 0)
  %252 = add i64 %251, 48
  %253 = trunc i64 %252 to i32
  %254 = call i32 @putchar(i32 %253)
  %255 = sext i32 %254 to i64
  %256 = trunc i64 10 to i32
  %257 = call i32 @putchar(i32 %256)
  %258 = sext i32 %257 to i64
  %259 = call i32 @puts(i8* getelementptr ([18 x i8], [18 x i8]* @.str.22, i64 0, i64 0))
  %260 = sext i32 %259 to i64
  %261 = call i64 @match_bind(i64 5)
  %262 = sdiv i64 %261, 10
  %263 = add i64 %262, 48
  %264 = trunc i64 %263 to i32
  %265 = call i32 @putchar(i32 %264)
  %266 = sext i32 %265 to i64
  %267 = call i64 @match_bind(i64 5)
  %268 = srem i64 %267, 10
  %269 = add i64 %268, 48
  %270 = trunc i64 %269 to i32
  %271 = call i32 @putchar(i32 %270)
  %272 = sext i32 %271 to i64
  %273 = trunc i64 10 to i32
  %274 = call i32 @putchar(i32 %273)
  %275 = sext i32 %274 to i64
  %276 = call i32 @puts(i8* getelementptr ([26 x i8], [26 x i8]* @.str.23, i64 0, i64 0))
  %277 = sext i32 %276 to i64
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

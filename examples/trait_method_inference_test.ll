; ModuleID = 'trait_method_inference_test'
source_filename = "<vais>"

%SimpleCounter = type { i64 }
declare i64 @fputc(i64, i64)
declare i32 @putchar(i32)
declare i64 @strlen(i64)
declare i64 @ftell(i64)
declare i64 @fputs(i8*, i64)
declare i32 @sched_yield()
declare i64 @malloc(i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i32 @strcmp(i8*, i8*)
declare i32 @usleep(i64)
declare i64 @fseek(i64, i64, i64)
declare void @exit(i32)
declare i64 @fwrite(i64, i64, i64, i64)
declare i32 @puts(i64)
declare i64 @fflush(i64)
declare i64 @fopen(i8*, i8*)
declare i64 @feof(i64)
declare i64 @fread(i64, i64, i64, i64)
declare i32 @printf(i8*)
declare i64 @memcpy(i64, i64, i64)
declare i32 @fclose(i64)
declare i64 @fgets(i64, i64, i64)
declare void @free(i64)
declare i64 @fgetc(i64)
@.str.0 = private unnamed_addr constant [33 x i8] c"Testing trait method resolution:\00"
@.str.1 = private unnamed_addr constant [23 x i8] c"Calling trait methods:\00"
@.str.2 = private unnamed_addr constant [35 x i8] c"Trait method inference successful!\00"

define %SimpleCounter @SimpleCounter_new(i64 %v) {
entry:
  %0 = alloca %SimpleCounter
  %1 = getelementptr %SimpleCounter, %SimpleCounter* %0, i32 0, i32 0
  store i64 %v, i64* %1
  %ret.2 = load %SimpleCounter, %SimpleCounter* %0
  ret %SimpleCounter %ret.2
}

define i64 @SimpleCounter_next(%SimpleCounter* %self) {
entry:
  %0 = getelementptr %SimpleCounter, %SimpleCounter* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %current.2 = alloca i64
  store i64 %1, i64* %current.2
  %3 = getelementptr %SimpleCounter, %SimpleCounter* %self, i32 0, i32 0
  %4 = load i64, i64* %3
  %5 = add i64 %4, 1
  %6 = getelementptr %SimpleCounter, %SimpleCounter* %self, i32 0, i32 0
  store i64 %5, i64* %6
  %7 = load i64, i64* %current.2
  ret i64 %7
}

define i64 @SimpleCounter_has_next(%SimpleCounter* %self) {
entry:
  %0 = getelementptr %SimpleCounter, %SimpleCounter* %self, i32 0, i32 0
  %1 = load i64, i64* %0
  %2 = icmp slt i64 %1, 10
  %3 = zext i1 %2 to i64
  %4 = icmp ne i64 %3, 0
  br i1 %4, label %then0, label %else1
then0:
  br label %merge2
else1:
  br label %merge2
merge2:
  %5 = phi i64 [ 1, %then0 ], [ 0, %else1 ]
  ret i64 %5
}

define i64 @main() {
entry:
  %0 = call i32 @puts(i8* getelementptr ([33 x i8], [33 x i8]* @.str.0, i64 0, i64 0))
  %1 = call %SimpleCounter @SimpleCounter_new(i64 0)
  %counter.2.struct = alloca %SimpleCounter
  store %SimpleCounter %1, %SimpleCounter* %counter.2.struct
  %counter.2 = alloca %SimpleCounter*
  store %SimpleCounter* %counter.2.struct, %SimpleCounter** %counter.2
  %3 = call i32 @puts(i8* getelementptr ([23 x i8], [23 x i8]* @.str.1, i64 0, i64 0))
  %4 = load %SimpleCounter*, %SimpleCounter** %counter.2
  %5 = call i64 @SimpleCounter_has_next(%SimpleCounter* %4)
  %status.6 = alloca i64
  store i64 %5, i64* %status.6
  %7 = load i64, i64* %status.6
  %8 = add i64 %7, 48
  %9 = trunc i64 %8 to i32
  %10 = call i32 @putchar(i32 %9)
  %11 = trunc i64 10 to i32
  %12 = call i32 @putchar(i32 %11)
  %13 = load %SimpleCounter*, %SimpleCounter** %counter.2
  %14 = call i64 @SimpleCounter_next(%SimpleCounter* %13)
  %val.15 = alloca i64
  store i64 %14, i64* %val.15
  %16 = load i64, i64* %val.15
  %17 = add i64 %16, 48
  %18 = trunc i64 %17 to i32
  %19 = call i32 @putchar(i32 %18)
  %20 = trunc i64 10 to i32
  %21 = call i32 @putchar(i32 %20)
  %22 = load %SimpleCounter*, %SimpleCounter** %counter.2
  %23 = call i64 @SimpleCounter_next(%SimpleCounter* %22)
  %val2.24 = alloca i64
  store i64 %23, i64* %val2.24
  %25 = load i64, i64* %val2.24
  %26 = add i64 %25, 48
  %27 = trunc i64 %26 to i32
  %28 = call i32 @putchar(i32 %27)
  %29 = trunc i64 10 to i32
  %30 = call i32 @putchar(i32 %29)
  %31 = call i32 @puts(i8* getelementptr ([35 x i8], [35 x i8]* @.str.2, i64 0, i64 0))
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

; ModuleID = 'option_test'
source_filename = "<vais>"

%Option = type { i32, { i64 } }
declare i32 @putchar(i32)
declare i64 @feof(i64)
declare i64 @strlen(i64)
declare i32 @puts(i8*)
declare i32 @strcmp(i8*, i8*)
declare i64 @fgets(i64, i64, i64)
declare i64 @memcpy(i64, i64, i64)
declare i32 @printf(i8*)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @fputc(i64, i64)
declare i64 @fread(i64, i64, i64, i64)
declare i32 @usleep(i64)
declare void @exit(i32)
declare i64 @fputs(i8*, i64)
declare i64 @fgetc(i64)
declare i64 @fseek(i64, i64, i64)
declare i64 @fopen(i8*, i8*)
declare i32 @fclose(i64)
declare i32 @strncmp(i8*, i8*, i64)
declare void @free(i64)
declare i64 @malloc(i64)
declare i64 @fflush(i64)
declare i32 @sched_yield()
declare i64 @ftell(i64)
define i64 @Option_is_some(%Option* %self) {
entry:
  br label %match.check1
match.check1:
  %0 = getelementptr { i32 }, { i32 }* %self, i32 0, i32 0
  %1 = load i32, i32* %0
  %2 = icmp eq i32 %1, 1
  br i1 %2, label %match.arm3, label %match.check2
match.arm3:
  %3 = getelementptr { i32, i64 }, { i32, i64 }* %self, i32 0, i32 1
  %4 = load i64, i64* %3
  br label %match.merge0
match.check2:
  br i1 1, label %match.arm4, label %match.merge0
match.arm4:
  br label %match.merge0
match.merge0:
  %5 = phi i64 [ 1, %match.arm3 ], [ 0, %match.arm4 ]
  ret i64 %5
}

define i64 @Option_is_none(%Option* %self) {
entry:
  br label %match.check1
match.check1:
  %0 = getelementptr { i32 }, { i32 }* %self, i32 0, i32 0
  %1 = load i32, i32* %0
  %2 = icmp eq i32 %1, 1
  br i1 %2, label %match.arm3, label %match.check2
match.arm3:
  %3 = getelementptr { i32, i64 }, { i32, i64 }* %self, i32 0, i32 1
  %4 = load i64, i64* %3
  br label %match.merge0
match.check2:
  br i1 1, label %match.arm4, label %match.merge0
match.arm4:
  br label %match.merge0
match.merge0:
  %5 = phi i64 [ 0, %match.arm3 ], [ 1, %match.arm4 ]
  ret i64 %5
}

define i64 @Option_unwrap_or(%Option* %self, i64 %default) {
entry:
  br label %match.check1
match.check1:
  %0 = getelementptr { i32 }, { i32 }* %self, i32 0, i32 0
  %1 = load i32, i32* %0
  %2 = icmp eq i32 %1, 1
  br i1 %2, label %match.arm3, label %match.check2
match.arm3:
  %3 = getelementptr { i32, i64 }, { i32, i64 }* %self, i32 0, i32 1
  %4 = load i64, i64* %3
  %v.5 = alloca i64
  store i64 %4, i64* %v.5
  %6 = load i64, i64* %v.5
  br label %match.merge0
match.check2:
  br i1 1, label %match.arm4, label %match.merge0
match.arm4:
  br label %match.merge0
match.merge0:
  %7 = phi i64 [ %6, %match.arm3 ], [ %default, %match.arm4 ]
  ret i64 %7
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
  %v.10 = alloca i64
  store i64 %9, i64* %v.10
  %11 = load i64, i64* %v.10
  br label %match.merge0
match.check2:
  br i1 1, label %match.arm4, label %match.merge0
match.arm4:
  br label %match.merge0
match.merge0:
  %12 = phi i64 [ %11, %match.arm3 ], [ 0, %match.arm4 ]
  ret i64 %12
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

; ModuleID = 'control_flow'
source_filename = "<vais>"

declare void @free(i64)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @fputs(i8*, i64)
declare i64 @fgets(i64, i64, i64)
declare void @exit(i32)
declare i64 @feof(i64)
declare i32 @printf(i8*)
declare i64 @fgetc(i64)
declare i32 @fclose(i64)
declare i64 @malloc(i64)
declare i32 @strcmp(i8*, i8*)
declare i32 @puts(i8*)
declare i64 @ftell(i64)
declare i32 @putchar(i32)
declare i32 @sched_yield()
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @fseek(i64, i64, i64)
declare i64 @memcpy(i64, i64, i64)
declare i32 @usleep(i64)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @fopen(i8*, i8*)
declare i64 @fflush(i64)
declare i64 @strlen(i64)
declare i64 @fputc(i64, i64)
define i64 @max(i64 %a, i64 %b) !dbg !4 {
entry:
  %0 = icmp sgt i64 %a, %b
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %ternary.then0, label %ternary.else1
ternary.then0:
  br label %ternary.merge2
ternary.else1:
  br label %ternary.merge2
ternary.merge2:
  %3 = phi i64 [ %a, %ternary.then0 ], [ %b, %ternary.else1 ]
  ret i64 %3, !dbg !5
}

define i64 @countdown(i64 %n) !dbg !7 {
entry:
  %0 = icmp slt i64 %n, 1
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %ternary.then0, label %ternary.else1
ternary.then0:
  br label %ternary.merge2
ternary.else1:
  %3 = sub i64 %n, 1
  %4 = call i64 @countdown(i64 %3)
  br label %ternary.merge2
ternary.merge2:
  %5 = phi i64 [ 0, %ternary.then0 ], [ %4, %ternary.else1 ]
  ret i64 %5, !dbg !8
}

define i64 @factorial(i64 %n) !dbg !10 {
entry:
  %0 = icmp slt i64 %n, 2
  %1 = zext i1 %0 to i64
  %2 = icmp ne i64 %1, 0
  br i1 %2, label %ternary.then0, label %ternary.else1
ternary.then0:
  br label %ternary.merge2
ternary.else1:
  %3 = sub i64 %n, 1
  %4 = call i64 @factorial(i64 %3)
  %5 = mul i64 %n, %4
  br label %ternary.merge2
ternary.merge2:
  %6 = phi i64 [ 1, %ternary.then0 ], [ %5, %ternary.else1 ]
  ret i64 %6, !dbg !11
}

define i64 @main() !dbg !13 {
entry:
  %0 = call i64 @factorial(i64 5)
  ret i64 %0, !dbg !14
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

; Debug intrinsics
declare void @llvm.dbg.declare(metadata, metadata, metadata)
declare void @llvm.dbg.value(metadata, metadata, metadata)

; Debug Information
!llvm.dbg.cu = !{!1}
!llvm.module.flags = !{!2}

!0 = !DIFile(filename: "control_flow.vais", directory: "examples")
!1 = distinct !DICompileUnit(language: DW_LANG_C99, file: !0, producer: "vaisc 0.1.0", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false)
!2 = !{i32 2, !"Debug Info Version", i32 3}
!3 = !DISubroutineType(types: !{})
!4 = distinct !DISubprogram(name: "max", scope: !0, file: !0, line: 2, type: !3, scopeLine: 2, spFlags: DISPFlagDefinition, unit: !1, retainedNodes: !{})
!5 = !DILocation(line: 2, column: 28, scope: !4)
!6 = !DISubroutineType(types: !{})
!7 = distinct !DISubprogram(name: "countdown", scope: !0, file: !0, line: 5, type: !6, scopeLine: 5, spFlags: DISPFlagDefinition, unit: !1, retainedNodes: !{})
!8 = !DILocation(line: 5, column: 27, scope: !7)
!9 = !DISubroutineType(types: !{})
!10 = distinct !DISubprogram(name: "factorial", scope: !0, file: !0, line: 8, type: !9, scopeLine: 8, spFlags: DISPFlagDefinition, unit: !1, retainedNodes: !{})
!11 = !DILocation(line: 8, column: 27, scope: !10)
!12 = !DISubroutineType(types: !{})
!13 = distinct !DISubprogram(name: "main", scope: !0, file: !0, line: 11, type: !12, scopeLine: 11, spFlags: DISPFlagDefinition, unit: !1, retainedNodes: !{})
!14 = !DILocation(line: 11, column: 17, scope: !13)

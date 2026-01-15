; ModuleID = 'control_flow'
source_filename = "<vais>"

define i64 @max(i64 %a, i64 %b) {
entry:
  %0 = icmp sgt i64 %a, %b
  br i1 %0, label %ternary.then0, label %ternary.else1
ternary.then0:
  br label %ternary.merge2
ternary.else1:
  br label %ternary.merge2
ternary.merge2:
  %1 = phi i64 [ %a, %ternary.then0 ], [ %b, %ternary.else1 ]
  ret i64 %1
}

define i64 @countdown(i64 %n) {
entry:
  %0 = icmp slt i64 %n, 1
  br i1 %0, label %ternary.then0, label %ternary.else1
ternary.then0:
  br label %ternary.merge2
ternary.else1:
  %1 = sub i64 %n, 1
  %2 = call i64 @countdown(i64 %1)
  br label %ternary.merge2
ternary.merge2:
  %3 = phi i64 [ 0, %ternary.then0 ], [ %2, %ternary.else1 ]
  ret i64 %3
}

define i64 @factorial(i64 %n) {
entry:
  %0 = icmp slt i64 %n, 2
  br i1 %0, label %ternary.then0, label %ternary.else1
ternary.then0:
  br label %ternary.merge2
ternary.else1:
  %1 = sub i64 %n, 1
  %2 = call i64 @factorial(i64 %1)
  %3 = mul i64 %n, %2
  br label %ternary.merge2
ternary.merge2:
  %4 = phi i64 [ 1, %ternary.then0 ], [ %3, %ternary.else1 ]
  ret i64 %4
}

define i64 @main() {
entry:
  %0 = call i64 @factorial(i64 5)
  ret i64 %0
}


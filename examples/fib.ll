; ModuleID = 'fib'
source_filename = "<vais>"

define i64 @fib(i64 %n) {
entry:
  %0 = icmp slt i64 %n, 2
  br i1 %0, label %ternary.then0, label %ternary.else1
ternary.then0:
  br label %ternary.merge2
ternary.else1:
  %1 = sub i64 %n, 1
  %2 = call i64 @fib(i64 %1)
  %3 = sub i64 %n, 2
  %4 = call i64 @fib(i64 %3)
  %5 = add i64 %2, %4
  br label %ternary.merge2
ternary.merge2:
  %6 = phi i64 [ %n, %ternary.then0 ], [ %5, %ternary.else1 ]
  ret i64 %6
}

define i64 @add(i64 %a, i64 %b) {
entry:
  %0 = add i64 %a, %b
  ret i64 %0
}

define i64 @main() {
entry:
  %0 = call i64 @fib(i64 10)
  ret i64 %0
}


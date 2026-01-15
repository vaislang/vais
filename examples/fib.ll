; ModuleID = 'fib'
source_filename = "<vais>"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

define i64 @fib(i64 %n) {
entry:
  %0 = icmp slt i64 %n, 2
  %1 = sub i64 %n, 1
  %2 = call i64 @fib(i64 %1)
  %3 = sub i64 %n, 2
  %4 = call i64 @fib(i64 %3)
  %5 = add i64 %2, %4
  %6 = select i1 %0, i64 %n, i64 %5
  ret i64 %6
}

define i64 @add(i64 %a, i64 %b) {
entry:
  %0 = add i64 %a, %b
  ret i64 %0
}


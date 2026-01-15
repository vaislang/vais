; ModuleID = 'arrays'
source_filename = "<vais>"

define i64 @get_elem(i64* %arr, i64 %idx) {
entry:
  %0 = getelementptr i64, i64* %arr, i64 %idx
  %1 = load i64, i64* %0
  ret i64 %1
}

define i64 @main() {
entry:
  %0 = alloca [4  x i64]
  %1 = getelementptr [4  x i64], [4  x i64]* %0, i64 0, i64 0
  store i64 10, i64* %1
  %2 = getelementptr [4  x i64], [4  x i64]* %0, i64 0, i64 1
  store i64 20, i64* %2
  %3 = getelementptr [4  x i64], [4  x i64]* %0, i64 0, i64 2
  store i64 30, i64* %3
  %4 = getelementptr [4  x i64], [4  x i64]* %0, i64 0, i64 3
  store i64 40, i64* %4
  %5 = getelementptr [4  x i64], [4  x i64]* %0, i64 0, i64 0
  %arr = alloca i64*
  store i64* %5, i64** %arr
  %6 = load i64*, i64** %arr
  %7 = call i64 @get_elem(i64* %6, i64 2)
  ret i64 %7
}


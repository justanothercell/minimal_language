; ModuleID = 'testing/fibonacci.bc'
source_filename = "fibonacci"

@0 = private unnamed_addr constant [3 x i8] c"%d\00", align 1
@1 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1

define void @main() {
entry:
  call void @main.1()
  ret void
}

declare i32 @puts(ptr)

declare i32 @printf(ptr, ...)

define i32 @print_int(i32 %0) {
entry:
  %len = call i32 (ptr, ...) @printf(ptr @0, i32 %0)
  %1 = call i32 @puts(ptr @1)
  ret i32 %len
}

define i32 @fibonacci(i32 %0) {
entry:
  %1 = icmp sle i32 %0, 2
  br i1 %1, label %then, label %else

then:                                             ; preds = %entry
  ret i32 1

else:                                             ; preds = %entry
  %2 = sub i32 %0, 1
  %3 = call i32 @fibonacci(i32 %2)
  %4 = sub i32 %0, 2
  %5 = call i32 @fibonacci(i32 %4)
  %6 = add i32 %3, %5
  ret i32 %6

ifcont:                                           ; No predecessors!
}

define void @main.1() {
entry:
  %0 = call i32 @fibonacci(i32 1)
  %1 = call i32 @print_int(i32 %0)
  %2 = call i32 @fibonacci(i32 2)
  %3 = call i32 @print_int(i32 %2)
  %4 = call i32 @fibonacci(i32 3)
  %5 = call i32 @print_int(i32 %4)
  %6 = call i32 @fibonacci(i32 4)
  %7 = call i32 @print_int(i32 %6)
  %8 = call i32 @fibonacci(i32 5)
  %9 = call i32 @print_int(i32 %8)
  %10 = call i32 @fibonacci(i32 6)
  %11 = call i32 @print_int(i32 %10)
  %12 = call i32 @fibonacci(i32 7)
  %13 = call i32 @print_int(i32 %12)
  %14 = call i32 @fibonacci(i32 8)
  %15 = call i32 @print_int(i32 %14)
  %16 = call i32 @fibonacci(i32 9)
  %17 = call i32 @print_int(i32 %16)
  %18 = call i32 @fibonacci(i32 10)
  %19 = call i32 @print_int(i32 %18)
  %20 = call i32 @fibonacci(i32 11)
  %21 = call i32 @print_int(i32 %20)
  %22 = call i32 @fibonacci(i32 12)
  %23 = call i32 @print_int(i32 %22)
  ret void
}

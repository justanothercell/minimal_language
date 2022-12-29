; ModuleID = 'testing/is_prime.bc'
source_filename = "is_prime"

@0 = private unnamed_addr constant [3 x i8] c"%d\00", align 1
@1 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@2 = private unnamed_addr constant [11 x i8] c"is a prime\00", align 1
@3 = private unnamed_addr constant [15 x i8] c"is not a prime\00", align 1

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

define i32 @void(i32 %0, i32 %1) {
entry:
  %2 = sdiv i32 %0, %1
  %3 = mul i32 %2, %1
  %4 = sub i32 %0, %3
  ret i32 %4
}

define i1 @is_prime(i32 %0) {
entry:
  %1 = sdiv i32 %0, 2
  %i = add i32 %1, 1
  %i1 = alloca i32, align 4
  store i32 %i, ptr %i1, align 4
  br label %cond

cond:                                             ; preds = %ifcont, %entry
  %2 = load i32, ptr %i1, align 4
  %3 = icmp sgt i32 %2, 2
  br i1 %3, label %body, label %whilecont

body:                                             ; preds = %cond
  %4 = load i32, ptr %i1, align 4
  %i2 = sub i32 %4, 1
  store i32 %i2, ptr %i1, align 4
  %5 = load i32, ptr %i1, align 4
  %6 = call i32 @void(i32 %0, i32 %5)
  %7 = icmp eq i32 %6, 0
  br i1 %7, label %then, label %else

whilecont:                                        ; preds = %cond
  ret i1 true

then:                                             ; preds = %body
  ret i1 false

else:                                             ; preds = %body
  br label %ifcont

ifcont:                                           ; preds = %else
  br label %cond
}

define void @prime_checker(i32 %0) {
entry:
  %1 = call i32 @print_int(i32 %0)
  %2 = call i1 @is_prime(i32 %0)
  br i1 %2, label %then, label %else

then:                                             ; preds = %entry
  %3 = call i32 @puts(ptr @2)
  br label %ifcont

else:                                             ; preds = %entry
  %4 = call i32 @puts(ptr @3)
  br label %ifcont

ifcont:                                           ; preds = %else, %then
  ret void
}

define void @main.1() {
entry:
  call void @prime_checker(i32 479001599)
  ret void
}

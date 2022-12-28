##### A minimal-effort language made to test llvm.
##### [hello world](testing/hello_world.mi)
```haskell
#include lib/std

fn main do
    call puts with literal ptr "hello, worlds!" end
end

```

### How to use
- Install LLVM 16 (other versions are probably file aswell, 
you will just need to adjust [cargo.toml](cargo.toml) to use the matching
version of `llvm-sys`.<br>
You may need to compile LLVM by hand, as the current releases for windows lack some needed
tools such as `llvm-config.exe`
- set environment variable `LLVM_SYS_150_PREFIX` (maybe replace the 150) to the llvm root directory 
or make sure llvm is on PATH (the compiler will complain and will tell you which variable exactly
needs to be set)
- set the path of the source file and executable as demonstrated in [main.rs](src/main.rs)

##### code example:
(removed `#include` to show off more code)
```haskell
extern fn puts i32 with ptr str end
extern fn printf i32 with vararg ptr str end

const EMPTY_STR ptr ""
const INT_TO_STR_FMT ptr "%d" end

fn print_int i32 with i32 num do
    let i32 len be call printf with INT_TO_STR_FMT num end
    call puts with EMPTY_STR end
    return len
end

fn main do
    let i32 a be literal i32 42
    let i32 b be literal i32 69
    let i32 r be call + with a b end
    
    let i32 len_of_printed be call print_int with r end
    call print_int with call + with len_of_printed literal i32 100 end end
end
```
prints:
```
42
69
111
103
```



### DISCLAIMER: 

Seriously don't use this in production. This is only to test LLVM's features and 
to provide a small reference on how to do things, since the LLVM C API docs are
a bit lacking. Check out [compiler.rs](src/compiler.rs) for most LLVM usage.

This language does not even have an AST, it goes directly from tokens to LLVM.
This should help simplify the whole process and make it quite obvious where to
look to find certain llvm compiler usages. The compiler does not check for full
validity of the source code, it just expects stuff to work,
or it panics/makes llvm crash.

The syntax is intentionally focused on easy-to-parse to let the compiler stay as
lightweight and readable as possible.

I forbid this "[...] to become some dependency of some other crap".
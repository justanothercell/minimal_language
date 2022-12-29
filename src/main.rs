#![feature(pattern)]
#![feature(try_blocks)]
#![feature(stmt_expr_attributes)]

use std::process::Command;
use llvm_sys::bit_writer;
use llvm_sys::core;
use crate::compiler::compile;
use crate::source::{ParseError, Source};
use crate::tokens::tok_iter::TokIter;
use crate::tokens::tokenize::tokenize;

mod tokens;
mod source;
mod compiler;

#[macro_export]
macro_rules! c_str {
    ($s:literal) => (
        #[allow(unused_unsafe)]
        unsafe { std::ffi::CStr::from_ptr(concat!($s, "\0").as_ptr() as *const i8) }
    );
    ($s:expr) => (
        #[allow(unused_unsafe)]
        unsafe { std::ffi::CStr::from_ptr(($s.to_string() + "\0").as_ptr() as *const i8) }
    );
}

#[macro_export]
macro_rules! c_str_ptr {
    ($s:expr) => (
        $crate::c_str!($s).as_ptr()
    );
}

fn main() {
    match compile_full("testing/fibonacci") {
        Ok(_) => (),
        Err(e) => panic!("{}\n{:?}", e, e)
    }
    println!();
    let code = Command::new("testing/fibonacci.exe")
        .spawn().unwrap().wait().unwrap();
    println!("executed with {code}");
}

fn compile_full(src: &str) -> Result<(), ParseError>{
    let name = src.split("/").last().unwrap();
    let source = Source::from_file(src.to_string() + ".mi").expect("Could not read source file");
    let tokens = tokenize(source)?;
    let module = compile(TokIter::new(tokens), name)?;
    let bitcode_file = src.to_string() + ".bc";
    let success = unsafe { bit_writer::LLVMWriteBitcodeToFile(module, c_str_ptr!(bitcode_file)) };
    println!("wrote to file with exit code: {success}");
    println!();
    unsafe { core::LLVMDumpModule(module) }
    println!();
    unsafe { core::LLVMDisposeModule(module) }
    let dis_code = Command::new("C:/LLVM/llvm-project/build/Release/bin/llvm-dis.exe")
        .args([bitcode_file.clone()])
        .spawn().unwrap().wait().unwrap();
    println!("disassembled .bc to .ll with {dis_code}");
    println!();
    let compile_code = Command::new("C:/LLVM/llvm-project/build/Release/bin/clang.exe")
        .args([bitcode_file, "-v".to_string(), "-o".to_string(), src.to_string() + ".exe"])
        .spawn().unwrap().wait().unwrap();
    println!();
    println!("compiled to binary with {compile_code}");
    Ok(())
}
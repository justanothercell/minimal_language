use std::collections::HashMap;
use std::ffi::{c_uint, c_ulonglong};
use llvm_sys::{prelude, core};
use llvm_sys::prelude::{LLVMBool, LLVMTypeRef, LLVMValueRef};
use crate::{c_str, c_str_ptr};
use crate::source::{ParseError, ParseET, Span};
use crate::tokens::tok_iter::TokIter;
use crate::tokens::tokens::{Literal, NumLit, Token, TokenType};

macro_rules! expect_ident {
    ($tokens: ident, $expected: literal) => {
        {
            let tok = $tokens.this()?;
            if let TokenType::Ident(ident) = tok.tt {
                if ident == $expected {
                    $tokens.next();
                } else {
                    return Err(ParseET::ParseError($expected.to_string(), ident).at(tok.loc))
                }
            } else {
                return Err(ParseET::ParseError($expected.to_string(), format!("{:?}", tok.tt)).at(tok.loc))
            }
        }
    };
}

macro_rules! ident_next {
    ($tokens: ident, $expected: literal) => {
        {
            let tok = $tokens.this()?;
            if let TokenType::Ident(ident) = tok.tt {
                $tokens.next();
                ident
            } else {
                return Err(ParseET::ParseError($expected.to_string(), format!("{:?}", tok.tt)).at(tok.loc))
            }
        }
    };
}

pub(crate) fn compile(mut tokens: TokIter, name: &str) -> Result<prelude::LLVMModuleRef, ParseError> {
    let module = unsafe { core::LLVMModuleCreateWithName(c_str_ptr!(name)) };
    let function_name = c_str!("main");
    let function_type = unsafe {
        let mut param_types = [];
        core::LLVMFunctionType(core::LLVMVoidType(), param_types.as_mut_ptr(), param_types.len() as u32, 0)
    };
    let function = unsafe { core::LLVMAddFunction(module, function_name.as_ptr(), function_type) };
    let entry_block = unsafe { core::LLVMAppendBasicBlock(function, c_str_ptr!("entry")) };
    let builder = unsafe {
        let b = core::LLVMCreateBuilder();
        core::LLVMPositionBuilderAtEnd(b, entry_block);
        b
    };

    let mut varmap = HashMap::new();
    while tokens.this().is_ok() {
        let tok = tokens.this()?;
        match tok.tt {
            TokenType::Ident(ident) => match ident.as_str() {
                "const" => compile_global_const(&mut tokens, &module, &builder, &mut varmap),
                "extern" => compile_extern(&mut tokens, &module, &mut varmap),
                "fn" => compile_fn(&mut tokens, &module, &mut varmap),
                e => return Err(ParseET::ParseError("[const|extern|fn]".to_string(), e.to_string()).at(tok.loc))
            }
            e => return Err(ParseET::ParseError("keyword".to_string(), format!("{e:?}")).at(tok.loc))
        }?;
    }

    unsafe {
        let fun = varmap.get("main").unwrap();
        core::LLVMBuildCall2(builder, fun.0, fun.1, [].as_mut_ptr(), 0 as c_uint, c_str_ptr!(""));
        core::LLVMBuildRetVoid(builder);
        core::LLVMDisposeBuilder(builder)
    }
    Ok(module)
}

fn get_var(name: &str, varmap: &mut HashMap<String, (LLVMTypeRef, LLVMValueRef)>, local_varmap: &mut HashMap<String, (LLVMTypeRef, LLVMValueRef)>) -> Result<(LLVMTypeRef, LLVMValueRef), ParseError>{
    local_varmap.get(name).map(|t|Ok(t.clone()))
        .unwrap_or_else(||varmap.get(name).map(|t|t.clone()).ok_or(ParseET::VariableError(name.to_string()).error()))
}

fn compile_global_const(tokens: &mut TokIter, module: &prelude::LLVMModuleRef, builder: &prelude::LLVMBuilderRef, varmap: &mut HashMap<String, (LLVMTypeRef, LLVMValueRef)>) -> Result<(), ParseError>{
    expect_ident!(tokens, "const");
    let ty = ident_next!(tokens, "type");
    let name = ident_next!(tokens, "name");
    expect_ident!(tokens, "is");
    let tok = tokens.this()?;
    let val = match tok.tt {
        TokenType::Literal(lit) => match lit {
            Literal::String(s) => {
                Ok(s)
            },
            _ => Err(ParseET::ParseError("string literal [only literal type supported]".to_string(), format!("{lit:?}")).at(tok.loc))
        }
        tt => Err(ParseET::ParseError("literal".to_string(), format!("{tt:?}")).at(tok.loc))
    }?;
    tokens.next();
    let p = unsafe {core::LLVMBuildGlobalString(*builder, c_str_ptr!(val), c_str_ptr!(name))};
    varmap.insert(name, (unsafe{ core::LLVMPointerType(core::LLVMInt8Type(), 0) }, p));
    Ok(())
}

fn fn_sig(tokens: &mut TokIter) -> Result<(String, Option<String>, Vec<(String, String)>, bool), ParseError> {
    expect_ident!(tokens, "fn");
    let name = ident_next!(tokens, "name");
    let n = ident_next!(tokens, "[with|do|end|<type>]");
    match n.as_str() {
        "do" | "end"  => Ok((name, None, vec![], false)),
        "with" => {
            let vararg = if &ident_next!(tokens, "<vararg?>") == "vararg" {
                true
            } else {
                tokens.index -= 1;
                false
            };
            let mut args = vec![];
            loop {
                args.push((ident_next!(tokens, "[with|do|end|<type>]"), ident_next!(tokens, "name")));
                let n = ident_next!(tokens, "[do|end]");
                if n == "do" || n == "end" {
                    break
                }
                tokens.index -= 1
            }
            Ok((name, None, args, vararg))
        }
        _  => {
            tokens.index -= 1;
            let ty = ident_next!(tokens, "<type>");
            let n2 = ident_next!(tokens, "[with|do|end]");
            match n2.as_str() {
                "do" | "end"  => Ok((name, Some(ty), vec![], false)),
                "with" => {
                    let vararg = if &ident_next!(tokens, "<vararg?>") == "vararg" {
                        true
                    } else {
                        tokens.index -= 1;
                        false
                    };
                    let mut args = vec![];
                    loop {
                        args.push((ident_next!(tokens, "[with|do|end|<type>]"), ident_next!(tokens, "name")));
                        let n = ident_next!(tokens, "[do|end]");
                        if n == "do" || n == "end" {
                            break
                        }
                        tokens.index -= 1
                    }
                    Ok((name, Some(ty), args, vararg))
                }
                e => Err(ParseET::ParseError("[with|do|end]".to_string(), n2).at(tokens.this()?.loc))
            }
        }
    }
}

fn ty_str_to_ty(ty: &str) -> Result<prelude::LLVMTypeRef, ParseError>{
    unsafe {
        match ty {
            "void" => Ok(core::LLVMVoidType()),
            "ptr" => Ok(core::LLVMPointerType(core::LLVMInt8Type(), 0)),
            "i8" =>  Ok(core::LLVMInt8Type()),
            "i32" =>  Ok(core::LLVMInt32Type()),
            "i64" =>  Ok(core::LLVMInt64Type()),
            "i128" =>  Ok(core::LLVMInt128Type()),
            _ => Err(ParseET::ParseError("valid type".to_string(), ty.to_string()).error())
        }
    }
}

fn compile_extern(tokens: &mut TokIter, module: &prelude::LLVMModuleRef, varmap: &mut HashMap<String, (LLVMTypeRef, LLVMValueRef)>) -> Result<(), ParseError> {
    expect_ident!(tokens, "extern");
    let (name, ty, args, vararg) = fn_sig(tokens)?;
    let fn_name = c_str!(name);
    let ret_ty = ty_str_to_ty(&ty.unwrap_or("void".to_string()))?;
    let mut params = args.iter().map(|(t, _)| ty_str_to_ty(t.as_str())).collect::<Result<Vec<LLVMTypeRef>, _>>()?;
    unsafe {
        let puts_fn_ty = core::LLVMFunctionType(ret_ty, params.as_mut_ptr(), params.len() as c_uint, vararg as LLVMBool);
        let puts_fn = core::LLVMAddFunction(*module, fn_name.as_ptr(), puts_fn_ty.clone());
        varmap.insert(name, (puts_fn_ty, puts_fn));
    }
    Ok(())
}

fn compile_fn(tokens: &mut TokIter, module: &prelude::LLVMModuleRef,
              varmap: &mut HashMap<String, (LLVMTypeRef, LLVMValueRef)>) -> Result<(), ParseError> {
    let (name, ty, args, vararg) = fn_sig(tokens)?;
    let function_name = c_str!(name.as_str());
    let mut param_names = vec![];
    let mut param_types = vec![];
    let ret_ty = ty_str_to_ty(&ty.clone().unwrap_or(String::from("void")))?;
    let function_type = unsafe {
        for (ty, n) in args {
            param_types.push(ty_str_to_ty(&ty).unwrap());
            param_names.push(n);
        }
        core::LLVMFunctionType(ret_ty, param_types.as_mut_ptr(), param_types.len() as u32, vararg as LLVMBool)
    };
    let function = unsafe { core::LLVMAddFunction(*module, function_name.as_ptr(), function_type) };
    varmap.insert(name.clone(), (function_type, function));
    let mut local_varmap = HashMap::new();
    for (i, pn) in param_names.into_iter().enumerate() {
        let v = unsafe { core::LLVMGetParam(function, i as c_uint) };
        local_varmap.insert(pn, (param_types.remove(0), v));
    }
    let entry_block = unsafe { core::LLVMAppendBasicBlock(function, c_str_ptr!("entry")) };
    let builder = unsafe {
        let b = core::LLVMCreateBuilder();
        core::LLVMPositionBuilderAtEnd(b, entry_block);
        b
    };

    unsafe {
        while tokens.this()?.tt != TokenType::Ident(String::from("end")){
            compile_statement(tokens, module, &builder, varmap, &mut local_varmap)?;
        }
        if let None = ty {
            core::LLVMBuildRetVoid(builder);
        }
    }
    expect_ident!(tokens, "end");
    Ok(())
}

fn compile_statement(tokens: &mut TokIter, module: &prelude::LLVMModuleRef, builder: &prelude::LLVMBuilderRef,
                     varmap: &mut HashMap<String, (LLVMTypeRef, LLVMValueRef)>,
                     local_varmap: &mut HashMap<String, (LLVMTypeRef, LLVMValueRef)>) -> Result<(), ParseError> {
    match ident_next!(tokens, "[let|<expr>]").as_str() {
        "let" => compile_let_assign(tokens, module, builder, varmap, local_varmap)?,
        "return" => compile_return(tokens, module, builder, varmap, local_varmap)?,
        _ => {
            tokens.index -= 1;
            compile_expression(tokens, module, builder, varmap, local_varmap, "")?;
        }
    }
    Ok(())
}

fn compile_expression(tokens: &mut TokIter, module: &prelude::LLVMModuleRef, builder: &prelude::LLVMBuilderRef,
                     varmap: &mut HashMap<String, (LLVMTypeRef, LLVMValueRef)>,
                     local_varmap: &mut HashMap<String, (LLVMTypeRef, LLVMValueRef)>,
                     ret_name: &str) -> Result<LLVMValueRef, ParseError> {
    let r = match ident_next!(tokens, "[call|literal|<variable>]").as_str() {
        "call" => compile_fn_call(tokens, module, builder, varmap, local_varmap, ret_name)?,
        "literal" => compile_literal(tokens, module, builder, varmap, local_varmap)?,
        v => get_var(v, varmap, local_varmap)?.1
    };
    Ok(r)
}

fn compile_return(tokens: &mut TokIter, module: &prelude::LLVMModuleRef, builder: &prelude::LLVMBuilderRef,
                    varmap: &mut HashMap<String, (LLVMTypeRef, LLVMValueRef)>,
                    local_varmap: &mut HashMap<String, (LLVMTypeRef, LLVMValueRef)>) -> Result<(), ParseError> {
    unsafe {
        if &ident_next!(tokens, "[end|<var>]") == "end" {
            core::LLVMBuildRetVoid(*builder);
        }
        else {
            tokens.index -= 1;
            core::LLVMBuildRet(*builder, compile_expression(tokens, module, builder, varmap, local_varmap, "")?);
        }
    }
    Ok(())
}

fn compile_fn_call(tokens: &mut TokIter, module: &prelude::LLVMModuleRef, builder: &prelude::LLVMBuilderRef,
                    varmap: &mut HashMap<String, (LLVMTypeRef, LLVMValueRef)>,
                    local_varmap: &mut HashMap<String, (LLVMTypeRef, LLVMValueRef)>,
                    ret_name: &str) -> Result<LLVMValueRef, ParseError> {
    let name = tokens.this()?.tt;
    tokens.next();
    let n = ident_next!(tokens, "[with|end]");
    let mut args = vec![];
    if &n == "with" {
        while {
            let i = ident_next!(tokens, "[<arg>|end]");
            if i != "end" {
                tokens.index -= 1;
                args.push(compile_expression(tokens, module, builder, varmap, local_varmap, "")?);
                true
            } else { false }
        } {}
    }
    let r = if let TokenType::Particle(p, _) = name{
        let b = args.pop().unwrap();
        let a = args.pop().unwrap();
        unsafe {
            println!("BUILT {p}");
            match p {
                '+' => core::LLVMBuildAdd(*builder, a, b, c_str_ptr!(ret_name)),
                '-' => core::LLVMBuildSub(*builder, a, b, c_str_ptr!(ret_name)),
                '*' => core::LLVMBuildMul(*builder, a, b, c_str_ptr!(ret_name)),
                '/' => core::LLVMBuildSDiv(*builder, a, b, c_str_ptr!(ret_name)),
                '&' => core::LLVMBuildAnd(*builder, a, b, c_str_ptr!(ret_name)),
                '|' => core::LLVMBuildOr(*builder, a, b, c_str_ptr!(ret_name)),
                c => unreachable!("unknown literal func {c}")
            }
        }
    } else if let TokenType::Ident(name) = name{
        let fun = varmap.get(&name).unwrap();
        unsafe { core::LLVMBuildCall2(*builder, fun.0, fun.1, args.as_mut_ptr(), args.len() as c_uint, c_str_ptr!(ret_name)) }
    } else { unreachable!() };
    Ok(r)
}

fn compile_literal(tokens: &mut TokIter, module: &prelude::LLVMModuleRef, builder: &prelude::LLVMBuilderRef,
                    varmap: &mut HashMap<String, (LLVMTypeRef, LLVMValueRef)>,
                    local_varmap: &mut HashMap<String, (LLVMTypeRef, LLVMValueRef)>) -> Result<LLVMValueRef, ParseError> {
    let ty = ty_str_to_ty(&ident_next!(tokens, "type"))?;
    let (value, loc) = if let Token { tt: TokenType::Literal(lit), loc} = tokens.this()? {
        (lit, loc)
    } else { panic!("literal value is not a literal value") };
    tokens.next();
    let v = unsafe {
        match value {
            Literal::String(s) => core::LLVMBuildGlobalString(*builder, c_str_ptr!(s), c_str_ptr!("")),
            Literal::Char(c) => unimplemented!(),
            Literal::Number(n, _) => match n {
                NumLit::Float(f) => unimplemented!(),
                NumLit::Integer(i) => core::LLVMConstInt(ty, i as c_ulonglong, 0)
            }
            Literal::Bool(b) => core::LLVMConstInt(core::LLVMInt1Type(), b as c_ulonglong, 0)
        }
    };
    Ok(v)
}

fn compile_let_assign(tokens: &mut TokIter, module: &prelude::LLVMModuleRef, builder: &prelude::LLVMBuilderRef,
                   varmap: &mut HashMap<String, (LLVMTypeRef, LLVMValueRef)>,
                   local_varmap: &mut HashMap<String, (LLVMTypeRef, LLVMValueRef)>) -> Result<(), ParseError> {
    let ty = ty_str_to_ty(&ident_next!(tokens, "type"))?;
    let name = ident_next!(tokens, "name");
    expect_ident!(tokens, "be");
    let v = compile_expression(tokens, module, builder, varmap, local_varmap, &name)?;
    local_varmap.insert(name, (ty, v));
    Ok(())
}
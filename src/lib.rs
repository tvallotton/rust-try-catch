//! This crate provides a macro that enables the familiar `try-catch` syntax of other programming languages.
//! It can be used to easlily group errors and manage them dynamically by type rather than value.
//! 
//! ```rust
//! use try_catch::catch;
//! use std::*;
//! use serde_json::Value;
//! fn own<T>(_: Vec<T>) {}
//! let x: Vec<i32> = vec![];
//! catch! {
//!     try {
//!         own(x);
//!         let data = fs::read_to_string("data.json")?;
//!         let json: Value = serde_json::from_str(&data)?;
//!     }
//!     catch error: io::Error {
//!         println!("Failed to open the file: {}", error)
//!     }
//!     catch json_err: serde_json::Error {
//!         println!("Failed to serialize data: {}", json_err)
//!     }
//! };
//! ```

mod prelude;

use crate::prelude::*;
use proc_macro2::Span;

use quote::ToTokens;
use syn::{parse::Parse, spanned::Spanned};

#[proc_macro]
pub fn catch(input: TokenStream) -> TokenStream {
    let try_catch = parse_macro_input!(input as TryCatch);

    template(try_catch).into()
}

struct TryCatch {
    try_block: ExprBlock,
    catches: Vec<Catch>,
    is_async: bool,
}

impl Parse for TryCatch {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let _try_kw: Token![try] = input.parse()?;
        let try_block = parse_block(&input)?;
        let ts = try_block.to_token_stream();
        let is_async = is_async(ts);
        eprintln!("IS ASYNC: {}", is_async);
        let mut catches = vec![];
        while let Ok(catch) = input.parse() {
            catches.push(catch)
        }
        Ok(TryCatch {
            try_block,
            catches,
            is_async,
        })
    }
}

struct Catch {
    error: Ident,
    err_type: Type,
    block: ExprBlock,
}
impl Parse for Catch {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let catch_kw: Ident = input.parse()?;
        let error: Ident = input.parse()?;
        let _colon: Token![:] = input.parse()?;
        let err_type: Type = input.parse()?;
        let block = parse_block(&input)?;
        if catch_kw != "catch" {
            return Err(Error::new(catch_kw.span(), "Expected `catch`."));
        }
        Ok(Catch {
            error,
            err_type,
            block,
        })
    }
}
use syn::ExprBlock;

fn parse_block(input: &parse::ParseStream) -> Result<ExprBlock> {
    let out = input.parse().map(|block| match block {
        Expr::Block(block) => Ok(block),
        span => Err(Error::new(span.span(), "Expected a block `{ /* ... */ }`.")),
    })??;
    Ok(out)
}

fn template(try_catch: TryCatch) -> TokenStream2 {
    let try_block = try_catch.try_block;
    let result = Ident::new("__try_catch_block", Span::mixed_site());
    let result_err = Ident::new("__try_catch_error", Span::mixed_site());

    let mut template = if try_catch.is_async {
        quote![
            let #result: ::std::result::Result<_, Box<dyn ::std::error::Error>> = (|| async {Ok(#try_block)})().await;
        ]
    } else {
        quote![
            let #result: ::std::result::Result<_, Box<dyn ::std::error::Error>> = (|| Ok(#try_block))();
        ]
    };

    let mut else_if = false;
    let mut catch_template = quote!();
    for catch in try_catch.catches {
        let err_type = catch.err_type;
        let block = catch.block;
        let error_name = catch.error;

        if else_if {
            catch_template.extend(quote![else])
        } else {
            else_if = true;
        }
        catch_template.extend(quote![
            if  #result_err.is::<#err_type>() {
                let #error_name = #result_err.downcast::<#err_type>().unwrap();
                ::std::result::Result::Ok(#block)
            }
        ]);
    }
    catch_template.extend(quote![
        else {
            ::std::result::Result::Err(#result_err)
        }
    ]);
    template.extend(quote![
        if let ::std::result::Result::Err(#result_err) = #result {
            #catch_template
        } else {
            #result
        }
    ]);
    quote!({#template})
}

fn is_async(input: TokenStream2) -> bool {
    let mut out = false;
    for token in input {
        match token {
            proc_macro2::TokenTree::Ident(ident) => {
                if ident == "await" {
                    out = true;
                }
            }
            proc_macro2::TokenTree::Group(group) => {
                out |= is_async(group.stream());
            }
            _ => (),
        }
    }
    out
}

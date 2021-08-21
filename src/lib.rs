//! This crate provides a macro that enables the familiar `try-catch` syntax of other programming languages.
//! It can be used to easlily group errors and manage them dynamically by type rather than value.
//!
//! ```rust
//! use try_catch::catch;
//! use std::*;
//! use serde_json::Value;
//!
//! catch! {
//!     try {
//!         let number: i32 = "10".parse()?;
//!         let data = fs::read_to_string("data.json")?;
//!         let json: Value = serde_json::from_str(&data)?;
//!     }
//!     catch error: io::Error {
//!         println!("Failed to open the file: {}", error)
//!     }
//!     catch json_err: serde_json::Error {
//!         println!("Failed to serialize data: {}", json_err)
//!     }
//!     catch err {
//!         println!("Error of unknown type: {}", err)
//!     }
//! };
//!
//! ```
//! Note, if no wildcard is present then the compiler will warn about unused results.
//! It can also be used as an expression:
//! ```rust
//! // We can guarantee that all errors are catched 
//! // so the type of this expression is `i32`.
//! // It can be guaranteed because the final catch 
//! // does not specify an Error type. 
//! let number: i32 = catch! {
//!     try {
//!         let number: i32 = "10".parse()?;
//!         number
//!     } catch error {
//!         0
//!     }
//! };
//! // we can't know for sure if all possible errors are 
//! // handled so the type of this expression 
//! // is still Result. 
//! let result: Result<i32, _> = catch! {
//!     try {
//!         let number: i32 = "invalid number".parse()?;
//!         number
//!     } catch error: io::Error {
//!         0
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
struct Catch {
    error: Ident,
    err_type: Option<Type>,
    block: ExprBlock,
}

fn parse_block(input: &parse::ParseStream) -> Result<ExprBlock> {
    let out = input.parse().map(|block| match block {
        Expr::Block(block) => Ok(block),
        span => Err(Error::new(span.span(), "Expected a block `{ /* ... */ }`.")),
    })??;
    Ok(out)
}

impl Parse for TryCatch {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let _try_kw: Token![try] = input.parse()?;
        let try_block = parse_block(&input)?;
        let ts = try_block.to_token_stream();
        let is_async = is_async(ts);
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

impl Parse for Catch {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let catch_kw: Ident = input.parse()?;
        if catch_kw != "catch" {
            return Err(Error::new(catch_kw.span(), "Expected `catch`"));
        }
        let error: Ident = input.parse()?;
        let err_type = if input.peek(Token![:]) {
            eprintln!("Yes colon\n\n\n\n");
            let _colon: Token![:] = input.parse()?;
            Some(input.parse()?)
        } else {
            eprintln!("No colon\n\n\n\n");
            None
        };
        let block = parse_block(&input)?;
        Ok(Catch {
            error,
            err_type,
            block,
        })
    }
}
use syn::ExprBlock;

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

    let mut catch_template = quote!();
    let mut warn_unused_must_use = true;
    for catch in try_catch.catches {
        let block = catch.block;
        let error_name = catch.error;
        if let Some(err_type) = catch.err_type {
            catch_template.extend(quote![
                _ if  #result_err.is::<#err_type>() => {
                    let #error_name = #result_err.downcast::<#err_type>().unwrap();
                    ::std::result::Result::Ok(#block)
                }
            ]);
        } else {
            warn_unused_must_use = false;
            catch_template.extend(quote![
                _ => {
                    let #error_name = #result_err;
                    ::std::result::Result::Ok(#block)
                }
            ]);
        }
    }

    catch_template.extend(quote![
        _ => {
            ::std::result::Result::Err(#result_err)
        }
    ]);

    template.extend(quote![
        if let ::std::result::Result::Err(#result_err) = #result {
           match () { #catch_template }
        } else {
            #result
        }
    ]);

    if warn_unused_must_use {
        quote!({#template})
    } else {
        quote!({#template.ok().unwrap()})
    }
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

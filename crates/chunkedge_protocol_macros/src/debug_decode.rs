use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Block, ImplItem, ImplItemFn, ItemImpl, Result, Type};

pub(super) fn debug_decode_impl(_args: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let mut item: ItemImpl = parse2(input)?;

    // Derive the display name from the Self type.
    let self_ty: &Type = &item.self_ty;
    let type_name_str = quote!(#self_ty).to_string();

    // Find and rewrite the `decode` method.
    for impl_item in &mut item.items {
        if let ImplItem::Fn(method) = impl_item {
            if method.sig.ident == "decode" {
                rewrite_decode_body(method, &type_name_str);
                break;
            }
        }
    }

    Ok(quote! { #item })
}

/// Replaces the body of a `decode` method with the instrumented version.
///
/// The original body is wrapped in a closure and called; success/failure
/// paths log to the debug runtime.
fn rewrite_decode_body(method: &mut ImplItemFn, type_name_str: &str) {
    let original_block: Block = method.block.clone();

    method.block = syn::parse_quote! {
        {
            #[allow(unexpected_cfgs)]
            {
            // Pull the reader out by value so we can capture the start
            // pointer before any bytes are consumed.
            #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
            let __dbg_start: &[u8] = *r;

            #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
            ::chunkedge_protocol::debug::log_field_start(
                None,
                #type_name_str,
            );

            let __res: ::anyhow::Result<Self> = {
                #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                let _guard = ::chunkedge_protocol::debug::IndentGuard::new();

                // Original decode body, re-executed as a block expression.
                (|| #original_block)()
            };

            match __res {
                Ok(__val) => {
                    #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                    ::chunkedge_protocol::debug::log_field_success(
                        #type_name_str,
                        &__val,
                        __dbg_start,
                        *r,
                    );
                    Ok(__val)
                }
                Err(__e) => {
                    #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                    ::chunkedge_protocol::debug::log_field_error(
                        &__e,
                        __dbg_start,
                    );
                    Err(__e)
                }
            }
            }
        }
    };
}

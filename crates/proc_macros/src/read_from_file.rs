use crate::util::get_crate_name_of;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, quote, quote_spanned};
use syn::parse::{Parse, ParseBuffer, ParseStream, Parser};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Expr, Meta};

pub fn derive_read_from_file_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let binary_crate = &get_crate_name_of("pure_lang_binary", Span::call_site());
    let global_crate = &get_crate_name_of("pure_lang_global", Span::call_site());
    let data = &input.data;
    let name = &input.ident;
    let (impl_g, ty_g, wh) = input.generics.split_for_impl();
    match data {
        Data::Struct(s) => {
            let idents = s
                .fields
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    if let Some(ref id) = f.ident {
                        id.to_token_stream()
                    } else {
                        syn::Index::from(i).to_token_stream()
                    }
                })
                .collect::<Vec<_>>();
            let per_ident_expr = idents.iter().map(|x| {
                let binary_crate = &get_crate_name_of("pure_lang_binary", x.span());
                let global_crate = &get_crate_name_of("pure_lang_global", x.span());
                quote_spanned! {
                    x.span() => #x: #binary_crate::traits::ReadFromFile::read_from_file(file)?,
                }
            });
            Ok(quote! {
                impl #impl_g #binary_crate::traits::ReadFromFile for #name #ty_g #wh {
                    fn read_from_file(
                        file: &mut #binary_crate::core::File,
                    ) -> #global_crate::Result<Self> {
                        Ok(Self {
                            #(
                                #per_ident_expr
                            )*
                        })
                    }
                }
            })
        }
        Data::Enum(e) => {
            let variants = &e.variants;
            if let Some(repr) = input.attrs.iter().find_map(|x| {
                if x.path().get_ident()? == "repr" {
                    Some(&x.meta)
                } else {
                    None
                }
            }) {
                let repr = repr.require_list()?.parse_args::<syn::Ident>()?;
                let idents = variants.iter().map(|x| &x.ident);
                let vars = variants.iter().enumerate().map(|(i, x)| {
                    x.discriminant
                        .clone()
                        .map(|x| x.1)
                        .unwrap_or(Parser::parse_str(Expr::parse, &format!("{i}")).unwrap())
                });
                return Ok(quote! {
                    impl #impl_g #binary_crate::traits::ReadFromFile for #name #ty_g #wh {
                        fn read_from_file(
                            file: &mut #binary_crate::core::File,
                        ) -> #global_crate::Result<Self> {
                            let __i = #repr::read_from_file(file)?;
                            match __i {
                                #(
                                    #vars => Ok(#name::#idents),
                                )*
                                _ => Err(
                                    #binary_crate::Error::EnumOutOfBounds(std::any::type_name::<Self>())
                                        .throw()
                                        .into(),
                                ),
                            }
                        }
                    }
                });
            }
            let attrs = input.attrs.iter().filter_map(|x| {
                if let Some(n) = x.meta.path().get_ident()
                    && n == "with_type"
                {
                    Some(x.meta.clone())
                } else {
                    None
                }
            });
            let mut repr = quote!();
            for meta in attrs {
                let list = meta.require_list()?;
                let inner_meta = list.parse_args::<Meta>()?;
                let name_value = inner_meta.require_name_value()?;
                let i = name_value.path.require_ident();
                if i.is_err() {
                    continue;
                }
                let i = i.unwrap();
                if i == "repr" {
                    let v = name_value.value.to_token_stream();
                    repr = quote!(#[repr(#v)]);
                }
            }
            let mut ts = TokenStream::new();
            let type_ident = Ident::new(&format!("{name}Type"), Span::call_site());
            for v in variants {
                let f_idents = v
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        if let Some(ref id) = f.ident {
                            id.to_token_stream()
                        } else {
                            syn::Index::from(i).to_token_stream()
                        }
                    })
                    .collect::<Vec<_>>();
                let v_name = &v.ident;
                ts.extend(quote! {
                    #type_ident::#v_name => Ok(#name::#v_name {
                        #(#f_idents: #binary_crate::traits::ReadFromFile::read_from_file(file)?,)*
                    }),
                });
            }
            Ok(quote! {
                impl #impl_g #binary_crate::traits::ReadFromFile for #name #ty_g #wh {
                    fn read_from_file(
                        file: &mut #binary_crate::core::File,
                    ) -> #global_crate::Result<Self> {
                        let x = #type_ident::read_from_file(file)?;
                        match x {
                            #ts
                        }
                    }
                }
            })
        }
        Data::Union(_) => Err(syn::Error::new(
            Span::call_site(),
            "Unions are not supported",
        )),
    }
}

struct ReadFromFileForeignInput {
    t: syn::Type,
    i: syn::LitInt,
}

impl Parse for ReadFromFileForeignInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let t = input.parse()?;
        let i = input.parse()?;
        Ok(Self { t, i })
    }
}

pub fn read_from_file_foreign_with_new_impl(input: TokenStream) -> syn::Result<TokenStream> {
    let parsed = Parser::parse2(ReadFromFileForeignInput::parse, input)?;
    let binary_crate = &get_crate_name_of("pure_lang_binary", Span::call_site());
    let global_crate = &get_crate_name_of("pure_lang_global", Span::call_site());
    let t = &parsed.t;
    let mut token_stream = TokenStream::new();
    for _ in 0..parsed.i.base10_parse::<u64>()? {
        token_stream.extend(quote! {#binary_crate::traits::ReadFromFile::read_from_file(file)?,});
    }
    Ok(quote! {
        impl #binary_crate::traits::ReadFromFile for #t {
            fn read_from_file(
                file: &mut #binary_crate::core::File,
            ) -> #global_crate::Result<Self> {
                Ok(Self::new(
                    #token_stream
                ))
            }
        }
    })
}

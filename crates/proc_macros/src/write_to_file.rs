use crate::util::get_crate_name_of;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, quote};
use syn::parse::{Parse, Parser};
use syn::{Data, DeriveInput, Expr, Meta};

pub fn derive_write_to_file_impl(input: DeriveInput) -> syn::Result<TokenStream> {
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
            Ok(quote! {
                impl #impl_g #binary_crate::traits::WriteToFile for #name #ty_g #wh {
                    fn write_to_file(
                        &self,
                        file: &mut #binary_crate::core::File,
                    ) -> #global_crate::Result<()> {
                        #(
                            #binary_crate::traits::WriteToFile::write_to_file(&self.#idents, file)?;
                        )*
                        Ok(())
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
                    impl #impl_g #binary_crate::traits::WriteToFile for #name #ty_g #wh {
                        fn write_to_file(
                            &self,
                            file: &mut #binary_crate::core::File,
                        ) -> #global_crate::Result<()> {
                            let __i = *self as #repr;
                            __i.write_to_file(file)
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
                let mut is_unnamed = false;
                let f_idents = v
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        if let Some(ref id) = f.ident {
                            id.to_token_stream()
                        } else {
                            is_unnamed = true;
                            syn::Index::from(i).to_token_stream()
                        }
                    })
                    .collect::<Vec<_>>();
                let out_idents = if is_unnamed {
                    f_idents
                        .iter()
                        .map(|x| {
                            let s = Ident::new(&format!("_{x}"), Span::call_site());
                            quote!(#s)
                        })
                        .collect::<Vec<_>>()
                } else {
                    f_idents.clone()
                };
                let v_name = &v.ident;
                let matcher = if is_unnamed {
                    quote!(#name::#v_name(
                        #(#out_idents),*
                    ))
                } else {
                    quote!(#name::#v_name {
                        #(#f_idents: #out_idents),*
                    })
                };

                ts.extend(quote! {
                    #matcher => {
                        #(
                            #binary_crate::traits::WriteToFile::write_to_file(#out_idents, file)?;
                        )*
                    }
                });
            }
            Ok(quote! {
                impl #impl_g #binary_crate::traits::WriteToFile for #name #ty_g #wh {
                    fn write_to_file(
                        &self,
                        file: &mut #binary_crate::core::File,
                    ) -> #global_crate::Result<()> {
                        self.to_type().write_to_file(file)?;
                        match self {
                            #ts
                        }
                        Ok(())
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

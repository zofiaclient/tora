use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{Fields, Type, Variant};

/// Generates a `FromReader` implementation for the given `ident`.
fn impl_from_reader(ident: &Ident, impl_tokens: TokenStream) -> TokenStream {
    quote! {
        impl tora::read::FromReader for #ident {
            fn from_reader<R>(mut r: R) -> std::io::Result<Self>
            where R: std::io::Read
            {
                #impl_tokens
            }
        }
    }
}

/// Generates a `SerializeIo` implementation for the given `ident`.
fn impl_serialize_io(ident: &Ident, impl_tokens: TokenStream) -> TokenStream {
    quote! {
        impl tora::write::SerializeIo for #ident {
            fn serialize<W>(&self, mut w: W) -> std::io::Result<()>
            where W: std::io::Write
            {
                #impl_tokens
            }
        }
    }
}

fn to_reads_field(ident: Option<&Ident>) -> TokenStream {
    match ident {
        Some(ident) => quote! { #ident: tora::read::ToraRead::reads(&mut r)? },
        None => quote! { tora::read::ToraRead::reads(&mut r)? },
    }
}

fn to_params<I, T>(it: I, fields: &Fields) -> TokenStream
where
    I: Iterator<Item = T>,
    T: ToTokens,
{
    match fields {
        Fields::Named(_) => quote!({ #( #it, )* }),
        Fields::Unnamed(_) => quote!(( #( #it, )* )),
        Fields::Unit => TokenStream::new(),
    }
}

fn to_variant_match(variant_id: usize, ident: &Ident, fields: &Fields) -> TokenStream {
    let field_iterator = fields.iter().map(|f| to_reads_field(f.ident.as_ref()));
    let construction_method = to_params(field_iterator, fields);

    quote! {
        #variant_id => Self::#ident #construction_method
    }
}

fn to_write_variant(variant_id: usize, id_ty: &Type, ident: Ident, fields: Fields) -> TokenStream {
    let params = fields.iter().enumerate().map(|(i, f)| {
        f.ident
            .clone()
            .unwrap_or_else(|| Ident::new(&format!("x{i}"), f.span()))
    });

    let vars = params.clone();
    let param_style = to_params(params, &fields);

    quote! {
        Self::#ident #param_style => {
            tora::write::ToraWrite::writes(&mut w, &(#variant_id as #id_ty))?;
            #( tora::write::ToraWrite::writes(&mut w, #vars)?; )*
        }
    }
}

/// `derive(ReadStruct)` implementation for named structs.
pub fn impl_read_struct_named<I>(ident: Ident, field_idents: I) -> TokenStream
where
    I: Iterator<Item = Ident>,
{
    let construction_method =
        quote! { Ok(Self { #( #field_idents: tora::read::ToraRead::reads(&mut r)?, )* }) };
    impl_from_reader(&ident, construction_method)
}

/// `derive(ReadStruct)` implementation for tuple structs.
pub fn impl_read_struct_tuple<I>(ident: Ident, types: I) -> TokenStream
where
    I: Iterator<Item = Type>,
{
    let construction_method =
        quote! { Ok(Self( #( tora::read::ToraRead::reads::<#types>(&mut r)?, )*)) };
    impl_from_reader(&ident, construction_method)
}

/// `derive(ReadEnum)` implementation.
pub fn impl_read_enum<I>(ident: Ident, ty: TokenStream, variants: I) -> TokenStream
where
    I: Iterator<Item = Variant>,
{
    let variants = variants
        .enumerate()
        .map(|(i, v)| to_variant_match(i, &v.ident, &v.fields));

    impl_from_reader(
        &ident,
        quote! {
            std::result::Result::Ok(match tora::read::ToraRead::reads::<#ty>(&mut r)? as usize {
                #( #variants, )*
                _ => return std::result::Result::Err(
                    std::io::Error::new(std::io::ErrorKind::InvalidInput,
                    format!("Invalid {} variant id", stringify!(#ident)))
                )
            })
        },
    )
}

/// `derive(WriteStruct)` implementation.
pub fn impl_write_struct<I>(ident: Ident, fields: I) -> TokenStream
where
    I: Iterator<Item = TokenStream>,
{
    impl_serialize_io(
        &ident,
        quote! {
            #( tora::write::ToraWrite::writes(&mut w, &self.#fields)?; )*
            std::result::Result::Ok(())
        },
    )
}

/// `derive(WriteEnum)` implementation.
pub fn impl_write_enum<I>(ident: Ident, id_ty: Type, variants: I) -> TokenStream
where
    I: Iterator<Item = Variant>,
{
    let variants = variants
        .enumerate()
        .map(|(i, v)| to_write_variant(i, &id_ty, v.ident, v.fields));

    impl_serialize_io(
        &ident,
        quote! {
            match self {
                #( #variants )*
            }
            Ok(())
        },
    )
}

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

fn struct_derive_macro(tokens: TokenStream) -> (Fields, Ident) {
    let ast: DeriveInput = syn::parse(tokens).unwrap();

    let fields = if let Data::Struct(ds) = ast.data {
        ds.fields
    } else {
        panic!("This trait can only be derived on structs")
    };
    (fields, ast.ident)
}

#[proc_macro_derive(FromReader)]
pub fn derive_from_reader(tokens: TokenStream) -> TokenStream {
    let (fields, name) = struct_derive_macro(tokens);

    let idents = fields.iter().map(|f| &f.ident);
    let tys = fields.iter().map(|f| &f.ty);

    let t = quote! {
        impl tora::read::FromReader for #name {
            fn from_reader<R>(r: &mut R) -> std::io::Result<Self>
            where
                R: std::io::Read,
            {
                std::io::Result::Ok(Self {
                  #(
                    #idents: tora::read::ToraRead::reads::<#tys>(r)?,
                  )*
                })
            }
        }
    };
    t.into()
}

#[proc_macro_derive(SerializeIo)]
pub fn derive_serialize_io(tokens: TokenStream) -> TokenStream {
    let (fields, name) = struct_derive_macro(tokens);

    let idents = fields.iter().map(|f| &f.ident);
    let tys = fields.iter().map(|f| &f.ty);

    let t = quote! {
        impl tora::write::SerializeIo for #name {
            fn serialize<W>(&self, w: &mut W) -> std::io::Result<()>
            where
                W: std::io::Write,
            {
                #(
                tora::write::ToraWrite::writes::<#tys>(w, &self.#idents)?;
                )*
                std::io::Result::Ok(())
            }
        }
    };
    t.into()
}

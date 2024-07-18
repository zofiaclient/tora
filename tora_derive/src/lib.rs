use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse::Parse;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Attribute, Error, Fields, ItemEnum, ItemStruct, LitInt, Type,
};

mod derive_impl;

fn get_list_attr_or_default<T>(key: &str, default: T, attributes: &[Attribute]) -> T
where
    T: Parse,
{
    for attribute in attributes {
        if attribute.meta.path().is_ident(key) {
            return attribute
                .meta
                .require_list()
                .unwrap()
                .parse_args::<T>()
                .unwrap();
        }
    }
    default
}

fn derive_empty_item_error<T>(tokens: T) -> TokenStream
where
    T: ToTokens,
{
    Error::new_spanned(tokens, "This macro cannot be derived on empty items")
        .into_compile_error()
        .into()
}

/// The `ReadEnum` macro generates a `FromReader` implementation for enums.
///
/// For structs, use [ReadStruct].
///
/// # Attributes
///
/// ## `type_variant_id($ty)`
///
/// This attribute tells the macro what type represents the enum variant ID.
///
/// The enum variant ID is the number written to notify the reader of the variant they should expect
/// to receive.
///
/// ```
/// use tora_derive::ReadEnum;
///
/// #[derive(ReadEnum)]
/// enum Packet {
///     PlayerJoin, // 0
///     PlayerQuit // 1
/// }
/// ```
///
/// By default, this macro assumes [u8].
///
/// In the case that the enum deriving this macro contains more than [u8::MAX] variants, the user
/// will be required to specify this attribute manually.
///
/// # Usage
///
/// ```
/// use tora_derive::ReadEnum;
///
/// #[derive(ReadEnum)]
/// #[type_variant_id(u32)]
/// enum Packet {
///     Variant1,
///     Variant2,
/// }
/// ```
///
/// # Generated code
///
/// ```
/// use std::io;
/// use std::io::{ErrorKind, Read};
///
/// use tora::read::{ToraRead, FromReader};
///
/// enum Packet {
///     Variant1,
///     Variant2,
/// }
///
/// impl FromReader for Packet {
///     fn from_reader<R>(r: &mut R) -> io::Result<Self>
///     where R: Read
///     {
///         let id = r.reads::<u32>()?;
///         Ok(match id {
///             0 => Self::Variant1,
///             1 => Self::Variant2,
///             _ => return Err(io::Error::new(ErrorKind::InvalidInput, "Invalid packet ID"))
///         })
///     }
/// }
/// ```
#[proc_macro_derive(ReadEnum, attributes(type_variant_id))]
pub fn derive_read_enum(tokens: TokenStream) -> TokenStream {
    let item = parse_macro_input!(tokens as ItemEnum);

    if item.variants.is_empty() {
        return derive_empty_item_error(item);
    }

    let path = get_list_attr_or_default("type_variant_id", parse_quote!(u8), &item.attrs);
    derive_impl::impl_read_enum(item.ident, path, item.variants.into_iter()).into()
}

/// The `ReadStruct` derive macro generates a `FromReader` implementation for structs.
///
/// For enums, use [ReadEnum].
///
/// # Usage
///
/// ```
/// use tora_derive::ReadStruct;
///
/// #[derive(ReadStruct)]
/// struct Packet {
///     message: String,
/// }
/// ```
///
/// # Generated code
///
/// ```
/// use std::io;
/// use std::io::Read;
///
/// use tora::read::{ToraRead, FromReader};
///
/// struct Packet {
///     message: String,
/// }
///
/// impl FromReader for Packet {
///     fn from_reader<R>(r: &mut R) -> io::Result<Self>
///     where R: Read
///     {
///         Ok(Self { message: r.reads()? })
///     }
/// }
/// ```
#[proc_macro_derive(ReadStruct)]
pub fn derive_read_struct(tokens: TokenStream) -> TokenStream {
    let item = parse_macro_input!(tokens as ItemStruct);

    if item.fields.is_empty() {
        return derive_empty_item_error(item);
    }

    match item.fields {
        Fields::Named(f) => derive_impl::impl_read_struct_named(
            item.ident,
            f.named.into_iter().map(|f| f.ident.unwrap()),
        ),
        Fields::Unnamed(f) => {
            derive_impl::impl_read_struct_tuple(item.ident, f.unnamed.into_iter().map(|f| f.ty))
        }
        Fields::Unit => return derive_empty_item_error(item),
    }
    .into()
}

/// The `WriteStruct` derive macro generates a `SerializeIo` implementation for structs.
///
/// # Usage
///
/// ```
/// use tora_derive::WriteStruct;
///
/// #[derive(WriteStruct)]
/// struct Packet {
///     message: String,
/// }
/// ```
///
/// # Generated code
///
/// ```
/// use std::io;
/// use std::io::Write;
///
/// use tora::write::{ToraWrite, SerializeIo};
///
/// struct Packet {
///     message: String,
/// }
///
/// impl SerializeIo for Packet {
///     fn serialize<W>(&self, w: &mut W) -> io::Result<()>
///     where W: Write
///     {
///         w.writes(&self.message)
///     }
/// }
/// ```
#[proc_macro_derive(WriteStruct)]
pub fn derive_write_struct(tokens: TokenStream) -> TokenStream {
    let item = parse_macro_input!(tokens as ItemStruct);

    if item.fields.is_empty() {
        return derive_empty_item_error(item);
    }
    let types = item.fields.into_iter().enumerate().map(|(i, f)| {
        f.ident
            .as_ref()
            .map(|i| i.to_token_stream())
            .unwrap_or_else(|| LitInt::new(&i.to_string(), f.span()).to_token_stream())
    });
    derive_impl::impl_write_struct(item.ident, types).into()
}

/// The `WriteEnum` derive macro generates a `SerializeIo` implementation for enums.
///
/// Opposite of the `ReadEnum` macro.
///
/// # Attributes
///
/// ## `type_variant_id($ty)`
///
/// This attribute tells the macro what type represents the enum variant ID.
///
/// The enum variant ID is the number written to notify the reader of the variant they should expect
/// to receive.
///
/// ```
/// use tora_derive::WriteEnum;
///
/// #[derive(WriteEnum)]
/// enum Packet {
///     PlayerJoin, // 0
///     PlayerQuit // 1
/// }
/// ```
///
/// By default, this macro assumes [u8].
///
/// In the case that the enum deriving this macro contains more than [u8::MAX] variants, the user
/// will be required to specify this attribute manually.
#[proc_macro_derive(WriteEnum, attributes(type_variant_id))]
pub fn derive_write_enum(tokens: TokenStream) -> TokenStream {
    let item = parse_macro_input!(tokens as ItemEnum);

    if item.variants.is_empty() {
        return derive_empty_item_error(item);
    }

    let ty: Type = get_list_attr_or_default("type_variant_id", parse_quote!(u8), &item.attrs);
    derive_impl::impl_write_enum(item.ident, ty, item.variants.into_iter()).into()
}

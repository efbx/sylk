//! The `#[sylk::collect]` expansion.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Error, ItemTrait, LitInt, Token,
    parse::{ParseStream, Parser},
};

/// Expands `#[sylk::collect]` and `#[sylk::collect(N)]` on a trait into the
/// `Installer` that collects `dyn Trait` at startup and, with a count, the
/// `Cardinality` that `Resources::collect` checks.
pub struct Collect;

impl Collect {
    pub fn attribute(
        attribute: proc_macro::TokenStream,
        item: proc_macro::TokenStream,
    ) -> proc_macro::TokenStream {
        let generated = syn::parse::<ItemTrait>(item.clone())
            .map_err(|error| {
                Error::new(
                    error.span(),
                    "#[sylk::collect] goes on a trait: `pub trait Name: Send + Sync { .. }`",
                )
            })
            .and_then(|declaration| Self::expand(attribute.into(), &declaration))
            .unwrap_or_else(|error| error.to_compile_error());
        let mut tokens = item;
        tokens.extend(proc_macro::TokenStream::from(generated));
        tokens
    }

    /// The collector `#[sylk::register]` submits, so `Seq<dyn Trait>` exists,
    /// empty, when nothing registers; with `N`, also a record of the exact
    /// number of implementations.
    ///
    /// Returns a compile error for a non-integer, repeated or invalid count or
    /// a generic trait.
    fn expand(attribute: TokenStream, declaration: &ItemTrait) -> Result<TokenStream, Error> {
        let count = Self::count.parse2(attribute)?;
        if !declaration.generics.params.is_empty() {
            return Err(Error::new_spanned(
                &declaration.generics,
                "a #[sylk::collect] trait is not generic",
            ));
        }
        let name = &declaration.ident;
        let cardinality = count.map(|count| {
            quote! {
                ::sylk::__private::inventory::submit! {
                    ::sylk::__private::Cardinality::new(
                        ::sylk::__private::Need::of::<dyn #name>(),
                        #count,
                    )
                }
            }
        });
        Ok(quote! {
            ::sylk::__private::inventory::submit! {
                ::sylk::__private::Installer::new(
                    ::sylk::__private::Need::of::<::sylk::Seq<dyn #name>>(),
                    ::sylk::Resources::collect::<dyn #name>,
                )
            }

            #cardinality
        })
    }

    /// Parses the optional count, an integer literal that fits a `usize`,
    /// rejecting anything else and a second argument.
    fn count(input: ParseStream<'_>) -> Result<Option<usize>, Error> {
        if input.is_empty() {
            return Ok(None);
        }
        let count = input
            .parse::<LitInt>()
            .map_err(|error| {
                Error::new(
                    error.span(),
                    "a #[sylk::collect] count is an integer: `#[sylk::collect(1)]`",
                )
            })?
            .base10_parse::<usize>()?;
        input.parse::<Option<Token![,]>>()?;
        if !input.is_empty() {
            return Err(input.error("a #[sylk::collect] attribute takes one count"));
        }
        Ok(Some(count))
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::{ItemTrait, parse2};

    use super::Collect;

    struct Expansion;

    impl Expansion {
        fn of(attribute: TokenStream, declaration: TokenStream) -> Result<String, String> {
            let declaration = parse2::<ItemTrait>(declaration).unwrap();
            Collect::expand(attribute, &declaration)
                .map(|tokens| tokens.to_string())
                .map_err(|error| error.to_string())
        }
    }

    #[test]
    fn a_bare_attribute_submits_only_the_collector() {
        let tokens = Expansion::of(quote! {}, quote! { trait Port: Send + Sync {} }).unwrap();
        assert!(tokens.contains("Installer"));
        assert!(!tokens.contains("Cardinality"));
    }

    #[test]
    fn a_count_submits_the_collector_and_the_cardinality() {
        let tokens = Expansion::of(quote! { 2 }, quote! { trait Port: Send + Sync {} }).unwrap();
        assert!(tokens.contains("Installer"));
        assert!(tokens.contains("Cardinality"));
    }

    #[test]
    fn rejects_a_count_that_is_not_an_integer() {
        for attribute in [quote! { one }, quote! { "1" }, quote! { count = 1 }] {
            assert_eq!(
                Expansion::of(attribute, quote! { trait Port: Send + Sync {} }).unwrap_err(),
                "a #[sylk::collect] count is an integer: `#[sylk::collect(1)]`"
            );
        }
    }

    #[test]
    fn rejects_a_negative_count() {
        assert!(Expansion::of(quote! { -1 }, quote! { trait Port: Send + Sync {} }).is_err());
    }

    #[test]
    fn rejects_a_second_count() {
        assert_eq!(
            Expansion::of(quote! { 1, 2 }, quote! { trait Port: Send + Sync {} }).unwrap_err(),
            "a #[sylk::collect] attribute takes one count"
        );
    }

    #[test]
    fn rejects_a_generic_trait() {
        assert_eq!(
            Expansion::of(quote! {}, quote! { trait Port<T>: Send + Sync {} }).unwrap_err(),
            "a #[sylk::collect] trait is not generic"
        );
    }
}

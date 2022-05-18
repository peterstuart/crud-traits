use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemStruct};

#[derive(Debug, FromMeta)]
struct BelongsToOptions {
    parent: Ident,
    table: String,
}

/// Annotate a struct with `belongs_to` to generate an implementation
/// of [`BelongsTo`](crud_trait_macros::BelongsTo).
#[proc_macro_attribute]
pub fn belongs_to(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemStruct);

    let options = match BelongsToOptions::from_list(&attr_args) {
        Ok(value) => value,
        Err(error) => {
            return TokenStream::from(error.write_errors());
        }
    };

    let child = input.ident.clone();
    let parent = options.parent;
    let table = options.table;

    let parent_id_field = Ident::new(
        &format!("{}_id", parent.to_string().to_lowercase()),
        parent.span(),
    );
    let query = format!("SELECT * FROM {table} WHERE {parent_id_field} = ANY($1)");

    let result = quote! {
        #input

        #[async_trait::async_trait]
        impl crud_traits::BelongsTo<#parent> for #child {
            fn parent_id(&self) -> <#parent as crud_traits::Meta>::Id {
                self.#parent_id_field
            }

            async fn for_parent_ids(
                ids: &[<#parent as crud_traits::Meta>::Id],
                store: &<#parent as crud_traits::Meta>::Store
            ) -> Result<std::collections::HashMap<<#parent as crud_traits::Meta>::Id, Vec<Self>>, <Self as crud_traits::Meta>::Error> {
                let values = sqlx::query_as::<_, Self>(#query)
                    .bind(ids)
                    .fetch_all(store)
                    .await?;

                let mut hash_map = std::collections::HashMap::new();

                for value in values {
                    let parent_id = crud_traits::BelongsTo::<#parent>::parent_id(&value);
                    let mut children: Vec<Self> = hash_map.remove(&parent_id).unwrap_or_default();
                    children.push(value);
                    hash_map.insert(parent_id, children);
                }

                Ok(hash_map)
            }
        }
    };

    result.into()
}

#[derive(Debug, FromMeta)]
struct HasManyOptions {
    child: Ident,
}

/// Annotate a struct with `has_many` to generate an implementation
/// of [`HasMany`](crud_trait_macros::HasMany).
#[proc_macro_attribute]
pub fn has_many(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemStruct);

    let options = match HasManyOptions::from_list(&attr_args) {
        Ok(value) => value,
        Err(error) => {
            return TokenStream::from(error.write_errors());
        }
    };

    let parent = input.ident.clone();
    let child = options.child;

    let result = quote! {
        #input

        #[async_trait::async_trait]
        impl crud_traits::HasMany<#child> for #parent {}
    };

    result.into()
}

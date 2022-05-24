use darling::{FromDeriveInput, FromMeta};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, DeriveInput, ItemStruct};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(crud))]
struct CrudOptions {
    table: String,
}

#[proc_macro_derive(Read, attributes(crud))]
pub fn sqlx_read(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let options = CrudOptions::from_derive_input(&input).expect("Invalid options");
    let DeriveInput { ident, .. } = input;

    let query_one = format!("SELECT * FROM {} WHERE id = $1", options.table);
    let query_many = format!("SELECT * FROM {} WHERE id = ANY($1)", options.table);

    quote! {
        #[async_trait::async_trait]
        impl crud_traits::Read for #ident {
            async fn read(
                id: <Self as crud_traits::Meta>::Id,
                store: &<Self as crud_traits::Meta>::Store
            ) -> std::result::Result<Self, sqlx::Error> {
                Ok(
                    sqlx::query_as!(Self, #query_one, id)
                        .fetch_one(store)
                        .await?,
                )
            }

            async fn maybe_read(
                id: <Self as crud_traits::Meta>::Id,
                store: &<Self as crud_traits::Meta>::Store
            ) -> std::result::Result<std::option::Option<Self>, sqlx::Error> {
                Ok(
                    sqlx::query_as!(Self, #query_one, id)
                        .fetch_optional(store)
                        .await?,
                )
            }

            async fn read_many(
                ids: &[<Self as crud_traits::Meta>::Id],
                store: &<Self as crud_traits::Meta>::Store
            ) -> std::result::Result<Vec<Self>, sqlx::Error> {
                Ok(
                    sqlx::query_as!(Self, #query_many, ids)
                        .fetch_all(store)
                        .await?,
                )
            }

        }
    }
    .into()
}

#[proc_macro_derive(Delete, attributes(crud))]
pub fn sqlx_delete(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let options = CrudOptions::from_derive_input(&input).expect("Invalid options");
    let DeriveInput { ident, .. } = input;

    let query = format!("DELETE FROM {} WHERE id = $1", options.table);

    quote! {
        #[async_trait::async_trait]
        impl crud_traits::Delete for #ident {
            async fn delete_by_id(
                id: <Self as crud_traits::Meta>::Id,
                store: &<Self as crud_traits::Meta>::Store
            ) -> std::result::Result<(), sqlx::Error> {
                sqlx::query_as!(Self, #query, id)
                    .execute(store)
                    .await?;
                Ok(())
            }
        }
    }
    .into()
}

fn alias_from_ident(ident: &Ident) -> String {
    ident.to_string().to_lowercase()
}

fn plural_alias_from_alias(alias: &str) -> String {
    format!("{alias}s")
}

#[derive(Debug, FromMeta)]
struct BelongsToOptions {
    parent: Ident,
    table: String,
    alias: Option<String>,
    plural_alias: Option<String>,
}

/// Annotate a struct with `belongs_to` to generate an implementation
/// of `BelongsTo`.
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
    let alias = options.alias.unwrap_or_else(|| alias_from_ident(&parent));
    let plural_alias = options
        .plural_alias
        .unwrap_or_else(|| plural_alias_from_alias(&alias));

    let parent_id_field = Ident::new(
        &format!("{}_id", parent.to_string().to_lowercase()),
        parent.span(),
    );
    let query = format!("SELECT * FROM {table} WHERE {parent_id_field} = ANY($1)");

    let parent_id_alias = Ident::new(&format!("{}_id", alias), Span::call_site());
    let for_parent_ids_alias = Ident::new(&format!("for_{}_ids", alias), Span::call_site());
    let for_parent_alias = Ident::new(&format!("for_{}", alias), Span::call_site());
    let for_parents_alias = Ident::new(&format!("for_{}", plural_alias), Span::call_site());
    let parent_alias = Ident::new(&alias, Span::call_site());
    let parents_for_many_alias =
        Ident::new(&format!("{}_for_many", plural_alias), Span::call_site());

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
            ) -> std::result::Result<std::collections::HashMap<<#parent as crud_traits::Meta>::Id, Vec<Self>>, <Self as crud_traits::Meta>::Error> {
                let values = sqlx::query_as::<_, Self>(#query)
                    .bind(ids)
                    .fetch_all(store)
                    .await?;

                let mut hash_map = std::collections::HashMap::new();

                for value in values {
                    let parent_id = crud_traits::BelongsTo::<#parent>::parent_id(&value);
                    hash_map.entry(parent_id).or_insert(Vec::new()).push(value);
                }

                Ok(hash_map)
            }
        }

        impl #child {
            fn #parent_id_alias(&self) -> <#parent as crud_traits::Meta>::Id {
                <Self as crud_traits::BelongsTo<#parent>>::parent_id(self)
            }

            pub async fn #for_parent_ids_alias(
                ids: &[<#parent as crud_traits::Meta>::Id],
                store: &<Self as crud_traits::Meta>::Store,
            ) -> std::result::Result<
                    std::collections::HashMap<<#parent as crud_traits::Meta>::Id, Vec<Self>>,
                <Self as crud_traits::Meta>::Error,
                > {
                <Self as crud_traits::BelongsTo<#parent>>::for_parent_ids(ids, store).await
            }

            pub async fn #for_parent_alias<T>(
                parent: &T,
                store: &<Self as crud_traits::Meta>::Store,
            ) -> std::result::Result<Vec<Self>, <Self as crud_traits::Meta>::Error>
            where
                T: crud_traits::AsId<Id = <#parent as crud_traits::Meta>::Id> + Send + Sync,
            {
                <Self as crud_traits::BelongsTo<#parent>>::for_parent(parent, store).await
            }

            pub async fn #for_parents_alias<'a, T, I>(
                parents: I,
                store: &<Self as crud_traits::Meta>::Store,
            ) -> std::result::Result<
                    std::collections::HashMap<<#parent as crud_traits::Meta>::Id, Vec<Self>>,
                <Self as crud_traits::Meta>::Error,
                >
            where
                T: 'a + crud_traits::AsId<Id = <#parent as crud_traits::Meta>::Id> + Send + Sync,
                I: std::iter::IntoIterator<Item = &'a T> + Send + Sync,
            {
                <Self as crud_traits::BelongsTo<#parent>>::for_parents(parents, store).await
            }

            pub async fn #parent_alias(
                &self,
                store: &<#parent as crud_traits::Meta>::Store,
            ) -> std::result::Result<#parent, <#parent as crud_traits::Meta>::Error> {
                <Self as crud_traits::BelongsTo<#parent>>::parent(&self, store).await
            }

            pub async fn #parents_for_many_alias<'a, I>(
                values: I,
                store: &<#parent as crud_traits::Meta>::Store,
            ) -> std::result::Result<
                    std::collections::HashMap<<Self as crud_traits::Meta>::Id, #parent>,
                <#parent as crud_traits::Meta>::Error,
                >
            where
                Self:'a,
                I: IntoIterator<Item = &'a Self> + Send + Sync,{
                <Self as crud_traits::BelongsTo<#parent>>::parents_for_many(values, store).await
            }
        }
    };

    result.into()
}

#[derive(Debug, FromMeta)]
struct HasManyOptions {
    child: Ident,
    alias: Option<String>,
    plural_alias: Option<String>,
}

/// Annotate a struct with `has_many` to generate an implementation
/// of `HasMany`.
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

    let alias = options.alias.unwrap_or_else(|| alias_from_ident(&child));
    let plural_alias = options
        .plural_alias
        .unwrap_or_else(|| plural_alias_from_alias(&alias));

    let children_alias = Ident::new(&plural_alias, Span::call_site());

    let result = quote! {
        #input

        #[async_trait::async_trait]
        impl crud_traits::HasMany<#child> for #parent {}

        impl #parent {
            pub async fn #children_alias(&self, store: &<Self as crud_traits::Meta>::Store) -> std::result::Result<Vec<#child>, <#child as crud_traits::Meta>::Error> {
                <Self as crud_traits::HasMany<#child>>::children(&self, store).await
            }
        }
    };

    result.into()
}

#[derive(Debug, FromMeta)]
struct HasOneOptions {
    child: Ident,
    alias: Option<String>,
}

/// Annotate a struct with `has_one` to generate an implementation of
/// `HasOne`.
#[proc_macro_attribute]
pub fn has_one(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemStruct);

    let options = match HasOneOptions::from_list(&attr_args) {
        Ok(value) => value,
        Err(error) => {
            return TokenStream::from(error.write_errors());
        }
    };

    let parent = input.ident.clone();
    let child = options.child;

    let alias = options.alias.unwrap_or_else(|| alias_from_ident(&child));

    let child_alias = Ident::new(&alias, Span::call_site());

    let result = quote! {
        #input

        #[async_trait::async_trait]
        impl crud_traits::HasOne<#child> for #parent {}

        impl #parent {
            pub async fn #child_alias(&self, store: &<Self as crud_traits::Meta>::Store) -> std::result::Result<std::option::Option<#child>, <#child as crud_traits::Meta>::Error> {
                <Self as crud_traits::HasOne<#child>>::child(&self, store).await
            }
        }
    };

    result.into()
}

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
            ) -> Result<Vec<Self>, <Self as crud_traits::Meta>::Error> {
                sqlx::query_as::<_, Dog>(#query)
                    .bind(ids)
                    .fetch_all(store)
                    .await
            }
        }
    };

    result.into()
}

#[derive(Debug, FromMeta)]
struct HasManyOptions {
    child: Ident,
}

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
        impl HasMany<#child> for #parent {}
    };

    result.into()
}

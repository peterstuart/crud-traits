use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, LitStr, Token,
};

struct BelongsToOptions {
    child: Ident,
    parent: Ident,
    table: LitStr,
    function_name: Ident,
}

impl Parse for BelongsToOptions {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let child = input.parse()?;
        input.parse::<Token![,]>()?;
        let parent = input.parse()?;
        input.parse::<Token![,]>()?;
        let table = input.parse()?;
        input.parse::<Token![,]>()?;
        let function_name = input.parse()?;

        Ok(Self {
            child,
            parent,
            table,
            function_name,
        })
    }
}

#[proc_macro]
pub fn belongs_to(input: TokenStream) -> TokenStream {
    let options = parse_macro_input!(input as BelongsToOptions);

    let child = options.child;
    let table = options.table.value();
    let parent = options.parent;
    let function_name = options.function_name;

    let parent_id_field = Ident::new(
        &format!("{}_id", parent.to_string().to_lowercase()),
        parent.span(),
    );
    let query = format!("SELECT * FROM {table} WHERE {parent_id_field} = ANY($1)");

    let result = quote! {
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

        impl #child {
            pub async fn #function_name(
                &self,
                store: &<#parent as crud_traits::Meta>::Store
            ) -> Result<#parent, <Self as crud_traits::Meta>::Error> {
                self.parent(store).await
            }
        }
    };

    result.into()
}

struct HasManyOptions {
    parent: Ident,
    child: Ident,
}

impl Parse for HasManyOptions {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let parent = input.parse()?;
        input.parse::<Token![,]>()?;
        let child = input.parse()?;

        Ok(Self { parent, child })
    }
}

#[proc_macro]
pub fn has_many(input: TokenStream) -> TokenStream {
    let options = parse_macro_input!(input as HasManyOptions);

    let parent = options.parent;
    let child = options.child;

    let result = quote! {
        #[async_trait::async_trait]
        impl HasMany<#child> for #parent {}
    };

    result.into()
}

//! SeaOrmResource derive macro for automatic REST API resource trait implementation.
//!
//! This crate provides the [`SeaOrmResource`](macro@SeaOrmResource) derive macro that automatically
//! implements resource metadata traits for sea-orm entities by extracting the table name.
//!
//! # Examples
//!
//! Basic usage - extracts table_name from sea_orm attribute:
//!
//! ```ignore
//! use sea_orm::entity::prelude::*;
//! use proc_macros::SeaOrmResource;
//!
//! #[derive(Clone, Debug, DeriveEntityModel, SeaOrmResource)]
//! #[sea_orm(table_name = "projects")]
//! pub struct Model {
//!     #[sea_orm(primary_key)]
//!     pub id: Uuid,
//!     pub title: String,
//! }
//!
//! // Auto-generated constants (using table_name):
//! assert_eq!(Model::COLLECTION, "projects");
//! assert_eq!(Model::URL, "/projects");
//! assert_eq!(Model::URL_WITH_ID, "/projects/{id}");
//! assert_eq!(Model::TAG, "Projects");
//! ```
//!
//! Customizing resource configuration:
//!
//! ```ignore
//! use sea_orm::entity::prelude::*;
//! use proc_macros::SeaOrmResource;
//!
//! #[derive(Clone, Debug, DeriveEntityModel, SeaOrmResource)]
//! #[sea_orm(table_name = "projects")]
//! #[sea_orm_resource(
//!     url = "/api/v1/projects",
//!     tag = "Project Management"
//! )]
//! pub struct Model {
//!     #[sea_orm(primary_key)]
//!     pub id: Uuid,
//! }
//!
//! assert_eq!(Model::COLLECTION, "projects");
//! assert_eq!(Model::URL, "/api/v1/projects");
//! assert_eq!(Model::URL_WITH_ID, "/api/v1/projects/{id}");
//! assert_eq!(Model::TAG, "Project Management");
//! ```

extern crate proc_macro;

use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Lit, Meta};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(sea_orm_resource), forward_attrs(sea_orm))]
struct SeaOrmResourceInput {
    ident: syn::Ident,
    attrs: Vec<syn::Attribute>,
    #[darling(default)]
    collection: Option<String>,
    #[darling(default)]
    url: Option<String>,
    #[darling(default)]
    tag: Option<String>,
}

/// Derives the `ApiResource` trait implementation for sea-orm entities.
///
/// This macro extracts the `table_name` from the `#[sea_orm(table_name = "...")]` attribute
/// and generates REST API resource constants.
///
/// # Attributes
///
/// - `collection`: Override the collection name (default: table_name from sea_orm)
/// - `url`: Override the default URL path (default: `/table_name`)
/// - `tag`: Override the default API tag (default: capitalized table_name)
///
/// # Generated Constants
///
/// - `URL`: The base URL path for this resource (plural, e.g., "/projects")
/// - `URL_WITH_ID`: The URL path with an `{id}` parameter appended
/// - `COLLECTION`: The database collection or table name
/// - `TAG`: The API documentation tag
///
/// # Requirements
///
/// The struct must have a `#[sea_orm(table_name = "...")]` attribute.
///
/// # Examples
///
/// ```ignore
/// #[derive(DeriveEntityModel, SeaOrmResource)]
/// #[sea_orm(table_name = "users")]
/// pub struct Model {
///     #[sea_orm(primary_key)]
///     pub id: Uuid,
///     pub email: String,
/// }
///
/// assert_eq!(Model::COLLECTION, "users");
/// assert_eq!(Model::URL, "/users");
/// assert_eq!(Model::URL_WITH_ID, "/users/{id}");
/// assert_eq!(Model::TAG, "Users");
/// ```
#[proc_macro_derive(SeaOrmResource, attributes(sea_orm_resource))]
pub fn sea_orm_resource_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse_macro_input!(input as DeriveInput);
    let receiver = match SeaOrmResourceInput::from_derive_input(&ast) {
        Ok(receiver) => receiver,
        Err(err) => return TokenStream::from(err.write_errors()),
    };

    match impl_sea_orm_resource(receiver) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn capitalize_first_letter(input: &str) -> String {
    if input.is_empty() {
        return input.to_owned();
    }

    input
        .char_indices()
        .fold(String::with_capacity(input.len()), |mut acc, (i, c)| {
            if i == 0 {
                acc.push_str(&c.to_uppercase().to_string());
            } else {
                acc.push(c);
            }
            acc
        })
}

fn extract_table_name(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("sea_orm") {
            if let Meta::List(meta_list) = &attr.meta {
                let mut table_name = None;
                let _ = meta_list.parse_nested_meta(|meta| {
                    if meta.path.is_ident("table_name") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(lit_str) = lit {
                            table_name = Some(lit_str.value());
                        }
                    }
                    Ok(())
                });
                if table_name.is_some() {
                    return table_name;
                }
            }
        }
    }
    None
}

fn impl_sea_orm_resource(receiver: SeaOrmResourceInput) -> syn::Result<proc_macro2::TokenStream> {
    let ident = &receiver.ident;

    // Extract table_name from #[sea_orm(table_name = "...")]
    let table_name = extract_table_name(&receiver.attrs).ok_or_else(|| {
        syn::Error::new_spanned(
            ident,
            "SeaOrmResource requires #[sea_orm(table_name = \"...\")] attribute",
        )
    })?;

    // Generate defaults based on table_name
    let collection = receiver.collection.unwrap_or_else(|| table_name.clone());

    // URL uses plural (table_name is already plural by convention)
    let url = receiver
        .url
        .unwrap_or_else(|| format!("/api/{}", table_name));

    let tag = receiver
        .tag
        .unwrap_or_else(|| capitalize_first_letter(&collection));

    let url_with_id = format!("{}/{{id}}", url);

    Ok(quote! {
        impl proc_macros::ApiResource for #ident {
            const URL: &'static str = #url;
            const URL_WITH_ID: &'static str = #url_with_id;
            const COLLECTION: &'static str = #collection;
            const TAG: &'static str = #tag;
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_extract_table_name() {
        let input = quote! {
            #[sea_orm(table_name = "projects")]
            pub struct Model {
                id: String,
            }
        };

        let ast: DeriveInput = syn::parse2(input).unwrap();
        let table_name = extract_table_name(&ast.attrs);
        assert_eq!(table_name, Some("projects".to_string()));
    }

    #[test]
    fn test_extract_table_name_with_other_attrs() {
        let input = quote! {
            #[derive(Clone, Debug)]
            #[sea_orm(table_name = "users")]
            pub struct Model {
                id: String,
            }
        };

        let ast: DeriveInput = syn::parse2(input).unwrap();
        let table_name = extract_table_name(&ast.attrs);
        assert_eq!(table_name, Some("users".to_string()));
    }

    #[test]
    fn test_basic_struct() {
        let input = quote! {
            #[sea_orm(table_name = "projects")]
            pub struct Model {
                id: String,
                title: String,
            }
        };

        let ast: DeriveInput = syn::parse2(input).unwrap();
        let receiver = SeaOrmResourceInput::from_derive_input(&ast).unwrap();
        let output = impl_sea_orm_resource(receiver).unwrap();
        let output_str = output.to_string();

        assert!(output_str.contains("impl proc_macros :: ApiResource for Model"));
        assert!(output_str.contains(r#"const COLLECTION : & 'static str = "projects""#));
        assert!(output_str.contains(r#"const URL : & 'static str = "/api/projects""#));
        assert!(output_str.contains(r#"const URL_WITH_ID : & 'static str = "/api/projects/{id}""#));
        assert!(output_str.contains(r#"const TAG : & 'static str = "Projects""#));
    }

    #[test]
    fn test_custom_url() {
        let input = quote! {
            #[sea_orm(table_name = "projects")]
            #[sea_orm_resource(url = "/api/v1/projects")]
            pub struct Model {
                id: String,
            }
        };

        let ast: DeriveInput = syn::parse2(input).unwrap();
        let receiver = SeaOrmResourceInput::from_derive_input(&ast).unwrap();
        let output = impl_sea_orm_resource(receiver).unwrap();
        let output_str = output.to_string();

        assert!(output_str.contains(r#"const URL : & 'static str = "/api/v1/projects""#));
        assert!(
            output_str.contains(r#"const URL_WITH_ID : & 'static str = "/api/v1/projects/{id}""#)
        );
    }

    #[test]
    fn test_custom_tag() {
        let input = quote! {
            #[sea_orm(table_name = "projects")]
            #[sea_orm_resource(tag = "Project Management")]
            pub struct Model {
                id: String,
            }
        };

        let ast: DeriveInput = syn::parse2(input).unwrap();
        let receiver = SeaOrmResourceInput::from_derive_input(&ast).unwrap();
        let output = impl_sea_orm_resource(receiver).unwrap();
        let output_str = output.to_string();

        assert!(output_str.contains(r#"const TAG : & 'static str = "Project Management""#));
    }

    #[test]
    fn test_all_custom_attributes() {
        let input = quote! {
            #[sea_orm(table_name = "projects")]
            #[sea_orm_resource(
                collection = "project_items",
                url = "/api/v1/projects",
                tag = "Project Catalog"
            )]
            pub struct Model {
                id: String,
            }
        };

        let ast: DeriveInput = syn::parse2(input).unwrap();
        let receiver = SeaOrmResourceInput::from_derive_input(&ast).unwrap();
        let output = impl_sea_orm_resource(receiver).unwrap();
        let output_str = output.to_string();

        assert!(output_str.contains(r#"const COLLECTION : & 'static str = "project_items""#));
        assert!(output_str.contains(r#"const URL : & 'static str = "/api/v1/projects""#));
        assert!(
            output_str.contains(r#"const URL_WITH_ID : & 'static str = "/api/v1/projects/{id}""#)
        );
        assert!(output_str.contains(r#"const TAG : & 'static str = "Project Catalog""#));
    }

    #[test]
    fn test_missing_table_name() {
        let input = quote! {
            pub struct Model {
                id: String,
            }
        };

        let ast: DeriveInput = syn::parse2(input).unwrap();
        let receiver = SeaOrmResourceInput::from_derive_input(&ast).unwrap();
        let result = impl_sea_orm_resource(receiver);

        assert!(result.is_err());
    }

    #[test]
    fn test_capitalize_first_letter() {
        assert_eq!(capitalize_first_letter(""), "");
        assert_eq!(capitalize_first_letter("a"), "A");
        assert_eq!(capitalize_first_letter("projects"), "Projects");
        assert_eq!(capitalize_first_letter("users"), "Users");
    }
}

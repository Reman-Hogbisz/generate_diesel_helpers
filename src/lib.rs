use proc_macro::{self, TokenStream};
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::parse::Parser;

fn camel_case_to_snake_case(camel_case: &str) -> String {
    let mut snake_case = String::new();
    for (i, c) in camel_case.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                snake_case.push('_');
            }
            snake_case.push(c.to_ascii_lowercase());
        } else {
            snake_case.push(c);
        }
    }
    snake_case
}

/**
 Generate SQL methods for a struct

 Args:
  - struct: The struct to generate SQL methods for
  - table_name: The name of the table in the database
    - id_type: The type of the ID field in the struct
*/
#[proc_macro]
pub fn generate_sql_methods(input: TokenStream) -> TokenStream {
    let mut data = syn::punctuated::Punctuated::<syn::Type, syn::Token![,]>::parse_terminated
        .parse(input)
        .expect("Failed to parse punctuated inpute")
        .into_iter();

    let struct_name = data.next().expect("Failed to get struct name from data");
    let table_name = data.next().expect("Failed to get table name from data");
    let id_type = data.next().expect("Failed to get id type from data");

    let (struct_name, snake_case_struct_name) = match struct_name {
        syn::Type::Path(p) => {
            let struct_name = p
                .path
                .segments
                .first()
                .expect("Failed to get first portion of struct_name segments")
                .ident
                .clone();
            let snake_case_struct_name =
                camel_case_to_snake_case(&struct_name.to_string()).to_lowercase();
            (struct_name, snake_case_struct_name)
        }
        _ => panic!("Expected a struct name"),
    };

    let table_name = match table_name {
        syn::Type::Path(p) => p.path.clone(),
        _ => panic!("Expected a table name"),
    };

    let get_struct_ident = Ident::new(&format!("get_{snake_case_struct_name}"), Span::call_site());
    let get_with_conn_struct_ident = Ident::new(
        &format!("get_{snake_case_struct_name}_with_conn"),
        Span::call_site(),
    );
    let insert_struct_ident = Ident::new(&format!("Insertable{struct_name}"), Span::call_site());
    let insert_struct_fn_ident = Ident::new(
        &format!("insert_{snake_case_struct_name}"),
        Span::call_site(),
    );
    let insert_with_conn_struct_fn_ident = Ident::new(
        &format!("insert_{snake_case_struct_name}_with_conn"),
        Span::call_site(),
    );
    let update_struct_ident = Ident::new(&format!("Updatable{struct_name}"), Span::call_site());
    let update_struct_fn_ident = Ident::new(
        &format!("update_{snake_case_struct_name}"),
        Span::call_site(),
    );
    let update_with_conn_struct_fn_ident = Ident::new(
        &format!("update_{snake_case_struct_name}_with_conn"),
        Span::call_site(),
    );
    let patch_struct_ident = Ident::new(
        &format!("patch_{snake_case_struct_name}"),
        Span::call_site(),
    );
    let patch_with_conn_struct_ident = Ident::new(
        &format!("patch_{snake_case_struct_name}_with_conn"),
        Span::call_site(),
    );
    let delete_struct_ident = Ident::new(
        &format!("delete_{snake_case_struct_name}"),
        Span::call_site(),
    );
    let delete_with_conn_struct_ident = Ident::new(
        &format!("delete_{snake_case_struct_name}_with_conn"),
        Span::call_site(),
    );

    let output = quote! {

        pub fn #get_struct_ident(struct_id: &#id_type, pool: &PgPool) -> Result<#struct_name, SqlError> {
            use diesel::prelude::*;
            let mut conn = get_connection!(pool);

            #get_with_conn_struct_ident(struct_id, &mut conn)
        }

        pub fn #get_with_conn_struct_ident(struct_id: &#id_type, conn: &mut PgPooledConnection) -> Result<#struct_name, SqlError> {
            use diesel::prelude::*;

            let result = #table_name::table
                .find(struct_id)
                .first(conn)
                .map_err(|e| {
                    log::error!("Failed to get {} with ID {struct_id} (error: {e})", stringify!(#struct_name));
                    SqlError::DieselError(e)
                })?;
            Ok(result)
        }

        pub fn #insert_struct_fn_ident(new_struct: &#insert_struct_ident, pool: &PgPool) -> Result<#struct_name, SqlError> {
            use diesel::prelude::*;

            let mut conn = get_connection!(pool);

            #insert_with_conn_struct_fn_ident(new_struct, &mut conn)
        }

        pub fn #insert_with_conn_struct_fn_ident(new_struct: &#insert_struct_ident, conn: &mut PgPooledConnection) -> Result<#struct_name, SqlError> {
            use #table_name::dsl::*;
            use diesel::prelude::*;

            let result = diesel::insert_into(#table_name::table)
                .values((
                    new_struct,
                    created_at.eq(diesel::dsl::now),
                    updated_at.eq(diesel::dsl::now)
                ))
                .get_result(conn).map_err(|e| {
                    log::error!("Failed to insert new {} (error: {e})", stringify!(#struct_name));
                    SqlError::DieselError(e)
                })?;
            Ok(result)
        }

        pub fn #update_struct_fn_ident(struct_id: &#id_type, updated_struct: &#insert_struct_ident, pool: &PgPool) -> Result<#struct_name, SqlError> {
            use diesel::prelude::*;

            let mut conn = get_connection!(pool);
            #update_with_conn_struct_fn_ident(struct_id, updated_struct, &mut conn)
        }

        pub fn #update_with_conn_struct_fn_ident(struct_id: &#id_type, updated_struct: &#insert_struct_ident, conn: &mut PgPooledConnection) -> Result<#struct_name, SqlError> {
            use #table_name::dsl::*;
            use diesel::prelude::*;

            let result = diesel::update(#table_name::table.find(struct_id))
                .set((
                    updated_struct,
                    updated_at.eq(diesel::dsl::now)
                ))
                .get_result(conn).map_err(|e| {
                    log::error!("Failed to update {} with ID {struct_id} (error: {e})", stringify!(#struct_name));
                    SqlError::DieselError(e)
                })?;
            Ok(result)
        }

        pub fn #patch_struct_ident(struct_id: &#id_type, updated_struct: &#update_struct_ident, pool: &PgPool) -> Result<#struct_name, SqlError> {
            use diesel::prelude::*;

            let mut conn = get_connection!(pool);
            #patch_with_conn_struct_ident(struct_id, updated_struct, &mut conn)
        }

        pub fn #patch_with_conn_struct_ident(struct_id: &#id_type, updated_struct: &#update_struct_ident, conn: &mut PgPooledConnection) -> Result<#struct_name, SqlError> {
            use #table_name::dsl::*;
            use diesel::prelude::*;

            let result = diesel::update(
                    #table_name::table
                        .find(struct_id)
                )
                .set((updated_struct, updated_at.eq(diesel::dsl::now)))
                .get_result(conn).map_err(|e| {
                    log::error!("Failed to patch {} with ID {struct_id} (error: {e})", stringify!(#struct_name));
                    SqlError::DieselError(e)
                })?;
            Ok(result)
        }

        pub fn #delete_struct_ident(struct_id: &#id_type, pool: &PgPool) -> Result<(), SqlError> {
            use #table_name::dsl::*;
            use diesel::prelude::*;

            let mut conn = get_connection!(pool);
            #delete_with_conn_struct_ident(struct_id, &mut conn)
        }

        pub fn #delete_with_conn_struct_ident(struct_id: &#id_type, conn: &mut PgPooledConnection) -> Result<(), SqlError> {
            use #table_name::dsl::*;
            use diesel::prelude::*;

            diesel::delete(#table_name::table.find(struct_id))
                .execute(conn).map_err(|e| {
                    log::error!("Failed to delete {} with ID {struct_id} (error: {e})", stringify!(#struct_name));
                    SqlError::DieselError(e)
                })?;
            Ok(())
        }
    };

    output.into()
}

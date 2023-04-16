use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    Expr,
    ExprArray,
    Result,
};

/// Converts an identifier in PascalCase to snake_case. (from `serde-derive`)
pub fn pascal_to_snake_case(name: &str) -> String {
    let mut snake = String::new();
    for (i, ch) in name.char_indices() {
        if i > 0 && ch.is_uppercase() {
            snake.push('_');
        }
        snake.push(ch.to_ascii_lowercase());
    }
    snake
}

/// Checks that an [`Expr`] is an array, returning the array if it is.
pub fn validate_expr_array(expr: Expr) -> Result<ExprArray> {
    match expr {
        Expr::Array(array) => Ok(array),
        _ => Err(syn::Error::new_spanned(expr, "expected an array here")),
    }
}

/// Wrap an [`Option<Token>`] in a token that actually wraps the token in an [`Option`].
///
/// This is different from the default implementation of [`ToTokens`] for [`Option`];
/// the default implementation never does any wrapping.
pub fn wrap<T: ToTokens>(option: Option<T>) -> TokenStream2 {
    match option {
        Some(token) => quote! { Some(#token) },
        None => quote! { None },
    }
}

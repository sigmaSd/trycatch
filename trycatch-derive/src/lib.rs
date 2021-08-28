use proc_macro::*;

#[proc_macro_derive(Exception)]
pub fn derive_exception(item: TokenStream) -> TokenStream {
    (|| -> Result<TokenStream, Box<dyn std::error::Error>> {
        let mut item = item.into_iter();
        // walk the items till we find a struct or an enum ident
        while !is_struct_or_enum(item.next()) {}
        let ident = item.next().ok_or("Could not find identifier")?.to_string();
        let impl_exception = format!(
            "impl Exception for {0} {{
                fn name(&self) -> &'static str {{
                    \"{0}\"
                }}
                fn into_any(self: Box<Self>) -> Box<dyn ::std::any::Any> {{
                    self
                }}
             }}",
            ident
        );
        impl_exception.parse().map_err(Into::into)
    })()
    .unwrap()
}

fn is_struct_or_enum(next: Option<TokenTree>) -> bool {
    if next.is_none() {
        return false;
    }
    matches!(next.unwrap(), TokenTree::Ident(ident) if ident.to_string() == "struct" || ident.to_string() == "enum")
}

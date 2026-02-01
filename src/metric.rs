use quote::quote;
use syn::{Error, Field, LitStr, Result, Type};

const COUNTER_TY: &str = "Counter";
const HISTOGRAM_TY: &str = "Histogram";
const GAUGE_TY: &str = "Gauge";

/// A parsed label pair (key, value) from `#[metric(labels = [("k", "v"), ...])]`.
pub(crate) type LabelPair = (LitStr, LitStr);

pub(crate) struct Metric<'a> {
    pub(crate) field: &'a Field,
    pub(crate) description: String,
    pub(crate) labels: Vec<LabelPair>,
    rename: Option<LitStr>,
}

impl<'a> Metric<'a> {
    pub(crate) fn new(
        field: &'a Field,
        description: String,
        rename: Option<LitStr>,
        labels: Vec<LabelPair>,
    ) -> Self {
        Self { field, description, rename, labels }
    }

    pub(crate) fn name(&self) -> String {
        match self.rename.as_ref() {
            Some(name) => name.value(),
            None => self.field.ident.as_ref().map(ToString::to_string).unwrap_or_default(),
        }
    }

    pub(crate) fn register_method(&self) -> Result<proc_macro2::TokenStream> {
        if let Type::Path(ref path_ty) = self.field.ty {
            if let Some(last) = path_ty.path.segments.last() {
                let method = match last.ident.to_string().as_str() {
                    COUNTER_TY => quote! { register_counter },
                    HISTOGRAM_TY => quote! { register_histogram },
                    GAUGE_TY => quote! { register_gauge },
                    _ => return Err(Error::new_spanned(path_ty, "unsupported metric type")),
                };

                return Ok(method);
            }
        }

        Err(Error::new_spanned(&self.field.ty, "unsupported metric type"))
    }

    pub(crate) fn describe_method(&self) -> Result<proc_macro2::TokenStream> {
        if let Type::Path(ref path_ty) = self.field.ty {
            if let Some(last) = path_ty.path.segments.last() {
                let method = match last.ident.to_string().as_str() {
                    COUNTER_TY => quote! { describe_counter },
                    HISTOGRAM_TY => quote! { describe_histogram },
                    GAUGE_TY => quote! { describe_gauge },
                    _ => return Err(Error::new_spanned(path_ty, "unsupported metric type")),
                };

                return Ok(method);
            }
        }

        Err(Error::new_spanned(&self.field.ty, "unsupported metric type"))
    }
}

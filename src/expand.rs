use crate::{
    metric::{LabelPair, Metric},
    with_attrs::WithAttrs,
};
use quote::{quote, ToTokens};
use syn::{
    punctuated::Punctuated, Attribute, Data, DeriveInput, Error, Expr, ExprTuple, Field, Lit,
    LitBool, LitStr, Meta, MetaNameValue, Result, Token,
};

enum MetricField<'a> {
    Included(Metric<'a>),
    Skipped(&'a Field),
}

impl<'a> MetricField<'a> {
    const fn field(&self) -> &'a Field {
        match self {
            MetricField::Included(Metric { field, .. }) | MetricField::Skipped(field) => field,
        }
    }
}

pub(crate) fn derive(node: &DeriveInput) -> Result<proc_macro2::TokenStream> {
    let ty = &node.ident;
    let vis = &node.vis;
    let ident_name = ty.to_string();

    let metrics_attr = parse_metrics_attr(node)?;
    let metric_fields = parse_metric_fields(node)?;

    let global_labels_init = if metrics_attr.labels.is_empty() {
        quote! {}
    } else {
        let label_keys = metrics_attr.labels.iter().map(|(k, _)| k);
        let label_values = metrics_attr.labels.iter().map(|(_, v)| v);
        quote! {
            __labels.extend([
                #(metrics::Label::new(#label_keys, #label_values)),*
            ]);
        }
    };

    let register_and_describe = match &metrics_attr.scope {
        MetricsScope::Static(scope) => {
            let mut field_inits = Vec::with_capacity(metric_fields.len());
            let mut describes = Vec::with_capacity(metric_fields.len());
            let mut partial_clone_inits = Vec::with_capacity(metric_fields.len());

            for metric in &metric_fields {
                let field_name = &metric.field().ident;
                match metric {
                    MetricField::Included(metric) => {
                        let metric_name = format!(
                            "{}{}{}",
                            scope.value(),
                            metrics_attr.separator(),
                            metric.name()
                        );
                        let register_method = metric.register_method()?;
                        let describe_method = metric.describe_method()?;
                        let description = &metric.description;

                        let field_init = if metric.labels.is_empty() {
                            quote! {
                                __recorder.#register_method(
                                    &metrics::Key::from_parts(#metric_name, __labels.clone()),
                                    __metadata,
                                )
                            }
                        } else {
                            let label_keys = metric.labels.iter().map(|(k, _)| k);
                            let label_values = metric.labels.iter().map(|(_, v)| v);
                            quote! {
                                {
                                    let mut __field_labels = __labels.clone();
                                    __field_labels.extend([
                                        #(metrics::Label::new(#label_keys, #label_values)),*
                                    ]);
                                    __recorder.#register_method(
                                        &metrics::Key::from_parts(#metric_name, __field_labels),
                                        __metadata,
                                    )
                                }
                            }
                        };

                        field_inits.push(quote! {
                            #field_name: #field_init,
                        });
                        describes.push(quote! {
                            __recorder.#describe_method(
                                ::core::convert::Into::into(#metric_name),
                                ::core::option::Option::None,
                                ::core::convert::Into::into(#description),
                            );
                        });
                        partial_clone_inits.push(quote! {
                            #field_name: ::core::clone::Clone::clone(&self.#field_name),
                        });
                    }
                    MetricField::Skipped(_) => {
                        field_inits.push(quote! {
                            #field_name: Default::default(),
                        });
                        partial_clone_inits.push(quote! {
                            #field_name: Default::default(),
                        });
                    }
                }
            }

            quote! {
                impl Default for #ty {
                    /// Creates a new instance of the metrics.
                    ///
                    /// This initializes all metrics and registers them with the global recorder.
                    /// Metrics are described only once; see [`Self::describe`].
                    ///
                    /// The result is cached in a static `OnceLock`, so subsequent calls will
                    /// return a clone of the same instance.
                    fn default() -> Self {
                        static __ONCE: ::std::sync::OnceLock<#ty> = ::std::sync::OnceLock::new();
                        __ONCE.get_or_init(|| {
                            Self::_new_with_labels(::std::vec::Vec::<metrics::Label>::new())
                        })._partial_clone()
                    }
                }

                impl #ty {
                    /// Creates a new instance of the metrics with the provided labels.
                    ///
                    /// Unlike [`Self::default`], this does not cache the result, so it can be used
                    /// to re-register metrics or register with different labels.
                    #vis fn new_with_labels(labels: impl metrics::IntoLabels) -> Self {
                        Self::_new_with_labels(labels.into_labels())
                    }

                    fn _new_with_labels(labels: ::std::vec::Vec<metrics::Label>) -> Self {
                        Self::describe();
                        metrics::with_recorder(|__recorder| {
                            static __METADATA: metrics::Metadata<'static> = metrics::Metadata::new(
                                module_path!(),
                                metrics::Level::INFO,
                                ::core::option::Option::Some(module_path!()),
                            );
                            let __metadata = &__METADATA;
                            #[allow(unused_mut)]
                            let mut __labels = labels;
                            #global_labels_init
                            Self {
                                #(#field_inits)*
                            }
                        })
                    }

                    /// Describe all exposed metrics.
                    ///
                    /// Internally, this calls the `describe_*` macros from the metrics crate
                    /// according to the metric type.
                    ///
                    /// This is done only once, to avoid multiple `describe` calls to the same
                    /// recorder. If this is not preferred, you can call [`Self::force_describe`]
                    /// directly.
                    ///
                    /// See: <https://docs.rs/metrics/latest/metrics/index.html#macros>
                    #vis fn describe() {
                        static __DESCRIBE_ONCE: ::std::sync::Once = ::std::sync::Once::new();
                        __DESCRIBE_ONCE.call_once(Self::force_describe);
                    }

                    /// Unconditionally describes all metrics.
                    ///
                    /// See [`Self::describe`].
                    #vis fn force_describe() {
                        metrics::with_recorder(|__recorder| {
                            #(#describes)*
                        });
                    }

                    fn _partial_clone(&self) -> Self {
                        Self {
                            #(#partial_clone_inits)*
                        }
                    }
                }
            }
        }
        MetricsScope::Dynamic => {
            let mut field_inits = Vec::with_capacity(metric_fields.len());
            let mut describes = Vec::with_capacity(metric_fields.len());

            for metric in &metric_fields {
                let field_name = &metric.field().ident;
                match metric {
                    MetricField::Included(metric) => {
                        let name = metric.name();
                        let separator = metrics_attr.separator();

                        let register_method = metric.register_method()?;
                        let describe_method = metric.describe_method()?;
                        let description = &metric.description;

                        let field_init = if metric.labels.is_empty() {
                            quote! {
                                __recorder.#register_method(
                                    &metrics::Key::from_parts(
                                        format!("{}{}{}", __scope, #separator, #name),
                                        __labels.clone(),
                                    ),
                                    __metadata,
                                )
                            }
                        } else {
                            let label_keys = metric.labels.iter().map(|(k, _)| k);
                            let label_values = metric.labels.iter().map(|(_, v)| v);
                            quote! {
                                {
                                    let mut __field_labels = __labels.clone();
                                    __field_labels.extend([
                                        #(metrics::Label::new(#label_keys, #label_values)),*
                                    ]);
                                    __recorder.#register_method(
                                        &metrics::Key::from_parts(
                                            format!("{}{}{}", __scope, #separator, #name),
                                            __field_labels,
                                        ),
                                        __metadata,
                                    )
                                }
                            }
                        };

                        field_inits.push(quote! {
                            #field_name: #field_init,
                        });
                        describes.push(quote! {
                            __recorder.#describe_method(
                                ::core::convert::Into::into(format!("{}{}{}", __scope, #separator, #name)),
                                ::core::option::Option::None,
                                ::core::convert::Into::into(#description),
                            );
                        });
                    }
                    MetricField::Skipped(_) => {
                        field_inits.push(quote! {
                            #field_name: Default::default(),
                        });
                    }
                }
            }

            quote! {
                impl #ty {
                    /// Creates a new instance of the metrics with the provided scope.
                    ///
                    /// This initializes all metrics and registers them with the global recorder.
                    #vis fn new(scope: &str) -> Self {
                        Self::_new_with_labels(scope, ::std::vec::Vec::<metrics::Label>::new())
                    }

                    /// Creates a new instance of the metrics with the provided scope and labels.
                    #vis fn new_with_labels(scope: &str, labels: impl metrics::IntoLabels) -> Self {
                        Self::_new_with_labels(scope, labels.into_labels())
                    }

                    fn _new_with_labels(scope: &str, labels: ::std::vec::Vec<metrics::Label>) -> Self {
                        Self::describe(scope);
                        metrics::with_recorder(|__recorder| {
                            static __METADATA: metrics::Metadata<'static> = metrics::Metadata::new(
                                module_path!(),
                                metrics::Level::INFO,
                                ::core::option::Option::Some(module_path!()),
                            );
                            let __metadata = &__METADATA;
                            let __scope = scope;
                            #[allow(unused_mut)]
                            let mut __labels = labels;
                            #global_labels_init
                            Self {
                                #(#field_inits)*
                            }
                        })
                    }

                    /// Describe all exposed metrics. Internally calls `describe_*` macros from
                    /// the metrics crate according to the metric type.
                    ///
                    /// See: <https://docs.rs/metrics/latest/metrics/index.html#macros>
                    #vis fn describe(scope: &str) {
                        metrics::with_recorder(|__recorder| {
                            let __scope = scope;
                            #(#describes)*
                        });
                    }
                }
            }
        }
    };

    Ok(quote! {
        #register_and_describe

        impl ::core::fmt::Debug for #ty {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct(#ident_name).finish()
            }
        }
    })
}

pub(crate) struct MetricsAttr {
    pub(crate) scope: MetricsScope,
    pub(crate) separator: Option<LitStr>,
    pub(crate) labels: Vec<LabelPair>,
}

impl MetricsAttr {
    const DEFAULT_SEPARATOR: &'static str = ".";

    fn separator(&self) -> String {
        match &self.separator {
            Some(sep) => sep.value(),
            None => Self::DEFAULT_SEPARATOR.to_owned(),
        }
    }
}

pub(crate) enum MetricsScope {
    Static(LitStr),
    Dynamic,
}

fn parse_metrics_attr(node: &DeriveInput) -> Result<MetricsAttr> {
    let metrics_attr = parse_single_required_attr(node, "metrics")?;
    let parsed =
        metrics_attr.parse_args_with(Punctuated::<MetaNameValue, Token![,]>::parse_terminated)?;
    let (mut scope, mut separator, mut dynamic, mut labels) = (None, None, None, None);
    for kv in parsed {
        if kv.path.is_ident("labels") {
            if labels.is_some() {
                return Err(Error::new_spanned(kv, "duplicate `labels` value provided"));
            }
            labels = Some(parse_labels_expr(&kv.value)?);
            continue;
        }

        let lit = match kv.value {
            Expr::Lit(ref expr) => &expr.lit,
            _ => return Err(Error::new_spanned(&kv.value, "value must be a literal")),
        };
        if kv.path.is_ident("scope") {
            if scope.is_some() {
                return Err(Error::new_spanned(kv, "duplicate `scope` value provided"));
            }
            scope = Some(parse_str_lit(lit)?);
        } else if kv.path.is_ident("separator") {
            if separator.is_some() {
                return Err(Error::new_spanned(kv, "duplicate `separator` value provided"));
            }
            separator = Some(parse_str_lit(lit)?);
        } else if kv.path.is_ident("dynamic") {
            if dynamic.is_some() {
                return Err(Error::new_spanned(kv, "duplicate `dynamic` flag provided"));
            }
            dynamic = Some(parse_bool_lit(lit)?.value);
        } else {
            return Err(Error::new_spanned(kv, "unsupported attribute entry"));
        }
    }

    let scope = match (scope, dynamic) {
        (Some(scope), None | Some(false)) => MetricsScope::Static(scope),
        (None, Some(true)) => MetricsScope::Dynamic,
        (Some(_), Some(_)) => {
            return Err(Error::new_spanned(node, "`scope = ..` conflicts with `dynamic = true`"))
        }
        _ => {
            return Err(Error::new_spanned(
                node,
                "either `scope = ..` or `dynamic = true` must be set",
            ))
        }
    };

    let labels = labels.unwrap_or_default();
    Ok(MetricsAttr { scope, separator, labels })
}

fn parse_metric_fields(node: &DeriveInput) -> Result<Vec<MetricField<'_>>> {
    let Data::Struct(ref data) = node.data else {
        return Err(Error::new_spanned(node, "only structs are supported"));
    };

    let mut metrics = Vec::with_capacity(data.fields.len());
    for field in &data.fields {
        let (mut describe, mut rename, mut labels, mut skip) = (None, None, None, false);
        if let Some(metric_attr) = parse_single_attr(field, "metric")? {
            let parsed =
                metric_attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
            for meta in parsed {
                match meta {
                    Meta::Path(path) if path.is_ident("skip") => skip = true,
                    Meta::NameValue(kv) => {
                        if kv.path.is_ident("labels") {
                            if labels.is_some() {
                                return Err(Error::new_spanned(
                                    kv,
                                    "duplicate `labels` value provided",
                                ));
                            }
                            labels = Some(parse_labels_expr(&kv.value)?);
                        } else if kv.path.is_ident("describe") {
                            if describe.is_some() {
                                return Err(Error::new_spanned(
                                    kv,
                                    "duplicate `describe` value provided",
                                ));
                            }
                            let lit = parse_expr_lit(&kv.value)?;
                            describe = Some(parse_str_lit(lit)?);
                        } else if kv.path.is_ident("rename") {
                            if rename.is_some() {
                                return Err(Error::new_spanned(
                                    kv,
                                    "duplicate `rename` value provided",
                                ));
                            }
                            let lit = parse_expr_lit(&kv.value)?;
                            rename = Some(parse_str_lit(lit)?);
                        } else {
                            return Err(Error::new_spanned(kv, "unsupported attribute entry"));
                        }
                    }
                    _ => return Err(Error::new_spanned(meta, "unsupported attribute entry")),
                }
            }
        }

        if skip {
            metrics.push(MetricField::Skipped(field));
            continue;
        }

        let description = match describe {
            Some(lit_str) => lit_str.value(),
            // Parse docs only if `describe` attribute was not provided
            None => match parse_docs_to_string(field)? {
                Some(docs_str) => docs_str,
                None => {
                    return Err(Error::new_spanned(
                        field,
                        "either doc comment or `describe = ..` must be set",
                    ))
                }
            },
        };

        let labels = labels.unwrap_or_default();
        metrics.push(MetricField::Included(Metric::new(field, description, rename, labels)));
    }

    Ok(metrics)
}

fn parse_single_attr<'a, T: WithAttrs + ToTokens>(
    token: &'a T,
    ident: &str,
) -> Result<Option<&'a Attribute>> {
    let mut attr_iter = token.attrs().iter().filter(|a| a.path().is_ident(ident));
    if let Some(attr) = attr_iter.next() {
        if let Some(next_attr) = attr_iter.next() {
            Err(Error::new_spanned(
                next_attr,
                format!("duplicate `#[{ident}(..)]` attribute provided"),
            ))
        } else {
            Ok(Some(attr))
        }
    } else {
        Ok(None)
    }
}

fn parse_single_required_attr<'a, T: WithAttrs + ToTokens>(
    token: &'a T,
    ident: &str,
) -> Result<&'a Attribute> {
    if let Some(attr) = parse_single_attr(token, ident)? {
        Ok(attr)
    } else {
        Err(Error::new_spanned(token, format!("`#[{ident}(..)]` attribute must be provided")))
    }
}

fn parse_docs_to_string<T: WithAttrs>(token: &T) -> Result<Option<String>> {
    let mut doc_str = None;
    for attr in token.attrs() {
        if let syn::Meta::NameValue(ref meta) = attr.meta {
            if let Expr::Lit(ref lit) = meta.value {
                if let Lit::Str(ref doc) = lit.lit {
                    let doc_value = doc.value().trim().to_string();
                    doc_str = Some(
                        doc_str
                            .map(|prev_doc_value| format!("{prev_doc_value} {doc_value}"))
                            .unwrap_or(doc_value),
                    );
                }
            }
        }
    }
    Ok(doc_str)
}

fn parse_str_lit(lit: &Lit) -> Result<LitStr> {
    match lit {
        Lit::Str(lit_str) => Ok(lit_str.to_owned()),
        _ => Err(Error::new_spanned(lit, "value must be a string literal")),
    }
}

fn parse_bool_lit(lit: &Lit) -> Result<LitBool> {
    match lit {
        Lit::Bool(lit_bool) => Ok(lit_bool.to_owned()),
        _ => Err(Error::new_spanned(lit, "value must be a boolean literal")),
    }
}

fn parse_expr_lit(expr: &Expr) -> Result<&Lit> {
    match expr {
        Expr::Lit(expr_lit) => Ok(&expr_lit.lit),
        _ => Err(Error::new_spanned(expr, "value must be a literal")),
    }
}

/// Parses `labels = [("key", expr), ...]` into a vector of label pairs.
/// Keys must be string literals; values can be arbitrary expressions.
fn parse_labels_expr(expr: &Expr) -> Result<Vec<LabelPair>> {
    let Expr::Array(arr) = expr else {
        return Err(Error::new_spanned(
            expr,
            "labels must be an array of tuples, e.g. `labels = [(\"key\", \"value\")]`",
        ));
    };

    let mut labels = Vec::with_capacity(arr.elems.len());
    for elem in &arr.elems {
        let Expr::Tuple(ExprTuple { elems, .. }) = elem else {
            return Err(Error::new_spanned(
                elem,
                "each label must be a tuple, e.g. `(\"key\", \"value\")` or `(\"key\", CONST)`",
            ));
        };

        if elems.len() != 2 {
            return Err(Error::new_spanned(
                elem,
                "each label tuple must have exactly two elements: (key, value)",
            ));
        }

        // Key must be a string literal
        let key = parse_str_lit(parse_expr_lit(&elems[0])?)?;
        // Value can be any expression (string literal, constant, etc.)
        let value = elems[1].clone();
        labels.push((key, value));
    }

    Ok(labels)
}

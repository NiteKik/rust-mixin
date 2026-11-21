//! Only signatures/attributes are inspected. rustc compiles the original body.
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use syn::{
    Error, ItemFn, ItemMod, LitStr, Meta, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
};

#[proc_macro_attribute]
pub fn targets(args: TokenStream, input: TokenStream) -> TokenStream {
    let namespace = parse_macro_input!(args as LitStr);
    let module = parse_macro_input!(input as ItemMod);
    expand_targets(namespace.value(), module)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
pub fn modify_call_result(args: TokenStream, input: TokenStream) -> TokenStream {
    let metas = parse_macro_input!(args with syn::punctuated::Punctuated::<Meta, Token![,]>::parse_terminated);
    let function = parse_macro_input!(input as ItemFn);
    expand_handler(metas.into_iter().collect(), function)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
pub fn modify_return_result(args: TokenStream, input: TokenStream) -> TokenStream {
    let metas = parse_macro_input!(args with syn::punctuated::Punctuated::<Meta, Token![,]>::parse_terminated);
    let function = parse_macro_input!(input as ItemFn);
    expand_handler_kind(
        metas.into_iter().collect(),
        function,
        "modify_return_result",
        false,
    )
    .unwrap_or_else(Error::into_compile_error)
    .into()
}

#[proc_macro_attribute]
pub fn modify_parameter(args: TokenStream, input: TokenStream) -> TokenStream {
    let metas = parse_macro_input!(args with syn::punctuated::Punctuated::<Meta, Token![,]>::parse_terminated);
    let function = parse_macro_input!(input as ItemFn);
    expand_handler_kind(
        metas.into_iter().collect(),
        function,
        "modify_parameter",
        false,
    )
    .unwrap_or_else(Error::into_compile_error)
    .into()
}

#[proc_macro_attribute]
pub fn modify_head(args: TokenStream, input: TokenStream) -> TokenStream {
    let metas = parse_macro_input!(args with syn::punctuated::Punctuated::<Meta, Token![,]>::parse_terminated);
    let function = parse_macro_input!(input as ItemFn);
    expand_handler_kind(metas.into_iter().collect(), function, "modify_head", false)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
pub fn inject_head(args: TokenStream, input: TokenStream) -> TokenStream {
    let metas = parse_macro_input!(args as InjectionArgs).0;
    let function = parse_macro_input!(input as ItemFn);
    expand_handler_kind(metas, function, "inject_head", false)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Run a callback after a call that returns `()` inside the target function.
#[proc_macro_attribute]
pub fn inject_after_call(args: TokenStream, input: TokenStream) -> TokenStream {
    let metas = parse_macro_input!(args as InjectionArgs).0;
    let function = parse_macro_input!(input as ItemFn);
    expand_unit_callback(metas, function, "modify_call_result", true)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Run a callback before a normal return from a target that returns `()`.
#[proc_macro_attribute]
pub fn inject_return(args: TokenStream, input: TokenStream) -> TokenStream {
    let metas = parse_macro_input!(args as InjectionArgs).0;
    let function = parse_macro_input!(input as ItemFn);
    expand_unit_callback(metas, function, "modify_return_result", false)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

struct InjectionArgs(Vec<Meta>);

impl Parse for InjectionArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut metas = Vec::new();
        if input.peek(LitStr) {
            let target: LitStr = input.parse()?;
            metas.push(syn::parse_quote!(target = #target));
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        metas.extend(Punctuated::<Meta, Token![,]>::parse_terminated(input)?);
        Ok(Self(metas))
    }
}

fn expand_unit_callback(
    metas: Vec<Meta>,
    function: ItemFn,
    operation: &str,
    requires_callee: bool,
) -> syn::Result<proc_macro2::TokenStream> {
    if !function.sig.inputs.is_empty() || !returns_unit(&function.sig.output) {
        return Err(Error::new(
            function.sig.span(),
            "expected a callback with signature fn()",
        ));
    }
    expand_handler_kind(metas, function, operation, requires_callee)
}

fn hash(id: &str) -> String {
    format!("{:x}", Sha256::digest(id.as_bytes()))[..24].into()
}

fn scalar_type(ty: &syn::Type) -> syn::Result<syn::Ident> {
    if is_sdk_type(ty, "EntityId") {
        return Ok(syn::Ident::new("entity_id", ty.span()));
    }
    if let syn::Type::Path(path) = ty
        && path.qself.is_none()
        && let Some(name) = path.path.get_ident()
        && matches!(
            name.to_string().as_str(),
            "bool"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "isize"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "usize"
                | "f32"
                | "f64"
        )
    {
        return Ok(name.clone());
    }
    Err(Error::new(
        ty.span(),
        "expected a supported primitive scalar type or mixin_sdk::EntityId",
    ))
}

fn is_sdk_type(ty: &syn::Type, name: &str) -> bool {
    let syn::Type::Path(ty) = ty else {
        return false;
    };
    let path = &ty.path;
    ty.qself.is_none()
        && (path.is_ident(name)
            || (path.segments.len() == 2
                && path.segments[0].ident == "mixin_sdk"
                && path.segments[0].arguments.is_empty()
                && path.segments[1].ident == name
                && path.segments[1].arguments.is_empty()))
}

fn check_function(function: &ItemFn) -> syn::Result<()> {
    let s = &function.sig;
    if s.constness.is_some()
        || s.asyncness.is_some()
        || s.unsafety.is_some()
        || s.abi.is_some()
        || s.variadic.is_some()
        || !s.generics.params.is_empty()
        || s.generics.where_clause.is_some()
        || s.inputs.len() > 4
    {
        return Err(Error::new(
            s.span(),
            "indexed boundary must be an ordinary safe function with 0..=4 scalar parameters; put other helpers outside the indexed module",
        ));
    }
    for attr in &function.attrs {
        if !attr.path().is_ident("doc") {
            return Err(Error::new(
                attr.span(),
                "attributes on indexed functions are not supported in this SDK revision",
            ));
        }
    }
    Ok(())
}

fn parameters(function: &ItemFn, allow_return_value: bool) -> syn::Result<Vec<syn::Ident>> {
    check_function(function)?;
    function
        .sig
        .inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Typed(param) if allow_return_value => value_type(&param.ty),
            syn::FnArg::Typed(param) => scalar_type(&param.ty),
            syn::FnArg::Receiver(_) => Err(Error::new(arg.span(), "expected a scalar parameter")),
        })
        .collect()
}

fn value_type(ty: &syn::Type) -> syn::Result<syn::Ident> {
    if matches!(ty, syn::Type::Tuple(tuple) if tuple.elems.is_empty()) {
        return Ok(syn::Ident::new("unit", ty.span()));
    }
    if is_optional_entity(ty) {
        return Ok(syn::Ident::new("option_entity_id", ty.span()));
    }
    if is_sdk_type(ty, "Vec2") {
        return Ok(syn::Ident::new("vec2", ty.span()));
    }
    scalar_type(ty)
}

fn is_optional_entity(ty: &syn::Type) -> bool {
    if let syn::Type::Path(path) = ty
        && path.qself.is_none()
        && let Some(segment) = path.path.segments.last()
        && segment.ident == "Option"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && args.args.len() == 1
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        return is_sdk_type(inner, "EntityId");
    }
    false
}

fn rust_type(kind: &syn::Ident) -> proc_macro2::TokenStream {
    match kind.to_string().as_str() {
        "entity_id" => quote!(::mixin_sdk::EntityId),
        "option_entity_id" => quote!(::core::option::Option<::mixin_sdk::EntityId>),
        "vec2" => quote!(::mixin_sdk::Vec2),
        _ => quote!(::core::primitive::#kind),
    }
}

fn result_type(function: &ItemFn) -> syn::Result<syn::Ident> {
    match &function.sig.output {
        syn::ReturnType::Default => Ok(syn::Ident::new("unit", function.sig.ident.span())),
        syn::ReturnType::Type(_, result) => value_type(result),
    }
}

fn returns_unit(output: &syn::ReturnType) -> bool {
    match output {
        syn::ReturnType::Default => true,
        syn::ReturnType::Type(_, ty) => {
            matches!(ty.as_ref(), syn::Type::Tuple(tuple) if tuple.elems.is_empty())
        }
    }
}

fn head_result(function: &ItemFn) -> syn::Result<syn::Ident> {
    if let syn::ReturnType::Type(_, ty) = &function.sig.output
        && let syn::Type::Path(path) = ty.as_ref()
        && let Some(segment) = path.path.segments.last()
        && segment.ident == "ControlFlow"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && args.args.len() == 1
        && let Some(syn::GenericArgument::Type(result)) = args.args.first()
    {
        return value_type(result);
    }
    Err(Error::new(
        function.sig.output.span(),
        "inject_head expects () or ControlFlow<T>, where T is the target return type",
    ))
}

fn metadata(name: &str, descriptor: serde_json::Value) -> proc_macro2::TokenStream {
    let ident = format_ident!("__MIXIN_METADATA_{}", hash(name));
    let symbol = format!("__rsmx_meta_{}", hash(name));
    let data = serde_json::to_vec(&descriptor).expect("literal metadata");
    let len = data.len();
    let bytes = syn::LitByteStr::new(&data, proc_macro2::Span::call_site());
    quote! {
        #[cfg(mixin_retained)]
        #[used]
        #[unsafe(export_name = #symbol)]
        static #ident: [u8; #len] = *#bytes;
    }
}

fn expand_targets(namespace: String, mut module: ItemMod) -> syn::Result<proc_macro2::TokenStream> {
    if namespace.is_empty() {
        return Err(Error::new(module.span(), "empty namespace"));
    }
    let (_, items) = module
        .content
        .as_mut()
        .ok_or_else(|| Error::new(module.ident.span(), "targets requires an inline module"))?;
    let mut generated = Vec::new();
    for item in items.iter_mut() {
        match item {
            syn::Item::Fn(function) => {
                generated.push(expand_target(&namespace, function)?);
            }
            syn::Item::Mod(child) if child.content.is_some() => {
                let expanded =
                    expand_targets(format!("{namespace}::{}", child.ident), child.clone())?;
                *item = syn::Item::Verbatim(expanded);
            }
            _ => {}
        }
    }
    items.push(syn::Item::Verbatim(quote!(#(#generated)*)));
    Ok(quote!(#module))
}

fn expand_target(namespace: &str, function: &mut ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    // Accept the former opt-in marker as a no-op for existing game source.
    if let Some(index) = function
        .attrs
        .iter()
        .position(|attr| attr.path().is_ident("typed_return"))
    {
        let marker = function.attrs.remove(index);
        if !matches!(marker.meta, Meta::Path(_)) {
            return Err(Error::new(marker.span(), "expected #[typed_return]"));
        }
    }
    let parameters = parameters(function, false)?;
    let scalar_result = result_type(function).ok();
    let ty = concrete_return_type(&function.sig.output)?;
    let id = format!("{namespace}::{}", function.sig.ident);
    let key = hash(&id);
    let name = &function.sig.ident;
    let symbol = format!("__rsmx_fn_{key}");
    let entry = format!("__rsmx_entry_{key}");
    let hook = format!("__rsmx_return_{key}");
    let probe = format!("__rsmx_target_type_{key}");
    let body_symbol = format!("__rsmx_body_{key}");
    let body_fn = format_ident!("__mixin_body_{name}");
    let entry_fn = format_ident!("__mixin_entry_{name}");
    let hook_fn = format_ident!("__mixin_return_{name}");
    let mut body_signature = function.sig.clone();
    body_signature.ident = body_fn.clone();
    let mut retained = function.clone();
    let mut arguments = Vec::new();
    for (index, parameter) in retained.sig.inputs.iter_mut().enumerate() {
        let syn::FnArg::Typed(parameter) = parameter else {
            unreachable!("validated target parameter");
        };
        let arg = format_ident!("__mixin_input{index}");
        *parameter.pat = syn::parse_quote!(#arg);
        arguments.push(arg);
    }
    let body = &function.block;
    // Keep the original body available for call selectors. Its normal exits
    // rejoin at the return hook, including early returns and ?.
    let body_code = quote! {
        #[cfg(mixin_retained)]
        #[inline(never)]
        #[unsafe(export_name = #body_symbol)]
        #body_signature #body
    };
    *retained.block = syn::parse_quote!({
        let mut __result: #ty = #body_fn(#(#arguments),*);
        #hook_fn((&raw mut __result).cast());
        __result
    });
    function
        .attrs
        .push(syn::parse_quote!(#[cfg(not(mixin_retained))]));
    let arity = parameters.len();
    let parameter_names: Vec<_> = parameters.iter().map(ToString::to_string).collect();
    let return_hook = json!({"symbol": hook, "type_probe": probe, "body": body_symbol});
    let (entry_code, descriptor) = if let Some(result) = scalar_result {
        let mut descriptor = json!({
            "schema": 1, "kind": "target", "logical_id": id, "symbol": symbol,
            "entry": entry, "arity": arity, "retention_noinline": true, "return_hook": return_hook,
        });
        let entry_code = if result == "f64" && parameters.iter().all(|kind| kind == "f64") {
            let args: Vec<_> = (0..arity).map(|i| format_ident!("__arg{i}")).collect();
            quote! { pub extern "C" fn #entry_fn(#(#args: f64),*) -> f64 { #name(#(#args),*) } }
        } else {
            descriptor["schema"] = json!(2);
            descriptor["signature"] =
                json!({"parameters": parameter_names, "result": result.to_string()});
            scalar_entry(&entry_fn, name, &parameters, &result)
        };
        (entry_code, descriptor)
    } else {
        let args: Vec<_> = (0..4).map(|i| format_ident!("__arg{i}")).collect();
        let decoded = decode_parameters(&args, &parameters, quote!(false));
        let entry_code = quote! {
            unsafe extern "C" fn #entry_fn(
                #(#args: ::core::primitive::u64,)*
                __output: *mut ::core::ffi::c_void,
            ) -> bool {
                unsafe { __output.cast::<#ty>().write(#name(#(#decoded),*)); }
                true
            }
        };
        let descriptor = json!({
            "schema": 3, "kind": "typed_return_target", "logical_id": id, "symbol": symbol,
            "entry": entry, "parameters": parameter_names, "return_hook": return_hook,
        });
        (entry_code, descriptor)
    };
    let probe_code = type_probe(
        format_ident!("__mixin_target_type_{name}"),
        &probe,
        quote!(#ty),
    );
    let record = metadata(&format!("target:{id}"), descriptor);
    Ok(quote! {
        #body_code
        #[cfg(mixin_retained)]
        #[inline(never)]
        #[unsafe(export_name = #symbol)]
        #retained

        #[cfg(mixin_retained)]
        #[inline(never)]
        #[unsafe(export_name = #hook)]
        extern "C" fn #hook_fn(result: *mut ::core::ffi::c_void) {
            ::core::hint::black_box(result);
        }

        #[cfg(mixin_retained)]
        #[unsafe(export_name = #entry)]
        #entry_code
        #probe_code
        #record
    })
}

fn type_probe(
    name: syn::Ident,
    symbol: &str,
    ty: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        #[cfg(mixin_retained)]
        #[unsafe(export_name = #symbol)]
        unsafe extern "C" fn #name(__output: *mut ::core::ffi::c_void) {
            unsafe {
                __output.cast::<::core::any::TypeId>().write(::core::any::TypeId::of::<#ty>());
            }
        }
    }
}

fn concrete_return_type(output: &syn::ReturnType) -> syn::Result<proc_macro2::TokenStream> {
    match output {
        syn::ReturnType::Default => Ok(quote!(())),
        syn::ReturnType::Type(_, ty)
            if !matches!(ty.as_ref(), syn::Type::ImplTrait(_) | syn::Type::Never(_)) =>
        {
            Ok(quote!(#ty))
        }
        _ => Err(Error::new(
            output.span(),
            "Mixin return modification requires a concrete return type",
        )),
    }
}

fn scalar_entry(
    bridge: &syn::Ident,
    name: &syn::Ident,
    parameters: &[syn::Ident],
    result: &syn::Ident,
) -> proc_macro2::TokenStream {
    let args: Vec<_> = (0..4).map(|i| format_ident!("__arg{i}")).collect();
    let decoded = decode_parameters(&args, parameters, quote!(0));
    let body = if result == "unit" {
        quote! {
            let (): () = #name(#(#decoded),*);
            0
        }
    } else {
        let encoded = match result.to_string().as_str() {
            "f32" | "f64" => quote!(__value.to_bits() as ::core::primitive::u64),
            "entity_id" => quote!(__value.get() as ::core::primitive::u64),
            "option_entity_id" => quote!(match __value {
                ::core::option::Option::Some(id) => id.get() as ::core::primitive::u64,
                ::core::option::Option::None => 0,
            }),
            "vec2" => quote! {
                (__value.x.to_bits() as ::core::primitive::u64)
                    | ((__value.y.to_bits() as ::core::primitive::u64) << 32)
            },
            _ => quote!(__value as ::core::primitive::u64),
        };
        let result = rust_type(result);
        quote! {
            let __value: #result = #name(#(#decoded),*);
            #encoded
        }
    };
    quote! {
        pub extern "C" fn #bridge(#(#args: ::core::primitive::u64),*) -> ::core::primitive::u64 {
            #body
        }
    }
}

fn decode_parameters<'a>(
    args: &'a [syn::Ident],
    parameters: &'a [syn::Ident],
    invalid: proc_macro2::TokenStream,
) -> impl Iterator<Item = proc_macro2::TokenStream> + 'a {
    args
        .iter()
        .zip(parameters)
        .map(move |(arg, kind)| match kind.to_string().as_str() {
            "bool" => quote!(#arg != 0),
            "f32" => quote!(::core::primitive::f32::from_bits(#arg as ::core::primitive::u32)),
            "f64" => quote!(::core::primitive::f64::from_bits(#arg)),
            "entity_id" => quote!({
                let ::core::option::Option::Some(id) = ::mixin_sdk::EntityId::new(#arg as ::core::primitive::u32) else {
                    return #invalid;
                };
                id
            }),
            _ => quote!(#arg as ::core::primitive::#kind),
        })
}

fn string(value: &syn::Expr) -> syn::Result<String> {
    match value {
        syn::Expr::Lit(l) => match &l.lit {
            syn::Lit::Str(s) if !s.value().is_empty() => Ok(s.value()),
            _ => Err(Error::new(value.span(), "expected a nonempty string")),
        },
        _ => Err(Error::new(value.span(), "expected a string literal")),
    }
}

fn unsigned(value: &syn::Expr) -> syn::Result<usize> {
    match value {
        syn::Expr::Lit(l) => match &l.lit {
            syn::Lit::Int(n) => n.base10_parse(),
            _ => Err(Error::new(value.span(), "expected a nonnegative integer")),
        },
        _ => Err(Error::new(value.span(), "expected an integer literal")),
    }
}

fn expand_handler(metas: Vec<Meta>, function: ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    expand_handler_kind(metas, function, "modify_call_result", true)
}

fn expand_handler_kind(
    mut metas: Vec<Meta>,
    function: ItemFn,
    operation: &str,
    requires_callee: bool,
) -> syn::Result<proc_macro2::TokenStream> {
    let inject_head = operation == "inject_head";
    let continue_at_head = inject_head && returns_unit(&function.sig.output);
    let modify_return = operation == "modify_return_result";
    let (return_ty, kind, parameters) = if modify_return {
        // The former opt-in flag is redundant, but existing mod source can still build.
        if let Some(index) = metas
            .iter()
            .position(|meta| matches!(meta, Meta::Path(path) if path.is_ident("typed_return")))
        {
            metas.remove(index);
        }
        check_function(&function)?;
        (
            Some(concrete_return_type(&function.sig.output)?),
            None,
            Vec::new(),
        )
    } else {
        let parameters = parameters(&function, requires_callee)?;
        let kind = if inject_head && !continue_at_head {
            head_result(&function)?
        } else {
            result_type(&function)?
        };
        let expected_parameters = if kind == "unit" && requires_callee {
            &[][..]
        } else {
            std::slice::from_ref(&kind)
        };
        if !inject_head && parameters != expected_parameters {
            return Err(Error::new(
                function.sig.span(),
                "handler must use the same supported value type for its parameter and result, or fn() for a unit call/return",
            ));
        }
        (None, Some(kind), parameters)
    };
    let mut seen = BTreeSet::new();
    let (mut id, mut target, mut callee) = (None, None, None);
    let mut parameter = None;
    let (mut ordinal, mut require_matches, mut priority) = (0, 1, 0_i32);
    for meta in metas {
        let Meta::NameValue(nv) = meta else {
            return Err(Error::new(meta.span(), "expected key = value"));
        };
        let key = nv
            .path
            .get_ident()
            .ok_or_else(|| Error::new(nv.path.span(), "unknown key"))?
            .to_string();
        if !seen.insert(key.clone()) {
            return Err(Error::new(nv.span(), "duplicate key"));
        }
        match key.as_str() {
            "id" => id = Some(string(&nv.value)?),
            "target" => target = Some(string(&nv.value)?),
            "callee" if requires_callee => callee = Some(string(&nv.value)?),
            "callee" => {
                return Err(Error::new(
                    nv.path.span(),
                    "callee is only valid for modify_call_result",
                ));
            }
            "parameter" if operation == "modify_parameter" => {
                let index = unsigned(&nv.value)?;
                if index > 3 {
                    return Err(Error::new(nv.span(), "parameter must be in 0..=3"));
                }
                parameter = Some(index);
            }
            "parameter" => {
                return Err(Error::new(
                    nv.path.span(),
                    "parameter is only valid for modify_parameter",
                ));
            }
            "ordinal" if requires_callee || modify_return => ordinal = unsigned(&nv.value)?,
            "require_matches" if requires_callee || modify_return => {
                require_matches = unsigned(&nv.value)?
            }
            "ordinal" | "require_matches" => {
                return Err(Error::new(
                    nv.path.span(),
                    "ordinal and require_matches are not valid for this operation",
                ));
            }
            "priority" => {
                priority = i32::try_from(unsigned(&nv.value)?).map_err(|_| {
                    Error::new(nv.span(), "priority must fit i32 and be nonnegative")
                })?
            }
            _ => return Err(Error::new(nv.path.span(), "unknown Mixin attribute key")),
        }
    }
    if require_matches == 0 || ordinal >= require_matches {
        return Err(Error::new(
            function.span(),
            "require_matches must be positive and ordinal must be smaller",
        ));
    }
    if modify_return && require_matches != 1 {
        return Err(Error::new(
            function.sig.span(),
            "modify_return_result has one hook for all normal returns",
        ));
    }
    let missing = |key: &str| Error::new(function.sig.span(), format!("missing {key}"));
    let name = &function.sig.ident;
    let id = id.unwrap_or_else(|| {
        let crate_name = std::env::var("CARGO_CRATE_NAME").unwrap_or_default();
        if crate_name.is_empty() {
            name.to_string()
        } else {
            format!("{crate_name}::{name}")
        }
    });
    let target = target.ok_or_else(|| missing("target"))?;
    if operation == "modify_parameter" && parameter.is_none() {
        return Err(missing("parameter"));
    }
    let callee = if requires_callee {
        Some(callee.ok_or_else(|| missing("callee"))?)
    } else {
        None
    };
    let symbol = format!("__rsmx_handler_{}", hash(&id));
    let bridge = format_ident!("__mixin_handler_{}", name);
    let mut descriptor = json!({
            "schema":1,"kind":operation,"operation":operation,"id":id,"target":target,"callee":callee,
            "ordinal":ordinal,"require_matches":require_matches,"priority":priority,"symbol":symbol,
            "parameter":parameter
    });
    let (bridge_code, check) = if let Some(ty) = return_ty {
        let probe = format!("__rsmx_handler_type_{}", hash(&id));
        descriptor["schema"] = json!(3);
        descriptor["return_type_probe"] = json!(probe);
        let probe_code = type_probe(
            format_ident!("__mixin_handler_type_{name}"),
            &probe,
            quote!(#ty),
        );
        let (apply, check) = if function.sig.inputs.is_empty() {
            (quote!(#name();), quote!(const _: fn() = #name;))
        } else {
            (
                quote! {
                    let slot = __address.cast::<#ty>();
                    // The loader verifies the type before making the JIT callable.
                    unsafe { slot.write(#name(slot.read())); }
                },
                quote!(const _: fn(#ty) -> #ty = #name;),
            )
        };
        let code = quote! {
            unsafe extern "C" fn #bridge(__address: *mut ::core::ffi::c_void) {
                #apply
            }
            #probe_code
        };
        (code, check)
    } else {
        let kind = kind.expect("legacy handler type");
        let code = if inject_head {
            descriptor["schema"] = json!(2);
            descriptor["head_signature"] = json!({
                "parameters": parameters.iter().map(ToString::to_string).collect::<Vec<_>>(),
                "result": kind.to_string(),
            });
            head_bridge(&bridge, name, &parameters, &kind, continue_at_head)
        } else if kind == "f64" {
            quote! { pub extern "C" fn #bridge(value: f64) -> f64 { #name(value) } }
        } else if kind == "unit" {
            descriptor["schema"] = json!(2);
            descriptor["value_type"] = json!("unit");
            quote! { pub extern "C" fn #bridge() { #name() } }
        } else if kind == "vec2" {
            descriptor["schema"] = json!(2);
            descriptor["value_type"] = json!("vec2");
            // This bridge is called from LLVM with the pinned rustc's ABI.
            quote! {
                pub fn #bridge(x: f32, y: f32) -> ::mixin_sdk::Vec2 {
                    #name(::mixin_sdk::Vec2 { x, y })
                }
            }
        } else {
            descriptor["schema"] = json!(2);
            descriptor["value_type"] = json!(kind.to_string());
            let kind = rust_type(&kind);
            quote! {
                pub extern "C" fn #bridge(value: #kind) -> #kind {
                    #name(value)
                }
            }
        };
        (code, quote! {})
    };
    let record = metadata(&format!("handler:{id}"), descriptor);
    Ok(quote! {
        #[cfg_attr(not(mixin_retained), allow(dead_code))]
        #function
        #check
        #[cfg(mixin_retained)]
        #[unsafe(export_name = #symbol)]
        #bridge_code
        #record
    })
}

fn head_bridge(
    bridge: &syn::Ident,
    name: &syn::Ident,
    parameters: &[syn::Ident],
    result: &syn::Ident,
    continue_at_head: bool,
) -> proc_macro2::TokenStream {
    let args: Vec<_> = (0..parameters.len())
        .map(|i| format_ident!("__arg{i}"))
        .collect();
    let inputs = || {
        args.iter().zip(parameters).map(|(arg, kind)| {
            let kind = rust_type(kind);
            quote!(#arg: #kind)
        })
    };
    let input = inputs();
    if continue_at_head {
        quote! {
            pub extern "C" fn #bridge(#(#input),*) -> bool {
                #name(#(#args),*);
                false
            }
        }
    } else if result == "unit" {
        quote! {
            pub extern "C" fn #bridge(#(#input),*) -> bool {
                let decision: ::core::ops::ControlFlow<()> = #name(#(#args),*);
                decision.is_break()
            }
        }
    } else {
        let result = rust_type(result);
        quote! {
            unsafe extern "C" fn #bridge(#(#input,)* __output: *mut #result) -> bool {
                let decision: ::core::ops::ControlFlow<#result> = #name(#(#args),*);
                if let ::core::ops::ControlFlow::Break(value) = decision {
                    unsafe { __output.write(value) };
                    true
                } else {
                    false
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn handler_body_is_left_for_rustc_and_bad_attributes_are_rejected() {
        let f: ItemFn = syn::parse_quote!(
            fn adjust(x: f64) -> f64 {
                let values = [x, 2.0];
                values.iter().sum()
            }
        );
        let args: syn::punctuated::Punctuated<Meta, Token![,]> =
            syn::parse_quote!(id = "test", target = "g::a", callee = "g::b");
        let text = expand_handler(args.into_iter().collect(), f.clone())
            .unwrap()
            .to_string();
        assert!(text.contains("iter () . sum ()"));
        let bad: syn::punctuated::Punctuated<Meta, Token![,]> =
            syn::parse_quote!(id = "x", id = "y");
        assert!(expand_handler(bad.into_iter().collect(), f).is_err());
    }
    #[test]
    fn module_generates_logical_index_and_rejects_unsupported_boundary() {
        let module = syn::parse_quote!(
            mod combat {
                pub fn damage(x: f64) -> f64 {
                    x
                }
            }
        );
        let result = expand_targets("game::combat".into(), module)
            .unwrap()
            .to_string();
        assert!(result.contains("export_name"));
        assert!(result.contains("mixin_retained"));
        let invalid = syn::parse_quote!(
            mod combat {
                fn wrong(x: &mut f64) -> f64 {
                    *x
                }
            }
        );
        assert!(expand_targets("game::combat".into(), invalid).is_err());
    }

    #[test]
    fn head_handler_emits_explicit_head_operation_without_selector_fields() {
        let function: ItemFn = syn::parse_quote!(
            fn adjust(value: f64) -> f64 {
                value + 4.0
            }
        );
        let args: syn::punctuated::Punctuated<Meta, Token![,]> =
            syn::parse_quote!(id = "head", target = "demo::combat::apply_damage");
        let output =
            expand_handler_kind(args.into_iter().collect(), function, "modify_head", false)
                .unwrap()
                .to_string();
        assert!(output.contains("modify_head"));
        assert!(output.contains("__mixin_handler_adjust"));
    }
}

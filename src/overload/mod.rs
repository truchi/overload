mod errors;

use errors::*;

use proc_macro2::{Delimiter, Group, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    token::{Brace, Paren},
    Abi, AttrStyle, Attribute, Block, Expr, ExprMethodCall, ExprPath, FnArg, GenericParam,
    Generics, Ident, ImplItem, ImplItemMethod, ImplItemType, ItemImpl, ItemTrait, Pat, PatIdent,
    PatType, Path, PathArguments, PathSegment, QSelf, Receiver, ReturnType, Signature, Stmt, Token,
    TraitBound, TraitBoundModifier, TraitItem, TraitItemMethod, TraitItemType, Type, TypeParam,
    TypeParamBound, TypePath, TypeReference, TypeTuple, VisPublic, Visibility,
};

pub fn overload(item_impl: ItemImpl) -> syn::Result<TokenStream> {
    let mut tokens = TokenStream::new();

    dbg!(&item_impl);

    // First pass with validation
    // Creer le trait
    // Implementer le trait pour tous les overload

    // let ctx = dbg!(first_pass(&item_impl)).unwrap();
    // tokens.append_all([&declare_trait(ctx, &item_impl)]);
    // tokens.append_all(impls_trait(ctx, &item_impl));

    let overload = Overload::new(&item_impl)?;
    let (item_impl, rest) = overload.make();

    tokens.append_all([item_impl]);

    if let Some((item_trait, item_impls)) = rest {
        tokens.append_all([item_trait]);
        tokens.append_all(item_impls);
    }

    Ok(tokens)
}

type Out = (ItemImpl, Option<(ItemTrait, Vec<ItemImpl>)>);

#[derive(Clone, Debug)]
struct Overload {
    item_impl: ItemImpl,
    first: Option<ImplItemMethod>,
}

impl Overload {
    fn new(item_impl: &ItemImpl) -> Result<Self, syn::Error> {
        if let Some(error) = Errors::new(item_impl).into_syn_error() {
            return Err(error);
        }

        Ok(Self {
            first: item_impl.items.iter().find_map(|item| {
                if let ImplItem::Method(method) = item {
                    Some(method.clone())
                } else {
                    None
                }
            }),
            item_impl: item_impl.clone(),
        })
    }

    fn make(&self) -> Out {
        let mut item_impl = self.item_impl.clone();

        let first = if let Some(first) = &self.first {
            first
        } else {
            return (item_impl, None);
        };

        item_impl
            .items
            .iter_mut()
            .filter_map(|item| {
                if let ImplItem::Method(method) = item {
                    Some(method)
                } else {
                    None
                }
            })
            .enumerate()
            .for_each(|(i, method)| {
                // Mangle ident
                let ident = &mut method.sig.ident;
                *ident = Ident::new(&format!("__{ident}_{i}"), ident.span());

                // Add #[doc(hidden)]
                method.attrs.push(parse_quote! { #[doc(hidden)] });
            });

        let ImplItemMethod {
            attrs,
            vis,
            sig:
                Signature {
                    constness,
                    asyncness,
                    unsafety,
                    abi,
                    ident: fun,
                    inputs,
                    ..
                },
            ..
        } = first;
        let receiver = inputs.first().and_then(|input| match input {
            FnArg::Receiver(receiver) => Some(receiver),
            _ => None,
        });
        let args_trait = to_trait_ident(fun);
        let args_generic = ident("T"); // TODO no conflicts

        let method: ImplItemMethod = parse_quote! {
            #vis #constness #unsafety #abi
            fn #fun<#args_generic: #args_trait>(#receiver, args: #args_generic) -> #args_generic::Output {
                args.#fun(self)
            }
        };
        item_impl.items.push(method.into());

        item_impl.items.push(ImplItem::Method(ImplItemMethod {
            attrs: default(),
            vis: vis.clone(),
            defaultness: None,
            sig: Signature {
                constness: *constness,
                asyncness: *asyncness,
                unsafety: *unsafety,
                abi: abi.clone(),
                fn_token: default(),
                ident: fun.clone(),
                generics: Generics {
                    lt_token: Some(default()),
                    params: [GenericParam::Type(TypeParam {
                        attrs: default(),
                        ident: args_generic.clone(),
                        colon_token: Some(default()),
                        bounds: punctuated([TypeParamBound::Trait(TraitBound {
                            paren_token: None,
                            modifier: TraitBoundModifier::None,
                            lifetimes: None,
                            path: Path {
                                leading_colon: None,
                                segments: punctuated([PathSegment {
                                    ident: args_trait.clone(),
                                    arguments: default(),
                                }]),
                            },
                        })]),
                        eq_token: None,
                        default: None,
                    })]
                    .into_iter()
                    .collect(),
                    gt_token: Some(default()),
                    where_clause: None,
                },
                paren_token: default(),
                inputs: punctuated(receiver.cloned().map(FnArg::Receiver).into_iter().chain([
                    FnArg::Typed(PatType {
                        attrs: default(),
                        pat: Box::new(Pat::Ident(PatIdent {
                            attrs: default(),
                            by_ref: None,
                            mutability: None,
                            ident: ident("args"),
                            subpat: None,
                        })),
                        colon_token: default(),
                        ty: Box::new(Type::Path(TypePath {
                            qself: None,
                            path: Path {
                                leading_colon: None,
                                segments: punctuated([PathSegment {
                                    ident: args_generic.clone(),
                                    arguments: default(),
                                }]),
                            },
                        })),
                    }),
                ])),
                variadic: None,
                output: ReturnType::Type(
                    default(),
                    Box::new(Type::Path(TypePath {
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments: punctuated([
                                PathSegment {
                                    ident: args_generic,
                                    arguments: default(),
                                },
                                PathSegment {
                                    ident: ident("Output"),
                                    arguments: default(),
                                },
                            ]),
                        },
                    })),
                ),
            },
            block: Block {
                brace_token: default(),
                stmts: vec![Stmt::Expr(Expr::MethodCall(ExprMethodCall {
                    attrs: default(),
                    receiver: Box::new(Expr::Path(ExprPath {
                        attrs: default(),
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments: punctuated([PathSegment {
                                ident: ident("args"),
                                arguments: default(),
                            }]),
                        },
                    })),
                    dot_token: default(),
                    method: fun.clone(),
                    turbofish: None,
                    paren_token: default(),
                    args: punctuated([Expr::Path(ExprPath {
                        attrs: default(),
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments: punctuated([PathSegment {
                                ident: ident("self"),
                                arguments: default(),
                            }]),
                        },
                    })]),
                }))],
            },
        }));

        // =====================================================================

        let self_ty = &self.item_impl.self_ty;

        let item_trait = parse_quote! {
            #vis trait #args_trait {
                type Output;

                #constness #unsafety #abi
                fn #fun(self, stuff: #self_ty) -> Self::Output;
            }
        };

        // =============================

        let item_impls = vec![];

        (item_impl, Some((item_trait, item_impls)))
    }
}

// ============================================================================================== //

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct Context<'a> {
    function_name: &'a Ident,
    receiver: Option<&'a Receiver>,
    vis: &'a Visibility,
    constness: &'a Option<Token![const]>,
    asyncness: &'a Option<Token![async]>,
    unsafety: &'a Option<Token![unsafe]>, // We should disallow `unsafe impl Overloaded {}` actually
}

impl<'a> From<&'a ImplItemMethod> for Context<'a> {
    fn from(method: &'a ImplItemMethod) -> Self {
        Self {
            function_name: &method.sig.ident,
            receiver: method.sig.inputs.first().and_then(|input| {
                if let FnArg::Receiver(receiver) = input {
                    Some(receiver)
                } else {
                    None
                }
            }),
            vis: &method.vis,
            constness: &method.sig.constness,
            asyncness: &method.sig.asyncness,
            unsafety: &method.sig.unsafety,
        }
    }
}

fn first_pass(item_impl: &ItemImpl) -> Result<Context, ()> {
    item_impl
        .items
        .iter()
        .filter_map(|item| {
            if let ImplItem::Method(method) = item {
                Some(method)
            } else {
                None
            }
        })
        .try_fold(None, |ctx, item| {
            let ctx2 = Some(Context::from(item));

            if ctx.is_none() || ctx == ctx2 {
                Ok(ctx2)
            } else {
                Err(())
            }
        })?
        .ok_or(())
}

fn declare_trait(ctx: Context, item_impl: &ItemImpl) -> ItemTrait {
    ItemTrait {
        attrs: item_impl.attrs.clone(),
        vis: ctx.vis.clone(),
        unsafety: None,
        auto_token: None,
        trait_token: Token![trait](span()),
        ident: Ident::new("FunNameArgs", span()),
        generics: item_impl.generics.clone(), // Also use generics from function?
        colon_token: None,
        supertraits: default(),
        brace_token: Brace { span: span() },
        items: vec![
            TraitItemType {
                attrs: vec![],
                type_token: Token![type](span()),
                ident: Ident::new("Output", span()),
                generics: default(),
                colon_token: None,
                bounds: default(),
                default: None,
                semi_token: Token![;](span()),
            }
            .into(),
            TraitItemMethod {
                attrs: vec![],
                sig: Signature {
                    constness: *ctx.constness,
                    asyncness: *ctx.asyncness,
                    unsafety: *ctx.unsafety,
                    abi: None,
                    fn_token: Token![fn](span()),
                    ident: ctx.function_name.clone(),
                    generics: default(),
                    paren_token: Paren { span: span() },
                    inputs: {
                        let receiver = Receiver {
                            attrs: vec![],
                            reference: None,
                            mutability: None,
                            self_token: Token![self](span()),
                        };

                        if let Some(overloaded) = ctx.receiver {
                            let overloaded = PatType {
                                attrs: vec![],
                                pat: Box::new(
                                    PatIdent {
                                        attrs: overloaded.attrs.clone(),
                                        by_ref: None,
                                        mutability: None,
                                        ident: Ident::new("todo", span()),
                                        subpat: None,
                                    }
                                    .into(),
                                ),
                                colon_token: Token![:](span()),
                                ty: if let Some(reference) = overloaded.reference.as_ref() {
                                    Box::new(Type::Reference(TypeReference {
                                        and_token: reference.0,
                                        lifetime: None, // We can do things here
                                        mutability: overloaded.mutability,
                                        elem: item_impl.self_ty.clone(),
                                    }))
                                } else {
                                    item_impl.self_ty.clone()
                                },
                            };

                            [FnArg::Receiver(receiver), FnArg::Typed(overloaded)]
                                .into_iter()
                                .collect()
                        } else {
                            [FnArg::Receiver(receiver)].into_iter().collect()
                        }
                    },
                    variadic: None,
                    output: ReturnType::Default,
                },
                default: None,
                semi_token: Some(Token![;](span())),
            }
            .into(),
        ],
    }
}

fn impls_trait<'a>(
    ctx: Context<'a>,
    item_impl: &'a ItemImpl,
) -> impl 'a + Iterator<Item = ItemImpl> {
    item_impl
        .items
        .iter()
        .flat_map(|item| {
            if let ImplItem::Method(method) = item {
                Some(method)
            } else {
                None
            }
        })
        .map(move |method| impl_trait(ctx, item_impl, method))
}

fn impl_trait(ctx: Context, item_impl: &ItemImpl, method: &ImplItemMethod) -> ItemImpl {
    ItemImpl {
        attrs: item_impl.attrs.clone(),
        defaultness: None,
        unsafety: None,
        impl_token: Token![impl](span()),
        generics: default(),
        trait_: Some((
            None,
            Ident::new("FunNameArgs", span()).into(),
            Token![for](span()),
        )),
        self_ty: item_impl.self_ty.clone(),
        brace_token: Brace { span: span() },
        items: vec![ImplItem::Type(ImplItemType {
            attrs: item_impl.attrs.clone(),
            vis: Visibility::Inherited,
            defaultness: None,
            type_token: Token![type](span()),
            ident: Ident::new("Output", span()),
            generics: default(),
            eq_token: Token![=](span()),
            ty: match &method.sig.output {
                ReturnType::Default => Type::Tuple(TypeTuple {
                    paren_token: Paren { span: span() },
                    elems: default(),
                }),
                ReturnType::Type(_, ty) => (**ty).clone(),
            },
            semi_token: Token![;](span()),
        })],
    }
}

// ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓ //
// ┃                                            Utils                                           ┃ //
// ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛ //

fn default<T: Default>() -> T {
    T::default()
}

fn span() -> Span {
    Span::call_site()
}

fn ident(name: &str) -> Ident {
    Ident::new(name, span())
}

fn to_trait_ident(ident: &Ident) -> Ident {
    Ident::new(
        &heck::ToPascalCase::to_pascal_case(format!("{ident}Args").as_str()),
        span(),
    )
}

fn to_snake_case(ident: &Ident) -> Ident {
    Ident::new(
        &heck::ToSnakeCase::to_snake_case(ident.to_string().as_str()),
        span(),
    )
}

fn punctuated<T, U: Default, I: IntoIterator<Item = T>>(items: I) -> Punctuated<T, U> {
    items.into_iter().collect()
}

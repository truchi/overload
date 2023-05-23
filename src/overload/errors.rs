use super::*;

#[derive(Clone, Default, Debug)]
pub struct Errors {
    // Forbidden:
    defaultness: (Vec<Token![default]>, bool),

    // Coherency:
    visibility: (Vec<Visibility>, bool),
    constness: (Vec<Option<Token![const]>>, bool),
    asyncness: (Vec<Option<Token![async]>>, bool),
    unsafety: (Vec<Option<Token![unsafe]>>, bool),
    abi: (Vec<Option<Abi>>, bool),
    ident: (Vec<Ident>, bool),
    receiver: (
        Vec<Option<(Option<Token![&]>, Option<Token![mut]>, Token![self])>>,
        bool,
    ),
}

impl Errors {
    pub const DEFAULTNESS: &str = "Overload: Forbidden default";
    pub const VISIBILITY: &str = "Overload: Incoherent visibility";
    pub const CONSTNESS: &str = "Overload: Incoherent constness";
    pub const ASYNCNESS: &str = "Overload: Incoherent asyncness";
    pub const UNSAFETY: &str = "Overload: Incoherent unsafety";
    pub const ABI: &str = "Overload: Incoherent abi";
    pub const IDENT: &str = "Overload: Incoherent identifier";

    pub fn new(item_impl: &ItemImpl) -> Self {
        let mut errors = Self::default();

        for method in item_impl.items.iter().filter_map(|item| {
            if let ImplItem::Method(method) = item {
                Some(method)
            } else {
                None
            }
        }) {
            // Defaultness
            if let Some(defaultness) = method.defaultness {
                errors.defaultness.1 = true;
                errors.defaultness.0.push(defaultness);
            }

            // Visibility
            let visibility = &method.vis;
            if let Some(last) = errors.visibility.0.last() {
                if last != visibility {
                    errors.visibility.1 = true;
                }
            }
            errors.visibility.0.push(visibility.clone());

            // Constness
            let constness = method.sig.constness;
            if let Some(last) = errors.constness.0.last() {
                if last.is_some() != constness.is_some() {
                    errors.constness.1 = true;
                }
            }
            errors.constness.0.push(constness);

            // Asyncness
            let asyncness = method.sig.asyncness;
            if let Some(last) = errors.asyncness.0.last() {
                if last.is_some() != asyncness.is_some() {
                    errors.asyncness.1 = true;
                }
            }
            errors.asyncness.0.push(asyncness);

            // Unsafety
            let unsafety = method.sig.unsafety;
            if let Some(last) = errors.unsafety.0.last() {
                if last.is_some() != method.sig.unsafety.is_some() {
                    errors.unsafety.1 = true;
                }
            }
            errors.unsafety.0.push(unsafety);

            // Abi
            let abi = &method.sig.abi;
            if let Some(last) = errors.abi.0.last() {
                if last
                    .as_ref()
                    .map(|abi| abi.name.as_ref().map(|name| name.value()))
                    != abi
                        .as_ref()
                        .map(|abi| abi.name.as_ref().map(|name| name.value()))
                {
                    errors.abi.1 = true;
                }
            }
            errors.abi.0.push(abi.clone());

            // Ident
            let ident = &method.sig.ident;
            if let Some(last) = errors.ident.0.last() {
                if last.to_string() != ident.to_string() {
                    errors.ident.1 = true;
                }
            }
            errors.ident.0.push(ident.clone());

            // Receiver
            let receiver = method.sig.inputs.first().and_then(|input| match input {
                FnArg::Receiver(receiver) => Some((
                    receiver.reference.as_ref().map(|reference| reference.0),
                    receiver.mutability,
                    receiver.self_token,
                )),
                FnArg::Typed(_) => None,
            });
            if let Some(last) = errors.receiver.0.last() {
                if last != &receiver {
                    errors.receiver.1 = true;
                }
            }
            errors.receiver.0.push(receiver);
        }

        errors
    }

    pub fn into_syn_error(self) -> Option<syn::Error> {
        let mut errors = vec![];

        if self.defaultness.1 {
            errors.extend(
                self.defaultness
                    .0
                    .into_iter()
                    .map(|defaultness| (defaultness.span, Self::DEFAULTNESS)),
            );
        }

        if self.visibility.1 {
            errors.extend(self.visibility.0.into_iter().filter_map(
                |visibility| match visibility {
                    Visibility::Public(visibility) => {
                        Some((visibility.pub_token.span, Self::VISIBILITY))
                    }
                    Visibility::Crate(visibility) => {
                        Some((visibility.crate_token.span, Self::VISIBILITY))
                    }
                    Visibility::Restricted(visibility) => {
                        Some((visibility.pub_token.span, Self::VISIBILITY))
                    }
                    Visibility::Inherited => None,
                },
            ));
        }

        if self.constness.1 {
            errors.extend(self.constness.0.into_iter().filter_map(|constness| {
                constness.map(|constness| (constness.span, Self::CONSTNESS))
            }));
        }

        if self.asyncness.1 {
            errors.extend(self.asyncness.0.into_iter().filter_map(|asyncness| {
                asyncness.map(|asyncness| (asyncness.span, Self::ASYNCNESS))
            }));
        }

        if self.unsafety.1 {
            errors.extend(
                self.unsafety.0.into_iter().filter_map(|unsafety| {
                    unsafety.map(|unsafety| (unsafety.span, Self::UNSAFETY))
                }),
            );
        }

        if self.abi.1 {
            errors.extend(
                self.abi
                    .0
                    .into_iter()
                    .filter_map(|abi| abi.map(|abi| (abi.extern_token.span, Self::ABI))),
            );
        }

        if self.ident.1 {
            errors.extend(
                self.ident
                    .0
                    .into_iter()
                    .map(|ident| (ident.span(), Self::IDENT)),
            );
        }

        let mut errors = errors.into_iter();
        let mut syn_error = {
            let error = errors.next()?;
            syn::Error::new(error.0, error.1)
        };

        for error in errors {
            syn_error.combine(syn::Error::new(error.0, error.1));
        }

        Some(syn_error)
    }

    pub fn has_errors(&self) -> bool {
        self.defaultness.1
            || self.visibility.1
            || self.constness.1
            || self.asyncness.1
            || self.unsafety.1
            || self.abi.1
            || self.ident.1
            || self.receiver.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaultness() {
        let errors = Errors::new(&parse_quote! {
            impl Foo {
                default fn foo(&self) {}
            }
        });
        assert!(errors.has_errors());
        assert!(errors.defaultness.1);
    }

    #[test]
    fn visibility() {
        let errors = Errors::new(&parse_quote! {
            impl Foo {
                pub fn foo(&self) {}
                    fn foo(&self) {}
            }
        });
        assert!(errors.has_errors());
        assert!(errors.visibility.1);

        let errors = Errors::new(&parse_quote! {
            impl Foo {
                pub(crate) fn foo(&self) {}
                pub(super) fn foo(&self) {}
            }
        });
        assert!(errors.has_errors());
        assert!(errors.visibility.1);
    }

    #[test]
    fn constness() {
        let errors = Errors::new(&parse_quote! {
            impl Foo {
                const fn foo(&self) {}
                      fn foo(&self) {}
            }
        });
        assert!(errors.has_errors());
        assert!(errors.constness.1);
    }

    #[test]
    fn asyncness() {
        let errors = Errors::new(&parse_quote! {
            impl Foo {
                async fn foo(&self) {}
                      fn foo(&self) {}
            }
        });
        assert!(errors.has_errors());
        assert!(errors.asyncness.1);
    }

    #[test]
    fn unsafety() {
        let errors = Errors::new(&parse_quote! {
            impl Foo {
                unsafe fn foo(&self) {}
                       fn foo(&self) {}
            }
        });
        assert!(errors.has_errors());
        assert!(errors.unsafety.1);
    }

    #[test]
    fn abi() {
        let errors = Errors::new(&parse_quote! {
            impl Foo {
                extern fn foo(&self) {}
                       fn foo(&self) {}
            }
        });
        assert!(errors.has_errors());
        assert!(errors.abi.1);

        let errors = Errors::new(&parse_quote! {
            impl Foo {
                extern "C" fn foo(&self) {}
                           fn foo(&self) {}
            }
        });
        assert!(errors.has_errors());
        assert!(errors.abi.1);

        let errors = Errors::new(&parse_quote! {
            impl Foo {
                extern "C" fn foo(&self) {}
                extern "D" fn foo(&self) {}
            }
        });
        assert!(errors.has_errors());
        assert!(errors.abi.1);
    }

    #[test]
    fn ident() {
        let errors = Errors::new(&parse_quote! {
            impl Foo {
                fn foo(&self) {}
                fn bar(&self) {}
            }
        });
        assert!(errors.has_errors());
        assert!(errors.ident.1);
    }

    #[test]
    fn receiver() {
        let errors = Errors::new(&parse_quote! {
            impl Foo {
                fn foo(self) {}
                fn foo(    ) {}
            }
        });
        assert!(errors.has_errors());
        assert!(errors.receiver.1);

        let errors = Errors::new(&parse_quote! {
            impl Foo {
                fn foo(&self) {}
                fn foo( self) {}
            }
        });
        assert!(errors.has_errors());
        assert!(errors.receiver.1);
    }

    #[test]
    fn all() {
        let errors = Errors::new(&parse_quote! {
            impl Foo {
                pub default const async fn foo(&self) {}
                unsafe extern "C"       fn bar( self) {}
            }
        });
        assert!(errors.has_errors());
        assert!(errors.defaultness.1);
        assert!(errors.visibility.1);
        assert!(errors.constness.1);
        assert!(errors.asyncness.1);
        assert!(errors.unsafety.1);
        assert!(errors.abi.1);
        assert!(errors.ident.1);
        assert!(errors.receiver.1);
    }
}

//! `#[cfg_attr(fptrace, melee_fptrace::trace)]` on a game function: in a trace build (`RUSTFLAGS=--cfg fptrace`) every
//! `a + b`, `a - b`, `a * b`, `a / b` (and the compound assignments) in its body becomes a `melee_core::fptrace` call
//! that computes the same value and logs it when the operands are f32 / f64, and every `e as f32` / `e as f64` a logged
//! conversion (the frsp points). The body is wrapped in a scope guard that turns the log on for the call the trace
//! wants (MELEE_FPTRACE_FN / _FRAME). Normal builds never expand it (the cfg is off), so they are unchanged.
//! Matched against the Gekko interpreter's log by tools/rust-port/fptrace/match.py.
use proc_macro::TokenStream;
use quote::quote;
use syn::visit_mut::{self, VisitMut};
use syn::{parse_macro_input, parse_quote, BinOp, Expr, ItemFn, Type};

struct Rw;

fn op_fn(op: &BinOp) -> Option<(&'static str, bool)> {
    Some(match op {
        BinOp::Add(_) => ("add", false),
        BinOp::Sub(_) => ("sub", false),
        BinOp::Mul(_) => ("mul", false),
        BinOp::Div(_) => ("div", false),
        BinOp::AddAssign(_) => ("add", true),
        BinOp::SubAssign(_) => ("sub", true),
        BinOp::MulAssign(_) => ("mul", true),
        BinOp::DivAssign(_) => ("div", true),
        _ => return None,
    })
}

impl VisitMut for Rw {
    fn visit_expr_mut(&mut self, e: &mut Expr) {
        visit_mut::visit_expr_mut(self, e);
        match e {
            Expr::Binary(b) => {
                if let Some((f, assign)) = op_fn(&b.op) {
                    let f = syn::Ident::new(f, proc_macro2::Span::call_site());
                    let (l, r) = (&b.left, &b.right);
                    let opn = f.to_string();
                    let opsym = &b.op;
                    *e = if assign {
                        // primitives evaluate the right operand before the place, as here
                        parse_quote!({
                            let __r = #r;
                            let __p = &mut #l;
                            let (__pb, __rb) = (::melee_core::fptrace::bits(&*__p), ::melee_core::fptrace::bits(&__r));
                            *__p #opsym __r;
                            ::melee_core::fptrace::log2(#opn, __pb, __rb, ::melee_core::fptrace::bits(&*__p));
                        })
                    } else {
                        // the real operator stays, so literal and operand type inference is the original's
                        parse_quote!(match (#l, #r) {
                            (__a, __b) => {
                                let (__ab, __bb) = (::melee_core::fptrace::bits(&__a), ::melee_core::fptrace::bits(&__b));
                                let __v = __a #opsym __b;
                                ::melee_core::fptrace::log2(#opn, __ab, __bb, ::melee_core::fptrace::bits(&__v));
                                __v
                            }
                        })
                    };
                }
            }
            Expr::Cast(c) => {
                if let Type::Path(p) = &*c.ty {
                    if p.path.is_ident("f32") || p.path.is_ident("f64") {
                        let inner = &c.expr;
                        let ty = &c.ty;
                        *e = parse_quote!({
                            let __c = #inner;
                            let __o = __c as #ty;
                            ::melee_core::fptrace::log_cast(&__c, ::melee_core::fptrace::bits(&__o));
                            __o
                        });
                    }
                }
            }
            _ => {}
        }
    }
    // const / static initialisers and nested fns stay as they are
    fn visit_item_mut(&mut self, _i: &mut syn::Item) {}
}

#[proc_macro_attribute]
pub fn trace(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut f = parse_macro_input!(item as ItemFn);
    Rw.visit_block_mut(&mut f.block);
    let name = f.sig.ident.to_string();
    let block = &f.block;
    f.block = parse_quote!({ let __fptrace_guard = ::melee_core::fptrace::enter(#name); #block });
    quote!(#f).into()
}

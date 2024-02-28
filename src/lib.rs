use serde::Deserialize;
use swc_core::common::{BytePos, Spanned};
use swc_core::ecma::ast::{CallExpr, Callee, Expr, ExprOrSpread, Lit, MemberProp, Program};
use swc_core::ecma::visit::{VisitMut, VisitMutWith};
use swc_core::common::{comments::{Comment, Comments, CommentKind}, DUMMY_SP};
use swc_core::plugin::{plugin_transform, proxies::{TransformPluginProgramMetadata, PluginCommentsProxy}};

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    title: String,
    functions: Vec<String>,
    objects: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            title: "markExpression".into(),
            functions: vec!["markedFunction".into()],
            objects: vec!["window".into()],
        }
    }
}

pub struct MarkExpression<'a, C: Comments> {
    comments: C,
    config: &'a Config,
}

struct MarkValue<'a> {
    pos: BytePos,
    value: &'a str,
}

impl<'a, C: Comments> MarkExpression<'a, C> {
    pub fn new(comments: C, config: &'a Config) -> Self {
        Self {
            comments,
            config,
        }
    }

    fn check_fn_call(&self, fn_name: &str) -> bool {
        self.config.functions.iter().any(|function| function == fn_name)
    }

    fn check_obj(&self, obj_name: &str) -> bool {
        self.config.objects.iter().any(|object| object == obj_name)
    }
}

impl<'a, C: Comments> VisitMut for MarkExpression<'a, C> {
    fn visit_mut_call_expr(&mut self, call_expr: &mut CallExpr) {
        call_expr.visit_mut_children_with(self);
        let mut maybe_lang_value: Option<MarkValue> = None;
        if let Callee::Expr(callee) = &call_expr.callee {
            let mut maybe_first_arg: Option<&ExprOrSpread> = None;

            if let Expr::Member(member_expr) = callee.as_ref() {
                if let MemberProp::Computed(prop) = &member_expr.prop { // window['markedFunction']
                    if let Expr::Lit(prop_literal) = prop.expr.as_ref() {
                        if let Lit::Str(prop) = prop_literal {
                            let fn_name: &str = &prop.value;
                            if self.check_fn_call(fn_name) {
                                if let Expr::Ident(obj) = &member_expr.obj.as_ref() {
                                    let obj_name: &str = &obj.sym;
                                    if self.check_obj(obj_name) {
                                        maybe_first_arg = call_expr.args.first();
                                    }
                                }
                            }
                        }
                    }
                }
                if let MemberProp::Ident(prop) = &member_expr.prop { // window.markedFunction(...)
                    let func_name: &str = &prop.sym;
                    if self.check_fn_call(func_name) {
                        if let Expr::Ident(obj) = &member_expr.obj.as_ref() {
                            let obj_name: &str = &obj.sym;
                            if self.check_obj(obj_name) {
                                maybe_first_arg = call_expr.args.first();
                            }
                        }
                    }
                }
            }

            if let Expr::Ident(func_ident) = callee.as_ref() { // markedFunction(...)
                let func_name: &str = &func_ident.sym;
                if self.check_fn_call(func_name) {
                    maybe_first_arg = call_expr.args.first();
                }
            }

            if let Some(first_arg) = maybe_first_arg {
                if let Expr::Lit(literal) = first_arg.expr.as_ref() { // markedFunction('str')
                    if let Lit::Str(s) = literal {
                        let value: &str = &s.value;
                        maybe_lang_value = MarkValue {
                            pos: literal.span_lo(),
                            value,
                        }.into();
                    }
                }
            }
        }

        if let Some(lang_value) = maybe_lang_value {
            self.comments.add_leading(lang_value.pos, Comment {
                span: DUMMY_SP,
                kind: CommentKind::Block,
                text: format!(" {}: {} ", self.config.title, lang_value.value).into(),
            });
        }
    }
}

#[plugin_transform]
pub fn process_transform(mut program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let config_str = &metadata.get_transform_plugin_config().expect("Failed to resolve config");
    let config = serde_json::from_str::<Option<Config>>(config_str.as_str()).expect("Invalid config").unwrap_or_default();

    let comments = match metadata.comments {
        Some(comments) => comments.clone(),
        None => PluginCommentsProxy,
    };

    program.visit_mut_with(&mut MarkExpression::new(comments, &config));
    program
}

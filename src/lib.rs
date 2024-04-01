use std::collections::{HashMap, HashSet};
use serde::Deserialize;
use regex::Regex;
use swc_core::common::{BytePos, Spanned};
use swc_core::ecma::ast::{CallExpr, Callee, Expr, Ident, Lit, MemberExpr, MemberProp, Module, Program, SpanExt};
use swc_core::ecma::visit::{VisitMut, VisitMutWith};
use swc_core::common::comments::{Comment, Comments, CommentKind};
use swc_core::plugin::{plugin_transform, proxies::{TransformPluginProgramMetadata, PluginCommentsProxy}};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    title: String,
    functions: Vec<String>,
    methods: HashMap<String, Vec<String>>,
    dynamic_imports: Vec<String>,
}

pub struct MarkExpression<C: Comments> {
    comments: C,
    title: String,
    functions: HashSet<String>,
    methods: HashMap<String, HashSet<String>>,
    dynamic_imports: Option<Regex>,
    results: Vec<String>,
}

impl<C: Comments> MarkExpression<C> {
    pub fn new(comments: C, config: &Config) -> Self {
        let functions = HashSet::from_iter(config.functions.iter().cloned());
        let methods = config.methods.iter().map(|(k, v)| (k.clone(), HashSet::from_iter(v.iter().cloned()))).collect();

        if config.dynamic_imports.is_empty() {
            return Self {
                title: config.title.clone(),
                comments,
                functions,
                methods,
                dynamic_imports: None,
                results: Vec::new(),
            };
        }

        let regex_str = config.dynamic_imports.join("|");
        let dynamic_imports = Regex::new(regex_str.as_str()).ok();

        Self {
            title: config.title.clone(),
            comments,
            functions,
            methods,
            dynamic_imports,
            results: Vec::new(),
        }
    }

    fn check_fn_call(&self, callee: &Ident) -> Option<String> {
        let name = &callee.sym;
        if self.functions.contains(name.as_str()) {
            return Some(name.as_str().into());
        }
        None
    }

    fn check_method_call(&self, callee: &MemberExpr) -> Option<(String, String)> {
        let (obj_name, methods) = match &callee.obj.as_ref() {
            Expr::Ident(obj) => {
                let name = obj.sym.as_str();
                (name, self.methods.get(name))
            }
            Expr::This(_) => ("this", self.methods.get("this")),
            _ => ("", None)
        };
        if let Some(methods) = methods {
            if let MemberProp::Ident(prop) = &callee.prop {
                let name = &prop.sym;
                if methods.contains(name.as_str()) {
                    return Some((obj_name.into(), name.as_str().into()));
                }
            }
        }
        None
    }

    fn check_dynamic_import(&self, pos: BytePos) -> Option<String> {
        let mut result: Vec<String> = Vec::new();
        let mut found = if let Some(_) = &self.dynamic_imports { false } else { true };
        self.comments.with_leading(pos, |comments| {
            for comment in comments {
                if let CommentKind::Block = comment.kind {
                    let text = comment.text.as_str();
                    result.push(text.trim().to_string().replace("\n", ""));
                    if !found {
                        if let Some(regex) = &self.dynamic_imports {
                            if regex.is_match(text) {
                                found = true;
                            }
                        }
                    }
                }
            }
            false
        });

        if found {
            return Some(result.join(", "));
        }

        None
    }

    fn get_args(&self, call_expr: &CallExpr) -> String {
        let mut args = Vec::new();
        for arg in &call_expr.args {
            if let Expr::Lit(lit) = arg.expr.as_ref() {
                match lit {
                    Lit::Str(str_lit) => {
                        let val = format!("\"{}\"", str_lit.value);
                        args.push(val);
                    },
                    Lit::Num(num_lit) => {
                        args.push(num_lit.to_string());
                    },
                    Lit::Bool(bool_lit) => {
                        args.push(bool_lit.value.to_string());
                    },
                    _ => { args.push("null".into()); }
                }
            } else {
                args.push("null".into());
            }
        }
        args.join(", ")
    }
}

impl<C: Comments> VisitMut for MarkExpression<C> {
    fn visit_mut_module(&mut self, module: &mut Module) {
        let span = module.comment_range();

        module.visit_mut_children_with(self);
        if !self.results.is_empty() {
            let text = format!(
                "---BEGIN {}–-- \n[\n    {}\n]\n ---END {}---",
                self.title,  
                self.results.join(",\n    "),
                self.title,
            );
            
            self.comments.add_leading(module.span_lo(), Comment {
                span,
                kind: CommentKind::Block,
                text: text.into(),
            });
        }
    }

    fn visit_mut_call_expr(&mut self, call_expr: &mut CallExpr) {
        call_expr.visit_mut_children_with(self);
        match &call_expr.callee {
            Callee::Expr(callee) => {
                match callee.as_ref() {
                    Expr::Ident(ident) => {
                        if let Some(name) = self.check_fn_call(ident) {
                            let args = format!("[\"{}\", [{}]]", name, self.get_args(call_expr));
                            self.results.push(args);
                        }
                    },
                    Expr::Member(member_expr) => {
                        if let Some((obj_name, fn_name)) = self.check_method_call(member_expr) {
                            let args = format!("[\"{}\", \"{}\", [{}]]", fn_name, obj_name, self.get_args(call_expr));
                            self.results.push(args);
                        }
                    },
                    _ => {}
                }
            },
            Callee::Import(callee) => {
                let first_arg = call_expr.args.first();
                let pos = first_arg.map_or(callee.span_lo(), |arg| arg.span_lo());
                if let Some(magic_comments) = self.check_dynamic_import(pos) {
                    let args = self.get_args(call_expr);
                    let args = format!("[\"import\", {{ {} }}, [{}]]", magic_comments, args);
                    self.results.push(args);
                }
            },
            _ => {}
        }
    }
}

#[plugin_transform]
pub fn process_transform(mut program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let config_str = &metadata.get_transform_plugin_config().expect("Failed to resolve config");
    let config = serde_json::from_str::<Option<Config>>(config_str.as_str()).expect("Invalid config").unwrap();

    let comments = match metadata.comments {
        Some(comments) => comments.clone(),
        None => PluginCommentsProxy,
    };

    program.visit_mut_with(&mut MarkExpression::new(comments, &config));
    program
}

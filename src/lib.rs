use jsonc_parser::parse_to_serde_value;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use swc_core::common::comments::{Comment, CommentKind, Comments};
use swc_core::common::{BytePos, Spanned};
use swc_core::ecma::ast::{
    CallExpr, Callee, Expr, Ident, Lit, MemberExpr, MemberProp, Module, Program, SpanExt,
};
use swc_core::ecma::visit::{VisitMut, VisitMutWith};
use swc_core::plugin::{
    plugin_transform,
    proxies::{PluginCommentsProxy, TransformPluginProgramMetadata},
};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    title: Option<String>,
    functions: Option<Vec<String>>,
    methods: Option<HashMap<String, Vec<String>>>,
    dynamic_imports: Option<Vec<String>>,
}

pub struct MarkExpression<C: Comments> {
    comments: C,
    title: String,
    functions: HashSet<String>,
    methods: HashMap<String, HashSet<String>>,
    dynamic_imports: HashSet<String>,
    results: Vec<String>,
}

impl<C: Comments> MarkExpression<C> {
    pub fn new(comments: C, config: &Config) -> Self {
        let title = config.title.to_owned().unwrap_or_default();
        let functions = config
            .functions
            .to_owned()
            .unwrap_or_default()
            .iter()
            .cloned()
            .collect();
        let dynamic_imports = config
            .dynamic_imports
            .to_owned()
            .unwrap_or_default()
            .iter()
            .cloned()
            .collect();
        let methods = config
            .methods
            .to_owned()
            .unwrap_or_default()
            .iter()
            .map(|(k, v)| (k.clone(), HashSet::from_iter(v.iter().cloned())))
            .collect();

        return Self {
            title,
            comments,
            functions,
            dynamic_imports,
            methods,
            results: Default::default(),
        };
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
            _ => ("", None),
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
        self.comments.with_leading(pos, |comments| {
            for comment in comments {
                if let CommentKind::Block = comment.kind {
                    let text = comment.text.as_str();
                    let text = format!("{{{text}}}");
                    let maybe_value =
                        parse_to_serde_value(text.as_str(), &Default::default()).unwrap_or(None);
                    if let Some(value) = &maybe_value {
                        let mut should_add = false;
                        if let Value::Object(object) = value {
                            for key in &self.dynamic_imports {
                                if let Some(value) = object.get(key) {
                                    match value {
                                        Value::Bool(true) => {
                                            should_add = true;
                                            break;
                                        }
                                        Value::Number(number) => {
                                            if number.as_i64().unwrap_or_default() != 0
                                                || number.as_f64().unwrap_or_default() != 0.0
                                                || number.as_u64().unwrap_or_default() != 0
                                            {
                                                should_add = true;
                                                break;
                                            }
                                        }
                                        Value::String(str) => {
                                            if !str.is_empty() {
                                                should_add = true;
                                                break;
                                            }
                                        }
                                        Value::Array(_) => {
                                            should_add = true;
                                            break;
                                        }
                                        Value::Object(_) => {
                                            should_add = true;
                                            break;
                                        }
                                        _ => (),
                                    }
                                }
                            }
                        }
                        if should_add {
                            let json = serde_json::to_string(&value).unwrap();
                            result.push(json);
                        }
                    }
                }
            }
        });

        if !result.is_empty() {
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
                    }
                    Lit::Num(num_lit) => {
                        args.push(num_lit.to_string());
                    }
                    Lit::Bool(bool_lit) => {
                        args.push(bool_lit.value.to_string());
                    }
                    _ => {
                        args.push("null".into());
                    }
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
                "---BEGIN {}–--\n[\n    {}\n]\n---END {}---",
                self.title,
                self.results.join(",\n    "),
                self.title,
            );

            self.comments.add_leading(
                module.span_lo(),
                Comment {
                    span,
                    kind: CommentKind::Block,
                    text: text.into(),
                },
            );
        }
    }

    fn visit_mut_call_expr(&mut self, call_expr: &mut CallExpr) {
        call_expr.visit_mut_children_with(self);
        match &call_expr.callee {
            Callee::Expr(callee) => match callee.as_ref() {
                Expr::Ident(ident) => {
                    if let Some(name) = self.check_fn_call(ident) {
                        let args = format!("[\"{}\", [{}]]", name, self.get_args(call_expr));
                        self.results.push(args);
                    }
                }
                Expr::Member(member_expr) => {
                    if let Some((obj_name, fn_name)) = self.check_method_call(member_expr) {
                        let args = format!(
                            "[\"{}\", \"{}\", [{}]]",
                            fn_name,
                            obj_name,
                            self.get_args(call_expr)
                        );
                        self.results.push(args);
                    }
                }
                _ => {}
            },
            Callee::Import(callee) => {
                let first_arg = call_expr.args.first();
                let pos = first_arg.map_or(callee.span_lo(), |arg| arg.span_lo());
                if let Some(magic_comments) = self.check_dynamic_import(pos) {
                    let args = self.get_args(call_expr);
                    let args = format!("[\"import\", {}, [{}]]", magic_comments, args);
                    self.results.push(args);
                }
            }
            _ => {}
        }
    }
}

#[plugin_transform]
pub fn process_transform(
    mut program: Program,
    metadata: TransformPluginProgramMetadata,
) -> Program {
    let config_str = &metadata
        .get_transform_plugin_config()
        .expect("Failed to resolve config");
    let config = serde_json::from_str::<Option<Config>>(config_str.as_str())
        .expect("Invalid config")
        .unwrap();

    let comments = match metadata.comments {
        Some(comments) => comments.clone(),
        None => PluginCommentsProxy,
    };

    program.visit_mut_with(&mut MarkExpression::new(comments, &config));
    program
}

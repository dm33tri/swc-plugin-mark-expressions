use jsonc_parser::parse_to_serde_value;
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use swc_common::{
    comments::{Comment, CommentKind, Comments},
    BytePos, SourceMapperDyn, Spanned,
};
use swc_ecma_ast::{
    CallExpr, Callee, Expr, Ident, Lit, MemberExpr, MemberProp, Module, Program, SpanExt,
};
use swc_ecma_ast::{ExprOrSpread, Prop, PropName};
use swc_ecma_visit::{VisitMut, VisitMutWith};
use swc_plugin_macro::plugin_transform;
use swc_plugin_proxy::{PluginCommentsProxy, TransformPluginProgramMetadata};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    title: Option<String>,
    functions: Option<Vec<String>>,
    methods: Option<HashMap<String, Vec<String>>>,
    dynamic_imports: Option<Vec<String>>,
    pretty: Option<bool>,
}

pub struct MarkExpression<C: Comments> {
    comments: C,
    source_mapper: Arc<SourceMapperDyn>,
    title: String,
    functions: HashSet<String>,
    methods: HashMap<String, HashSet<String>>,
    dynamic_imports: HashSet<String>,
    pretty: bool,
    results: Vec<Value>,
}

fn get_args(call_expr: &CallExpr) -> Value {
    let mut args: Vec<Value> = Vec::new();

    fn expr_to_serde_value(expr: &Expr) -> Value {
        match expr {
            Expr::Lit(lit) => match lit {
                Lit::Str(str) => Value::from(str.value.as_str()),
                Lit::Bool(bool) => Value::from(bool.value),
                Lit::Num(num) => {
                    let str = num.raw.as_ref().unwrap().as_str();
                    if str.contains(".") || str.contains("e") {
                        str.parse::<f64>()
                            .map_or(Value::Null, |num| Value::from(num))
                    } else {
                        str.parse::<i32>().map_or_else(
                            |_| {
                                str.parse::<u32>()
                                    .map_or(Value::Null, |num| Value::from(num))
                            },
                            |num| Value::from(num),
                        )
                    }
                }
                _ => Value::Null,
            },
            Expr::Array(arr) => {
                let vals = arr
                    .elems
                    .iter()
                    .filter_map(|item| {
                        item.as_ref()
                            .map(|item| expr_or_spread_to_serde_value(item))
                    })
                    .collect();
                Value::Array(vals)
            }
            Expr::Object(obj) => {
                let vals = obj
                    .props
                    .iter()
                    .filter_map(|prop| {
                        prop.as_prop()
                            .map(|prop| {
                                if let Prop::KeyValue(kv) = prop.as_ref() {
                                    let key = match &kv.key {
                                        PropName::Ident(ident) => Some(ident.sym.to_string()),
                                        PropName::Num(num) => {
                                            Some(num.raw.as_ref().unwrap().to_string())
                                        }
                                        PropName::Str(str) => Some(str.value.to_string()),
                                        _ => None,
                                    };

                                    let value = expr_to_serde_value(kv.value.as_ref());

                                    if let Some(key) = key {
                                        return Some((key, value));
                                    }
                                }
                                None
                            })
                            .unwrap_or_default()
                    })
                    .collect();
                Value::Object(vals)
            }
            _ => Value::Null,
        }
    }

    fn expr_or_spread_to_serde_value(expr_or_spread: &ExprOrSpread) -> Value {
        if let Some(_) = &expr_or_spread.spread {
            Value::Null
        } else {
            expr_to_serde_value(expr_or_spread.expr.as_ref())
        }
    }

    for arg in &call_expr.args {
        args.push(expr_or_spread_to_serde_value(arg))
    }

    Value::Array(args)
}

impl<C: Comments> MarkExpression<C> {
    pub fn new(comments: C, source_mapper: Arc<SourceMapperDyn>, config: &Config) -> Self {
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
        let pretty = config.pretty.unwrap_or_default();

        return Self {
            title,
            comments,
            source_mapper,
            functions,
            dynamic_imports,
            methods,
            pretty,
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

    fn check_dynamic_import(&self, pos: BytePos) -> Option<Value> {
        let mut result: Vec<Value> = Vec::new();
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
                            result.push(value.to_owned());
                        }
                    }
                }
            }
        });

        if !result.is_empty() {
            return Some(Value::from(result));
        }

        None
    }

    fn format_pos(&self, expr: &dyn Spanned) -> Value {
        let pos = self.source_mapper.lookup_char_pos(expr.span_lo());
        let str = format!(
            "{}:{}:{}",
            pos.file.name.to_string(),
            pos.line,
            pos.col_display
        );
        Value::from(str)
    }
}

impl<C: Comments> VisitMut for MarkExpression<C> {
    fn visit_mut_module(&mut self, module: &mut Module) {
        let span = module.comment_range();

        module.visit_mut_children_with(self);
        if !self.results.is_empty() {
            let json = if self.pretty {
                serde_json::to_string_pretty(&self.results).unwrap()
            } else {
                serde_json::to_string(&self.results).unwrap()
            };

            let text = format!(
                "---BEGIN {}–--\n{}\n---END {}---",
                self.title, json, self.title,
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
                        let args = Value::from(vec![
                            Value::from(name),
                            get_args(call_expr),
                            self.format_pos(call_expr),
                        ]);
                        self.results.push(args);
                    }
                }
                Expr::Member(member_expr) => {
                    if let Some((obj_name, fn_name)) = self.check_method_call(member_expr) {
                        let args = Value::from(vec![
                            Value::from(obj_name),
                            Value::from(fn_name),
                            get_args(call_expr),
                            self.format_pos(call_expr),
                        ]);
                        self.results.push(args);
                    }
                }
                _ => {}
            },
            Callee::Import(callee) => {
                let first_arg = call_expr.args.first();
                let pos = first_arg.map_or(callee.span_lo(), |arg| arg.span_lo());
                if let Some(magic_comments) = self.check_dynamic_import(pos) {
                    let args = Value::from(vec![
                        Value::from("import"),
                        magic_comments,
                        get_args(call_expr),
                        self.format_pos(call_expr),
                    ]);
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

    let source_map = std::sync::Arc::new(metadata.source_map);

    program.visit_mut_with(&mut MarkExpression::new(comments, source_map, &config));

    program
}

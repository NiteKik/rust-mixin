use crate::{Diagnostic, DiagnosticResult, ScalarType, Stage};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    #[serde(rename = "modify_call_result")]
    CallResult,
    #[serde(rename = "modify_return_result")]
    ReturnResult,
    #[serde(rename = "modify_parameter")]
    Parameter,
    #[serde(rename = "modify_head")]
    Head,
    InjectHead,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub schema: u32,
    pub logical_id: String,
    pub symbol: String,
    pub entry: String,
    pub arity: usize,
    pub retention_noinline: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<Signature>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub return_hook: Option<ReturnHook>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TypedReturnTarget {
    pub schema: u32,
    pub logical_id: String,
    pub symbol: String,
    pub entry: String,
    pub return_hook: ReturnHook,
    pub parameters: Vec<ScalarType>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReturnHook {
    pub symbol: String,
    pub type_probe: String,
    pub body: String,
}

impl ReturnHook {
    fn valid(&self) -> bool {
        self.symbol.starts_with("__rsmx_return_")
            && self.type_probe.starts_with("__rsmx_target_type_")
            && self.body.starts_with("__rsmx_body_")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Signature {
    pub parameters: Vec<ScalarType>,
    pub result: ScalarType,
}

impl Signature {
    fn valid_parameters(&self) -> bool {
        valid_parameters(&self.parameters)
    }
}

fn valid_parameters(parameters: &[ScalarType]) -> bool {
    parameters.len() <= 4
        && parameters.iter().all(|kind| {
            !matches!(
                kind,
                ScalarType::Unit | ScalarType::OptionEntityId | ScalarType::Vec2
            )
        })
}

impl Target {
    pub fn parameter_types(&self) -> &[ScalarType] {
        match &self.signature {
            Some(signature) => &signature.parameters,
            None => &[ScalarType::F64; 4][..self.arity],
        }
    }

    pub fn return_type(&self) -> ScalarType {
        self.signature
            .as_ref()
            .map_or(ScalarType::F64, |signature| signature.result)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Handler {
    pub schema: u32,
    pub id: String,
    pub target: String,
    pub operation: Operation,
    #[serde(default)]
    pub callee: Option<String>,
    pub ordinal: usize,
    pub require_matches: usize,
    pub priority: i32,
    pub symbol: String,
    #[serde(default)]
    pub parameter: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_type: Option<ScalarType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_signature: Option<Signature>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub return_type_probe: Option<String>,
}

impl Handler {
    pub fn value_type(&self) -> ScalarType {
        self.value_type.unwrap_or(ScalarType::F64)
    }
}

#[derive(Deserialize)]
#[serde(tag = "kind")]
pub enum Record {
    #[serde(rename = "target")]
    Target(Target),
    #[serde(rename = "typed_return_target")]
    TypedReturnTarget(TypedReturnTarget),
    #[serde(rename = "modify_call_result")]
    Handler(Handler),
    #[serde(rename = "modify_return_result")]
    ReturnHandler(Handler),
    #[serde(rename = "modify_parameter")]
    ParameterHandler(Handler),
    #[serde(rename = "modify_head")]
    HeadHandler(Handler),
    #[serde(rename = "inject_head")]
    EntryHandler(Handler),
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Index {
    pub targets: BTreeMap<String, Target>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub typed_return_targets: BTreeMap<String, TypedReturnTarget>,
    pub handlers: Vec<Handler>,
}

impl Index {
    pub fn append(&mut self, records: Vec<Record>, game: bool) -> Result<(), String> {
        self.append_detailed(records, game)
            .map_err(|error| error.message)
    }

    pub(crate) fn append_detailed(
        &mut self,
        records: Vec<Record>,
        game: bool,
    ) -> DiagnosticResult<()> {
        let error = |message| Diagnostic::new(Stage::Index, message);
        let mut ids: BTreeSet<_> = self.handlers.iter().map(|h| h.id.clone()).collect();
        let mut handlers = Vec::new();
        for record in records {
            match &record {
                Record::Handler(h) if h.operation != Operation::CallResult => {
                    return Err(error("kind/operation mismatch").with_handler(h, None));
                }
                Record::ReturnHandler(h) if h.operation != Operation::ReturnResult => {
                    return Err(error("kind/operation mismatch").with_handler(h, None));
                }
                Record::ParameterHandler(h) if h.operation != Operation::Parameter => {
                    return Err(error("kind/operation mismatch").with_handler(h, None));
                }
                Record::HeadHandler(h) if h.operation != Operation::Head => {
                    return Err(error("kind/operation mismatch").with_handler(h, None));
                }
                Record::EntryHandler(h) if h.operation != Operation::InjectHead => {
                    return Err(error("kind/operation mismatch").with_handler(h, None));
                }
                _ => {}
            }
            match record {
                Record::Target(t) if game => {
                    if !matches!((t.schema, &t.signature), (1, None) | (2, Some(_)))
                        || t.logical_id.is_empty()
                        || t.arity > 4
                        || t.signature.as_ref().is_some_and(|signature| {
                            signature.parameters.len() != t.arity || !signature.valid_parameters()
                        })
                        || !t.symbol.starts_with("__rsmx_fn_")
                        || !t.entry.starts_with("__rsmx_entry_")
                        || t.return_hook.as_ref().is_some_and(|hook| !hook.valid())
                        || !t.retention_noinline
                    {
                        return Err(Diagnostic::new(
                            Stage::Index,
                            format!("invalid target metadata: {}", t.logical_id),
                        ));
                    }
                    let id = t.logical_id.clone();
                    if self.typed_return_targets.contains_key(&id)
                        || self.targets.insert(id.clone(), t).is_some()
                    {
                        return Err(Diagnostic::new(
                            Stage::Index,
                            format!("duplicate target: {id}"),
                        ));
                    }
                }
                Record::TypedReturnTarget(t) if game => {
                    let id = t.logical_id.clone();
                    if t.schema != 3
                        || id.is_empty()
                        || !t.symbol.starts_with("__rsmx_fn_")
                        || !t.entry.starts_with("__rsmx_entry_")
                        || !t.return_hook.valid()
                        || !valid_parameters(&t.parameters)
                    {
                        return Err(error(&format!(
                            "invalid typed return target metadata: {id}"
                        )));
                    }
                    if self.targets.contains_key(&id)
                        || self.typed_return_targets.insert(id.clone(), t).is_some()
                    {
                        return Err(error(&format!("duplicate target: {id}")));
                    }
                }
                Record::Handler(h)
                | Record::ReturnHandler(h)
                | Record::ParameterHandler(h)
                | Record::HeadHandler(h)
                | Record::EntryHandler(h)
                    if !game =>
                {
                    let entry_callback = h.operation == Operation::InjectHead;
                    let valid_signature = if h.operation == Operation::ReturnResult {
                        h.schema == 3
                            && h.value_type.is_none()
                            && h.head_signature.is_none()
                            && h.ordinal == 0
                            && h.require_matches == 1
                            && h.return_type_probe
                                .as_deref()
                                .is_some_and(|probe| probe.starts_with("__rsmx_handler_type_"))
                    } else if h.return_type_probe.is_some() {
                        false
                    } else if entry_callback {
                        h.schema == 2
                            && h.value_type.is_none()
                            && h.head_signature
                                .as_ref()
                                .is_some_and(Signature::valid_parameters)
                    } else {
                        h.head_signature.is_none()
                            && matches!((h.schema, h.value_type), (1, None) | (2, Some(_)))
                            && (!matches!(
                                h.value_type,
                                Some(
                                    ScalarType::Unit
                                        | ScalarType::OptionEntityId
                                        | ScalarType::Vec2
                                )
                            ) || h.operation == Operation::CallResult)
                    };
                    if h.operation == Operation::ReturnResult && h.schema < 3 {
                        return Err(error(
                            "old modify_return_result ABI; rebuild the mod with the current SDK",
                        )
                        .with_handler(&h, None));
                    }
                    if !valid_signature
                        || h.id.is_empty()
                        || h.target.is_empty()
                        || (h.operation == Operation::CallResult
                            && h.callee.as_deref().is_none_or(str::is_empty))
                        || (h.operation == Operation::ReturnResult && h.callee.is_some())
                        || (h.operation == Operation::Parameter
                            && (h.callee.is_some() || h.parameter.is_none()))
                        || (matches!(h.operation, Operation::Head | Operation::InjectHead)
                            && (h.callee.is_some() || h.parameter.is_some()))
                        || (h.operation != Operation::Parameter && h.parameter.is_some())
                        || (matches!(h.operation, Operation::Head | Operation::InjectHead)
                            && (h.ordinal != 0 || h.require_matches != 1))
                        || (h.operation == Operation::Parameter
                            && (h.parameter.is_none_or(|p| p > 3)
                                || h.ordinal != 0
                                || h.require_matches != 1))
                        || h.priority < 0
                        || h.require_matches == 0
                        || h.ordinal >= h.require_matches
                        || !h.symbol.starts_with("__rsmx_handler_")
                    {
                        return Err(error("invalid handler metadata").with_handler(&h, None));
                    }
                    if !ids.insert(h.id.clone()) {
                        return Err(Diagnostic::new(
                            Stage::Index,
                            format!("duplicate handler: {}", h.id),
                        )
                        .with_handler(&h, None));
                    }
                    handlers.push(h);
                }
                _ => {
                    return Err(error(
                        "game must contain targets; mod must contain only handlers",
                    ));
                }
            }
        }
        handlers.sort_by(|a, b| a.id.cmp(&b.id));
        self.handlers.extend(handlers);
        Ok(())
    }

    pub fn target(&self, id: &str) -> Result<&Target, String> {
        self.targets.get(id).ok_or_else(|| {
            if self.typed_return_targets.contains_key(id) {
                format!("target {id} has a typed return; use Jit::call_typed to call it")
            } else {
                format!("logical target not found: {id}")
            }
        })
    }

    pub(crate) fn return_hook(&self, id: &str) -> Option<(&str, &ReturnHook)> {
        if let Some(target) = self.typed_return_targets.get(id) {
            return Some((&target.symbol, &target.return_hook));
        }
        let target = self.targets.get(id)?;
        Some((&target.symbol, target.return_hook.as_ref()?))
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct PlannedPatch {
    pub id: String,
    pub target: String,
    pub target_symbol: String,
    pub operation: Operation,
    pub callee: String,
    pub callee_symbol: String,
    pub handler_symbol: String,
    pub ordinal: usize,
    pub matches: usize,
    pub priority: i32,
    pub parameter: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_type: Option<ScalarType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head_signature: Option<Signature>,
}

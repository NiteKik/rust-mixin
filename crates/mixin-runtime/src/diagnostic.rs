use crate::metadata::{Handler, Operation};
use serde::Serialize;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Manifest,
    ModManifest,
    LoadLlvm,
    InitializeLlvm,
    Parse,
    Index,
    Link,
    ResolveSelector,
    Transform,
    Verify,
    Optimize,
    Jit,
    Reload,
    Call,
    WriteReport,
    WriteEvidence,
}

#[derive(Debug, Serialize)]
pub struct Diagnostic {
    pub stage: Stage,
    pub message: String,
    /// Zero-based index in the input mod list, when the error identifies one mod.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mod_index: Option<usize>,
    #[serde(flatten)]
    pub handler: Option<Box<HandlerContext>>,
}

#[derive(Debug, Serialize)]
pub struct HandlerContext {
    pub handler_id: String,
    pub operation: Operation,
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callee: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter: Option<usize>,
    pub ordinal: usize,
    pub expected_matches: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_matches: Option<usize>,
}

impl Diagnostic {
    pub fn new(stage: Stage, message: impl Into<String>) -> Self {
        Self {
            stage,
            message: message.into(),
            mod_index: None,
            handler: None,
        }
    }

    pub(crate) fn with_mod(mut self, mod_index: Option<usize>) -> Self {
        self.mod_index = mod_index;
        self
    }

    pub(crate) fn with_handler(mut self, handler: &Handler, actual_matches: Option<usize>) -> Self {
        self.handler = Some(Box::new(HandlerContext {
            handler_id: handler.id.clone(),
            operation: handler.operation,
            target: handler.target.clone(),
            callee: handler.callee.clone(),
            parameter: handler.parameter,
            ordinal: handler.ordinal,
            expected_matches: handler.require_matches,
            actual_matches,
        }));
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}", self.stage)?;
        if let Some(index) = self.mod_index {
            write!(formatter, " mod #{}", index + 1)?;
        }
        if let Some(handler) = &self.handler {
            write!(
                formatter,
                " handler {} targeting {}",
                handler.handler_id, handler.target
            )?;
        }
        write!(formatter, ": {}", self.message)
    }
}

impl std::error::Error for Diagnostic {}

pub type DiagnosticResult<T> = Result<T, Diagnostic>;

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use super::levels::Level;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u16)]
pub enum ErrorCode {
    ComponentCall = 1,
    RuntimeCall = 2,
    UnresolvedCall = 3,
    LintConfiguration = 4,
    PrivateApi = 5,
}

impl ErrorCode {
    const ALL: [Self; 5] = [
        Self::ComponentCall,
        Self::RuntimeCall,
        Self::UnresolvedCall,
        Self::LintConfiguration,
        Self::PrivateApi,
    ];

    pub fn explanation(self) -> &'static str {
        match self {
            Self::ComponentCall => include_str!("error_codes/T0001.md"),
            Self::RuntimeCall => include_str!("error_codes/T0002.md"),
            Self::UnresolvedCall => include_str!("error_codes/T0003.md"),
            Self::LintConfiguration => include_str!("error_codes/T0004.md"),
            Self::PrivateApi => include_str!("error_codes/T0005.md"),
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "T{:04}", *self as u16)
    }
}

impl FromStr for ErrorCode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|code| code.to_string() == value)
            .ok_or_else(|| format!("unknown Tessera diagnostic code `{value}`"))
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lint {
    RedundantRenderSlot,
}

impl Lint {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::RedundantRenderSlot => "tessera::redundant_render_slot",
        }
    }

    pub(crate) fn default_level(self) -> Level {
        Level::Warn
    }
}

impl fmt::Display for Lint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

impl FromStr for Lint {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        [Self::RedundantRenderSlot]
            .into_iter()
            .find(|lint| lint.name() == value)
            .ok_or_else(|| format!("unknown Tessera lint `{value}`"))
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum DiagnosticCode {
    Error(ErrorCode),
    Lint(Lint),
}

impl DiagnosticCode {
    pub(crate) fn error_code(self) -> Option<ErrorCode> {
        match self {
            Self::Error(code) => Some(code),
            Self::Lint(_) => None,
        }
    }

    pub(crate) fn explanation(self) -> Option<&'static str> {
        self.error_code().map(ErrorCode::explanation)
    }
}

impl From<ErrorCode> for DiagnosticCode {
    fn from(code: ErrorCode) -> Self {
        Self::Error(code)
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error(code) => code.fmt(formatter),
            Self::Lint(lint) => lint.fmt(formatter),
        }
    }
}

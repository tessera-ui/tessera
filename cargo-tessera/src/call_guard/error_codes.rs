use std::{fmt, str::FromStr};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub enum ErrorCode {
    ComponentCall = 1,
    RuntimeCall = 2,
    UnresolvedCall = 3,
}

impl ErrorCode {
    const ALL: [Self; 3] = [Self::ComponentCall, Self::RuntimeCall, Self::UnresolvedCall];

    pub fn explanation(self) -> &'static str {
        match self {
            Self::ComponentCall => include_str!("error_codes/T0001.md"),
            Self::RuntimeCall => include_str!("error_codes/T0002.md"),
            Self::UnresolvedCall => include_str!("error_codes/T0003.md"),
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

use std::str::FromStr;

use cargo_metadata::diagnostic::DiagnosticLevel as Severity;

use super::error_codes::Lint;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Allow,
    Warn,
    ForceWarn,
    Deny,
    Forbid,
}

impl Level {
    pub(super) fn severity(self) -> Option<Severity> {
        match self {
            Self::Allow => None,
            Self::Warn | Self::ForceWarn => Some(Severity::Warning),
            Self::Deny | Self::Forbid => Some(Severity::Error),
        }
    }
}

impl FromStr for Level {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "allow" => Ok(Self::Allow),
            "warn" => Ok(Self::Warn),
            "force-warn" => Ok(Self::ForceWarn),
            "deny" => Ok(Self::Deny),
            "forbid" => Ok(Self::Forbid),
            _ => Err(format!("unknown Tessera lint level `{value}`")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LintTarget {
    Lint(Lint),
    All,
    Warnings,
}

impl FromStr for LintTarget {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "tessera::all" => Ok(Self::All),
            "warnings" => Ok(Self::Warnings),
            _ => value.parse().map(Self::Lint),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LintOptions {
    pub overrides: Vec<(LintTarget, Level)>,
    pub cap: Option<Level>,
}

impl LintOptions {
    pub(super) fn command_line(&self, lint: Lint) -> (Level, Level) {
        let mut level = lint.default_level();
        let mut warnings = Level::Warn;
        for &(target, requested) in &self.overrides {
            let current = match target {
                LintTarget::Warnings => &mut warnings,
                LintTarget::All => &mut level,
                LintTarget::Lint(target) if target == lint => &mut level,
                _ => continue,
            };
            if !matches!(current, Level::ForceWarn | Level::Forbid) {
                *current = requested;
            }
        }
        (level, warnings)
    }

    pub(super) fn level(&self, (level, warnings): (Level, Level)) -> Level {
        self.cap_level(if level == Level::Warn {
            warnings
        } else {
            level
        })
    }

    pub(super) fn cap_level(&self, level: Level) -> Level {
        if level == Level::ForceWarn {
            level
        } else {
            self.cap.map_or(level, |cap| level.min(cap))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_line_order_locks_groups_and_caps_determine_the_effective_level() {
        use Level::*;
        let lint = Lint::RedundantRenderSlot;
        let target = LintTarget::Lint(lint);
        let warnings = LintTarget::Warnings;
        for (overrides, cap, expected) in [
            (vec![], None, Warn),
            (vec![(target, Allow), (target, Deny)], None, Deny),
            (vec![(target, Deny), (target, Allow)], None, Allow),
            (vec![(LintTarget::All, Deny), (target, Warn)], None, Warn),
            (vec![(target, Forbid), (target, Allow)], None, Forbid),
            (vec![(target, ForceWarn), (target, Forbid)], None, ForceWarn),
            (vec![(target, Forbid), (target, ForceWarn)], None, Forbid),
            (vec![(target, Forbid)], Some(Allow), Allow),
            (vec![(target, Forbid)], Some(Warn), Warn),
            (vec![(target, ForceWarn)], Some(Allow), ForceWarn),
            (vec![(warnings, Deny)], None, Deny),
            (vec![(warnings, Deny), (target, Allow)], None, Allow),
            (vec![(target, Allow), (warnings, Deny)], None, Allow),
            (vec![(target, ForceWarn), (warnings, Deny)], None, ForceWarn),
            (vec![(warnings, Forbid), (warnings, Allow)], None, Forbid),
        ] {
            let options = LintOptions { overrides, cap };
            assert_eq!(
                options.level(options.command_line(lint)),
                expected,
                "{options:?}"
            );
        }
    }
}

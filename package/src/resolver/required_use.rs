//! REQUIRED_USE constraint parsing and validation
//!
//! REQUIRED_USE specifies constraints on USE flags that must be satisfied for a package.
//! This module parses and evaluates these constraints, and can suggest changes to satisfy them.
//!
//! # Syntax
//!
//! - Simple flag: `foo` means foo must be enabled
//! - Negation: `!foo` means foo must be disabled
//! - Any-of group: `|| ( foo bar )` means at least one must be enabled
//! - Exactly-one-of: `^^ ( foo bar )` means exactly one must be enabled
//! - At-most-one-of: `?? ( foo bar )` means at most one can be enabled
//! - Use-conditional: `foo? ( bar )` means if foo is enabled, bar must be enabled
//! - Negative conditional: `!foo? ( bar )` means if foo is disabled, bar must be enabled

use std::collections::HashSet;

/// A REQUIRED_USE constraint
#[derive(Debug, Clone, PartialEq)]
pub enum RequiredUseConstraint {
    /// A flag must be enabled
    Enabled(String),
    /// A flag must be disabled
    Disabled(String),
    /// At least one of the constraints must be satisfied (||)
    AnyOf(Vec<RequiredUseConstraint>),
    /// Exactly one of the constraints must be satisfied (^^)
    ExactlyOneOf(Vec<RequiredUseConstraint>),
    /// At most one of the constraints can be satisfied (??)
    AtMostOneOf(Vec<RequiredUseConstraint>),
    /// All constraints must be satisfied (implicit grouping)
    AllOf(Vec<RequiredUseConstraint>),
    /// Conditional: if condition is satisfied, inner must be satisfied
    Conditional {
        /// The condition flag
        flag: String,
        /// Whether the condition is positive (flag?) or negative (!flag?)
        positive: bool,
        /// The constraints that must be satisfied if condition is met
        inner: Vec<RequiredUseConstraint>,
    },
}

/// Result of validating REQUIRED_USE constraints
#[derive(Debug, Clone)]
pub struct RequiredUseValidation {
    /// Whether all constraints are satisfied
    pub satisfied: bool,
    /// Human-readable explanation of unsatisfied constraints
    pub explanation: Option<String>,
    /// Suggested flags to enable to satisfy constraints
    pub suggest_enable: Vec<String>,
    /// Suggested flags to disable to satisfy constraints
    pub suggest_disable: Vec<String>,
}

impl RequiredUseConstraint {
    /// Parse a REQUIRED_USE string into constraints
    pub fn parse(input: &str) -> Result<Vec<Self>, String> {
        let input = input.trim();
        if input.is_empty() {
            return Ok(Vec::new());
        }

        let tokens = tokenize(input)?;
        parse_tokens(&tokens)
    }

    /// Evaluate if this constraint is satisfied given enabled USE flags
    pub fn is_satisfied(&self, enabled_flags: &HashSet<String>) -> bool {
        match self {
            RequiredUseConstraint::Enabled(flag) => enabled_flags.contains(flag),
            RequiredUseConstraint::Disabled(flag) => !enabled_flags.contains(flag),
            RequiredUseConstraint::AnyOf(constraints) => {
                constraints.iter().any(|c| c.is_satisfied(enabled_flags))
            }
            RequiredUseConstraint::ExactlyOneOf(constraints) => {
                constraints
                    .iter()
                    .filter(|c| c.is_satisfied(enabled_flags))
                    .count()
                    == 1
            }
            RequiredUseConstraint::AtMostOneOf(constraints) => {
                constraints
                    .iter()
                    .filter(|c| c.is_satisfied(enabled_flags))
                    .count()
                    <= 1
            }
            RequiredUseConstraint::AllOf(constraints) => {
                constraints.iter().all(|c| c.is_satisfied(enabled_flags))
            }
            RequiredUseConstraint::Conditional {
                flag,
                positive,
                inner,
            } => {
                let condition_met = if *positive {
                    enabled_flags.contains(flag)
                } else {
                    !enabled_flags.contains(flag)
                };

                // If condition is not met, the constraint is satisfied (vacuous truth)
                if !condition_met {
                    return true;
                }

                // If condition is met, all inner constraints must be satisfied
                inner.iter().all(|c| c.is_satisfied(enabled_flags))
            }
        }
    }

    /// Get a human-readable description of this constraint
    pub fn describe(&self) -> String {
        match self {
            RequiredUseConstraint::Enabled(flag) => format!("{} must be enabled", flag),
            RequiredUseConstraint::Disabled(flag) => format!("{} must be disabled", flag),
            RequiredUseConstraint::AnyOf(constraints) => {
                let items: Vec<_> = constraints.iter().map(|c| c.describe()).collect();
                format!("at least one of: {}", items.join(", "))
            }
            RequiredUseConstraint::ExactlyOneOf(constraints) => {
                let items: Vec<_> = constraints.iter().map(|c| c.describe()).collect();
                format!("exactly one of: {}", items.join(", "))
            }
            RequiredUseConstraint::AtMostOneOf(constraints) => {
                let items: Vec<_> = constraints.iter().map(|c| c.describe()).collect();
                format!("at most one of: {}", items.join(", "))
            }
            RequiredUseConstraint::AllOf(constraints) => {
                let items: Vec<_> = constraints.iter().map(|c| c.describe()).collect();
                format!("all of: {}", items.join(", "))
            }
            RequiredUseConstraint::Conditional {
                flag,
                positive,
                inner,
            } => {
                let items: Vec<_> = inner.iter().map(|c| c.describe()).collect();
                if *positive {
                    format!("if {} is enabled: {}", flag, items.join(", "))
                } else {
                    format!("if {} is disabled: {}", flag, items.join(", "))
                }
            }
        }
    }

    /// Get all flags referenced by this constraint
    ///
    /// Returns a list of all USE flag names that appear in this constraint,
    /// including flags in nested conditions and groups.
    pub fn get_flags(&self) -> Vec<&str> {
        match self {
            RequiredUseConstraint::Enabled(flag) | RequiredUseConstraint::Disabled(flag) => {
                vec![flag.as_str()]
            }
            RequiredUseConstraint::AnyOf(cs)
            | RequiredUseConstraint::ExactlyOneOf(cs)
            | RequiredUseConstraint::AtMostOneOf(cs)
            | RequiredUseConstraint::AllOf(cs) => cs.iter().flat_map(|c| c.get_flags()).collect(),
            RequiredUseConstraint::Conditional { flag, inner, .. } => {
                let mut flags = vec![flag.as_str()];
                flags.extend(inner.iter().flat_map(|c| c.get_flags()));
                flags
            }
        }
    }
}

/// Validate REQUIRED_USE constraints and suggest changes
pub fn validate_required_use(
    required_use: &str,
    enabled_flags: &HashSet<String>,
    available_flags: &HashSet<String>,
) -> RequiredUseValidation {
    let constraints = match RequiredUseConstraint::parse(required_use) {
        Ok(c) => c,
        Err(e) => {
            return RequiredUseValidation {
                satisfied: false,
                explanation: Some(format!("Failed to parse REQUIRED_USE: {}", e)),
                suggest_enable: Vec::new(),
                suggest_disable: Vec::new(),
            };
        }
    };

    if constraints.is_empty() {
        return RequiredUseValidation {
            satisfied: true,
            explanation: None,
            suggest_enable: Vec::new(),
            suggest_disable: Vec::new(),
        };
    }

    // Check if all constraints are satisfied
    let mut unsatisfied = Vec::new();
    for constraint in &constraints {
        if !constraint.is_satisfied(enabled_flags) {
            unsatisfied.push(constraint);
        }
    }

    if unsatisfied.is_empty() {
        return RequiredUseValidation {
            satisfied: true,
            explanation: None,
            suggest_enable: Vec::new(),
            suggest_disable: Vec::new(),
        };
    }

    // Generate suggestions to satisfy constraints
    let (suggest_enable, suggest_disable) =
        suggest_changes(&unsatisfied, enabled_flags, available_flags);

    let explanation = Some(format!(
        "Unsatisfied REQUIRED_USE constraints:\n{}",
        unsatisfied
            .iter()
            .map(|c| format!("  - {}", c.describe()))
            .collect::<Vec<_>>()
            .join("\n")
    ));

    RequiredUseValidation {
        satisfied: false,
        explanation,
        suggest_enable,
        suggest_disable,
    }
}

/// Suggest changes to satisfy unsatisfied constraints
fn suggest_changes(
    unsatisfied: &[&RequiredUseConstraint],
    enabled_flags: &HashSet<String>,
    available_flags: &HashSet<String>,
) -> (Vec<String>, Vec<String>) {
    let mut to_enable = Vec::new();
    let mut to_disable = Vec::new();

    for constraint in unsatisfied {
        suggest_for_constraint(
            constraint,
            enabled_flags,
            available_flags,
            &mut to_enable,
            &mut to_disable,
        );
    }

    // Remove duplicates while preserving order
    to_enable.sort();
    to_enable.dedup();
    to_disable.sort();
    to_disable.dedup();

    // Remove conflicts (if a flag is both suggested to enable and disable, remove it)
    to_enable.retain(|f| !to_disable.contains(f));
    to_disable.retain(|f| !to_enable.contains(f));

    (to_enable, to_disable)
}

fn suggest_for_constraint(
    constraint: &RequiredUseConstraint,
    enabled_flags: &HashSet<String>,
    available_flags: &HashSet<String>,
    to_enable: &mut Vec<String>,
    to_disable: &mut Vec<String>,
) {
    match constraint {
        RequiredUseConstraint::Enabled(flag) => {
            if available_flags.contains(flag) && !enabled_flags.contains(flag) {
                to_enable.push(flag.clone());
            }
        }
        RequiredUseConstraint::Disabled(flag) => {
            if enabled_flags.contains(flag) {
                to_disable.push(flag.clone());
            }
        }
        RequiredUseConstraint::AnyOf(constraints) => {
            // Suggest enabling the first available flag that would satisfy the constraint
            for c in constraints {
                if let RequiredUseConstraint::Enabled(flag) = c {
                    if available_flags.contains(flag) && !enabled_flags.contains(flag) {
                        to_enable.push(flag.clone());
                        return;
                    }
                }
            }
        }
        RequiredUseConstraint::ExactlyOneOf(constraints) => {
            // Count how many are currently satisfied
            let satisfied: Vec<_> = constraints
                .iter()
                .filter(|c| c.is_satisfied(enabled_flags))
                .collect();

            if satisfied.is_empty() {
                // None enabled - suggest enabling the first available
                for c in constraints {
                    if let RequiredUseConstraint::Enabled(flag) = c {
                        if available_flags.contains(flag) {
                            to_enable.push(flag.clone());
                            return;
                        }
                    }
                }
            } else if satisfied.len() > 1 {
                // Too many enabled - suggest disabling all but the first
                let mut first = true;
                for c in &satisfied {
                    if let RequiredUseConstraint::Enabled(flag) = c {
                        if first {
                            first = false;
                        } else {
                            to_disable.push(flag.clone());
                        }
                    }
                }
            }
        }
        RequiredUseConstraint::AtMostOneOf(constraints) => {
            // If more than one satisfied, disable extras
            let satisfied: Vec<_> = constraints
                .iter()
                .filter(|c| c.is_satisfied(enabled_flags))
                .collect();

            if satisfied.len() > 1 {
                let mut first = true;
                for c in &satisfied {
                    if let RequiredUseConstraint::Enabled(flag) = c {
                        if first {
                            first = false;
                        } else {
                            to_disable.push(flag.clone());
                        }
                    }
                }
            }
        }
        RequiredUseConstraint::AllOf(constraints) => {
            for c in constraints {
                if !c.is_satisfied(enabled_flags) {
                    suggest_for_constraint(
                        c,
                        enabled_flags,
                        available_flags,
                        to_enable,
                        to_disable,
                    );
                }
            }
        }
        RequiredUseConstraint::Conditional {
            flag,
            positive,
            inner,
        } => {
            let condition_met = if *positive {
                enabled_flags.contains(flag)
            } else {
                !enabled_flags.contains(flag)
            };

            if condition_met {
                // Condition is met but inner constraints are not satisfied
                // Either disable the condition flag or satisfy inner constraints
                // Prefer satisfying inner constraints
                for c in inner {
                    if !c.is_satisfied(enabled_flags) {
                        suggest_for_constraint(
                            c,
                            enabled_flags,
                            available_flags,
                            to_enable,
                            to_disable,
                        );
                    }
                }
            }
        }
    }
}

/// Token types for parsing
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Flag(String),
    NegatedFlag(String),
    Conditional(String, bool), // (flag, positive)
    OpenParen,
    CloseParen,
    AnyOf,        // ||
    ExactlyOneOf, // ^^
    AtMostOneOf,  // ??
}

/// Tokenize REQUIRED_USE string
fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            '(' => {
                chars.next();
                tokens.push(Token::OpenParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::CloseParen);
            }
            '|' => {
                chars.next();
                if chars.peek() == Some(&'|') {
                    chars.next();
                    tokens.push(Token::AnyOf);
                } else {
                    return Err("Expected '||' but found single '|'".to_string());
                }
            }
            '^' => {
                chars.next();
                if chars.peek() == Some(&'^') {
                    chars.next();
                    tokens.push(Token::ExactlyOneOf);
                } else {
                    return Err("Expected '^^' but found single '^'".to_string());
                }
            }
            '?' => {
                chars.next();
                if chars.peek() == Some(&'?') {
                    chars.next();
                    tokens.push(Token::AtMostOneOf);
                } else {
                    return Err("Unexpected '?' - use '??' for at-most-one-of".to_string());
                }
            }
            '!' => {
                chars.next();
                // Could be negated flag or negative conditional
                let mut flag = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' || c == '-' || c == '+' {
                        flag.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if flag.is_empty() {
                    return Err("Expected flag name after '!'".to_string());
                }
                // Check if it's a conditional
                if chars.peek() == Some(&'?') {
                    chars.next();
                    tokens.push(Token::Conditional(flag, false));
                } else {
                    tokens.push(Token::NegatedFlag(flag));
                }
            }
            _ if c.is_alphabetic() || c == '_' => {
                let mut flag = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' || c == '-' || c == '+' {
                        flag.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                // Check if it's a conditional
                if chars.peek() == Some(&'?') {
                    chars.next();
                    tokens.push(Token::Conditional(flag, true));
                } else {
                    tokens.push(Token::Flag(flag));
                }
            }
            _ => {
                return Err(format!("Unexpected character: '{}'", c));
            }
        }
    }

    Ok(tokens)
}

/// Parse tokens into constraints
fn parse_tokens(tokens: &[Token]) -> Result<Vec<RequiredUseConstraint>, String> {
    let mut pos = 0;
    let mut constraints = Vec::new();

    while pos < tokens.len() {
        let (constraint, new_pos) = parse_constraint(tokens, pos)?;
        constraints.push(constraint);
        pos = new_pos;
    }

    Ok(constraints)
}

/// Parse a single constraint (may consume multiple tokens)
fn parse_constraint(
    tokens: &[Token],
    pos: usize,
) -> Result<(RequiredUseConstraint, usize), String> {
    if pos >= tokens.len() {
        return Err("Unexpected end of input".to_string());
    }

    match &tokens[pos] {
        Token::Flag(flag) => Ok((RequiredUseConstraint::Enabled(flag.clone()), pos + 1)),
        Token::NegatedFlag(flag) => Ok((RequiredUseConstraint::Disabled(flag.clone()), pos + 1)),
        Token::AnyOf => {
            // Expect ( constraints )
            if pos + 1 >= tokens.len() || tokens[pos + 1] != Token::OpenParen {
                return Err("Expected '(' after '||'".to_string());
            }
            let (inner, new_pos) = parse_group(tokens, pos + 2)?;
            Ok((RequiredUseConstraint::AnyOf(inner), new_pos))
        }
        Token::ExactlyOneOf => {
            if pos + 1 >= tokens.len() || tokens[pos + 1] != Token::OpenParen {
                return Err("Expected '(' after '^^'".to_string());
            }
            let (inner, new_pos) = parse_group(tokens, pos + 2)?;
            Ok((RequiredUseConstraint::ExactlyOneOf(inner), new_pos))
        }
        Token::AtMostOneOf => {
            if pos + 1 >= tokens.len() || tokens[pos + 1] != Token::OpenParen {
                return Err("Expected '(' after '??'".to_string());
            }
            let (inner, new_pos) = parse_group(tokens, pos + 2)?;
            Ok((RequiredUseConstraint::AtMostOneOf(inner), new_pos))
        }
        Token::Conditional(flag, positive) => {
            // Expect ( constraints )
            if pos + 1 >= tokens.len() || tokens[pos + 1] != Token::OpenParen {
                return Err(format!("Expected '(' after '{}?'", flag));
            }
            let (inner, new_pos) = parse_group(tokens, pos + 2)?;
            Ok((
                RequiredUseConstraint::Conditional {
                    flag: flag.clone(),
                    positive: *positive,
                    inner,
                },
                new_pos,
            ))
        }
        Token::OpenParen => {
            let (inner, new_pos) = parse_group(tokens, pos + 1)?;
            Ok((RequiredUseConstraint::AllOf(inner), new_pos))
        }
        Token::CloseParen => Err("Unexpected ')'".to_string()),
    }
}

/// Parse a group of constraints until closing paren
fn parse_group(
    tokens: &[Token],
    start: usize,
) -> Result<(Vec<RequiredUseConstraint>, usize), String> {
    let mut constraints = Vec::new();
    let mut pos = start;

    while pos < tokens.len() {
        if tokens[pos] == Token::CloseParen {
            return Ok((constraints, pos + 1));
        }
        let (constraint, new_pos) = parse_constraint(tokens, pos)?;
        constraints.push(constraint);
        pos = new_pos;
    }

    Err("Unclosed parenthesis".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flags(names: &[&str]) -> HashSet<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_parse_simple_flag() {
        let constraints = RequiredUseConstraint::parse("foo").unwrap();
        assert_eq!(constraints.len(), 1);
        assert_eq!(
            constraints[0],
            RequiredUseConstraint::Enabled("foo".to_string())
        );
    }

    #[test]
    fn test_parse_negated_flag() {
        let constraints = RequiredUseConstraint::parse("!foo").unwrap();
        assert_eq!(constraints.len(), 1);
        assert_eq!(
            constraints[0],
            RequiredUseConstraint::Disabled("foo".to_string())
        );
    }

    #[test]
    fn test_parse_any_of() {
        let constraints = RequiredUseConstraint::parse("|| ( foo bar )").unwrap();
        assert_eq!(constraints.len(), 1);
        match &constraints[0] {
            RequiredUseConstraint::AnyOf(inner) => {
                assert_eq!(inner.len(), 2);
            }
            _ => panic!("Expected AnyOf"),
        }
    }

    #[test]
    fn test_parse_exactly_one_of() {
        let constraints = RequiredUseConstraint::parse("^^ ( a b c )").unwrap();
        assert_eq!(constraints.len(), 1);
        match &constraints[0] {
            RequiredUseConstraint::ExactlyOneOf(inner) => {
                assert_eq!(inner.len(), 3);
            }
            _ => panic!("Expected ExactlyOneOf"),
        }
    }

    #[test]
    fn test_parse_at_most_one_of() {
        let constraints = RequiredUseConstraint::parse("?? ( x y )").unwrap();
        assert_eq!(constraints.len(), 1);
        match &constraints[0] {
            RequiredUseConstraint::AtMostOneOf(inner) => {
                assert_eq!(inner.len(), 2);
            }
            _ => panic!("Expected AtMostOneOf"),
        }
    }

    #[test]
    fn test_parse_conditional() {
        let constraints = RequiredUseConstraint::parse("foo? ( bar )").unwrap();
        assert_eq!(constraints.len(), 1);
        match &constraints[0] {
            RequiredUseConstraint::Conditional {
                flag,
                positive,
                inner,
            } => {
                assert_eq!(flag, "foo");
                assert!(*positive);
                assert_eq!(inner.len(), 1);
            }
            _ => panic!("Expected Conditional"),
        }
    }

    #[test]
    fn test_parse_negative_conditional() {
        let constraints = RequiredUseConstraint::parse("!foo? ( bar )").unwrap();
        assert_eq!(constraints.len(), 1);
        match &constraints[0] {
            RequiredUseConstraint::Conditional {
                flag,
                positive,
                inner,
            } => {
                assert_eq!(flag, "foo");
                assert!(!*positive);
                assert_eq!(inner.len(), 1);
            }
            _ => panic!("Expected Conditional"),
        }
    }

    #[test]
    fn test_parse_multiple_constraints() {
        let constraints = RequiredUseConstraint::parse("foo bar !baz").unwrap();
        assert_eq!(constraints.len(), 3);
    }

    #[test]
    fn test_parse_complex() {
        let constraints =
            RequiredUseConstraint::parse("^^ ( ssl gnutls ) ssl? ( !libressl )").unwrap();
        assert_eq!(constraints.len(), 2);
    }

    #[test]
    fn test_evaluate_enabled() {
        let c = RequiredUseConstraint::Enabled("foo".to_string());
        assert!(c.is_satisfied(&flags(&["foo"])));
        assert!(!c.is_satisfied(&flags(&["bar"])));
    }

    #[test]
    fn test_evaluate_disabled() {
        let c = RequiredUseConstraint::Disabled("foo".to_string());
        assert!(!c.is_satisfied(&flags(&["foo"])));
        assert!(c.is_satisfied(&flags(&["bar"])));
    }

    #[test]
    fn test_evaluate_any_of() {
        let c = RequiredUseConstraint::AnyOf(vec![
            RequiredUseConstraint::Enabled("foo".to_string()),
            RequiredUseConstraint::Enabled("bar".to_string()),
        ]);
        assert!(c.is_satisfied(&flags(&["foo"])));
        assert!(c.is_satisfied(&flags(&["bar"])));
        assert!(c.is_satisfied(&flags(&["foo", "bar"])));
        assert!(!c.is_satisfied(&flags(&["baz"])));
    }

    #[test]
    fn test_evaluate_exactly_one_of() {
        let c = RequiredUseConstraint::ExactlyOneOf(vec![
            RequiredUseConstraint::Enabled("foo".to_string()),
            RequiredUseConstraint::Enabled("bar".to_string()),
        ]);
        assert!(c.is_satisfied(&flags(&["foo"])));
        assert!(c.is_satisfied(&flags(&["bar"])));
        assert!(!c.is_satisfied(&flags(&["foo", "bar"])));
        assert!(!c.is_satisfied(&flags(&["baz"])));
    }

    #[test]
    fn test_evaluate_at_most_one_of() {
        let c = RequiredUseConstraint::AtMostOneOf(vec![
            RequiredUseConstraint::Enabled("foo".to_string()),
            RequiredUseConstraint::Enabled("bar".to_string()),
        ]);
        assert!(c.is_satisfied(&flags(&["foo"])));
        assert!(c.is_satisfied(&flags(&["bar"])));
        assert!(!c.is_satisfied(&flags(&["foo", "bar"])));
        assert!(c.is_satisfied(&flags(&["baz"]))); // None enabled is OK
    }

    #[test]
    fn test_evaluate_conditional() {
        let c = RequiredUseConstraint::Conditional {
            flag: "foo".to_string(),
            positive: true,
            inner: vec![RequiredUseConstraint::Enabled("bar".to_string())],
        };
        // If foo is not enabled, constraint is satisfied (vacuous truth)
        assert!(c.is_satisfied(&flags(&[])));
        assert!(c.is_satisfied(&flags(&["baz"])));
        // If foo is enabled, bar must also be enabled
        assert!(!c.is_satisfied(&flags(&["foo"])));
        assert!(c.is_satisfied(&flags(&["foo", "bar"])));
    }

    #[test]
    fn test_evaluate_negative_conditional() {
        let c = RequiredUseConstraint::Conditional {
            flag: "foo".to_string(),
            positive: false,
            inner: vec![RequiredUseConstraint::Enabled("bar".to_string())],
        };
        // If foo is enabled, constraint is satisfied (condition not met)
        assert!(c.is_satisfied(&flags(&["foo"])));
        // If foo is not enabled, bar must be enabled
        assert!(!c.is_satisfied(&flags(&[])));
        assert!(c.is_satisfied(&flags(&["bar"])));
    }

    #[test]
    fn test_validate_satisfied() {
        let result = validate_required_use(
            "foo bar",
            &flags(&["foo", "bar"]),
            &flags(&["foo", "bar", "baz"]),
        );
        assert!(result.satisfied);
        assert!(result.suggest_enable.is_empty());
        assert!(result.suggest_disable.is_empty());
    }

    #[test]
    fn test_validate_unsatisfied() {
        let result =
            validate_required_use("foo bar", &flags(&["foo"]), &flags(&["foo", "bar", "baz"]));
        assert!(!result.satisfied);
        assert!(result.suggest_enable.contains(&"bar".to_string()));
    }

    #[test]
    fn test_validate_exactly_one_none() {
        let result = validate_required_use("^^ ( a b )", &flags(&[]), &flags(&["a", "b"]));
        assert!(!result.satisfied);
        assert!(!result.suggest_enable.is_empty());
    }

    #[test]
    fn test_validate_exactly_one_multiple() {
        let result = validate_required_use("^^ ( a b )", &flags(&["a", "b"]), &flags(&["a", "b"]));
        assert!(!result.satisfied);
        assert!(!result.suggest_disable.is_empty());
    }

    #[test]
    fn test_empty_required_use() {
        let result = validate_required_use("", &flags(&["foo"]), &flags(&["foo"]));
        assert!(result.satisfied);
    }
}

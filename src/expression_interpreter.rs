use crate::expression_parser::Expr;

pub fn evaluate(expression: &Expr, line: &str) -> bool {
    match expression {
        Expr::And(expressions) => {
            for expr in expressions {
                // AND evaluates to false if any of the expressions evaluates to false
                if !evaluate(expr, line) {
                    return false;
                }
            }
            true
        }
        Expr::Or(expressions) => {
            for expr in expressions {
                // OR evaluates to true if any of the expressions evaluates to true
                if evaluate(expr, line) {
                    return true;
                }
            }
            false
        }
        Expr::Not(expression) => {
            // NOT evaluates to the opposite of the expression
            return !evaluate(expression.as_ref(), line);
        }
        Expr::Regex(regex) => {
            // A regex evaluates to true if it matches the line
            regex.is_match(line)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::expression_parser::{parse};
    use super::*;

    #[test]
    fn test_eval() {
        let input = "AND(OR(abc,def),NOT(ghi))";
        let expression = parse(input).unwrap();
        assert!(evaluate(&expression, "abc"));
        assert!(evaluate(&expression, "def"));
        assert!(!evaluate(&expression, "ghi"));
    }

    #[test]
    fn test_eval2() {
        let input = "AND(OR(abc,def),ghi)";
        let expression = parse(input).unwrap();
        assert!(!evaluate(&expression, "abc"));
        assert!(evaluate(&expression, "abcghi"));
        assert!(evaluate(&expression, "defghi"));
    }

    #[test]
    fn test_eval3() {
        let input = "AND(OR(abc,def),lomb,par)";
        let expression = parse(input).unwrap();
        assert!(!evaluate(&expression, "abc"));
        assert!(!evaluate(&expression, "abclomb"));
        assert!(evaluate(&expression, "deflombxpar"));
    }
}
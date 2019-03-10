#[cfg(test)]
mod tests {
    use crate::dsl::DSLParser;
    use crate::dsl::Rule;
    use pest::Parser;

    #[test]
    fn test_pattern_parser() {
        let patterns = vec![
            "speed > 0",
            "speed > 0 for 3 min or (sin(angle) > 0.5 and voltage <= 220)",
            "speed > 0 for 3 min",
            "speed > 0 for 3 min andThen position = 5",
        ];
        for pattern in patterns {
            let result =
                DSLParser::parse(Rule::pattern, pattern).unwrap_or_else(|e| panic!("{}", e));
            println!("{:?}", result);
        }
    }
}

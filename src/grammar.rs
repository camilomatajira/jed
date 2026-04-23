
use pest::Parser as _;
use pest_derive::Parser;
use regex::Regex;
use anyhow::{Context, Result};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct SedParser;

#[derive(Clone)]
pub struct ArrayRange {
    pub begin: usize,
    pub end: usize,
}
#[derive(Clone)]
pub enum RangeType {
    Key(Regex),
    Array(ArrayRange),
    Value(Regex),
}
pub enum JedCommand {
    Substitute(SubstituteParams),
    SubstituteKeys(SubstituteParams),
    Print,
    Delete,
    Other(()),
}
pub struct SubstituteParams {
    pub pattern: Regex,
    pub replacement: String,
}

pub fn parse_grammar(input: &String) -> Result<(Vec<RangeType>, JedCommand)> {
    let mut stack = Vec::new();
    let parsed = SedParser::parse(Rule::substitute, input).with_context(|| format!("Parsing the jed command failed: {input}"))?;
    let mut pattern = Regex::new("")?;
    let mut replacement = String::from("");
    let mut sed_command = ' ';
    for pair in parsed.into_iter().next().context("Parsing the jed command failed")?.into_inner() {
        match pair.as_rule() {
            Rule::range_regex => {
                for inner_pair in pair.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::key_range_regex => {
                            stack.push(RangeType::Key(
                                Regex::new(inner_pair.as_str().trim_matches('/')).context("Parsing the regex expression failed")?,
                            ));
                        }
                        Rule::array_range_regex => {
                            let mut begin = 0;
                            let mut end = 0;
                            for ip in inner_pair.into_inner() {
                                match ip.as_rule() {
                                    Rule::array_range_regex_begin => {
                                        begin = ip.as_str().parse::<usize>()?;
                                    }
                                    Rule::array_range_regex_end => {
                                        end = ip.as_str().parse::<usize>()?;
                                    }
                                    _ => (),
                                }
                            }

                            stack.push(RangeType::Array(ArrayRange { begin, end }));
                        }
                        Rule::value_range_regex => {
                            stack.push(RangeType::Value(
                                Regex::new(inner_pair.as_str().trim_matches('/')).context("Parsing the regex expression failed")?,
                            ));
                        }
                        _ => (),
                    }
                }
            }
            Rule::sed_command => {
                sed_command = pair.as_str().chars().next().context("Failed to parse the Jed command")?;
            }
            Rule::pattern => pattern = Regex::new(pair.as_str()).context("Parsing the search pattern failed")?,
            Rule::replacement => replacement = pair.as_str().to_string(),
            _ => {}
        }
    }

    if sed_command == 's' {
        return Ok((
            stack,
            JedCommand::Substitute(SubstituteParams {
                pattern,
                replacement,
            }),
        ));
    }
    if sed_command == 'S' {
        return Ok((
            stack,
            JedCommand::SubstituteKeys(SubstituteParams {
                pattern,
                replacement,
            }),
        ));
    }
    if sed_command == 'p' {
        return Ok((stack, JedCommand::Print));
    }
    if sed_command == 'd' {
        return Ok((stack, JedCommand::Delete));
    }
    Ok((stack, JedCommand::Other(())))
}

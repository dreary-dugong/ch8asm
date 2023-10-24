use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Deref;

/// To save allocations, we represent the instructions as an enum after processing so some can use the original string views while others are new strings
pub enum PreprocessedInstruction<'a> {
    Unchanged(&'a str),
    Changed(String),
}

/// To seamlessly call functions on collections of instructions, we implement deref str
impl<'a> Deref for PreprocessedInstruction<'a> {
    type Target = str;
    fn deref(&self) -> &str {
        match self {
            Self::Changed(s) => s.deref(),
            Self::Unchanged(s) => s,
        }
    }
}

impl<'a> From<&'a str> for PreprocessedInstruction<'a> {
    fn from(s: &'a str) -> PreprocessedInstruction<'a> {
        Self::Unchanged(s)
    }
}

// TODO: add error info here
pub enum PreprocessingError {
    TooManyAliasArgs,
    TooFewAliasArgs,
    ReusedAlias,
}

pub fn preprocess(unprocessed: &str) -> Result<Vec<PreprocessedInstruction>, PreprocessingError> {
    // clean up the input before starting preprocessing
    let lines = unprocessed
        .lines()
        .map(|l| l.trim()) // remove leading and trailing whitespace
        .filter(|l| !l.is_empty()) // remove empty lines
        .filter(|l| !l.starts_with(';')) // remove comments
        // convert into preprocessedinstruction enums
        .fold(Vec::new(), |mut acc, l| {
            acc.push(PreprocessedInstruction::from(l));
            acc
        });

    evaluate_aliases(lines)
}

fn evaluate_aliases(
    mut lines: Vec<PreprocessedInstruction>,
) -> Result<Vec<PreprocessedInstruction>, PreprocessingError> {
    let mut alias_map: HashMap<String, String> = HashMap::new();

    // find aliases
    let mut to_remove = Vec::new();
    for (i, line) in lines.iter_mut().enumerate() {
        if line.starts_with("alias") {
            // check for a valid alias
            let tokens = line.split_whitespace().collect::<Vec<&str>>();
            match tokens.len().cmp(&3) {
                Ordering::Greater => return Err(PreprocessingError::TooManyAliasArgs),
                Ordering::Less => return Err(PreprocessingError::TooFewAliasArgs),

                Ordering::Equal => {
                    alias_map.insert(
                        // remove comma
                        tokens[1].trim_end_matches(',').to_string(),
                        tokens[2].to_string(),
                    );
                    to_remove.push(i);
                }
            }
        }
    }

    if alias_map.is_empty() {
        return Ok(lines);
    }

    // remove alias declarations from instructions
    for index in &to_remove {
        lines.remove(*index);
    }

    // replace aliases
    let mut to_replace: Vec<(usize, String)> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let mut replace_with = None;
        // a side effect of this is some weird undefined behavior when an alias contains another alias that's decared after it
        // I wanted to avoid alias ordering mattering but solving this problem is beyond what I feel like dealing with
        for token in line.split_whitespace().map(|s| s.trim_end_matches(',')) {
            if let Some(alias_val) = alias_map.get(token) {
                replace_with = match replace_with {
                    None => Some(line.to_string().replace(token, alias_val)),
                    Some(s) => Some(s.replace(token, alias_val)),
                }
            }
        }

        if let Some(replacement) = replace_with {
            to_replace.push((i, replacement));
        }
    }

    for (i, replacement) in to_replace.into_iter() {
        lines[i] = PreprocessedInstruction::Changed(replacement);
    }

    Ok(lines)
}

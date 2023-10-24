use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;

const RESERVED_WORDS: [&str; 21] = [
    "CLS", "RET", "SYS", "JP", "CALL", "SE", "LD", "ADD", "OR", "AND", "XOR", "SUB", "SHR", "SHL",
    "SUBN", "SNE", "RND", "DRW", "SKP", "SKNP", "alias",
];

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
    ReservedAlias,
    ReusedAlias,
    ReservedLabel,
    InvalidLabel,
    ReusedLabel,
}

pub fn preprocess(unprocessed: &str) -> Result<Vec<PreprocessedInstruction>, PreprocessingError> {
    // clean up the input before starting preprocessing
    let mut lines = unprocessed
        .lines()
        .map(|l| l.trim()) // remove leading and trailing whitespace
        .filter(|l| !l.is_empty()) // remove empty lines
        .filter(|l| !l.starts_with(';')) // remove comments
        // convert into preprocessedinstruction enums
        .fold(Vec::new(), |mut acc, l| {
            acc.push(PreprocessedInstruction::from(l));
            acc
        });

    lines = evaluate_aliases(lines)?;
    evaluate_labels(lines)
}

fn evaluate_aliases(
    mut lines: Vec<PreprocessedInstruction>,
) -> Result<Vec<PreprocessedInstruction>, PreprocessingError> {
    let reserved: HashSet<&str> = HashSet::from(RESERVED_WORDS);
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
                    let key = tokens[1].trim_end_matches(',').to_string(); // remove comma
                                                                           // check if the alias is a reserved word
                    if reserved.contains(&*key) {
                        return Err(PreprocessingError::ReservedAlias);
                    }
                    // check if the alias has already been declared
                    if alias_map.insert(key, tokens[2].to_string()).is_some() {
                        return Err(PreprocessingError::ReusedAlias);
                    } else {
                        to_remove.push(i);
                    }
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

/// Find label declarations in instructions, remove them, and replace references to them with corresponding memory addresses
/// Label syntax is `label:\n`
fn evaluate_labels(
    mut lines: Vec<PreprocessedInstruction>,
) -> Result<Vec<PreprocessedInstruction>, PreprocessingError> {
    let reserved = HashSet::from(RESERVED_WORDS);
    let mut label_map: HashMap<String, usize> = HashMap::new();
    let mut to_remove = Vec::new();

    // find labels, record where the point to, and remove them
    for (i, line) in lines.iter().enumerate() {
        if line.ends_with(':') {
            let label = line.trim_end_matches(':');
            // labels can't contain spaces because that's how we separate tokens
            if label.contains(char::is_whitespace) {
                return Err(PreprocessingError::InvalidLabel);
            // check if the label is a reserved word
            } else if reserved.contains(label) {
                return Err(PreprocessingError::ReservedLabel);

            // the program starts at 0x200 and each instruction is 2 bytes so our label address is 0x200 + 2 times the number of instructions before
            } else if label_map
                .insert(label.to_string(), (i - to_remove.len()) * 2 + 0x200)
                .is_some()
            {
                return Err(PreprocessingError::ReusedLabel);
            } else {
                to_remove.push(i);
            }
        }
    }

    for i in to_remove.into_iter() {
        lines.remove(i);
    }

    // find where labels are used and replace them with their addresses
    let mut to_replace: Vec<(usize, String)> = Vec::new();
    let mut curr_inst_as_string = String::new();
    for (i, line) in lines.iter().enumerate() {
        let mut replace_this_line = false;
        for token in line.split_whitespace() {
            if let Some(addr) = label_map.get(token) {
                curr_inst_as_string.push_str(&format!(" 0x{:x}", addr));
                replace_this_line = true;
            } else {
                curr_inst_as_string.push(' ');
                curr_inst_as_string.push_str(token);
            }
        }
        if replace_this_line {
            to_replace.push((i, curr_inst_as_string));
            curr_inst_as_string = String::new();
        } else {
            curr_inst_as_string.clear();
        }
    }

    for (i, replacement) in to_replace.into_iter() {
        lines[i] = PreprocessedInstruction::Changed(replacement);
    }

    Ok(lines)
}

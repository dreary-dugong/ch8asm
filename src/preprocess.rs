use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;

use thiserror::Error;

// the module path could be cleaned up a bit to make this nicer
use super::assemble::parse::{self, AsmArgParseError};

/// strings that shouldn't be used as aliases or labels because they have other meanings
const RESERVED_WORDS: [&str; 21] = [
    "CLS", "RET", "SYS", "JP", "CALL", "SE", "LD", "ADD", "OR", "AND", "XOR", "SUB", "SHR", "SHL",
    "SUBN", "SNE", "RND", "DRW", "SKP", "SKNP", "alias",
];

/// To save allocations, we represent the instructions as an enum after processing so some can use the original string views while others are new strings
#[derive(Debug)]
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

#[derive(Debug, Error)]
pub enum PreprocessingError {
    #[error("Too many arguments for `alias` preprocessor instruction: {0}")]
    TooManyAliasArgs(String),
    #[error("Too few arguments for `alias` preprocessor instruction: {0}")]
    TooFewAliasArgs(String),
    #[error("Use of reserved word in alias: {0}")]
    ReservedAlias(String),
    #[error("Reused alias in alias declaration: {0}")]
    ReusedAlias(String),
    #[error("Too many arguments for `sprite` preprocessor instruction: {0}")]
    TooManySpriteArgs(String),
    #[error("Too few arguments for `sprite` preprocessor instruction: {0}")]
    TooFewSpriteArgs(String),
    #[error("Missing 'endsprite' instruction for sprite delcared with {0}")]
    UnclosedSprite(String),
    #[error("Sprite of over 15 bytes delcared with {0}")]
    OversizedSprite(String),
    #[error("unable to parse byte in sprite: {0}")]
    InvalidSpriteByte(#[from] AsmArgParseError),
    #[error("Use of reserved word in label: {0}")]
    ReservedLabel(String),
    #[error("Invalid label (probably contains whitespace): {0}")]
    InvalidLabel(String),
    #[error("Invalid memory offset (probably contains nonnumeric characters): {0}")]
    InvalidOffset(String),
    #[error("Reused label in label declaration: {0}")]
    ReusedLabel(String),
}

pub fn preprocess(unprocessed: &str) -> Result<Vec<PreprocessedInstruction>, PreprocessingError> {
    // clean up the input before starting preprocessing
    let mut lines = unprocessed
        .lines()
        .map(|l| l.trim()) // remove leading and trailing whitespace
        .filter(|l| !l.is_empty()) // remove empty lines
        .filter(|l| !l.starts_with(';')) // remove comment lines
        // remove comments at the ends of lines
        .map(|l| match l.find(';') {
            None => l,
            Some(i) => &l[..i],
        })
        // convert into preprocessedinstruction enums
        .fold(Vec::new(), |mut acc, l| {
            acc.push(PreprocessedInstruction::from(l));
            acc
        });

    lines = evaluate_aliases(lines)?;
    lines = evaluate_sprites(lines)?;
    lines = evaluate_memory_offsets(lines)?;
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
                Ordering::Greater => {
                    return Err(PreprocessingError::TooManyAliasArgs(line.to_string()))
                }
                Ordering::Less => {
                    return Err(PreprocessingError::TooFewAliasArgs(line.to_string()))
                }

                Ordering::Equal => {
                    let key = tokens[1].trim_end_matches(',').to_string(); // remove comma
                                                                           // check if the alias is a reserved word
                    if reserved.contains(&*key) {
                        return Err(PreprocessingError::ReservedAlias(line.to_string()));
                    }
                    // check if the alias has already been declared
                    if alias_map.insert(key, tokens[2].to_string()).is_some() {
                        return Err(PreprocessingError::ReusedAlias(line.to_string()));
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
    for (i, index) in to_remove.into_iter().enumerate() {
        lines.remove(index - i);
    }

    // replace aliases
    let mut to_replace: Vec<(usize, String)> = Vec::new();
    let mut curr_inst_as_string = String::new();
    for (i, line) in lines.iter().enumerate() {
        let mut replace_this_line = false;

        for token in line.split_whitespace().map(|s| s.trim_end_matches(',')) {
            curr_inst_as_string.push(' ');
            if let Some(alias_val) = alias_map.get(token) {
                curr_inst_as_string.push_str(alias_val);
                replace_this_line = true;
            } else {
                curr_inst_as_string.push_str(token);
            }
        }

        if replace_this_line {
            to_replace.push((i, String::from(curr_inst_as_string.trim())));
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

/// Find sprite blocks, condense the bytes into raw hex strings and replace the sprite declaration with a label
/// sprite syntax is `sprite NAME` (with an optional colon), any number of bytes beginning with 0b then `endsprite`
fn evaluate_sprites(
    mut lines: Vec<PreprocessedInstruction>,
) -> Result<Vec<PreprocessedInstruction>, PreprocessingError> {
    let mut to_change: Vec<(usize, PreprocessedInstruction)> = Vec::new();
    let mut to_remove: Vec<usize> = Vec::new();
    // iterate over the lines, looking for sprite instructions
    let mut i = 0;
    while i < lines.len() {
        let cur_line = &lines[i];

        if cur_line.starts_with("sprite") {
            // once we have a sprite instruction, make sure it's valid
            let tokens = cur_line.split_whitespace().collect::<Vec<_>>();
            match tokens.len().cmp(&2) {
                Ordering::Less => {
                    return Err(PreprocessingError::TooFewSpriteArgs(cur_line.to_string()))
                }
                Ordering::Greater => {
                    return Err(PreprocessingError::TooManySpriteArgs(cur_line.to_string()))
                }

                // find the end of the sprite, pair up bytes, and convert to raws
                Ordering::Equal => {
                    let sprite_start = i;
                    while &*lines[i] != "endsprite" {
                        // this is cursed
                        i += 1;
                        if i == lines.len() {
                            return Err(PreprocessingError::UnclosedSprite(cur_line.to_string()));
                        }
                    }
                    let sprite_end = i;
                    if sprite_end - sprite_start > 16 {
                        return Err(PreprocessingError::OversizedSprite(cur_line.to_string()));
                    };
                    process_sprite(
                        &mut lines,
                        sprite_start,
                        sprite_end,
                        &mut to_change,
                        &mut to_remove,
                    )?;
                }
            }
        }
        i += 1;
    }

    for (i, change_to) in to_change.into_iter() {
        lines[i] = change_to;
    }

    for (i, index) in to_remove.into_iter().enumerate() {
        lines.remove(index - i);
    }

    Ok(lines)
}

/// Given the bounds of a sprite declared in lines, record the necessary changes to process it, or error if it can't be parsed
fn process_sprite(
    lines: &mut [PreprocessedInstruction],
    start: usize,
    end: usize,
    change_list: &mut Vec<(usize, PreprocessedInstruction)>,
    remove_list: &mut Vec<usize>,
) -> Result<(), PreprocessingError> {
    // I beg your forgiveness for this unholy abomination
    let sprite_bytes = parse::parse_asm_args(
        // convert our preprocessed instructions into string slices in order to use our parse module
        &(lines[start + 1..end]
            .iter()
            .map(|l| &**l)
            .collect::<Vec<_>>()),
    )?
    .into_iter()
    .map(|arg| parse::parse_valid_byte(&arg).map_err(PreprocessingError::from))
    .collect::<Result<Vec<u8>, PreprocessingError>>()?;

    // pair up bytes and convert to u16
    let raws = sprite_bytes
        .chunks(2)
        .map(|chunk| ((chunk[0] as u16) << 8) + if chunk.len() == 2 { chunk[1] as u16 } else { 0 });

    // we're going to convert the sprite block into a label and raws, so let's start with the label
    let mut new_label = lines[start]
        .strip_prefix("sprite")
        .expect("We check that this starts with sprite in the calling context")
        .trim()
        .to_string();
    if !new_label.ends_with(':') {
        new_label.push(':')
    };
    change_list.push((start, PreprocessedInstruction::Changed(new_label)));

    let remove_threshold = start + raws.len() + 1;

    // record changes from bytes to raws
    for (i, raw) in raws.into_iter().enumerate() {
        change_list.push((
            start + 1 + i,
            PreprocessedInstruction::Changed(format!("{raw:#X}")),
        ));
    }

    // record deletions for extra bytes
    for i in remove_threshold..=end {
        remove_list.push(i);
    }

    Ok(())
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
                return Err(PreprocessingError::InvalidLabel(line.to_string()));
            // check if the label is a reserved word
            } else if reserved.contains(label) {
                return Err(PreprocessingError::ReservedLabel(line.to_string()));

            // the program starts at 0x200 and each instruction is 2 bytes so our label address is 0x200 + 2 times the number of instructions before
            } else if label_map
                .insert(label.to_string(), (i - to_remove.len()) * 2 + 0x200)
                .is_some()
            {
                return Err(PreprocessingError::ReusedLabel(line.to_string()));
            } else {
                to_remove.push(i);
            }
        }
    }

    for (i, index) in to_remove.into_iter().enumerate() {
        lines.remove(index - i);
    }

    // find where labels are used and replace them with their addresses
    let mut to_replace: Vec<(usize, String)> = Vec::new();
    let mut curr_inst_as_string = String::new();
    for (i, line) in lines.iter().enumerate() {
        let mut replace_this_line = false;

        for token in line.split_whitespace() {
            if let Some(addr) = label_map.get(token.trim_end_matches(',')) {
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

/// Find instances of the #n free memory offset syntax and replace them with
/// correct addresses based on the length of the program
fn evaluate_memory_offsets(
    mut lines: Vec<PreprocessedInstruction>,
) -> Result<Vec<PreprocessedInstruction>, PreprocessingError> {
    // determine where offset #0 is
    let used_memory = 2 * lines.len();

    // iterate over the instructions until we stop finding offsets
    let mut found_changes = true;
    while found_changes {
        found_changes = false;

        // keep track of instructions to swap out with evaluated alternatives
        let mut to_replace = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if let Some(index) = line.find('#') {
                found_changes = true;

                // equivalent regex would be \#.+ but we throw an error if it's not numeric
                let start = index + 1;
                let mut end = index + 1;
                // i hate that we need copy and an allocation
                let chars = line.chars().collect::<Vec<_>>();
                while end < chars.len() && !chars[end].is_whitespace() {
                    end += 1;
                }

                if start >= chars.len() {
                    return Err(PreprocessingError::InvalidOffset(line.to_string()));
                }

                let offset: usize = str::parse(&line[start..end])
                    .map_err(|_| PreprocessingError::InvalidOffset(line.to_string()))?;

                // replace the instruction with one that uses a raw decimal number instead
                let mut replacement = line[0..index].to_string();
                replacement.push_str(&(used_memory + offset).to_string());
                if end < line.len() - 1 {
                    replacement.push_str(&line[(end + 1)..]);
                }

                to_replace.push((i, replacement))
            }
        }

        for (index, new_str) in to_replace.into_iter() {
            lines[index] = PreprocessedInstruction::Changed(new_str)
        }
    }

    Ok(lines)
}

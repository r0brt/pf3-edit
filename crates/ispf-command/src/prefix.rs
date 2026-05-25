#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrefixCommand {
    Insert(usize),
    Delete(usize),
    DeleteBlock,
    Repeat(usize),
    RepeatBlock,
    TextSplit(usize),
    TextFlow(Option<usize>),
    Copy(usize),
    CopyBlock,
    Move(usize),
    MoveBlock,
    Overlay,
    OverlayBlock,
    Show,
    Lowercase(usize),
    LowercaseBlock,
    Uppercase(usize),
    UppercaseBlock,
    After,
    Before,
    Exclude,
    ExcludeBlock,
}

pub fn parse_prefix(input: &str) -> Result<PrefixCommand, String> {
    let normalized = input.trim().to_ascii_uppercase();
    match normalized.as_str() {
        "DD" => Ok(PrefixCommand::DeleteBlock),
        "RR" => Ok(PrefixCommand::RepeatBlock),
        "CC" => Ok(PrefixCommand::CopyBlock),
        "MM" => Ok(PrefixCommand::MoveBlock),
        "OO" => Ok(PrefixCommand::OverlayBlock),
        "LCC" => Ok(PrefixCommand::LowercaseBlock),
        "UCC" => Ok(PrefixCommand::UppercaseBlock),
        "XX" => Ok(PrefixCommand::ExcludeBlock),
        "A" => Ok(PrefixCommand::After),
        "B" => Ok(PrefixCommand::Before),
        "O" => Ok(PrefixCommand::Overlay),
        "S" => Ok(PrefixCommand::Show),
        "X" => Ok(PrefixCommand::Exclude),
        other if matches_multi_letter_counted_line_command(other, "TS") => Ok(PrefixCommand::TextSplit(
            other
                .strip_prefix("TS")
                .filter(|suffix| !suffix.is_empty())
                .and_then(|suffix| suffix.parse().ok())
                .unwrap_or(0),
        )),
        other if matches_multi_letter_counted_line_command(other, "TF") => {
            let width = parse_multi_letter_count(other, 2);
            Ok(PrefixCommand::TextFlow((width != 1 || other.len() > 2).then_some(width)))
        }
        other if matches_multi_letter_counted_line_command(other, "LC") => {
            Ok(PrefixCommand::Lowercase(parse_multi_letter_count(other, 2)))
        }
        other if matches_multi_letter_counted_line_command(other, "UC") => {
            Ok(PrefixCommand::Uppercase(parse_multi_letter_count(other, 2)))
        }
        other if matches_counted_line_command(other, 'I') => {
            Ok(PrefixCommand::Insert(parse_line_command_count(other)))
        }
        other if matches_counted_line_command(other, 'D') => {
            Ok(PrefixCommand::Delete(parse_line_command_count(other)))
        }
        other if matches_counted_line_command(other, 'R') => {
            Ok(PrefixCommand::Repeat(parse_line_command_count(other)))
        }
        other if matches_counted_line_command(other, 'C') => {
            Ok(PrefixCommand::Copy(parse_line_command_count(other)))
        }
        other if matches_counted_line_command(other, 'M') => {
            Ok(PrefixCommand::Move(parse_line_command_count(other)))
        }
        _ => Err(format!("unknown line command: {}", input.trim())),
    }
}

fn matches_counted_line_command(input: &str, command: char) -> bool {
    let mut chars = input.chars();
    matches!(chars.next(), Some(first) if first == command)
        && chars.all(|ch| ch.is_ascii_digit())
}

fn parse_line_command_count(input: &str) -> usize {
    let suffix = &input[1..];
    if suffix.is_empty() {
        1
    } else {
        suffix.parse().unwrap_or(1)
    }
}

fn matches_multi_letter_counted_line_command(input: &str, command: &str) -> bool {
    input
        .strip_prefix(command)
        .is_some_and(|suffix| !suffix.starts_with(command) && suffix.chars().all(|ch| ch.is_ascii_digit()))
}

fn parse_multi_letter_count(input: &str, prefix_len: usize) -> usize {
    let suffix = &input[prefix_len..];
    if suffix.is_empty() {
        1
    } else {
        suffix.parse().unwrap_or(1)
    }
}

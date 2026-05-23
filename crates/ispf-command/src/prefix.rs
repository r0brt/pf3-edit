#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrefixCommand {
    Insert,
    Delete,
    Repeat,
    After,
    Before,
    Exclude,
    ExcludeBlock,
}

pub fn parse_prefix(input: &str) -> Result<PrefixCommand, String> {
    match input.trim() {
        "I" => Ok(PrefixCommand::Insert),
        "D" => Ok(PrefixCommand::Delete),
        "R" => Ok(PrefixCommand::Repeat),
        "A" => Ok(PrefixCommand::After),
        "B" => Ok(PrefixCommand::Before),
        "X" => Ok(PrefixCommand::Exclude),
        "XX" => Ok(PrefixCommand::ExcludeBlock),
        other => Err(format!("unknown prefix command: {other}")),
    }
}

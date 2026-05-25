#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrimaryCommand {
    Save,
    Cancel,
    End,
    Find { pattern: String },
    RFind,
    Change { from: String, to: String },
    RChange,
    Locate { target: usize },
    Cols,
    Reset,
    Up(usize),
    Down(usize),
    Left(usize),
    Right(usize),
    Bounds(Option<(usize, usize)>),
    Number(bool),
    Caps(bool),
    Undo,
}

pub fn parse_primary(input: &str) -> Result<PrimaryCommand, String> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let upper: Vec<String> = parts.iter().map(|part| part.to_ascii_uppercase()).collect();
    let upper_parts: Vec<&str> = upper.iter().map(String::as_str).collect();

    match upper_parts.as_slice() {
        ["SAVE"] => Ok(PrimaryCommand::Save),
        ["CANCEL"] => Ok(PrimaryCommand::Cancel),
        ["END"] => Ok(PrimaryCommand::End),
        ["RFIND"] => Ok(PrimaryCommand::RFind),
        ["RCHANGE"] => Ok(PrimaryCommand::RChange),
        ["LOCATE", _] | ["L", _] => Ok(PrimaryCommand::Locate {
            target: parts[1]
                .parse()
                .map_err(|_| "invalid LOCATE target".to_string())?,
        }),
        ["COLS"] => Ok(PrimaryCommand::Cols),
        ["RESET"] => Ok(PrimaryCommand::Reset),
        ["UNDO"] => Ok(PrimaryCommand::Undo),
        ["UNNUM"] => Ok(PrimaryCommand::Number(false)),
        ["NUMBER"] => Ok(PrimaryCommand::Number(true)),
        ["CAPS", "ON"] => Ok(PrimaryCommand::Caps(true)),
        ["CAPS", "OFF"] => Ok(PrimaryCommand::Caps(false)),
        ["FIND", rest @ ..] if !rest.is_empty() => Ok(PrimaryCommand::Find {
            pattern: parts[1..].join(" "),
        }),
        ["CHANGE", _, _] => Ok(PrimaryCommand::Change {
            from: parts[1].into(),
            to: parts[2].into(),
        }),
        ["BOUNDS"] => Ok(PrimaryCommand::Bounds(None)),
        ["BOUNDS", _, _] => Ok(PrimaryCommand::Bounds(Some((
            parts[1]
                .parse()
                .map_err(|_| "invalid BOUNDS left column".to_string())?,
            if upper_parts[2] == "*" {
                144
            } else {
                parts[2]
                    .parse()
                    .map_err(|_| "invalid BOUNDS right column".to_string())?
            },
        )))),
        ["UP", _] => Ok(PrimaryCommand::Up(
            parts[1]
                .parse()
                .map_err(|_| "invalid UP count".to_string())?,
        )),
        ["DOWN", _] => Ok(PrimaryCommand::Down(
            parts[1]
                .parse()
                .map_err(|_| "invalid DOWN count".to_string())?,
        )),
        ["LEFT", _] => Ok(PrimaryCommand::Left(
            parts[1]
                .parse()
                .map_err(|_| "invalid LEFT count".to_string())?,
        )),
        ["RIGHT", _] => Ok(PrimaryCommand::Right(
            parts[1]
                .parse()
                .map_err(|_| "invalid RIGHT count".to_string())?,
        )),
        _ => Err(format!("unknown primary command: {input}")),
    }
}

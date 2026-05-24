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
    match parts.as_slice() {
        ["SAVE"] => Ok(PrimaryCommand::Save),
        ["CANCEL"] => Ok(PrimaryCommand::Cancel),
        ["END"] => Ok(PrimaryCommand::End),
        ["RFIND"] => Ok(PrimaryCommand::RFind),
        ["RCHANGE"] => Ok(PrimaryCommand::RChange),
        ["LOCATE", target] | ["L", target] => Ok(PrimaryCommand::Locate {
            target: target
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
            pattern: rest.join(" "),
        }),
        ["CHANGE", from, to] => Ok(PrimaryCommand::Change {
            from: (*from).into(),
            to: (*to).into(),
        }),
        ["BOUNDS"] => Ok(PrimaryCommand::Bounds(None)),
        ["BOUNDS", left, right] => Ok(PrimaryCommand::Bounds(Some((
            left.parse()
                .map_err(|_| "invalid BOUNDS left column".to_string())?,
            if *right == "*" {
                144
            } else {
                right
                    .parse()
                    .map_err(|_| "invalid BOUNDS right column".to_string())?
            },
        )))),
        ["UP", count] => Ok(PrimaryCommand::Up(
            count
                .parse()
                .map_err(|_| "invalid UP count".to_string())?,
        )),
        ["DOWN", count] => Ok(PrimaryCommand::Down(
            count
                .parse()
                .map_err(|_| "invalid DOWN count".to_string())?,
        )),
        ["LEFT", count] => Ok(PrimaryCommand::Left(
            count
                .parse()
                .map_err(|_| "invalid LEFT count".to_string())?,
        )),
        ["RIGHT", count] => Ok(PrimaryCommand::Right(
            count
                .parse()
                .map_err(|_| "invalid RIGHT count".to_string())?,
        )),
        _ => Err(format!("unknown primary command: {input}")),
    }
}

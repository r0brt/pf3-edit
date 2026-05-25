#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollMode {
    Page,
    Half,
    Csr,
}

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
    Scroll(ScrollMode),
    Reset,
    Up(Option<usize>),
    Down(Option<usize>),
    Left(usize),
    Right(usize),
    Bounds(Option<(usize, usize)>),
    Number(bool),
    Caps(bool),
    Undo,
}

pub fn parse_primary(input: &str) -> Result<PrimaryCommand, String> {
    let parts = tokenize_primary(input)?;
    let part_refs: Vec<&str> = parts.iter().map(String::as_str).collect();
    let upper: Vec<String> = part_refs
        .iter()
        .map(|part| part.to_ascii_uppercase())
        .collect();
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
        ["SCROLL", "PAGE"] => Ok(PrimaryCommand::Scroll(ScrollMode::Page)),
        ["SCROLL", "HALF"] => Ok(PrimaryCommand::Scroll(ScrollMode::Half)),
        ["SCROLL", "CSR"] => Ok(PrimaryCommand::Scroll(ScrollMode::Csr)),
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
            from: parts[1].clone(),
            to: parts[2].clone(),
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
        ["UP"] => Ok(PrimaryCommand::Up(None)),
        ["UP", _] => Ok(PrimaryCommand::Up(Some(
            parts[1]
                .parse()
                .map_err(|_| "invalid UP count".to_string())?,
        ))),
        ["DOWN"] => Ok(PrimaryCommand::Down(None)),
        ["DOWN", _] => Ok(PrimaryCommand::Down(Some(
            parts[1]
                .parse()
                .map_err(|_| "invalid DOWN count".to_string())?,
        ))),
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

fn tokenize_primary(input: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;

    for ch in input.chars() {
        match quote {
            Some(active) if ch == active => {
                quote = None;
            }
            Some(_) => current.push(ch),
            None if ch == '"' || ch == '\'' => {
                quote = Some(ch);
            }
            None if ch.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            None => current.push(ch),
        }
    }

    if quote.is_some() {
        return Err("unterminated quoted argument".into());
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    Ok(tokens)
}

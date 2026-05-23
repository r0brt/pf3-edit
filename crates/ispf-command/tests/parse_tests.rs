use ispf_command::{parse_prefix, parse_primary, PrefixCommand, PrimaryCommand};

#[test]
fn parses_basic_primary_commands() {
    assert_eq!(parse_primary("SAVE").unwrap(), PrimaryCommand::Save);
    assert_eq!(parse_primary("UNNUM").unwrap(), PrimaryCommand::Number(false));
    assert_eq!(parse_primary("CAPS ON").unwrap(), PrimaryCommand::Caps(true));
}

#[test]
fn parses_find_and_change_commands() {
    assert_eq!(
        parse_primary("FIND ALPHA").unwrap(),
        PrimaryCommand::Find {
            pattern: "ALPHA".into()
        }
    );
    assert_eq!(
        parse_primary("CHANGE OLD NEW").unwrap(),
        PrimaryCommand::Change {
            from: "OLD".into(),
            to: "NEW".into()
        }
    );
}

#[test]
fn parses_prefix_commands() {
    assert_eq!(parse_prefix("I").unwrap(), PrefixCommand::Insert);
    assert_eq!(parse_prefix("XX").unwrap(), PrefixCommand::ExcludeBlock);
}

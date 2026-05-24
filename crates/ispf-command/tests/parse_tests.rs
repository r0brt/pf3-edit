use ispf_command::{parse_prefix, parse_primary, PrefixCommand, PrimaryCommand};

#[test]
fn parses_basic_primary_commands() {
    assert_eq!(parse_primary("SAVE").unwrap(), PrimaryCommand::Save);
    assert_eq!(parse_primary("UNNUM").unwrap(), PrimaryCommand::Number(false));
    assert_eq!(parse_primary("CAPS ON").unwrap(), PrimaryCommand::Caps(true));
    assert_eq!(parse_primary("COLS").unwrap(), PrimaryCommand::Cols);
    assert_eq!(parse_primary("BOUNDS").unwrap(), PrimaryCommand::Bounds(None));
    assert_eq!(
        parse_primary("BOUNDS 7 70").unwrap(),
        PrimaryCommand::Bounds(Some((7, 70)))
    );
    assert_eq!(
        parse_primary("BOUNDS 10 *").unwrap(),
        PrimaryCommand::Bounds(Some((10, 144)))
    );
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
    assert_eq!(
        parse_primary("LOCATE 12").unwrap(),
        PrimaryCommand::Locate { target: 12 }
    );
    assert_eq!(
        parse_primary("L 7").unwrap(),
        PrimaryCommand::Locate { target: 7 }
    );
}

#[test]
fn parses_prefix_commands() {
    assert_eq!(parse_prefix("I").unwrap(), PrefixCommand::Insert(1));
    assert_eq!(parse_prefix("I3").unwrap(), PrefixCommand::Insert(3));
    assert_eq!(parse_prefix("D").unwrap(), PrefixCommand::Delete(1));
    assert_eq!(parse_prefix("D4").unwrap(), PrefixCommand::Delete(4));
    assert_eq!(parse_prefix("DD").unwrap(), PrefixCommand::DeleteBlock);
    assert_eq!(parse_prefix("R").unwrap(), PrefixCommand::Repeat(1));
    assert_eq!(parse_prefix("R2").unwrap(), PrefixCommand::Repeat(2));
    assert_eq!(parse_prefix("RR").unwrap(), PrefixCommand::RepeatBlock);
    assert_eq!(parse_prefix("C").unwrap(), PrefixCommand::Copy(1));
    assert_eq!(parse_prefix("C3").unwrap(), PrefixCommand::Copy(3));
    assert_eq!(parse_prefix("CC").unwrap(), PrefixCommand::CopyBlock);
    assert_eq!(parse_prefix("M").unwrap(), PrefixCommand::Move(1));
    assert_eq!(parse_prefix("M2").unwrap(), PrefixCommand::Move(2));
    assert_eq!(parse_prefix("MM").unwrap(), PrefixCommand::MoveBlock);
    assert_eq!(parse_prefix("O").unwrap(), PrefixCommand::Overlay);
    assert_eq!(parse_prefix("OO").unwrap(), PrefixCommand::OverlayBlock);
    assert_eq!(parse_prefix("LC").unwrap(), PrefixCommand::Lowercase(1));
    assert_eq!(parse_prefix("LC3").unwrap(), PrefixCommand::Lowercase(3));
    assert_eq!(parse_prefix("LCC").unwrap(), PrefixCommand::LowercaseBlock);
    assert_eq!(parse_prefix("UC").unwrap(), PrefixCommand::Uppercase(1));
    assert_eq!(parse_prefix("UC2").unwrap(), PrefixCommand::Uppercase(2));
    assert_eq!(parse_prefix("UCC").unwrap(), PrefixCommand::UppercaseBlock);
    assert_eq!(parse_prefix("A").unwrap(), PrefixCommand::After);
    assert_eq!(parse_prefix("B").unwrap(), PrefixCommand::Before);
    assert_eq!(parse_prefix("XX").unwrap(), PrefixCommand::ExcludeBlock);
    assert_eq!(
        parse_prefix("ZZ").unwrap_err(),
        "unknown line command: ZZ"
    );
}

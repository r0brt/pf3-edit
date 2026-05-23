use ispf_core::{ActiveArea, CapsMode, EditBuffer, EditProfile, EditorSession};

#[test]
fn new_session_starts_in_data_area() {
    let buffer = EditBuffer::from_text("A\n").unwrap();
    let session = EditorSession::new(buffer);
    assert_eq!(session.view().active_area, ActiveArea::DataArea);
}

#[test]
fn profile_defaults_match_v0_1_design() {
    let profile = EditProfile::default();
    assert_eq!(profile.caps_mode, CapsMode::Off);
    assert!(!profile.number_mode);
    assert_eq!(profile.bounds, None);
}

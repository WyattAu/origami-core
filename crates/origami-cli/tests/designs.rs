//! Design-library gate (REQ-O-004): every `designs/**/*.fold` in the
//! repository must load and pass both flat-foldability gates. This is the
//! executable oracle for the civilian design library.

use std::path::Path;

#[test]
fn every_design_in_library_is_valid() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../designs");
    let walked = origami_cli::run_fold_check(&root, true);
    assert!(
        walked.is_ok(),
        "design library must be walkable: {walked:?}"
    );
    let Ok(reports) = walked else {
        unreachable!("walk success already asserted")
    };
    assert!(
        !reports.is_empty(),
        "design library must contain at least one .fold file"
    );
    let failures: Vec<String> = reports
        .iter()
        .filter(|r| !r.is_pass())
        .map(|r| format!("{}", r.path.display()))
        .collect();
    assert!(
        failures.is_empty(),
        "invalid designs in library: {failures:?}"
    );
    println!("validated {} design file(s)", reports.len());
}

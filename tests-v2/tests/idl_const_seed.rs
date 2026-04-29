use std::{process::Command, sync::OnceLock};

static OUTPUT: OnceLock<(String, String)> = OnceLock::new();

fn idl_output() -> &'static (String, String) {
    OUTPUT.get_or_init(|| {
        let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("programs/idl-const-seed/Cargo.toml");
        let target = std::env::temp_dir().join("anchor-bug-idl-const-seed");

        let out = Command::new("cargo")
            .args([
                "test",
                "--manifest-path",
                manifest.to_str().unwrap(),
                "--target-dir",
                target.to_str().unwrap(),
                "--features",
                "idl-build",
                "__anchor_private_print_idl_program",
                "--",
                "--nocapture",
            ])
            .output()
            .expect("cargo ran");
        (
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    })
}

#[test]
fn bug_classify_warning_in_stderr() {
    let (_stdout, stderr) = idl_output();
    // FAIL after fix: warning gone; PASS today: macro emits warning for `MY_SEED`.
    assert!(
        stderr.contains("unable to classify seed expression `MY_SEED`"),
        "expected classify-failure warning. stderr={stderr}"
    );
}

#[test]
fn bug_idl_seed_value_is_empty() {
    let (stdout, _stderr) = idl_output();
    let begin = stdout
        .find("--- IDL begin program ---")
        .expect("idl begin marker");
    let end = stdout
        .find("--- IDL end program ---")
        .expect("idl end marker");
    let idl = &stdout[begin..end];

    // Fail today: const seed bytes not emitted
    assert!(
        idl.contains("[104,101,108,108,111]"),
        "expected seed value in IDL. idl={idl}"
    );
}

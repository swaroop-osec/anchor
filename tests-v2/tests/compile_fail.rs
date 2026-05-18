use std::{fs, path::PathBuf, process::Command};

fn compile_fail_case(name: &str, source: &str, snippets: &[&str]) {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let crate_dir = manifest_dir.join("target/compile-fail").join(name);
    let src_dir = crate_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    let anchor_lang_v2 = manifest_dir
        .parent()
        .expect("tests-v2 should live under the workspace root")
        .join("lang-v2");

    fs::write(
        crate_dir.join("Cargo.toml"),
        format!(
            r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
anchor-lang-v2 = {{ path = "{}" }}
wincode = {{ version = "0.5", features = ["derive"] }}

[workspace]
"#,
            anchor_lang_v2.display()
        ),
    )
    .unwrap();
    fs::write(src_dir.join("lib.rs"), source).unwrap();

    let output = Command::new("cargo")
        .args(["check", "--offline", "--manifest-path"])
        .arg(crate_dir.join("Cargo.toml"))
        .output()
        .unwrap_or_else(|err| panic!("failed to run cargo check for {name}: {err}"));

    assert!(
        !output.status.success(),
        "{name} unexpectedly compiled successfully"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    for snippet in snippets {
        assert!(
            stderr.contains(snippet),
            "{name} stderr did not contain {snippet:?}\n\nstderr:\n{stderr}"
        );
    }
}

#[test]
fn instruction_args_without_handler_args_do_not_compile() {
    compile_fail_case(
        "instruction_args_without_handler_args",
        r#"
use anchor_lang_v2::prelude::*;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod instruction_args_without_handler_args {
    use super::*;

    pub fn ix(ctx: &mut Context<Bad>) -> Result<()> {
        let _ = ctx;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(value: u64)]
pub struct Bad {
    #[account(constraint = value > 0)]
    pub data: UncheckedAccount,
}
"#,
        &["expected `()`, found `(u64,)`"],
    );
}

#[test]
fn extra_instruction_args_do_not_compile() {
    compile_fail_case(
        "extra_instruction_args",
        r#"
use anchor_lang_v2::prelude::*;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod extra_instruction_args {
    use super::*;

    pub fn ix(ctx: &mut Context<Bad>, value: u64) -> Result<()> {
        let _ = (ctx, value);
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(value: u64, other: u8)]
pub struct Bad {
    #[account(constraint = value > 0 && other > 0)]
    pub data: UncheckedAccount,
}
"#,
        &["the trait bound", "__AnchorIxArgCoerce"],
    );
}

#[test]
fn missing_instruction_args_do_not_compile() {
    compile_fail_case(
        "missing_instruction_args",
        r#"
use anchor_lang_v2::prelude::*;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod missing_instruction_args {
    use super::*;

    pub fn ix(ctx: &mut Context<Bad>, value: u64, other: u8) -> Result<()> {
        let _ = (ctx, value, other);
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(value: u64)]
pub struct Bad {
    #[account(constraint = value > 0)]
    pub data: UncheckedAccount,
}
"#,
        &["the trait bound", "__AnchorIxArgCoerce"],
    );
}

#[test]
fn wrong_instruction_arg_type_does_not_compile() {
    compile_fail_case(
        "wrong_instruction_arg_type",
        r#"
use anchor_lang_v2::prelude::*;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod wrong_instruction_arg_type {
    use super::*;

    pub fn ix(ctx: &mut Context<Bad>, value: u64) -> Result<()> {
        let _ = (ctx, value);
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(value: u8)]
pub struct Bad {
    #[account(constraint = value > 0)]
    pub data: UncheckedAccount,
}
"#,
        &["the trait bound", "__AnchorIxArgCoerce"],
    );
}

#[test]
fn swapped_instruction_arg_types_do_not_compile() {
    compile_fail_case(
        "swapped_instruction_arg_types",
        r#"
use anchor_lang_v2::prelude::*;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod swapped_instruction_arg_types {
    use super::*;

    pub fn ix(ctx: &mut Context<Bad>, amount: u64, flag: u8) -> Result<()> {
        let _ = (ctx, amount, flag);
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(amount: u8, flag: u64)]
pub struct Bad {
    #[account(constraint = amount > 0 && flag > 0)]
    pub data: UncheckedAccount,
}
"#,
        &["the trait bound", "__AnchorIxArgCoerce"],
    );
}

use std::fs::{File, read_to_string};
use std::io::{Result, Write};
use serde_derive::Deserialize;
use std::collections::HashMap;

fn main() {
    println!("cargo:rerun-if-changed=../user/src/");
    println!("cargo:rerun-if-changed=/src/");
    println!("cargo:rerun-if-changed=../lib_so/src/");
    println!("cargo:rerun-if-changed=../user/cases.toml");
    insert_app_data().unwrap();
}

static TARGET_PATH: &str = "../user/target/riscv64gc-unknown-none-elf/release/";
#[derive(Deserialize, Default, Debug)]
struct Cases {
    pub cases: Option<Vec<String>>,
}

fn insert_app_data() -> Result<()> {
    let mut f = File::create("src/link_app.asm").unwrap();
    let cfg = read_to_string("../user/cases.toml").unwrap();
    let cases = toml::from_str::<HashMap<String, Cases>>(&cfg)
        .unwrap()
        .remove(&format!("usercases"))
        .unwrap_or_default();
    println!("{:?}", cases);
    let apps: Vec<_> = cases.cases.unwrap();
    
    writeln!(
        f,
        r#"
    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad {}"#,
        apps.len()
    )?;

    for i in 0..apps.len() {
        writeln!(f, r#"    .quad app_{}_start"#, i)?;
    }
    writeln!(f, r#"    .quad app_{}_end"#, apps.len() - 1)?;

    writeln!(
        f,
        r#"
    .global _app_names
_app_names:"#
    )?;
    for app in apps.iter() {
        writeln!(f, r#"    .string "{}""#, app)?;
    }

    for (idx, app) in apps.iter().enumerate() {
        println!("app_{}: {}", idx, app);
        writeln!(
            f,
            r#"
    .section .data
    .global app_{0}_start
    .global app_{0}_end
    .align 3
app_{0}_start:
    .incbin "{2}{1}"
app_{0}_end:"#,
            idx, app, TARGET_PATH
        )?;
    }
    Ok(())
}

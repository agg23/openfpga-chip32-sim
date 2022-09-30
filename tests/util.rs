use std::{collections::HashMap, env, fs, process::Command};

use chip32_sim::cpu::CPU;
use regex::Regex;

// Testing

pub fn test_command<TS: FnOnce(&mut CPU) -> (), TA: FnOnce(&CPU) -> ()>(
    asm_path: &str,
    output_path: &str,
    replacements: HashMap<&str, &str>,
    step_count: u32,
    setup: TS,
    assertions: TA,
) -> CPU {
    let mut cpu = prep_and_load(asm_path, output_path, replacements);

    setup(&mut cpu);

    for _ in 0..step_count {
        cpu.step();
    }

    assertions(&cpu);

    cpu
}

pub fn test_command_without_setup<T: FnOnce(&CPU) -> ()>(
    asm_path: &str,
    output_path: &str,
    replacements: HashMap<&str, &str>,
    step_count: u32,
    assertions: T,
) -> CPU {
    test_command(
        asm_path,
        output_path,
        replacements,
        step_count,
        |_| {},
        assertions,
    )
}

// Loading

pub fn tmp_path() -> String {
    "tests/chip32-tmp.asm".to_string()
}

pub fn prep_test(asm_path: &str, replacements: HashMap<&str, &str>) {
    let tmp_path = tmp_path();

    let mut asm = fs::read_to_string(asm_path).expect(&format!("Unable to read {asm_path}"));

    // TODO: This is naive
    for (original, replacement) in replacements.into_iter() {
        let match_regex = Regex::new(&format!("\\{{{original}\\}}")).unwrap();

        asm = match_regex.replace_all(&asm, replacement).to_string();
    }

    fs::write(&tmp_path, asm).expect(&format!("Unable to write to {tmp_path}"));
}

pub fn build_and_load(asm_path: &str, output_path: &str) -> CPU {
    let bass_path = env::var("BASS_PATH").expect("No BASS_PATH envvar found");

    let output = Command::new(bass_path)
        .args(["-strict", asm_path, "-o", output_path])
        .output()
        .expect("Compilation failed");

    assert!(output.status.success(), "Compilation failed: {output:?}");

    CPU::load_file(output_path, None).expect(&format!("Could not load bin file at {output_path}"))
}

pub fn prep_and_load(asm_path: &str, output_path: &str, replacements: HashMap<&str, &str>) -> CPU {
    prep_test(asm_path, replacements);
    build_and_load(&tmp_path(), output_path)
}

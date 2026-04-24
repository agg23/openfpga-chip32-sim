#![allow(dead_code)]

use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

use chip32_sim::cpu::CPU;

static NEXT_TEST_FILE_ID: AtomicUsize = AtomicUsize::new(0);

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

    cpu.logs.iter().for_each(|log| println!("{log}"));

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

fn unique_temp_path(base_path: &str) -> PathBuf {
    let base_path = Path::new(base_path);
    let stem = base_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("chip32");
    let extension = base_path
        .extension()
        .and_then(|extension| extension.to_str());
    let unique_id = NEXT_TEST_FILE_ID.fetch_add(1, Ordering::Relaxed);
    let file_name = match extension {
        Some(extension) => format!(
            "chip32-{stem}-{}-{unique_id}.{extension}",
            std::process::id()
        ),
        None => format!("chip32-{stem}-{}-{unique_id}", std::process::id()),
    };

    env::temp_dir().join(file_name)
}

pub fn tmp_path() -> PathBuf {
    unique_temp_path("chip32-tmp.asm")
}

pub fn prep_test(asm_path: &str, replacements: HashMap<&str, &str>) -> PathBuf {
    let tmp_path = tmp_path();

    let mut asm = fs::read_to_string(asm_path).expect(&format!("Unable to read {asm_path}"));

    for (original, replacement) in replacements.into_iter() {
        asm = asm.replace(&format!("{{{original}}}"), replacement);
    }

    fs::write(&tmp_path, asm).expect(&format!("Unable to write to {}", tmp_path.display()));

    tmp_path
}

pub fn build_and_load(asm_path: &Path, output_path: &Path) -> CPU {
    let bass_path = env::var("BASS_PATH").unwrap_or_else(|_| {
        let local_bass = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("manifest dir should have parent")
            .join("bass-chip32")
            .join("bass");
        local_bass.to_string_lossy().into_owned()
    });

    let output = Command::new(bass_path)
        .args([
            "-strict",
            asm_path.to_str().expect("asm path should be valid utf-8"),
            "-o",
            output_path
                .to_str()
                .expect("output path should be valid utf-8"),
        ])
        .output()
        .expect("Compilation failed");

    assert!(output.status.success(), "Compilation failed: {output:?}");

    CPU::load_file(
        output_path
            .to_str()
            .expect("output path should be valid utf-8"),
        None,
        None,
    )
    .expect(&format!(
        "Could not load bin file at {}",
        output_path.display()
    ))
}

pub fn prep_and_load(asm_path: &str, output_path: &str, replacements: HashMap<&str, &str>) -> CPU {
    let asm_path = prep_test(asm_path, replacements);
    let output_path = unique_temp_path(output_path);

    build_and_load(&asm_path, &output_path)
}

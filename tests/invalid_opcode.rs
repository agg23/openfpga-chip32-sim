use std::{env, fs, path::PathBuf};

use chip32_sim::cpu::{HaltState, CPU};

fn run_until_halt(cpu: &mut CPU) {
    for _ in 0..1_000_000 {
        cpu.step();
        if !matches!(cpu.halt, HaltState::Running) {
            return;
        }
    }

    panic!("CPU did not halt");
}

#[test]
fn it_routes_unknown_opcodes_to_error_vector() {
    let path = PathBuf::from(env::temp_dir()).join(format!(
        "chip32_invalid_opcode_{}.bin",
        std::process::id()
    ));

    // 0x0000: 0x4601 (exit 1)
    // 0x0002: 0x0100 (invalid/reserved opcode)
    fs::write(&path, [0x01, 0x46, 0x00, 0x01]).expect("write temp bin");

    let mut cpu =
        CPU::load_file(path.to_str().expect("temp path utf8"), None, None).expect("load temp bin");
    run_until_halt(&mut cpu);

    assert!(matches!(cpu.halt, HaltState::Failure));
    assert!(
        cpu.logs
            .iter()
            .any(|log| log.contains("Unimplemented opcode 0x1")),
        "expected unknown opcode log, got {:?}",
        cpu.logs
    );
}

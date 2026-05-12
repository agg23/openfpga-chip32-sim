use std::{collections::HashMap, env, fs};

use util::{build_and_load, execute_until_halt, prep_and_load, prep_test};

mod util;

#[test]
fn it_preserves_registers_across_reload_slots() {
    let mut cpu = prep_and_load(
        "tests/asm/reload.asm",
        "tests/bin/reload.bin",
        HashMap::new(),
    );

    execute_until_halt(&mut cpu);
    assert!(matches!(cpu.halt, chip32_sim::cpu::HaltState::Success));
    assert_eq!(cpu.work_regs[1], 0x1234);

    cpu.restart_for_slot(2).expect("restart for slot 2");
    execute_until_halt(&mut cpu);

    assert!(matches!(cpu.halt, chip32_sim::cpu::HaltState::Success));
    assert_eq!(cpu.work_regs[0], 2);
    assert_eq!(cpu.work_regs[1], 0x1234);
    assert!(
        cpu.logs
            .iter()
            .any(|log| log.contains("Restarting CHIP32 with reload slot 0x2")),
        "expected reload log entry, got {:?}",
        cpu.logs
    );
}

#[test]
fn it_reloads_program_from_disk_for_reload_slots() {
    let asm_path = prep_test("tests/asm/reload.asm", HashMap::new());
    let output_path =
        env::temp_dir().join(format!("chip32-reload-disk-{}.bin", std::process::id()));
    let mut cpu = build_and_load(&asm_path, &output_path);

    execute_until_halt(&mut cpu);
    assert!(matches!(cpu.halt, chip32_sim::cpu::HaltState::Success));

    // 0x0000: nop
    // 0x0002: 0x4601 (exit 1)
    fs::write(&output_path, [0x00, 0x00, 0x01, 0x46]).expect("rewrite reload bin");

    cpu.restart_for_slot(2).expect("restart from rewritten bin");
    execute_until_halt(&mut cpu);

    assert!(matches!(cpu.halt, chip32_sim::cpu::HaltState::Failure));
}

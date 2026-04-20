use std::collections::HashMap;

use util::prep_and_load;

mod util;

fn run_until_halt(cpu: &mut chip32_sim::cpu::CPU) {
    for _ in 0..1_000_000 {
        cpu.step();
        if !matches!(cpu.halt, chip32_sim::cpu::HaltState::Running) {
            return;
        }
    }
    panic!("CPU did not halt");
}

#[test]
fn it_preserves_registers_across_reload_slots() {
    let mut cpu = prep_and_load(
        "tests/asm/reload.asm",
        "tests/bin/reload.bin",
        HashMap::new(),
    );

    run_until_halt(&mut cpu);
    assert!(matches!(cpu.halt, chip32_sim::cpu::HaltState::Success));
    assert_eq!(cpu.work_regs[1], 0x1234);

    cpu.restart_for_slot(2);
    run_until_halt(&mut cpu);

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

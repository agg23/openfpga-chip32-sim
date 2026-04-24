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
fn it_executes_crc16() {
    let mut cpu = prep_and_load("tests/asm/crc.asm", "tests/bin/crc.bin", HashMap::new());

    run_until_halt(&mut cpu);

    assert!(matches!(cpu.halt, chip32_sim::cpu::HaltState::Success));
    assert_eq!(cpu.work_regs[3], 0x29B1);
}

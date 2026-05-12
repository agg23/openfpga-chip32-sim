use std::collections::HashMap;

use util::{execute_until_halt, prep_and_load};

mod util;

#[test]
fn it_executes_crc16() {
    let mut cpu = prep_and_load("tests/asm/crc.asm", "tests/bin/crc.bin", HashMap::new());

    execute_until_halt(&mut cpu);

    assert!(matches!(cpu.halt, chip32_sim::cpu::HaltState::Success));
    assert_eq!(cpu.work_regs[3], 0x29B1);
}

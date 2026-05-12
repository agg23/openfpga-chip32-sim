use std::{env, fs, path::PathBuf};

use chip32_sim::cpu::{HaltState, CPU};
use util::execute_until_halt;

mod util;

#[test]
fn it_treats_4703_as_legacy_sec() {
    let path = PathBuf::from(env::temp_dir()).join("chip32_legacy_sec.bin");

    // 0x0000: nop
    // 0x0002: 0x4703 (legacy Spiritualized SEC form)
    // 0x0004: 0x4600 (exit 0)
    fs::write(&path, [0x00, 0x00, 0x03, 0x47, 0x00, 0x46]).expect("write temp bin");

    let mut cpu =
        CPU::load_file(path.to_str().expect("temp path utf8"), None, None).expect("load temp bin");
    execute_until_halt(&mut cpu);

    assert!(matches!(cpu.halt, HaltState::Success));
    assert!(cpu.carry, "legacy 0x4703 should set carry like SEC");
}

architecture chip32.vm

// Error vector
                jp error

// Start vector
start:
                bit r13,#1
                jp z,cold_boot
                cmp r0,#2
                jp z,reload_slot_2
                exit 1

cold_boot:
                or r13,#1
                ld r1,#0x1234
                exit 0

reload_slot_2:
                cmp r1,#0x1234
                jp z,reload_ok
                exit 1

reload_ok:
                exit 0

error:
                exit 1

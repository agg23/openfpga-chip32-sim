architecture chip32.vm

// Error vector
                exit 1

// Start vector
start:
                ld r1,#data
                ld r2,#9
                ld r3,#0xFFFF
                crc r1,r2,r3,#0x1021
                exit 0

data:
                db "123456789"

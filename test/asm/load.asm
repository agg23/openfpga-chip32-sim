architecture chip32.vm
output "load.bin", create

// Error vector (0x0)
nop

// Init vector (0x2)
ld.b r1,(data)

data:
db 0xDEADBEEF

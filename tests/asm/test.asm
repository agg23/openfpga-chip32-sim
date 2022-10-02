architecture chip32.vm

// Error vector (0x0)
nop

// Init vector (0x2)
ld r1,#{r1value}
ld r2,#{r2value}
{command} {targets}

HW1:
db "Hello world",0

HW2:
db "Hello world",0

HW_Short:
db "Hello wor",0

HW_Long:
db "Hello world longer",0

Random:
db "No match",0

Partial:
db "Hello no",0
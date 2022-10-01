# Simulator for Analogue's CHIP32 VM

This is a simulator for Analogue's CHIP32 virtual 32-bit CPU used as a data preprocessor for openFPGA. It allows you to step through instructions, view processor and memory state, and constantly log data (as opposed to CHIP32 only letting you print when exiting).

Some of the more obscure APF opcodes are not currently implemented, but their implementation will come soon.

![Main Screen](../assets/mainscreen.png)

## Usage

The TUI currently isn't as polished as I would like, and as such it doesn't display the valid commands. They are:

| Input          | Action                                                                                                  |
|----------------|---------------------------------------------------------------------------------------------------------|
| **s**          | Step through this instruction to the next                                                               |
| **m**          | Switch the display mode to/from memory. Arrow keys up/down will allow you to scroll memory when visible |
| **q**/**quit** | Quit the simulator                                                                                      |

## Docs

[The official Analogue CHIP32 docs can be found here](https://www.analogue.co/developer/docs/chip32-vm). Unfortunately, the opcode page hasn't been published for some reason, but hopefully it will be soon.
architecture chip32.vm
output "example_chip32.bin", create

// Example Chip32 project

// we will put data into here that we're working on.  It's the last 1K of the 8K chip32 memory
constant rambuf = 0x1b00

// json slot ID
constant datslot = 0x1
constant soundslot = 99

// core ID as listed in the core.json file
constant core_id = 0x0

// host commands
constant host_reset = 0x4000
constant host_run = 0x4001
constant host_init = 0x4002

// bridge address of our "video enable" register
constant videoenable = 0x51000000

// how many picture slots we have
constant numslots = 0x3
                
// bitfield entries
variable bit_coreloaded = 0x1

// how large the slot files are
constant slotsize = 184320
// how many bytes we'll load
constant copysize = 46080


// the error handler always starts here. (0x0000) DO NOT MOVE OR CHANGE ITS LOCATION
                jp error                    // jump to error handler

// code execution always starts here. (0x0002) DO NOT MOVE OR CHANGE ITS LOCATION
start:          
                cmp r0,#datslot             // does our slot ID match the one we're looking for?
                jp z,loaddata
                exit 0                      // other slots- do nothing and just exit
                
loaddata:       
                bit r13,#bit_coreloaded     // is the core already loaded? if so, skip loading it
                jp nz,was_loaded
                ld r0,#core_id              // load core ID 0
                core r0                     // we will set the bit_coreloaded later.  you'll see why

was_loaded:     
                ld r3,#datslot              // get data slot into R3
                ld r2,#file_err             // point to file error message preemptively
                open r3,r0                  // open file R3, length goes into R0
                jp nz,spec_err              // file could not be open for some reason
                ld r2,#size_err
                cmp r0,#8
                jp nz,spec_err              // it has to be exactly 8 bytes
                ld r0,#rambuf               // point to our RAM buffer
                ld r1,#8                    // read 8 bytes
                ld r2,#file_err
                read r0,r1                  // read in the header
                close
                jp nz,spec_err              // if there was an issue reading, show the file error
                
                ld r2,#head_err             // point to error message
                ld r1,#header               // point to header that we will test against
                ld r0,#rambuf               // point to the RAM buffer where our header we loaded lives
                test r1,r0                  // test to see if they match
                jp nz,spec_err              // jump if they don't, showing the error in R2
                
                ld r0,#host_reset           // we're reasonably sure that our files are OK, so reset if this is not the first run.
                bit r13,#bit_coreloaded     // check to see if the core's been loaded before
                jp z,firstrun
                host r0,r0                  // stop CPU if not the first time
                
                ld r0,#videoenable          // if not the first run, disable the video scanout to prevent ram clobbering during loads
                ld r9,#0
                pmpw r0,r9                  // write 0 to the display enable register (disable video scanout)
                
firstrun:       
                ld r4,#0                    // start loading slots, at ID 0
                ld r5,#rambuf+4             // start at slot 0 (next byte after the 4 byte header)
                
slotloop:       
                ld.b r0,(r5)                // get slot ID
                ld r2,#index_err            // point to the index error message
                cmp r0,#numslots            // is our slot ID larger or equal to the total number?
                jp nc,spec_err              // carry will be set if we can successfully subtract the count and not underflow
                
                asl r0,#1                   // the slot ID table is 16 bit words, so we multiply by 2
                add r0,#picslots            // add the start of the table to our index
                ld.w r3,(r0)                // get the slot ID
                
                ld r0,r4
                asl r0,#2                   // we need to multiply the slot ID by 4, so we can index our 32 bit offset table
                add r0,#offsets             // add the start of the table to our index
                ld.l r1,(r0)                // load the buffer start address to put the data we're going to write
                
                ld r2,#file_err             // point to file error message preemptively
                open r3,r0                  // open the file
                jp nz,spec_err              // if the file didn't exist or there was some other problem, error out
                ld r2,#slot_err
                cmp r0,#slotsize            // it must be the right size
                jp nz,spec_err
                
                
                ld r2,#file_err
                seek r1                     // seek to the same offset in the file
                jp nz,spec_err              // if something went wrong, error out

                ld r0,#copysize             // we will copy 1/4th of the image
                copy r1,r0                  // copy the data from the file, into the frame buffer (pointed to by R1) for length R0
                jp nz,spec_err              // if something went wrong, show an error
                close                       // close the file we copied over
                
                add r5,#1                   // increment entry in our .dat
                add r4,#1                   // increment count
                cmp r4,#4
                jp nz,slotloop              // do all slots
                
                ld r2,#sound_err            // sound playback is not implemented in the core. you may add it if you wish from the "Basic Assets" example
                ld r0,#soundslot            // for demo purposes, show loading an additional file the easier way with LOADF
                loadf r0                    // load the sound slot's data (sound is not implemented in the core itself)
                jp nz,spec_err
                
                ld r0,#videoenable
                ld r9,#0x123456FF
                pmpw r0,r9                  // write 1 to the display enable register (enable video scanout)
                
                ld r0,#host_run             // we will just run if the coreloaded bit is set
                bit r13,#bit_coreloaded     // this is why we didn't set the bit first.
                jp nz,runonly
                
                ld r0,#host_init                
                or r13,#bit_coreloaded      // NOW set the bit, so we only run host_init once.  then call host_run after that point.
runonly:        
                host r0,r0                  // run the core
                exit 0                      // done             
                
// specific errors we will display
spec_err:       
                printf r2
                exit 1

// main error handler if something REALLY BAD happened
error:
                ld r0,#err1     //  print "Error 0x12 at 0x1234"
                printf r0
                err r0,r1
                hex.b r0
                ld r0,#err2
                printf r0
                hex.w r1
                ld r0,#err3
                printf r0
                exit 1
                


// table of offsets into the frame buffer where our segments should be loaded
offsets:        
                dl 0x00000
                dl 0x0B400
                dl 0x16800
                dl 0x21C00

// table of picture slot IDs in the json file
picslots:       
                dw 0x20
                dw 0x21
                dw 0x22
                
// header for our .dat files                
header:         
                db "TEST",0x0
                
// error messages
size_err:       
                db "File Not 8 Bytes!",10,0
file_err:       
                db "Error Opening File!",10,0
head_err:       
                db "Invalid Header!",10,0
index_err:      
                db "Invalid Index!",10,0
slot_err:       
                db "Slot Size Error!",10,0
sound_err:      
                db "Sound Slot Error!",10,0

// parts for showing the error message
err1:           
                db "Error 0x",0
err2:           
                db " at 0x",0
err3:           
                db 10,0

                

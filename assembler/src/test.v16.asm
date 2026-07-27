; hello_world.v16.asm
; Pure Assembly Hello World - Builds its own font in memory!
; (No commas version)

; ---------------------------------------------------------
; 1. CLEAR TILE MEMORY
; Tilesheet starts at $1000. We will define 8 custom tiles
; starting at ID 1 (Address 1016). 8 tiles * 16 words = 128 words.
; ---------------------------------------------------------
mov *A #1016     ; Start address for Tile 1
mov *B #128      ; Number of 16-bit words to clear

clear_loop:
mov $*A #0       ; Write 0 to the address pointed to by Register A
add *A #1        ; Move to the next memory address
sub *B #1        ; Decrement our counter
jne clear_loop *B ; Loop until Register B hits 0

; ---------------------------------------------------------
; 2. BUILD FONT DATA
; Writing the non-zero pixel data (using color index 2: Cloud White).
; Each pixel is a 4-bit nibble. 
; ---------------------------------------------------------

; --- Tile 1: 'H' ---
mov $#1016 #0x0200
mov $#1017 #0x0020
mov $#1018 #0x0200
mov $#1019 #0x0020
mov $#1020 #0x0200
mov $#1021 #0x0020
mov $#1022 #0x0222
mov $#1023 #0x2220
mov $#1024 #0x0200
mov $#1025 #0x0020
mov $#1026 #0x0200
mov $#1027 #0x0020
mov $#1028 #0x0200
mov $#1029 #0x0020

; --- Tile 2: 'e' ---
mov $#1036 #0x0022
mov $#1037 #0x2200
mov $#1038 #0x0200
mov $#1039 #0x0020
mov $#1040 #0x0222
mov $#1041 #0x2220
mov $#1042 #0x0200
mov $#1044 #0x0022
mov $#1045 #0x2200

; --- Tile 3: 'l' ---
mov $#1048 #0x0220
mov $#1050 #0x0020
mov $#1052 #0x0020
mov $#1054 #0x0020
mov $#1056 #0x0020
mov $#1058 #0x0020
mov $#1060 #0x0222

; --- Tile 4: 'o' ---
mov $#1068 #0x0022
mov $#1069 #0x2200
mov $#1070 #0x0200
mov $#1071 #0x0020
mov $#1072 #0x0200
mov $#1073 #0x0020
mov $#1074 #0x0200
mov $#1075 #0x0020
mov $#1076 #0x0022
mov $#1077 #0x2200

; --- Tile 5: ' ' (Space) ---
; Intentionally skipped - left as all 0s from the clear loop

; --- Tile 6: 'W' ---
mov $#1096 #0x0200
mov $#1097 #0x0002
mov $#1098 #0x0200
mov $#1099 #0x0002
mov $#1100 #0x0200
mov $#1101 #0x0002
mov $#1102 #0x0200
mov $#1103 #0x2002
mov $#1104 #0x0202
mov $#1105 #0x0202
mov $#1106 #0x0202
mov $#1107 #0x0202
mov $#1108 #0x0020
mov $#1109 #0x0020

; --- Tile 7: 'r' ---
mov $#1116 #0x0020
mov $#1117 #0x2220
mov $#1118 #0x0022
mov $#1120 #0x0020
mov $#1122 #0x0020
mov $#1124 #0x0020

; --- Tile 8: 'd' ---
mov $#1129 #0x0020
mov $#1131 #0x0020
mov $#1132 #0x0022
mov $#1133 #0x2220
mov $#1134 #0x0200
mov $#1135 #0x0020
mov $#1136 #0x0200
mov $#1137 #0x0020
mov $#1138 #0x0200
mov $#1139 #0x0020
mov $#1140 #0x0022
mov $#1141 #0x2220

; ---------------------------------------------------------
; 3. DRAW TO SCREEN
; Map our 8 new Tile IDs into the video memory offset ($3560)
; ---------------------------------------------------------
mov $#3560 #1 ; 'H' mapped to x=0
mov $#3561 #2 ; 'e' mapped to x=1
mov $#3562 #3 ; 'l' mapped to x=2
mov $#3563 #3 ; 'l' mapped to x=3
mov $#3564 #4 ; 'o' mapped to x=4
mov $#3565 #5 ; ' ' mapped to x=5
mov $#3566 #6 ; 'W' mapped to x=6
mov $#3567 #4 ; 'o' mapped to x=7
mov $#3568 #7 ; 'r' mapped to x=8
mov $#3569 #3 ; 'l' mapped to x=9
mov $#3570 #8 ; 'd' mapped to x=10

; ---------------------------------------------------------
; 4. HALT LOOP
; Keep the screen drawn forever
; ---------------------------------------------------------
halt_loop:
stall #10000A
jmp halt_loop
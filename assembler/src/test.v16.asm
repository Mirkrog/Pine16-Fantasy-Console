; ==========================================================
; 1. SYSTEM INIT: Load Palette Registers ($#301 - $#315)
; ==========================================================
init_palette:
    mov $#301 #6373     ; Index 1:  Deep Night
    mov $#302 #63423    ; Index 2:  Cloud White
    mov $#303 #16970    ; Index 3:  Charcoal
    mov $#304 #42293    ; Index 4:  Silver
    mov $#305 #64080    ; Index 5:  Cyber Pink  (#64080)
    mov $#306 #13439    ; Index 6:  Ocean Blue
    mov $#307 #16062    ; Index 7:  Sky Cyan
    mov $#308 #34695    ; Index 8:  Slime Green
    mov $#309 #47869    ; Index 9:  Magic Violet
    mov $#310 #27358    ; Index 10: Electric Indigo
    mov $#311 #60591    ; Index 11: Toasted Peach
    mov $#312 #64331    ; Index 12: Neon Coral  (#64331)
    mov $#313 #64966    ; Index 13: Sunny Amber
    mov $#314 #63303    ; Index 14: Electric Lemon
    mov $#315 #12017    ; Index 15: Minty Green

; ==========================================================
; 2. TILE DEFINITIONS (Tilesheet @ Address 1000)
; ==========================================================
init_tile_1:
    mov $#1016 #21845
    mov $#1017 #52428
    mov $#1018 #21845
    mov $#1019 #52428
    mov $#1020 #21845
    mov $#1021 #52428
    mov $#1022 #21845
    mov $#1023 #52428
    mov $#1024 #52428
    mov $#1025 #21845
    mov $#1026 #52428
    mov $#1027 #21845
    mov $#1028 #52428
    mov $#1029 #21845
    mov $#1030 #52428
    mov $#1031 #21845

init_tile_2:
    mov $#1032 #52428
    mov $#1033 #21845
    mov $#1034 #52428
    mov $#1035 #21845
    mov $#1036 #52428
    mov $#1037 #21845
    mov $#1038 #52428
    mov $#1039 #21845
    mov $#1040 #21845
    mov $#1041 #52428
    mov $#1042 #21845
    mov $#1043 #52428
    mov $#1044 #21845
    mov $#1045 #52428
    mov $#1046 #21845
    mov $#1047 #52428

; ==========================================================
; 3. POPULATE FULL SCREEN TILEMAP (#3560 - #4559)
; ==========================================================
init_tilemap:
    mov *A #3560

start_even_row:
    mov *B #20
even_row_loop:
    mov $*A #1
    add *A #1
    mov $*A #2
    add *A #1
    sub *B #1
    jne even_row_loop *B

    mov *C *A
    sub *C #4560
    jeq main_loop *C

start_odd_row:
    mov *B #20
odd_row_loop:
    mov $*A #2
    add *A #1
    mov $*A #1
    add *A #1
    sub *B #1
    jne odd_row_loop *B

    mov *C *A
    sub *C #4560
    jeq main_loop *C
    jmp start_even_row

; ==========================================================
; 4. ANIMATED MAIN LOOP (Hardware Color Cycling)
; Shifting 4px right/down by toggling palette registers!
; ==========================================================
main_loop:
    stall #0b00000000_00000001

    jmp main_loop       ; Loop forever!
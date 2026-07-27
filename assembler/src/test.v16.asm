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
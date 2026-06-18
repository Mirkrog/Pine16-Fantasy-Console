; ===================================================
; Test Suite
; Goal: Verify every single opcode and addressing mode.
; Returns: Exit code 0 on absolute success.
; ===================================================

test_mov:
    ; Test 1: Immediate to Register, Register to Address, Immediate to Address
    Mov *A #10          ; *A = 10
    Mov $#100 *A        ; Memory[100] = 10
    Mov $#101 #2        ; Memory[101] = 2
    Mov $#101

test_add:
    ; Test 2: Add memory locations together
    Add $#100 $#101     ; Memory[100] = 10 + 2 = 12
    Mov *B $#100        ; *B = 12
    Sub *B #12          ; *B = 12 - 12 = 0
    Jne fail_add *B     ; If result is not 0, Add failed!

test_sub:
    ; Test 3: Standard subtraction
    Mov *B #50          ; *B = 50
    Sub *B #20          ; *B = 50 - 20 = 30
    Sub *B #30          ; *B = 30 - 30 = 0
    Jne fail_sub *B     ; If result is not 0, Sub failed!

test_mul:
    ; Test 4: Multiplication and Register Indirect addressing ($*)
    Mov *C #200         ; Point *C to Memory address 200
    Mov $*C #5          ; Memory[200] = 5
    Mul $*C #4          ; Memory[200] = 5 * 4 = 20
    Mov *B $*C          ; *B = 20
    Sub *B #20          ; *B = 20 - 20 = 0
    Jne fail_mul *B     ; If result is not 0, Mul failed!

test_div:
    ; Test 5: Standard division
    Mov *B #100         ; *B = 100
    Div *B #4          ; *B = 100 / 4 = 25
    Sub *B #25          ; *B = 25 - 25 = 0
    Jne fail_div *B     ; If result is not 0, Div failed!

test_stall:
    ; Test 6: Ensure Stall parses correctly (interpreter halts execution loops)
    Stall #5            ; Sleep for 5 cycles

test_conditionals:
    ; Test 7: Confirm Jeq and Jne logic states
    Mov *B #0           ; *B = 0
    Jeq next_check *B   ; Should jump because *B == 0
    Jmp fail_jeq        ; Fallback if Jeq failed

next_check:
    Mov *B #99          ; *B = 99
    Jne global_success *B ; Should jump because *B != 0
    Jmp fail_jne        ; Fallback if Jne failed


; ===================================================
; EXIT ROUTINES & FAIL SLOTS
; ===================================================

global_success:
    Exit #0             ; ALL TESTS PASSED! Returns 0.

fail_add:
    Exit #1             ; Add opcode configuration broken.

fail_sub:
    Exit #2             ; Sub opcode configuration broken.

fail_mul:
    Exit #3             ; Mul or Register Indirect ($*) broken.

fail_div:
    Exit #4             ; Div opcode configuration broken.

fail_jeq:
    Exit #5             ; Jeq conditional pipeline broken.

fail_jne:
    Exit #6             ; Jne conditional pipeline broken.
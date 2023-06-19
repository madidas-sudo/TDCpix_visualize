# Generate a single chunk of data words and one frame word
# Output to chunky_chunk.txt

# Dataword definition:
# 47    : data selector
# 46..40: address
# 39..35: address_arbiter
# 34..30: address_pileup
# 29    : leading_coarse_time_selector
# 28..17: leading_coarse_time
# 16..12: leading_fine_time
# 11    : trailing_coarse_time_selector
# 10..5 : trailing_coarse_time
# 4..0  : trailing_fine_time

# Frameword definition allways has bit 47:45 = "111"
# 42..37: qchip_collision_count
# 36..28: hit_counter
# 27..0 : frame_counter

# Example of chunk with 5 data words and one frameword
# 29081a3c48ae 17081a3c20cd 5083a3ca0d7 4d081a3cd0cf 3b081a3c40c0 e8005000eb00

from random import randint

def g_dataword() -> str:
    # Addres is in range 0 89
    addr = randint(0, 89)
    arb = 1<<randint(0, 4)
    pileup = (1 if randint(0, 332) == 0 else 0) << randint(0, 4)
    lcts = 0
    lct = randint(0, 4095)
    lft = randint(0, 31)
    tcts = 0
    tct = randint(0, 63)
    tft = randint(0, 31)
    # remember that the word is 48 bits long
    # So the length of the string is 12 characters
    num = 1<<47 | addr<<40 | arb<<35 | pileup<<30 | lcts<<29 | lct<<17 | lft<<12 | tcts<<11 | tct<<5 | tft
    return "{:012x}".format(num)

def g_frameword(num_dw) -> str:
    return "e8005000eb00"
    

def main():
    NUM_CHUNKS = 10

    NUM_DW = 254

    with open("chunky_chunk.txt", "w") as f:
        for i in range(NUM_CHUNKS):
            for j in range(NUM_DW):
                f.write(g_dataword() + " ")
            f.write(g_frameword(NUM_DW) + "\n")

if __name__ == "__main__":
    main()
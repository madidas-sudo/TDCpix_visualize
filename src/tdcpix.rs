// 42..37: qchip_collision_count
// 36..28: hit_counter
// 27..0 : frame_counter
#[derive(Clone, Copy, Debug)]
pub struct FrameWord {
    raw: u64,
    qchip_collision_count: u8,
    hit_counter: u16,
    frame_counter: u32,
}

impl From<&str> for FrameWord {
    fn from(value: &str) -> Self {
        let raw = u64::from_str_radix(value, 16).unwrap();
        let qchip_collision_count = ((raw >> 37) & 0x3F) as u8;
        let hit_counter = ((raw >> 28) & 0xFF) as u16;
        let frame_counter = (raw & 0x7FFFFFF) as u32;
        FrameWord {
            raw,
            qchip_collision_count,
            hit_counter,
            frame_counter,
        }
    }
}

// 47    : data selector
// 46..40: address
// 39..35: address_arbiter
// 34..30: address_pileup
// 29    : leading_coarse_time_selector
// 28..17: leading_coarse_time
// 16..12: leading_fine_time
// 11    : trailing_coarse_time_selector
// 10..5 : trailing_coarse_time
// 4..0  : trailing_fine_time
#[derive(Clone, Copy, Debug)]
pub struct DataWord {
    pub raw: u64,
    pub data_selector: u8,
    pub address: u8,
    pub address_arbiter: u8,
    pub address_pileup: u8,
    pub leading_coarse_time_selector: u8,
    pub leading_coarse_time: u16,
    pub leading_fine_time: u8,
    pub trailing_coarse_time_selector: u8,
    pub trailing_coarse_time: u8,
    pub trailing_fine_time: u8,
}

impl DataWord {
    pub fn get_time(&self) -> u64 {
        // leading coarse time = 1 bit rollover indicator + 2048(11bit)*3.125 ns =6.4us
        // leading fine time = 98ps -> 3.125ns
        // trailing coarse time selector
        // trailing coarse time = 64*3.125ns = 200ns
        // trailing fine time = 98ps -> 3.125ns
        let leading_coarse_time = self.leading_coarse_time as u64 * 3_125;
        let leading_fine_time = self.leading_fine_time as u64 * 98;
        let trailing_coarse_time = self.trailing_coarse_time as u64 * 3_125;
        let trailing_fine_time = self.trailing_fine_time as u64 * 98;
        leading_coarse_time + leading_fine_time + trailing_coarse_time + trailing_fine_time
        // This returns the time in ps
    }

    pub fn get_duration(&self) -> u64 {
        // leading coarse time = 1 bit rollover indicator + 2048(11bit)*3.125 ns =6.4us
        // leading fine time = 98ps -> 3.125ns
        // trailing coarse time selector
        // trailing coarse time = 64*3.125ns = 200ns
        // trailing fine time = 98ps -> 3.125ns
        let trailing_coarse_time = self.trailing_coarse_time as u64 * 3_125;
        let trailing_fine_time = self.trailing_fine_time as u64 * 98;
        trailing_coarse_time + trailing_fine_time
        // This returns the time in ps
    }

    pub fn get_start_time(&self) -> u64 {
        // leading coarse time = 1 bit rollover indicator + 2048(11bit)*3.125 ns =6.4us
        // leading fine time = 98ps -> 3.125ns
        // trailing coarse time selector
        // trailing coarse time = 64*3.125ns = 200ns
        // trailing fine time = 98ps -> 3.125ns
        let leading_coarse_time = self.leading_coarse_time as u64 * 3_125;
        let leading_fine_time = self.leading_fine_time as u64 * 98;
        leading_coarse_time + leading_fine_time
        // This returns the time in ps
    }
}

impl From<&str> for DataWord {
    fn from(value: &str) -> Self {
        let raw = u64::from_str_radix(value, 16).unwrap();
        let data_selector = ((raw >> 47) & 0x1) as u8;
        let address = ((raw >> 40) & 0x7F) as u8;
        let address_arbiter = ((raw >> 35) & 0x1F) as u8;
        let address_pileup = ((raw >> 30) & 0x1F) as u8;
        let leading_coarse_time_selector = ((raw >> 29) & 0x1) as u8;
        let leading_coarse_time = ((raw >> 17) & 0xFFF) as u16;
        let leading_fine_time = ((raw >> 12) & 0x1F) as u8;
        let trailing_coarse_time_selector = ((raw >> 11) & 0x1) as u8;
        let trailing_coarse_time = ((raw >> 5) & 0x3F) as u8;
        let trailing_fine_time = (raw & 0x1F) as u8;
        DataWord {
            raw,
            data_selector,
            address,
            address_arbiter,
            address_pileup,
            leading_coarse_time_selector,
            leading_coarse_time,
            leading_fine_time,
            trailing_coarse_time_selector,
            trailing_coarse_time,
            trailing_fine_time,
        }
    }
}

pub enum TDCpixWord {
    FrameWord(FrameWord),
    DataWord(DataWord),
}

#[derive(Clone)]
pub struct Chunk {
    pub data_words: Vec<DataWord>,
    pub frame_word: FrameWord,
}

pub fn parse_tdcpix_txt(file: &str, chunks: &mut Vec<Chunk>) -> () {
    for line in std::fs::read_to_string(file).unwrap().lines() {
        let mut words: Vec<&str> = line.split_whitespace().collect();

        let frame_word = FrameWord::from(words.pop().unwrap());

        let mut data_words: Vec<DataWord> = Vec::new();
        for word in words {
            data_words.push(DataWord::from(word));
        }

        chunks.push(Chunk {
            data_words,
            frame_word,
        });
    }
}
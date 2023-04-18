pub struct Header {
    version: i32,
    previous_block_header_hash: String,
    merkle_root_hash: String,
    time: u32,
    n_bits: u32,
    none: u32
}

fn is_valid_version(version: i32) -> bool {
    true
}

fn is_existin_header(previous_block_header_hash: &str) -> bool {
    true
}

fn is_valid_hash(merkle_root_hash: &str) -> bool {
    true
}

fn has_correct_threshlod_format(n_bits: u32) -> bool {
    true
}

impl Header {
    fn new(version: i32,
           previous_block_header_hash: String,
           merkle_root_hash: String,
           time: u32,
           n_bits: u32,
           none: u32) -> Result<Header, &'static str>{
            if(is_valid_version(version) && is_existin_header(&previous_block_header_hash) &&
               is_valid_hash(&merkle_root_hash) && has_correct_threshlod_format(n_bits)) {
                return Ok(Header {version, previous_block_header_hash, merkle_root_hash, time, n_bits, none});
               }
            Err("Los parámetros introducidos no son válidos")
           }
}
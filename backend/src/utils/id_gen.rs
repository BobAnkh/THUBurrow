use idgenerator::{IdGeneratorOptions, IdHelper};

pub fn init(worker_id: u32) {
    IdHelper::init();
    let mut options: IdGeneratorOptions = IdGeneratorOptions::new(worker_id);
    options.worker_id_bit_len = 6;
    options.seq_bit_len = 16;
    IdHelper::set_id_generator(options);
}

use idgenerator::{IdGeneratorOptions, IdHelper};

pub fn init(worker_id: u32) {
    IdHelper::init();
    let mut options: IdGeneratorOptions = IdGeneratorOptions::new(worker_id);
    options.worker_id_bit_len = 8;
    IdHelper::set_id_generator(options);
}

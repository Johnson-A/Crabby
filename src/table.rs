use rand::{Rng, ThreadRng, thread_rng};

pub struct Entry {
	hash: u64,
	data: u64
}

//pub static mut table:
static mut piece_keys: [u64; 64*6*2] = [0; 64*6*2];
static mut castle_keys: [u64; 16] = [0; 16];
static mut ep_keys: [u64; 8] = [0; 8];
static mut color_key: u64 = 0;

fn set_random(arr: &mut [u64], rng: &mut ThreadRng) {
	for elem in arr.iter_mut() {
		*elem = rng.gen();
	}
}

pub unsafe fn init() {
	let mut rng = thread_rng();
	set_random(&mut piece_keys,  &mut rng);
	set_random(&mut castle_keys, &mut rng);
	set_random(&mut ep_keys,     &mut rng);
	color_key = rng.gen();
}
use std::thread::sleep;

pub struct OrderPoller {

}
impl OrderPoller {
	pub fn new() -> Self {
		Self {

		}
	}
	pub fn run(&mut self) {
		loop {
			println!("Polling for orders...");
			sleep(std::time::Duration::from_millis(1000));
		}
	}
}
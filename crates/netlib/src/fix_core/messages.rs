use crate::fix_core::types::NewOrder;
use chrono::Local;

pub fn write_header(
    write_buf: &mut Vec<u8>,
    msg_type: &[u8],
    outbound_sequence_number: &u32,
    sender_comp_id: &String,
    target_comp_id: &String,
) {
    let mut itoa_buf = itoa::Buffer::new();

    // Message Type
    write_buf.extend_from_slice(b"35=");
    write_buf.extend_from_slice(msg_type);
    write_buf.push(0x01);

    // Message Sequence Number
    write_buf.extend_from_slice(b"34=");
    write_buf.extend_from_slice(itoa_buf.format(*outbound_sequence_number).as_bytes());
    write_buf.push(0x01);

    // Sender Comp ID
    write_buf.extend_from_slice(b"49=");
    write_buf.extend_from_slice(sender_comp_id.as_bytes());
    write_buf.push(0x01);

    // Sending Time
    write_buf.extend_from_slice(b"52=");
    write_buf.extend_from_slice(get_timestamp().as_bytes());
    write_buf.push(0x01);

    // Target Comp ID
    write_buf.extend_from_slice(b"56=");
    write_buf.extend_from_slice(target_comp_id.as_bytes());
    write_buf.push(0x01);
}

pub fn calculate_checksum(message: &[u8]) -> u32 {
    let mut sum: u32 = 0;

    for &byte in message.iter() {
        sum += byte as u32;
    }

    sum % 256
}

pub fn write_trailer(write_buf: &mut Vec<u8>) {
    // Checksum
    let checksum: u32 = calculate_checksum(write_buf.as_slice());
    write_buf.extend_from_slice(b"10=");
    write_buf.push(b'0' + (checksum / 100) as u8);
    write_buf.push(b'0' + ((checksum / 10) % 10) as u8);
    write_buf.push(b'0' + (checksum % 10) as u8);
    write_buf.push(0x01);
}

pub fn print_message(message: &Vec<u8>) {
    let mut output = Vec::with_capacity(message.len());

    for &byte in message.iter() {
        let c = if byte == 0x01 { b'|' } else { byte };
        output.push(c);
    }

    println!("{}", String::from_utf8_lossy(&output));
}

pub fn get_timestamp() -> String {
    let now = Local::now();
    now.format("%Y%m%d-%H:%M:%S.%3f").to_string()
}

pub fn get_maturity_month_year() -> String {
    let now = Local::now();
    now.format("%Y%m").to_string()
}

pub fn get_maturity_month_year_day() -> String {
    let now = Local::now();
    now.format("%Y%m%d").to_string()
}

pub trait IntoBytes {
    fn as_bytes(&self) -> Vec<u8>;
}

impl NewOrder {
    pub fn new(cl_ord_id: u64, qty: u32, price: u32, side: u8, symbol: String) -> Self {
        Self {
            cl_ord_id,
            qty,
            price,
            side,
            symbol,
        }
    }
}
impl IntoBytes for NewOrder {
    fn as_bytes(&self) -> Vec<u8> {
        let mut itoa_buf = itoa::Buffer::new();
        let mut buf = Vec::new();

        buf.extend_from_slice(b"11=");
        buf.extend_from_slice(itoa_buf.format(self.cl_ord_id).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(b"21=1\x01");

        buf.extend_from_slice(b"38=");
        buf.extend_from_slice(itoa_buf.format(self.qty).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(b"40=2\x01");

        buf.extend_from_slice(b"44=");
        buf.extend_from_slice(itoa_buf.format(self.price).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(b"54=");
        buf.extend_from_slice(itoa_buf.format(self.side).as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(b"55=");
        buf.extend_from_slice(&self.symbol.as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(b"60=");
        buf.extend_from_slice(get_timestamp().as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(b"77=O\x01");
        buf.extend_from_slice(b"167=OPT\x01");

        buf.extend_from_slice(b"200=");
        buf.extend_from_slice(get_maturity_month_year().as_bytes());
        buf.push(0x01);

        buf.extend_from_slice(b"201=1\x01");
        buf.extend_from_slice(b"202=10\x01");
        buf.extend_from_slice(b"204=0\x01");
        buf.extend_from_slice(b"205=10\x01");

        buf
    }
}

pub enum FIXCommand {
    NewOrder(NewOrder),
}

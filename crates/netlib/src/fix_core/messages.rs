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

pub fn calculate_checksum(write_buf: &Vec<u8>) -> u32 {
    let mut sum: u32 = 0;

    for &byte in write_buf.iter() {
        sum += byte as u32;
    }

    sum % 256
}

pub fn write_trailer(write_buf: &mut Vec<u8>) {
    // Checksum
    let checksum: u32 = calculate_checksum(write_buf);
    write_buf.extend_from_slice(b"10=");
    write_buf.push(b'0' + (checksum / 100) as u8);
    write_buf.push(b'0' + ((checksum / 10) % 10) as u8);
    write_buf.push(b'0' + (checksum % 10) as u8);
    write_buf.push(0x01);
}

pub fn print_message(write_buf: &Vec<u8>) {
    let mut output = Vec::with_capacity(write_buf.len());

    for &byte in write_buf.iter() {
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

use chrono::{Local, NaiveDateTime, TimeZone, Utc};
use std::str::from_utf8;

pub fn write_fix_message(
    write_buf: &mut Vec<u8>,
    msg_type: &'static [u8],
    outbound_sequence_number: &u32,
    sender_comp_id: &String,
    target_comp_id: &String,
    body: &Vec<u8>,
) {
    write_buf.clear();

    write_header(
        write_buf,
        msg_type,
        outbound_sequence_number,
        sender_comp_id,
        target_comp_id,
    );

    write_buf.extend_from_slice(&body);

    write_wrapper(write_buf);
}

pub fn write_header(
    write_buf: &mut Vec<u8>,
    msg_type: &'static [u8],
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

pub fn write_wrapper(write_buf: &mut Vec<u8>) {
    let body_length = write_buf.len();
    let mut itoa_buf = itoa::Buffer::new();
    let mut final_buf: Vec<u8> = Vec::with_capacity(body_length + 64);

    final_buf.extend_from_slice(b"8=FIX.4.2\x01");

    final_buf.extend_from_slice(b"9=");
    final_buf.extend_from_slice(itoa_buf.format(body_length).as_bytes());
    final_buf.push(0x01);

    final_buf.extend_from_slice(write_buf);
    *write_buf = final_buf;

    write_trailer(write_buf);
    print_message(write_buf);
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

pub fn convert_timestamp(value: &[u8]) -> Option<u64> {
    let timestamp = from_utf8(value).ok()?;
    let ndt = NaiveDateTime::parse_from_str(timestamp, "%Y%m%d-%H:%M:%S%.3f").ok()?;
    Some(Utc.from_utc_datetime(&ndt).timestamp_millis() as u64)
}

pub fn get_maturity_month_year() -> String {
    let now = Local::now();
    now.format("%Y%m").to_string()
}

pub fn get_maturity_month_year_day() -> String {
    let now = Local::now();
    now.format("%Y%m%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fix_core::messages::FIX_MESSAGE_TYPE_NEW_ORDER;

    #[test]
    fn test_write_header_field_values() {
        let mut buf = Vec::new();

        write_header(
            &mut buf,
            &FIX_MESSAGE_TYPE_NEW_ORDER,
            &1,
            &"str1".to_string(),
            &"str2".to_string(),
        );

        let s = String::from_utf8_lossy(&buf);

        assert!(s.contains("35=D"));
        assert!(s.contains("34=1"));
        assert!(s.contains("49=str1"));
        assert!(s.contains("52="));
        assert!(s.contains("56=str2"));
        assert!(buf.contains(&0x01));
    }

    #[test]
    fn test_calculate_checksum() {
        let b = b"ABC";
        let c: u32 = (b'A' as u32 + b'B' as u32 + b'C' as u32) % 256;

        assert_eq!(calculate_checksum(b), c);
    }

    #[test]
    fn test_write_trailer_appends_checksum() {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"35=D\x01");

        write_trailer(&mut buf);

        let s = String::from_utf8_lossy(&buf);

        assert!(s.contains("10="));
        assert!(buf.ends_with(&[0x01]));
    }
}

use std::str::from_utf8;
use time::{OffsetDateTime, PrimitiveDateTime, macros::format_description};

/// Constructs a complete FIX message byte vector by prepending the standard header
/// (BeginString, BodyLength, MsgType, SeqNum, Sender/Target IDs, SendingTime)
/// to the provided body and appending the calculated checksum trailer.
pub fn write_fix_message(
    msg_type: u8,
    outbound_sequence_number: &u32,
    sender_comp_id: &str,
    target_comp_id: &str,
    body: &[u8],
) -> Vec<u8> {
    let mut itoa_buf = itoa::Buffer::new();

    let seq_str = itoa_buf.format(*outbound_sequence_number);
    let timestamp = get_timestamp();

    let body_length = 21
        + seq_str.len()
        + sender_comp_id.len()
        + timestamp.len()
        + target_comp_id.len()
        + body.len();

    let mut buf = Vec::with_capacity(body_length + 30);

    buf.extend_from_slice(b"8=FIX.4.2\x01");

    // BodyLength
    buf.extend_from_slice(b"9=");
    buf.extend_from_slice(itoa_buf.format(body_length).as_bytes());
    buf.push(0x01);

    // Message Type
    buf.extend_from_slice(b"35=");
    buf.push(msg_type);
    buf.push(0x01);

    // Message Sequence Number
    buf.extend_from_slice(b"34=");
    buf.extend_from_slice(itoa_buf.format(*outbound_sequence_number).as_bytes());
    buf.push(0x01);

    // Sender Comp ID
    buf.extend_from_slice(b"49=");
    buf.extend_from_slice(sender_comp_id.as_bytes());
    buf.push(0x01);

    // Sending Time
    buf.extend_from_slice(b"52=");
    buf.extend_from_slice(timestamp.as_bytes());
    buf.push(0x01);

    // Target Comp ID
    buf.extend_from_slice(b"56=");
    buf.extend_from_slice(target_comp_id.as_bytes());
    buf.push(0x01);

    // Body
    buf.extend_from_slice(body);

    // Trailer
    let checksum: u32 = calculate_checksum(buf.as_slice());
    buf.extend_from_slice(b"10=");
    buf.push(b'0' + (checksum / 100) as u8);
    buf.push(b'0' + ((checksum / 10) % 10) as u8);
    buf.push(b'0' + (checksum % 10) as u8);
    buf.push(0x01);

    buf
}

/// Computes the standard FIX checksum by summing the byte values of the message
/// up to the checksum field, modulo 256.
pub fn calculate_checksum(message: &[u8]) -> u32 {
    let mut sum: u32 = 0;

    for &byte in message.iter() {
        sum += byte as u32;
    }

    sum % 256
}

/// Prints a raw FIX message byte slice to standard output, substituting the
/// SOH delimiter (`\x01`) with a pipe (`|`) for readability.
pub fn print_message(message: &[u8]) {
    let mut output = Vec::with_capacity(message.len());

    for &byte in message.iter() {
        let c = if byte == 0x01 { b'|' } else { byte };
        output.push(c);
    }

    println!("{}", String::from_utf8_lossy(&output));
}

/// Helper function to drop the first byte of a malformed buffer sequence,
/// advancing the parser state to look for the next valid message.
fn invalidate_message(read_buffer: &mut Vec<u8>) -> Option<Vec<u8>> {
    read_buffer.drain(0..1);
    None
}

/// Scans the read buffer for a complete FIX message. If a message is found with a valid
/// `BeginString`, `BodyLength`, and `Checksum`, it is extracted from the buffer and returned.
/// Malformed bytes are discarded.
pub fn extract_message(read_buffer: &mut Vec<u8>) -> Option<Vec<u8>> {
    if !read_buffer.starts_with(b"8=FIX") {
        if let Some(position) = read_buffer.windows(5).position(|f| f == b"8=FIX") {
            read_buffer.drain(0..position);
        } else {
            read_buffer.clear();
        }
        return None;
    }

    let first_delimiter = read_buffer.iter().position(|&b| b == 0x01)?;
    if !read_buffer[first_delimiter + 1..].starts_with(b"9=") {
        return invalidate_message(read_buffer);
    }
    let body_len_start = first_delimiter + 1;
    let body_len_end = read_buffer[body_len_start..]
        .iter()
        .position(|&b| b == 0x01)?
        + body_len_start;

    let body_len: usize = match from_utf8(&read_buffer[body_len_start + 2..body_len_end])
        .ok()
        .and_then(|s| s.parse().ok())
    {
        Some(n) => n,
        None => return invalidate_message(read_buffer),
    };

    if body_len == 0 {
        return invalidate_message(read_buffer);
    }

    let body_start = body_len_end + 1;
    let body_end = body_start + body_len;

    let recv_checksum_start = body_end + 3;
    let recv_checksum_end = body_end + 6;
    let total_len = body_end + 7;

    if read_buffer.len() < total_len {
        return None;
    }

    if !read_buffer[body_end..].starts_with(b"10=") {
        return invalidate_message(read_buffer);
    }

    let recv_checksum: u32 = match from_utf8(&read_buffer[recv_checksum_start..recv_checksum_end])
        .ok()
        .and_then(|s| s.parse().ok())
    {
        Some(n) => n,
        None => return invalidate_message(read_buffer),
    };

    let checksum = calculate_checksum(&read_buffer[..body_end]);

    if recv_checksum != checksum {
        return invalidate_message(read_buffer);
    }

    Some(read_buffer.drain(0..total_len).collect())
}

/// Generates a current timestamp string formatted for standard FIX headers (YYYYMMDD-HH:MM:SS.mmm).
pub fn get_timestamp() -> String {
    let now = OffsetDateTime::now_utc();
    let format =
        format_description!("[year][month][day]-[hour]:[minute]:[second].[subsecond digits:3]");
    now.format(&format).unwrap_or_default()
}

/// Converts a millisecond UNIX timestamp into a standard FIX timestamp string.
pub fn to_timestamp(timestamp: u64) -> String {
    let now =
        time::OffsetDateTime::from_unix_timestamp_nanos((timestamp as i128) * 1_000_000).unwrap();
    let format =
        format_description!("[year][month][day]-[hour]:[minute]:[second].[subsecond digits:3]");
    now.format(&format).unwrap_or_default()
}

/// Parses a standard FIX timestamp string back into a millisecond UNIX timestamp.
pub fn convert_timestamp(timestamp_str: String) -> Option<u64> {
    let format =
        format_description!("[year][month][day]-[hour]:[minute]:[second].[subsecond digits:3]");

    let parsed = PrimitiveDateTime::parse(&timestamp_str, &format).ok()?;
    let offset_dt = parsed.assume_utc();

    Some((offset_dt.unix_timestamp_nanos() / 1_000_000) as u64)
}

/// Returns the current year and month formatted as YYYYMM.
pub fn get_maturity_month_year() -> String {
    let now = OffsetDateTime::now_utc();
    let format = format_description!("[year][month]");
    now.format(&format).unwrap_or_default()
}

/// Returns the current date formatted as YYYYMMDD.
pub fn get_maturity_month_year_day() -> String {
    let now = OffsetDateTime::now_utc();
    let format = format_description!("[year][month][day]");
    now.format(&format).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_checksum() {
        let b = b"ABC";
        let c: u32 = (b'A' as u32 + b'B' as u32 + b'C' as u32) % 256;

        assert_eq!(calculate_checksum(b), c);
    }
}

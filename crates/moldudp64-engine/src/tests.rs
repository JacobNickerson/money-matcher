#[cfg(test)]
use crate::engine::*;
use rand::Rng;
use rand::distributions::Alphanumeric;
use rand::rngs::OsRng;

#[tokio::test]
async fn test_packet_engine() {
    let mut mold: MOLDPRODUCER = MOLDPRODUCER::new().await;
    let n = 100;
    let mut test_messages: Vec<String> = Vec::with_capacity(n);

    for _i in 1..=n {
        let message: String = OsRng
            .sample_iter(&Alphanumeric)
            .take(128)
            .map(char::from)
            .collect();
        test_messages.push(message.clone());
        mold.enqueue_message(message.as_bytes().to_vec());
    }

    let packet = mold.flush();

    let header = packet.header;
    let message_blocks = packet.message_blocks;
    println!(
        "Header Session ID: {:?} {:?}",
        std::str::from_utf8(&header.session_id).unwrap(),
        header.session_id
    );
    println!(
        "Header Sequence Number: {:?} {:?}",
        u64::from_be_bytes(header.sequence_number) as usize,
        header.sequence_number
    );
    let message_count = u16::from_be_bytes(header.message_count) as usize;
    println!(
        "Header Message Count: {:?} {:?}",
        message_count, header.message_count
    );
    assert_eq!(n, message_count);

    let mut k = 1;
    for msg in message_blocks {
        let message: String = std::str::from_utf8(&msg.message_data).unwrap().to_string();
        assert_eq!(&message, test_messages.get(k - 1).unwrap());

        k += 1;
    }
}

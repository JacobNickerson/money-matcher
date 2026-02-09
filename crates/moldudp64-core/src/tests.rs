use crate::Header;
use crate::MessageBlock;
use crate::Packet;
use crate::SessionTable;

#[test]
fn test_ids() {
    let mut my_struct = SessionTable::new();

    for _ in 0..5 {
        let session_id = my_struct.generate_session_id();
        my_struct.add_session(session_id, [0; 8]);
        println!(
            "Generated session ID: {:?}",
            std::str::from_utf8(&session_id).unwrap()
        );
    }
}

#[test]
fn test_packet() {
    let mut my_struct = SessionTable::new();

    let one: u16 = 1;
    let one2: u64 = 1;
    let header = Header {
        session_id: my_struct.generate_session_id(),
        sequence_number: one2.to_be_bytes(),
        message_count: one.to_be_bytes(),
    };

    println!("Header Session ID: {:?}", header.session_id);
    println!("Header Sequence Number: {:?}", header.sequence_number);
    println!("Header Message Count: {:?}", header.message_count);

    let message = "TESTING";
    let size: u16 = message.len() as u16;
    let message = MessageBlock {
        message_length: size.to_be_bytes(),
        message_data: message.as_bytes().to_vec(),
    };

    let mut m: Vec<MessageBlock> = Vec::new();
    m.push(message);
    let packet = Packet {
        header: Header {
            session_id: header.session_id,
            sequence_number: header.sequence_number,
            message_count: header.message_count,
        },
        messages: m,
    };

    let bytes: Vec<u8> = packet.to_bytes();
    println!("Packet as bytes: {:?}", &bytes);

    let reverse_packet: Packet = Packet::from_bytes(&bytes);

    println!("Header Session ID: {:?}", reverse_packet.header.session_id);
    println!(
        "Header Sequence Number: {:?}",
        reverse_packet.header.sequence_number
    );
    println!(
        "Header Message Count: {:?}",
        reverse_packet.header.message_count
    );

    assert_eq!(
        header.session_id, reverse_packet.header.session_id,
        "Session ID Match"
    );
    assert_eq!(
        header.sequence_number,
        reverse_packet.header.sequence_number
    );
    assert_eq!(header.message_count, reverse_packet.header.message_count);
}

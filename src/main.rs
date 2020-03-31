
use std::io::prelude::*;
use std::net::TcpStream;

use mqtt::packet::*;
use mqtt::control::*;
use mqtt::*;

use std::env;

use uuid::Uuid;

use std::str;

fn main()
{
    let broker_address = "localhost:1883"; //1883 is VerneMQ's default port
    let client_id = format!("/MQTT/rust/{}", Uuid::new_v4());
    let topic_name = "test";


    let args: Vec<String> = env::args().collect();
    let is_subscriber: bool = args.len() > 1 && args[1] == "--subscriber";


    let mut stream = TcpStream::connect(broker_address).unwrap();

    //Connection
    let mut conn = ConnectPacket::new("MQTT", client_id);
    conn.set_clean_session(true);

    let mut buf = Vec::new();
    conn.encode(&mut buf).unwrap();
    stream.write_all(&buf[..]).unwrap();

    let connack: ConnackPacket = ConnackPacket::decode(&mut stream).unwrap();

    if connack.connect_return_code() != ConnectReturnCode::ConnectionAccepted
    {
        panic!("failed to connect to server, {:?}", connack.connect_return_code());
    }

    println!("Connected!");


    if is_subscriber
    {
        //Read packets

        let topic_filter = TopicFilter::new(topic_name).unwrap();
        let sub_packet = SubscribePacket::new(10, vec![(topic_filter, QualityOfService::Level0)]);
        
        let mut buf = Vec::new();
        sub_packet.encode(&mut buf).unwrap();
        
        stream.write_all(&buf[..]).unwrap();

        loop
        {
            let packet = VariablePacket::decode(&mut stream).unwrap();

            match packet
            {
                VariablePacket::PublishPacket(ref pkt) =>
                {
                    let payload = str::from_utf8(pkt.payload_ref()).unwrap();

                    println!("Got something! {:?}", payload);
                }

                VariablePacket::SubackPacket(..) =>
                {
                    println!("Subscribed successfully!");
                }

                _ => {}
            }
        }
    }
    else
    {
        //Send packets

        let topic = TopicName::new(topic_name).unwrap();

        let mut counter = 0;

        loop
        {
            println!("Send {:?}!", counter);

            let msg = format!("Data from MQTT sensor {:?}", counter);
            let pub_packet = PublishPacket::new(topic.clone(), QoSWithPacketIdentifier::Level0, msg);

            let mut buf = Vec::new();
            pub_packet.encode(&mut buf).unwrap();
            stream.write_all(&buf[..]).unwrap();

            counter += 1;

            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }
}
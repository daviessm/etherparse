use etherparse::*;
use std;
use std::io;

#[test]
fn ipv4_new() {
    let result = Ipv4Header::new(15, 4, IpTrafficClass::Udp, [1,2,3,4], [5,6,7,8]);
    assert_eq!(Ipv4Header {
        header_length: 0,
        differentiated_services_code_point: 0,
        explicit_congestion_notification: 0,
        total_length: 15 + 20,
        identification: 0,
        dont_fragment: true,
        more_fragments: false,
        fragments_offset: 0,
        time_to_live: 4,
        protocol: IpTrafficClass::Udp as u8,
        header_checksum: 0,
        source: [1,2,3,4],
        destination: [5,6,7,8]
    }, result.unwrap());
}

#[test]
fn ipv4_new_error() {
    //border case check (no error)
    match Ipv4Header::new(
        (std::u16::MAX as usize) - 20,
        4,
        IpTrafficClass::Udp,
        [1,2,3,4],
        [5,6,7,8]
    ) {
        Ok(_) => {}, //all good
        value => assert!(false, format!("Expected an Ipv4Header but received {:?}", value))
    }
    //check that a too large payload generates an error
    const TOO_LARGE_PAYLOAD: usize = (std::u16::MAX as usize) - 19;
    match Ipv4Header::new(
        (std::u16::MAX as usize) - 19,
        4,
        IpTrafficClass::Udp,
        [1,2,3,4],
        [5,6,7,8]
    ) {
        Err(ValueError::Ipv4PayloadAndOptionsLengthTooLarge(TOO_LARGE_PAYLOAD)) => {}, //all good
        value => assert!(false, format!("Expected an Ipv4PayloadAndOptionsLengthTooLarge error but received {:?}", value))
    }
}

#[test]
fn ipv4_calc_header_checksum() {
    use ValueError::*;
    use ErrorField::*;
    //without options
    {
        let header = Ipv4Header {
            header_length: 5,
            differentiated_services_code_point: 0,
            explicit_congestion_notification: 0,
            total_length: 40 + 20,
            identification: 0,
            dont_fragment: true,
            more_fragments: false,
            fragments_offset: 0,
            time_to_live: 4,
            protocol: IpTrafficClass::Udp as u8,
            header_checksum: 0,
            source: [192, 168, 1, 1],
            destination: [212, 10, 11, 123]
        };
        assert_eq!(0xd582, header.calc_header_checksum(&[]).unwrap());
    }
    //with options
    {
        let header = Ipv4Header {
            header_length: 7,
            differentiated_services_code_point: 0,
            explicit_congestion_notification: 0,
            total_length: 40 + 20,
            identification: 0,
            dont_fragment: true,
            more_fragments: false,
            fragments_offset: 0,
            time_to_live: 4,
            protocol: IpTrafficClass::Udp as u8,
            header_checksum: 0,
            source: [192, 168, 1, 1],
            destination: [212, 10, 11, 123]
        };
        assert_eq!(0xc36e, header.calc_header_checksum(&[1,2,3,4,5,6,7,8]).unwrap());
    }
    //check errors
    {
        //max value check header length
        {
            let mut header = Ipv4Header::new(15, 4, IpTrafficClass::Udp, [1,2,3,4], [5,6,7,8]).unwrap();
            header.header_length = 0x10;
            match header.calc_header_checksum(&[]) {
                Err(U8TooLarge{value: 0x10, max: 0xf, field: Ipv4HeaderLength}) => {}, //all good
                value => assert!(false, format!("Expected a ValueU8TooLarge error but received {:?}", value))
            }
        }
        //max check differentiated_services_code_point
        {
            let mut header = Ipv4Header::new(15, 4, IpTrafficClass::Udp, [1,2,3,4], [5,6,7,8]).unwrap();
            header.differentiated_services_code_point = 0x40;
            match header.calc_header_checksum(&[]) {
                Err(U8TooLarge{value: 0x40, max: 0x3f, field: Ipv4Dscp}) => {}, //all good
                value => assert!(false, format!("Expected a ValueU8TooLarge error but received {:?}", value))
            }
        }
        //max check explicit_congestion_notification
        {
            let mut header = Ipv4Header::new(15, 4, IpTrafficClass::Udp, [1,2,3,4], [5,6,7,8]).unwrap();
            header.explicit_congestion_notification = 0x4;
            match header.calc_header_checksum(&[]) {
                Err(U8TooLarge{value: 0x4, max: 0x3, field: Ipv4Ecn}) => {}, //all good
                value => assert!(false, format!("Expected a ValueU8TooLarge error but received {:?}", value))
            }
        }
        //max check fragments_offset
        {
            let mut header = Ipv4Header::new(15, 4, IpTrafficClass::Udp, [1,2,3,4], [5,6,7,8]).unwrap();
            header.fragments_offset = 0x2000;
            match header.calc_header_checksum(&[]) {
                Err(U16TooLarge{value: 0x2000, max: 0x1fff, field: Ipv4FragmentsOffset}) => {}, //all good
                value => assert!(false, format!("Expected a ValueU8TooLarge error but received {:?}", value))
            }
        }
        //non 4 byte aligned options check
        {
            let header = Ipv4Header::new(15, 4, IpTrafficClass::Udp, [1,2,3,4], [5,6,7,8]).unwrap();
            let options = vec![0;9]; //9 is non 4 byte aligned
            match header.calc_header_checksum(&options) {
                Err(Ipv4OptionsLengthBad(9)) => {}, //all good
                value => assert!(false, format!("Expected a Ipv4OptionsLengthBad error but received {:?}", value))
            }
        }
        //options too large test
        {
            let header = Ipv4Header::new(15, 4, IpTrafficClass::Udp, [1,2,3,4], [5,6,7,8]).unwrap();
            let options = vec![0;11*4]; //11 is a too big number to store in the ipv4 header
            match header.calc_header_checksum(&options) {
                Err(Ipv4OptionsLengthBad(44)) => {}, //all good
                value => assert!(false, format!("Expected a Ipv4OptionsLengthBad error but received {:?}", value))
            }
        }
    }
}

#[test]
fn read_ip_header_ipv4() {
    use std::io::Cursor;

    let inputs = [
        Ipv4Header {
            header_length: 10,
            differentiated_services_code_point: 42,
            explicit_congestion_notification: 3,
            total_length: 1234,
            identification: 4321,
            dont_fragment: true,
            more_fragments: false,
            fragments_offset: 4367,
            time_to_live: 8,
            protocol: 1,
            header_checksum: 2345,
            source: [192, 168, 1, 1],
            destination: [212, 10, 11, 123]
        },
        Ipv4Header {
            header_length: 10,
            differentiated_services_code_point: 42,
            explicit_congestion_notification: 3,
            total_length: 1234,
            identification: 4321,
            dont_fragment: false,
            more_fragments: true,
            fragments_offset: 4367,
            time_to_live: 8,
            protocol: 1,
            header_checksum: 2345,
            source: [192, 168, 1, 1],
            destination: [212, 10, 11, 123]
        }
    ];
    for input in &inputs {
        //serialize
        let mut buffer: Vec<u8> = Vec::with_capacity(20);
        buffer.write_ipv4_header_raw(&input, &[]).unwrap();
        assert_eq!(20, buffer.len());

        //deserialize
        let mut cursor = Cursor::new(&buffer);
        let result = cursor.read_ip_header().unwrap();
        assert_eq!(20, cursor.position());

        match result {
            IpHeader::Version4(result) => assert_eq!(*input, result),
            value => assert!(false, format!("Expected IpHeaderV4 but received {:?}", value))
        }
    }
}

#[test]
fn read_ip_header_ipv6() {
    use std::io::Cursor;
    let input = Ipv6Header {
        traffic_class: 1,
        flow_label: 0x81806,
        payload_length: 0x8021,
        next_header: 30,
        hop_limit: 40,
        source: [1, 2, 3, 4, 5, 6, 7, 8,
                 9,10,11,12,13,14,15,16],
        destination: [21,22,23,24,25,26,27,28,
                      29,30,31,32,33,34,35,36]
    };
    //serialize
    let mut buffer: Vec<u8> = Vec::with_capacity(20);
    buffer.write_ipv6_header(&input).unwrap();
    assert_eq!(40, buffer.len());

    //deserialize
    let mut cursor = Cursor::new(&buffer);
    let result = cursor.read_ip_header().unwrap();
    assert_eq!(40, cursor.position());

    match result {
        IpHeader::Version6(result) => assert_eq!(input, result),
        value => assert!(false, format!("Expected IpHeaderV6 but received {:?}", value))
    }
}

#[test]
fn read_ip_header_error() {
    use std::io::Cursor;
    let input = Ipv6Header {
        traffic_class: 1,
        flow_label: 0x81806,
        payload_length: 0x8021,
        next_header: 30,
        hop_limit: 40,
        source: [1, 2, 3, 4, 5, 6, 7, 8,
                 9,10,11,12,13,14,15,16],
        destination: [21,22,23,24,25,26,27,28,
                      29,30,31,32,33,34,35,36]
    };
    //serialize
    let mut buffer: Vec<u8> = Vec::with_capacity(20);
    buffer.write_ipv6_header(&input).unwrap();
    assert_eq!(40, buffer.len());

    //corrupt the version
    buffer[0] = 0xff;

    //deserialize
    let mut cursor = Cursor::new(&buffer);
    match cursor.read_ip_header() {
        Err(ReadError::IpUnsupportedVersion(0xf)) => {}, //all good
        value => assert!(false, format!("Expected a IpUnsupportedVersion error but received {:?}", value))
    }
}

#[test]
fn readwrite_ipv4_header_raw() {
    use std::io::Cursor;

    let input = Ipv4Header {
        header_length: 10,
        differentiated_services_code_point: 42,
        explicit_congestion_notification: 3,
        total_length: 1234,
        identification: 4321,
        dont_fragment: true,
        more_fragments: false,
        fragments_offset: 4367,
        time_to_live: 8,
        protocol: 1,
        header_checksum: 2345,
        source: [192, 168, 1, 1],
        destination: [212, 10, 11, 123]
    };
    //serialize
    let mut buffer: Vec<u8> = Vec::with_capacity(20);
    buffer.write_ipv4_header_raw(&input, &[]).unwrap();
    assert_eq!(20, buffer.len());

    //deserialize
    let mut cursor = Cursor::new(&buffer);
    let result = cursor.read_ipv4_header().unwrap();
    assert_eq!(20, cursor.position());

    //check equivalence
    assert_eq!(input, result);
}

#[test]
fn write_ipv4_raw_header_errors() {
    use WriteError::ValueError;
    use ValueError::*;
    use ErrorField::*;
    fn base() -> Ipv4Header {
        Ipv4Header{
            header_length: 10,
            differentiated_services_code_point: 42,
            explicit_congestion_notification: 3,
            total_length: 1234,
            identification: 4321,
            dont_fragment: true,
            more_fragments: false,
            fragments_offset: 4367,
            time_to_live: 8,
            protocol: 1,
            header_checksum: 2345,
            source: [192, 168, 1, 1],
            destination: [212, 10, 11, 123]
        }
    };

    fn test_write(input: &Ipv4Header) -> Result<(), WriteError> {
        let mut buffer: Vec<u8> = Vec::new();
        let result = buffer.write_ipv4_header_raw(input, &[]);
        assert_eq!(0, buffer.len());
        result
    };
    //header_length
    match test_write(&{
        let mut value = base();
        value.header_length = 0x10;
        value
    }) {
        Err(ValueError(U8TooLarge{value: 0x10, max: 0xf, field: Ipv4HeaderLength})) => {}, //all good
        value => assert!(false, format!("Expected a range error but received {:?}", value))
    }
    //dscp
    match test_write(&{
        let mut value = base();
        value.differentiated_services_code_point = 0x40;
        value
    }) {
        Err(ValueError(U8TooLarge{value: 0x40, max: 0x3f, field: Ipv4Dscp})) => {}, //all good
        value => assert!(false, format!("Expected a range error but received {:?}", value))
    }
    //ecn
    match test_write(&{
        let mut value = base();
        value.explicit_congestion_notification = 0x4;
        value
    }) {
        Err(ValueError(U8TooLarge{value: 0x4, max: 0x3, field: Ipv4Ecn})) => {}, //all good
        value => assert!(false, format!("Expected a range error but received {:?}", value))
    }
    //fragmentation offset
    match test_write(&{
        let mut value = base();
        value.fragments_offset = 0x2000;
        value
    }) {
        Err(ValueError(U16TooLarge{value: 0x2000, max: 0x1FFF, field: Ipv4FragmentsOffset})) => {}, //all good
        value => assert!(false, format!("Expected a range error but received {:?}", value))
    }
    //options header length (non 4 modulo)
    {
        let mut buffer: Vec<u8> = Vec::new();
        let result = buffer.write_ipv4_header_raw(&base(), &[1,2]);
        assert_eq!(0, buffer.len());
        match result {
            Err(ValueError(Ipv4OptionsLengthBad(2))) => {}, //all good
            value => assert!(false, format!("Expected a Ipv4OptionsLengthBad error but received {:?}", value))
        }
    }
    //options header length (non 4 modulo)
    {
        let mut buffer: Vec<u8> = Vec::new();
        let result = buffer.write_ipv4_header_raw(&base(), &vec![0;44]);
        assert_eq!(0, buffer.len());
        match result {
            Err(ValueError(Ipv4OptionsLengthBad(44))) => {}, //all good
            value => assert!(false, format!("Expected a Ipv4OptionsLengthBad error but received {:?}", value))
        }
    }
}

#[test]
fn write_ipv4_header() {
    use std::io::Cursor;

    let mut input = Ipv4Header {
        header_length: 0,
        differentiated_services_code_point: 42,
        explicit_congestion_notification: 3,
        total_length: 1234,
        identification: 4321,
        dont_fragment: true,
        more_fragments: false,
        fragments_offset: 4367,
        time_to_live: 8,
        protocol: 1,
        header_checksum: 0,
        source: [192, 168, 1, 1],
        destination: [212, 10, 11, 123]
    };
    //serialize
    let mut buffer: Vec<u8> = Vec::with_capacity(20);
    buffer.write_ipv4_header(&input, &[]).unwrap();
    assert_eq!(20, buffer.len());

    //deserialize
    let mut cursor = Cursor::new(&buffer);
    let result = cursor.read_ipv4_header().unwrap();
    assert_eq!(20, cursor.position());

    //check equivalence (with calculated checksum & header_length)
    input.header_length = 5;
    input.header_checksum = input.calc_header_checksum(&[]).unwrap();
    assert_eq!(input, result);
}

#[test]
fn read_ipv4_error_header() {
    let buffer: [u8;20] = [0;20];
    let mut cursor = io::Cursor::new(&buffer);
    let result = cursor.read_ipv4_header();
    match result {
        Err(ReadError::Ipv4UnexpectedVersion(0)) => {},
        _ => assert!(false, format!("Expected ipv 4 version error but received {:?}", result))
    }
}

#[test]
fn readwrite_ipv6_header() {
    use std::io::Cursor;

    let input = Ipv6Header {
        traffic_class: 1,
        flow_label: 0x81806,
        payload_length: 0x8021,
        next_header: 30,
        hop_limit: 40,
        source: [1, 2, 3, 4, 5, 6, 7, 8,
                 9,10,11,12,13,14,15,16],
        destination: [21,22,23,24,25,26,27,28,
                      29,30,31,32,33,34,35,36]
    };
    //serialize
    let mut buffer: Vec<u8> = Vec::with_capacity(20);
    buffer.write_ipv6_header(&input).unwrap();
    //deserialize
    let result = {
        let mut cursor = Cursor::new(&buffer);
        cursor.read_ipv6_header().unwrap()
    };
    //check equivalence
    assert_eq!(input, result);
}

#[test]
fn write_ipv6_header_errors() {
    use WriteError::ValueError;
    use ValueError::*;
    use ErrorField::*;
    fn base() -> Ipv6Header {
        Ipv6Header {
            traffic_class: 1,
            flow_label: 0x201806,
            payload_length: 0x8021,
            next_header: 30,
            hop_limit: 40,
            source: [1, 2, 3, 4, 5, 6, 7, 8,
                     9,10,11,12,13,14,15,16],
            destination: [21,22,23,24,25,26,27,28,
                          29,30,31,32,33,34,35,36]
        }
    };

    fn test_write(input: &Ipv6Header) -> Result<(), WriteError> {
        let mut buffer: Vec<u8> = Vec::with_capacity(20);
        buffer.write_ipv6_header(input)
    };
    //flow label
    match test_write(&{
        let mut value = base();
        value.flow_label = 0x100000;
        value
    }) {
        Err(ValueError(U32TooLarge{value: 0x100000, max: 0xFFFFF, field: Ipv6FlowLabel})) => {}, //all good
        value => assert!(false, format!("Expected a range error but received {:?}", value))
    }
}

#[test]
fn read_ipv6_error_header() {
    let buffer: [u8;20] = [0;20];
    let mut cursor = io::Cursor::new(&buffer);
    let result = cursor.read_ipv6_header();
    match result {
        Err(ReadError::Ipv6UnexpectedVersion(0)) => {},
        _ => assert!(false, format!("Expected ipv 6 version error but received {:?}", result))
    }
}

#[test]
fn skip_ipv6_header_extension() {
    use std::io::Cursor;
    {
        let buffer: [u8; 8] = [0;8];
        let mut cursor = Cursor::new(&buffer);
        match cursor.skip_ipv6_header_extension() {
            Ok(0) => {},
            value => assert!(false, format!("Expected Ok(0) but received {:?}", value))
        }
        assert_eq!(8, cursor.position());
    }
    {
        let buffer: [u8; 8*3] = [
            4,2,0,0, 0,0,0,0,
            0,0,0,0, 0,0,0,0,
            0,0,0,0, 0,0,0,0,
        ];
        let mut cursor = Cursor::new(&buffer);
        match cursor.skip_ipv6_header_extension() {
            Ok(4) => {},
            value => assert!(false, format!("Expected Ok(4) but received {:?}", value))
        }
        assert_eq!(8*3, cursor.position());
    }
}

#[test]
fn skip_all_ipv6_header_extensions() {
    use io::Cursor;
    //extension header values
    use IpTrafficClass::*;
    //based on RFC 8200 4.1. Extension Header Order
    const EXTENSION_IDS: [u8;7] = [
        IPv6HeaderHopByHop as u8,
        IPv6DestinationOptions as u8,
        IPv6RouteHeader as u8,
        IPv6FragmentationHeader as u8,
        IPv6AuthenticationHeader as u8,
        IPv6EncapSecurityPayload as u8,
        IPv6DestinationOptions as u8
    ];
    const UDP: u8 = Udp as u8;

    //no & single skipping
    {
        let buffer: [u8; 8*3] = [
            UDP,2,0,0, 0,0,0,0, //set next to udp
            0,0,0,0,   0,0,0,0,
            0,0,0,0,   0,0,0,0,
        ];

        for i_as16 in 0..((u8::max_value() as u16) + 1) {
            let i = i_as16 as u8; //note: I would prefer to use the inclusive range ..= but this feature is not yet marked as stable -> replace when stable
            let mut cursor = Cursor::new(&buffer);
            let result = cursor.skip_all_ipv6_header_extensions(i);

            match EXTENSION_IDS.iter().find(|&&x| x == i) {
                Some(_) => {
                    //ipv6 header extension -> expect skip
                    match result {
                        Ok(UDP) => {},
                        _ => assert!(false, format!("exepected udp as next traffic_class but received {:?}", result))
                    }
                    assert_eq!(buffer.len(), cursor.position() as usize);
                },
                None => {
                    //non ipv6 header expect no read movement and direct return
                    match result {
                        Ok(next) => assert_eq!(i, next),
                        _ => assert!(false, format!("exepected {} as next traffic_class but received {:?}", i, result))
                    }
                    assert_eq!(0, cursor.position());
                }
            }
        }

    }
    //skip 7 (max)
    {
        let buffer = vec![
            EXTENSION_IDS[1],0,0,0, 0,0,0,0,
            EXTENSION_IDS[2],1,0,0, 0,0,0,0,
            0,0,0,0,                0,0,0,0,
            EXTENSION_IDS[3],0,0,0, 0,0,0,0,
            EXTENSION_IDS[4],1,0,0, 0,0,0,0,

            0,0,0,0,                0,0,0,0,
            EXTENSION_IDS[5],0,0,0, 0,0,0,0,
            EXTENSION_IDS[6],0,0,0, 0,0,0,0,
            UDP,2,0,0, 0,0,0,0,

            0,0,0,0,   0,0,0,0,
            0,0,0,0,   0,0,0,0,
        ];
        let mut cursor = Cursor::new(&buffer);
        let result = cursor.skip_all_ipv6_header_extensions(EXTENSION_IDS[0]);
        match result {
            Ok(UDP) => {},
            result => assert!(false, format!("exepected udp as next traffic_class but received {:?}", result)) 
        }
        assert_eq!(buffer.len(), cursor.position() as usize);
    }
    //trigger "too many" error
    {
        let buffer = vec![
            EXTENSION_IDS[1],0,0,0, 0,0,0,0,
            EXTENSION_IDS[2],0,0,0, 0,0,0,0,
            EXTENSION_IDS[3],0,0,0, 0,0,0,0,
            EXTENSION_IDS[4],0,0,0, 0,0,0,0,
            EXTENSION_IDS[5],0,0,0, 0,0,0,0,
            EXTENSION_IDS[6],0,0,0, 0,0,0,0,
            EXTENSION_IDS[1],0,0,0, 0,0,0,0,
        ];
        let mut cursor = Cursor::new(&buffer);
        let result = cursor.skip_all_ipv6_header_extensions(EXTENSION_IDS[0]);
        match result {
            Err(ReadError::Ipv6TooManyHeaderExtensions) => {},
            result => assert!(false, format!("exepected error Ipv6TooManyHeaderExtensions but received {:?}", result)) 
        }
    }
}
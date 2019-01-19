use crate::ignite_error::IgniteResult;
use crate::net::end_point::EndPoint;

/// Parse endpoints string.
/// The format is `"<host>[:<port>[..<port>]][,...]"`.
pub fn parse_endpoints(end_points: &str) -> IgniteResult<Vec<EndPoint>> {
    let mut res = Vec::<EndPoint>::new();
    for sep in end_points.split(',') {
        res.push(EndPoint::from_string(sep)?);
    }
    Ok(res)
}

#[test]
fn test_parse_endpoints() {
    parse_endpoints("127.0.0.1").unwrap();
    parse_endpoints("example.com").unwrap();
    parse_endpoints("localhost").unwrap();
    parse_endpoints("127.0.0.1:10800").unwrap();
    parse_endpoints("127.0.0.1:10800..10810").unwrap();
    parse_endpoints("127.0.0.1,example.com").unwrap();
    parse_endpoints("127.0.0.1, example.com").unwrap();
    parse_endpoints("127.0.0.1, example.com ").unwrap();
    parse_endpoints("127.0.0.1,example.com,localhost").unwrap();
    parse_endpoints("127.0.0.1:12,example.com,localhost").unwrap();
    parse_endpoints("127.0.0.1,example.com:12,localhost").unwrap();
    parse_endpoints("127.0.0.1,example.com,localhost:12").unwrap();
    parse_endpoints("127.0.0.1:12,example.com:12,localhost").unwrap();
    parse_endpoints("127.0.0.1:12,example.com,localhost:12").unwrap();
    parse_endpoints("127.0.0.1:12,example.com:12,localhost:12").unwrap();
    parse_endpoints("127.0.0.1:12..42,example.com:12,localhost:12").unwrap();
    parse_endpoints("127.0.0.1:12..42,example.com:12..42,localhost:12").unwrap();
    parse_endpoints("127.0.0.1:12..42,example.com:12,localhost:12..42").unwrap();
    parse_endpoints("127.0.0.1:12..42,example.com:12..42,localhost:12..42").unwrap();
    parse_endpoints("127.0.0.1:12..42,example.com,localhost:12").unwrap();

    parse_endpoints("127.0.0.1:12..42,,localhost:12").unwrap_err();
    parse_endpoints("127.0.0.1:12..42,localhost:12,").unwrap_err();
    parse_endpoints(",127.0.0.1:12..42,localhost:12").unwrap_err();
}

use ignite_error::IgniteResult;
use net::end_point::EndPoint;

/// Parse endpoints string.
/// The format is `"<host>[:<port>[..<port>]][,...]"`.
pub fn parse_endpoints(end_points: &str) -> IgniteResult<Vec<EndPoint>> {
    let mut res = Vec::<EndPoint>::new();
    for sep in end_points.split(',') {
        res.push(EndPoint::from_string(sep)?);
    }
    Ok(res)
}

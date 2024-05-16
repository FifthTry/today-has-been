extern crate self as backend;

#[no_mangle]
pub extern "C" fn main_ft() {
    let req = ft_sdk::http::current_request();
    let resp = backend::route(req);
    ft_sdk::http::send_response(resp);
}


pub fn route(_r: http::Request<bytes::Bytes>) -> http::Response<bytes::Bytes> {
    todo!()
}

#![feature(test)]

#[macro_use]
extern crate json;
extern crate test;
extern crate serde;
extern crate serde_json;

use test::Bencher;

const JSON_STR: &'static str = r#"{"timestamp":2837513946597,"zone_id":123456,"zone_plan":1,"http":{"protocol":2,"status":200,"host_status":503,"up_status":520,"method":1,"content_type":"text/html","user_agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36","referer":"https://www.cloudflare.com/","request_uri":"/cdn-cgi/trace"},"origin":{"ip":"1.2.3.4","port":8000,"hostname":"www.example.com","protocol":2},"country":238,"cache_status":3,"server_ip":"192.168.1.1","server_name":"metal.cloudflare.com","remote_ip":"10.1.2.3","bytes_dlv":123456,"ray_id":"10c73629cce30078-LAX","true":true,"false":false,"null":null}"#;

#[bench]
fn bench_serde_value(b: &mut Bencher) {
    b.bytes = JSON_STR.len() as u64;

    b.iter(|| {
        serde_json::from_str::<serde_json::Value>(JSON_STR).unwrap()
    });
}

#[bench]
fn bench_json_rust(b: &mut Bencher) {
    extern crate json;

    b.bytes = JSON_STR.len() as u64;

    b.iter(|| {
        json::parse(JSON_STR).unwrap()
    });
}

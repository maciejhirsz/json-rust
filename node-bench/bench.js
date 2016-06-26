let json = `{"timestamp":2837513946597,"zone_id":123456,"zone_plan":1,"http":{"protocol":2,"status":200,"host_status":503,"up_status":520,"method":1,"content_type":"text/html","user_agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36","referer":"https://www.cloudflare.com/","request_uri":"/cdn-cgi/trace"},"origin":{"ip":"1.2.3.4","port":8000,"hostname":"www.example.com","protocol":2},"country":238,"cache_status":3,"server_ip":"192.168.1.1","server_name":"metal.cloudflare.com","remote_ip":"10.1.2.3","bytes_dlv":123456,"ray_id":"10c73629cce30078-LAX"}`;
let data = JSON.parse(json);
const bytes = json.length;
const maxIterations = 1;

function bench(iter) {
    let iterations = maxIterations;

    const start = Date.now();
    let totalNanos = 0;
    while (iterations--) {
        // data = JSON.parse(json);
        // json = JSON.stringify(data);
        const start = process.hrtime();
        iter();
        totalNanos += process.hrtime(start)[1];
    }

    const average = totalNanos / maxIterations;
    const iterPerSec = 1e9 / average;
    const throughput = (bytes * iterPerSec) / (1024 * 1024);

    console.log(`Benching ${iter.name}`);
    console.log(`- ${Math.round(average)}ns per iteration`);
    console.log(`- throughput ${Math.round(throughput * 100) / 100} MB/s`);
    console.log('');
}

bench(function parse() {
    out = JSON.parse(json);
});

bench(function serialize() {
    out = JSON.stringify(data);
});

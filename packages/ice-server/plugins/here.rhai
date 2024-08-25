fn id() {
    "here"
}

fn on_load(server) {
    print("on_load");
    // server.start();
    server.delay_call(1000, id(), "main");
}

fn on_server_log(server, content) {
    print("on_serverlog: " + content)
}

fn on_server_done(server) {
    print("on_server_done");
    server.delay_call(1000, id(), "main")
}

let cnt = 0;

fn main(server) {
    print(main);
    print(cnt);
    cnt += 1;
    server.delay_call(1000, id(), "main")
}
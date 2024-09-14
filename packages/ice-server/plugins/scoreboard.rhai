fn id() {
    "scoreboard"
}

const DEFAULT_CONFIG = #{
    interval: 10,
};

let config = DEFAULT_CONFIG + config();
let server = server();

let objectives = [];
let cur = 0;

const REGEX_OBJECTIVE_SET = "]: Set display slot sidebar to show objective";
const REGEX_OBJECTIVES_SET_UNCHANGED = "]: Nothing changed. That display slot is already showing that objective";
fn on_load() {
    server.add_log_filter(REGEX_OBJECTIVE_SET);
    server.add_log_filter(REGEX_OBJECTIVES_SET_UNCHANGED);
}

fn on_server_done() {
    server.writeln("scoreboard objectives list");
    server.delay_call(config.interval * 1000, "scoreboard", "change_objective");
}

const REGEX_OBJECTIVES = "]: There are \\d+ objective\\(s\\): (.*)";
const REGEX_OBJECTIVE = "\\[([^\\]]+)\\]";
fn on_server_log(content) {
    // Result of "scoreboard objectives list"
    if regex::regex_match(content, REGEX_OBJECTIVES) {
        let s = regex::regex_captures(content, REGEX_OBJECTIVES)[1];
        let objs = regex::regex_captures_iter(s, REGEX_OBJECTIVE);
        objectives = objs.map(|v| v[1]);
        cur = 0;
    }
}

fn change_objective() {
    if !objectives.is_empty() {
        cur = (cur + 1) % objectives.len();
        server.writeln("scoreboard objectives setdisplay sidebar " + objectives[cur]);
    }
    server.delay_call(config.interval * 1000, "scoreboard", "change_objective");
}

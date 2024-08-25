fn id() {
    "here"
}

let here_user = 0;

fn request_player_position(server, player) {
    server.writeln(`data get entity ${player}`);
}

// https://github.com/TISUnion/Here/blob/master/here/entry.py#L44
fn display(server, player, dimension, position) {
    // basic text: someone @ dimension [x, y, z]
    let text = `${player} @ ${dimension} [${position[0]}, ${position[1]}, ${position[2]}]`;
    let time = 10;
    server.say(text);
    server.writeln(`effect give ${player} minecraft:glowing ${time} 0 true`);
}

fn on_player_message(server, player, msg) {
    if msg == "#here" {
        here_user += 1;
        request_player_position(server, player);
    }
}

fn on_server_log(server, content) {
    if here_user > 0 && regex_match(content, "\\w+ has the following entity data: ") {
        let name = regex_captures(content, "(\\w+) has the following entity data: ")[1];

        let dimension = regex_captures(content, "Dimension: (.*?),")[1];
        dimension.replace("\"", "");
        dimension.replace("'", "");
        dimension.replace(",", "");

        let position = regex_captures(content, "Pos: \\[(.*?)d, (.*?)d, (.*?)d]");
        position = position.extract(1..=3).map(|s| parse_float(s));

        display(server, name, dimension, position);

        here_user -= 1;
    }
}

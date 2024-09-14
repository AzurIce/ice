fn id() {
    "here"
}

const DEFAULT_CONFIG = #{
    display_voxel_waypoint: true,
    display_xaero_waypoint: true
};

let config = DEFAULT_CONFIG + config();
let server = server();
let here_user = 0;

fn on_load() {
    request_player_position.call("_AzurIce_");
}

const request_player_position = |player| {
    server.writeln(`data get entity ${player}`);
};

// https://github.com/TISUnion/Here/blob/master/here/entry.py#L44
const display = |player, dimension, position| {
    let dimension_color = if dimension == "minecraft:overworld" {
        Color::DarkGreen
    } else if dimension == "minecraft:the_nether" {
        Color::DarkRed
    } else if dimension == "minecraft:the_end" {
        Color::DarkPurple
    };

    // basic text: someone @ dimension [x, y, z]
    let player_text = Rtext(player).color(Color::Yellow);
    let dimension_text = Rtext(dimension).color(dimension_color);

    let position_text = `${position[0]}, ${position[1]}, ${position[2]}`;
    let position_text = Rtext(`[${position_text}]`).color(Color::Aqua).underlined()
        .hover_event(HoverEvent::ShowText(Rtext("Click to copy")))
        .click_event(ClickEvent::CopyToClipboard(position_text));
    
    let text = ["", player_text, " @ ", dimension_text, " ", position_text];

    let x = position[0].to_int();
    let y = position[1].to_int();
    let z = position[2].to_int();

    let dimension_name = dimension.replace("minecraft:", "");
    if config.display_voxel_waypoint {
        let voxel_waypoint_command = `newWaypoint x:${x}, y:${y}, z:${z}, dim:${dimension}`;
        let voxel_waypoint_text = Rtext(`[+V]`).color(Color::Gold)
            .hover_event(HoverEvent::ShowText(Rtext("§6Voxel§r: 点击添加路径点")))
            .click_event(ClickEvent::ExecuteCommand(voxel_waypoint_command));
        text.push(voxel_waypoint_text);
    }
    if config.display_xaero_waypoint {
        let xaero_waypoint_command = `xaero_waypoint_add:${player}'s Location:${player[0]}:${x}:${y}:${z}:6:false:0`;
        xaero_waypoint_command += `:Internal_${dimension_name}_waypoints`;
        let xaero_waypoint_text = Rtext(`[+X]`).color(Color::Gold)
            .hover_event(HoverEvent::ShowText(Rtext("§6Xaeros Minimap§r: 点击添加路径点")))
            .click_event(ClickEvent::ExecuteCommand(xaero_waypoint_command));
        text.push(xaero_waypoint_text);
    }

    server.tellraw("@a", text);
    let time = 10;
    // server.say(text);
    server.writeln(`effect give ${player} minecraft:glowing ${time} 0 true`);
};

fn on_player_message(player, msg) {
    if msg == "#here" {
        here_user += 1;
        request_player_position.call(player);
    }
}

const REGEX = "(\\w+) has the following entity data: ";
const REGEX_DIMENSION = "Dimension: (.*?),";
const REGEX_POSITION = "Pos: \\[(.*?)d, (.*?)d, (.*?)d]";
fn on_server_log(content) {
    if here_user > 0 && regex::regex_match(content, REGEX) {
        let name = regex::regex_captures(content, REGEX)[1];

        let dimension = regex::regex_captures(content, REGEX_DIMENSION)[1];
        dimension.replace("\"", "");
        dimension.replace("'", "");
        dimension.replace(",", "");

        let position = regex::regex_captures(content, REGEX_POSITION);
        position = position.extract(1..=3).map(|s| parse_float(s));

        display.call(name, dimension, position);

        here_user -= 1;
    }
}

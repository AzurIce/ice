fn id() {
    "rtext"
}

fn on_load(server) {
    print(build_component("text"));
    print(build_component(["a", 0.3, 1, false, Rtext("text").color(Color::Green).obfuscated()]));
    // print(build_component(1));
    print(build_component(true));
    print(build_component(Rtext("text").color(Color::Green).obfuscated()));
    print(build_component(Rtext(1).color(Color::Green).underlined().italic()));
    let rtext = Rtext(true).color(Color::Green).bold().strikethrough();
    print(build_component(rtext));
    server.tellraw("@a", rtext)
}

fn on_player_message(server, player, msg) {
    if msg == "#rtext" {
        let rtext = Rtext(true).color(Color::Green).bold().strikethrough();
        server.tellraw(player, rtext)
    }
}
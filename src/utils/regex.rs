use std::sync::OnceLock;

use regex::Regex;

const FORWARD: &str = r"^(.+) *\| *(\S+?)\n";
pub fn forward_regex() -> &'static Regex {
    static FORWARD_REGEX: OnceLock<Regex> = OnceLock::new();
    FORWARD_REGEX.get_or_init(|| Regex::new(FORWARD).expect("regex err"))
}

/*
[16:00:01] [Server thread/INFO]: _AzurIce_[/127.0.0.1:58952] logged in with entity id 112 at (-21.5, 72.0, -7.5)
[16:00:01] [Server thread/INFO]: _AzurIce_ joined the game
[16:00:04] [Server thread/INFO]: <_AzurIce_> asd
[16:00:06] [Server thread/INFO]: _AzurIce_ lost connection: Disconnected
[16:00:06] [Server thread/INFO]: _AzurIce_ left the game
 */
/*
[19:23:48] [Server thread/INFO]: [Not Secure] <_AzurIce_> #bksnap make
 */
const PLAYER: &str = r"]: (?:\[Not Secure] )?<(.*?)> (.*)";
pub fn player_regex() -> &'static Regex {
    static PLAYER_REGEX: OnceLock<Regex> = OnceLock::new();
    PLAYER_REGEX.get_or_init(|| Regex::new(PLAYER).expect("regex err"))
}

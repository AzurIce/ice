use std::sync::OnceLock;

use regex::Regex;

/*
[17:19:12] [Server thread/INFO]: Time elapsed: 4208 ms
[17:19:12] [Server thread/INFO]: sleep_ignore_fake_players app loaded
[17:19:12] [Server thread/INFO]: App sleep_ignore_fake_players loaded in 51 ms
[17:19:12] [Server thread/INFO]: Done (7.109s)! For help, type "help"
*/
const DONE: &str = r"]: Done \(\d+.\d+s\)!";
pub fn done_regex() -> &'static Regex {
    static DONE_REGEX: OnceLock<Regex> = OnceLock::new();
    DONE_REGEX.get_or_init(|| Regex::new(DONE).unwrap())
}

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


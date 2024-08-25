# Rhai Plugin

使用 Rhai 脚本编写插件

## 事件

Ice 使用事件来在特定的时机以特定的信息调用插件实现的钩子函数。

- `on_server_log` 服务器日志输出
  `on_server_log(server: Server, log: string)`

- `on_server_done` 服务器启动完成
  > 由形如 `[17:19:12] [Server thread/INFO]: Done (7.109s)! For help, type "help"` 的一行输出触发，
  > 正则表达式为：`]: Done \(\d+.\d+s\)!`

  `on_server_done(server: Server)`

- `on_player_message` 玩家输出内容
  > 由形如 `[19:23:48] [Server thread/INFO]: [Not Secure] <_AzurIce_> #bksnap make` 的一行输出触发
  > 正则表达式为：`]: (?:\[Not Secure] )?<(.*?)> (.*)`

  `on_player_message(player: Player, message: string)`

- `on_timer` 定时器触发，有关定时器见 [定时器](#定时器)
  `on_timer()`

## api
### 定时器


  
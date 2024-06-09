```toml
command_prefix = "#"
backup_dir = "./backups"

[servers]
[servers."1.20.4 Survival"]
dir = "G:\\_MCServer\\1.20.4 Survival"
jvm_options = ""
version = "1.20.4"
[servers."1.20.4 Survival".properties]
difficulty = "hard"
enforce-secure-profile = "false"
gamemode = "survival"
level-seed = "20240110"
max-player = "7"
motd = "AzurCraft reborn"
online-mode = "true"
server-ip = "0.0.0.0"
server-port = "25565"
spawn-protection = "0"
view-distance = "16"
white-list = "true"
```

## 使用

使用如下命令来创建一个 Bish 目录：

```
bish new xxx
```

或者在一个目录下使用以下命令来将其初始化为 Bish 目录：

```
bish init xxx
```


使用如下命令来创建一个 Bish 目录：

```
bish new xxx
```

或者在一个目录下使用以下命令来将其初始化为 Bish 目录：

```
bish init xxx
```

Bish 目录结构如下：

```
my-server/
    ├─server/
    ├─mods/
    ├─backups/
    │    ├─snapshots/
    │    └─archives/
    ├─files/
    └─Bish.toml
```

- `server/` Minecraft 服务器的根目录
- `mods/` 离线 Mod 存放&读取位置
- `backups/` 备份
  - `snapshots` 快照（5个槽位，新的会将旧的替换）
  - `archives` 归档（无槽位限制）
- `files` 服务器共享文件（可以存放材质包、光影包等资源文件供下载）

使用以下命令启动服务器：

```
bish run
```

## 基本

Bish 的配置文件为 `Bish.toml`，其中定义了服务器相关的配置信息。最基本的，同时也是必须的选项如下：

- `name`：服务器的名称，一般与目录名相同，多个词之间以 `-` 隔开

- `version`：服务器使用的版本
- `loader`：服务器使用的 loader，可以是 `quilt` 或 `fabric`

初始的配置文件如下：

```toml
name = "server-name"
version = "1.20.4"
loader = "quilt"
```



```toml
jvm_options = ""
version = "1.20.4"
loader = "quilt"

[properties]
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



## Modrinth 相关

可以使用 `slug` 或 `id` 来唯一标识一个 Project，每个 Project 有多个 Version，

```mermaid
flowchart LR
A[配置文件中的 slug]
```

### config

```
[mods]
fabric-api = "0.94.0+1.20.4"
```

### cli

`ach mod install xxx` 来安装 mod

`ach mod update xxx` 来更新 mod

更新时会按照 mod 列表
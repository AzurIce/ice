## v0

有多个子命令：

- `mod`：结合 `mods.toml` 对 mod 进行管理
- `server`：结合 `ice.toml` 对服务器进行管理

### mod

- `ice mod init`：在当前目录初始化一个 `mods.toml`

- `ice mod sync`：将 mods 目录下的 mod 与 `mods.toml` 同步

    目前只做了，没有的下载，没有做多的删掉。

- `ice mod add <slug>`：下载符合 `loader` 和 `version` 的最新 mod，并添加到 `mods.toml` 中。
- `ice mod update`：更新 mod，并修改 `mods.toml`

### server

- `ice server init`：在当前目录初始化一个 `Ice.toml`

- `ice server install`：按照 `Ice.toml` 中设定的 `loader` 和 `version` 安装服务器。

- `ice server check`：

- `ice server run`：启动服务器

    ```mermaid
    flowchart
    p[Check server.roperties]
    p --> m
    m[Check mods]
    m --> e
    e[Check eula.txt]
    e --> r
    r[Run]
    ```

    





```
cargo run --manifest-path H:\ice\Cargo.toml -p ice-cli
```



- `ice mod`
    - `list`
    - `sync`

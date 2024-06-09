# Ice

A Minecraft Server Helper

重构ing...

---

# ACH

基于对 minecraft 服务端输入输出进行重定向实现的服务器 Helper，可以完成多个服务器备份管理等功能，并带有后台页面。

> AzurCraftHelper

## 构建

首先，需要 Go 语言的环境。

### 1. 安装 statik

本项目使用 statik 将静态文件打包入可执行文件中

```shell
go get github.com/rakyll/statik
go install github.com/rakyll/statik
```


### 2. 构建前端

```shell
git submodule update
cd assets
yarn
yarn build
```

### 3. 修改并运行 `build.bat`



## 使用

### 一、备份

在网页的 `Backups` 栏可以看到所有服务器的备份<font color="orange">（WIP）</font>。

#### 快照 与 归档

备份分为两种：**快照（snapshot）** 与 **归档（archive）**，前者有槽位限制，旧的快照会随着新的快照的创建而被替换，后者为永久存储。

##### 1. 快照

**快照保存目录**：`{backupDir}/snapshots`

**快照槽位机制**：有10个槽位，若槽位占满，会替换掉最老的一个槽位。

- **创建快照**：`#bksnap make[ <name>]`

  可以选择输入快照名称，会将 `world` 目录以 `<ServerName> - 2022-07-19 17-11-07[ <name>]` 为名称存储到 **快照保存目录**。

- **查看快照列表**：`#bksnap list`

  会列出所有的 **快照** 及其 **id**

- **加载快照**：`#bksnap load <id>`

  关闭服务器后，使用 **id** 为 `<id>` 的快照替换掉 `world` 目录，再启动服务器。

##### 2. 备份（旧的备份）

**备份保存目录**：`storedBackup/snapshots`

`#backup make[ <name>]` 同 `bksnap`

`#backup list[ <page>]` 显示第 `<page>` 页备份（不填则只显示第一页，一页10个）

`#backup load <id>`

##### 2. 归档<font color="orange">（WIP）</font>

归档目录

`#bkarch make <name>` 归档必须有名字。

`#bkarch list[ <page>]` 显示第 `<page>` 页快照（不填则只显示第一页，一页10个）

`#bkarch load <id>`

---

以下内容已经过时，是旧版 MCSH 的内容

- `command_prefix`

    This is what MCSH will look for while you enter something into it.

    If any input contains it at the beginning, it will be considered as a **MCSH Command**

    check more in **MCSH Commands**

- `servers`
    It contains the information of all your server need to be managed with ACH.
    For each server, the **key** should be a custom name for it.

  - `execOptions`
        e.g. `-Xms4G -Xmx4G`.
  - `execPath`
        The path to the `.jar` file of your server.
        >
        > - When doing bacnup jobs, MCSH will use the dir of this path to locate `world/` folder.
        > - Server will be using command `java execOptions -jar execPath --nogui` to start.

### IO for each server

- input:
    Write `xxx` to `serverName` with `serverName | xxx`.\
    Otherwise, MCSHGo will write the input to every server.
- output:
    It will present output from each server like this:

    ```
    YYYY-MM-DD HH:MM:SS [serverName/INFO]: ......
    YYYY-MM-DD HH:MM:SS [serverName/WARN]: ......
    ```

### MCSH Commands

A MCSH Commands is recognized with the prefix `#` for normal, you can configure it in `config.yml` .\

If you do `#xxx abc defgh ijk` \

`xxx abc defgh ijk` will be recognized as a **MCSH Command** \

> You can also use `serverName|#xxx` to run **MCSH Command** in specified server, or it will execute for every server.

- `backup [mode] [arg]`

  - enter ``(empty) as `[mode]`

  > Show the backup list, Not developed yet.

  - enter `make` as `[mode]`

  `arg` is `comment` , optional.\

  MCSH will copy your server's `serverRoot/world` to `Backups/` folder with a changed name in `servername - yyyy-mm-dd hh-mm-ss[ comment]` format

  - enter  `restore` as `[mode]`

  > stop the server, backup your server with comment `Restore2<backupName>` , Not developed yet.

- `run`
  - After you stoped some server, you can use this command to run it again.

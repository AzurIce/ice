# Draft

<!-- 专注于支持 Fabric/Quilt 服务端 -->

<!-- `fabric-server-launch.jar` -->
暂时先专注支持 quilt
`quilt-server-launch.jar`


## v1 `config.yml` 设计

### alpha12

```yaml
command_prefix: '#'
backup_dir: ./Backups
servers:
    test:
        execOptions: -Xms4G -Xmx4G
        execPath: D:/MCServer/server.jar
jwt_signing_string: dsqvre
```
### alpha13

```yaml
command_prefix: '#'
backup_dir: ./Backups
servers:
    test:
        dir: D:/MCServer # the folder to place exec jar
        version: 1.19.4 # automatically install, if empty, then use the latest version, and update this field
        # launcher: vanilla # or fabric/quilt, will automatic install
        jvmOptions: -Xms4G -Xmx4G
jwt_signing_string: dsqvre
```

## 服务器相关文件

- `banned-ips.json`
- `banned-players.json`
- `ops.json`
- `usercache.json`
- `whitelist.json`

- `mods/*.jar`
- `world/*`

- *shaderpacks*
- *resourcepacks*

## 配置文件

## 服务器版本管理

所有已发布的 Minecraft: Java Edition 的列表 `version_manifest_v2.json`
> https://minecraft.fandom.com/wiki/Version_manifest.json

API：`http://launchermeta.mojang.com/mc/game/version_manifest_v2.json`

可以获取到对应版本的 `<gameversion>.json`

其中便包含服务端文件的 url。


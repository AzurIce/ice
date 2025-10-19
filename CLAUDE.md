# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

Ice 是一个用 Rust 编写的 Minecraft 命令行助手工具，主要功能包括：
- Mod 管理（通过 Modrinth API）
- Minecraft 服务器管理
- 基于 Rhai 脚本的插件系统

项目从 MCSH -> ACH -> Ice 一路演进而来，目前版本为 0.1.0-alpha.40。

## 构建和开发命令

### 基础命令
```bash
# 构建项目
cargo build

# 运行项目
cargo run -- <COMMAND>

# 运行测试
cargo test

# 使用 cargo 安装
cargo install --git https://github.com/AzurIce/ice.git --locked
```

### 主要命令示例
```bash
# Modrinth 相关
ice modrinth init                  # 创建 mods.toml
ice modrinth sync                  # 同步 mod
ice modrinth update                # 更新 mod
ice modrinth add <slug>            # 添加 mod

# Server 相关
ice server init                    # 初始化服务器配置
ice server new <name>              # 创建新服务器
ice server install                 # 安装服务器
ice server run                     # 运行服务器
```

## 代码架构

### Workspace 结构
项目采用 Cargo workspace 架构，包含以下核心 package：

1. **ice (主包)** - 位于根目录
   - 入口：`src/bin/ice.rs`
   - CLI 定义：`src/bin/cli/mod.rs`
   - 命令实现：`src/bin/cli/modrinth.rs`、`src/bin/cli/server.rs`

2. **ice-core** - 核心功能包
   - 提供 `Loader` 枚举（Fabric/Quilt）
   - 处理服务器加载器的安装逻辑
   - 包含可选的 clap feature 以支持 CLI

3. **ice-server** - 服务器管理包
   - 服务器启动和管理
   - 插件系统（基于 Rhai）
   - 备份系统（快照和归档）
   - 内置命令：`#bksnap`、`#bkarch`
   - 内置插件：`here.rhai`、`rtext.rhai`、`scoreboard.rhai`

4. **ice-api-tool** - API 工具包
   - Modrinth API 封装
   - Fabric API 封装
   - Mojang API 封装

5. **ice-util** - 工具库
   - 文件系统工具（SHA1 哈希计算等）
   - Minecraft 相关工具（rtext 富文本）
   - 通用工具函数

### 关键设计模式

#### 异步运行时
- 项目已从 tokio 迁移到 smol 异步运行时
- 使用 `async-compat` 提供兼容性
- 主入口使用 `smol::block_on(Compat::new(...))`

#### 配置文件
- **mods.toml** - Mod 配置文件，包含 version、loader 和 mods 列表
- **Ice.toml** - 服务器配置文件，包含 server 配置、properties 和 mods

#### 插件系统
- 基于 Rhai 脚本语言
- 插件放置在 `plugins/` 目录
- 内置插件会在运行时自动复制到 plugins 目录
- 插件可以响应事件：`Event::ServerDown`、`Event::ServerDone`、`Event::PlayerMessage` 等
- 支持延迟调用和跨插件函数调用

#### Modrinth Mod 管理
- 使用 `slug` 作为 mod 的唯一标识（来自 Modrinth URL）
- 版本格式：`version_id#version_number`
- 通过 SHA1 哈希验证文件完整性
- 支持并发下载（使用 `futures::stream` 和 `buffer_unordered`）

## 重要约定

### 加载器兼容性
- Quilt 加载器允许使用 Fabric 的 mod
- 在查询 API 时，Quilt 会同时查询 Fabric 和 Quilt

### 日志和进度显示
- 使用 `tracing` + `tracing-indicatif` 进行日志和进度显示
- 支持 spinner 和层级日志
- 使用 `color-print` 提供彩色输出

### 备份系统
- **快照 (snapshot)**: 有 10 个槽位限制，自动替换最旧的
  - 保存路径：`backups/snapshots/`
- **归档 (archive)**: 永久存储，需要指定名称
  - 保存路径：`backups/archives/`

## 测试相关

- 测试目录位于各个 package 的 `src/` 中使用 `#[cfg(test)]`
- 部分测试需要实际的网络请求（Modrinth API）
- 服务器安装测试需要 Java 环境

## 注意事项

- 项目使用 Windows 路径风格（反斜杠），但 Rust 的 `Path` 会自动处理
- API 请求可能受 Modrinth API 速率限制影响
- 服务器安装需要下载 installer jar 并使用 Java 执行
- 插件系统 API 可能频繁变更（WIP 阶段）

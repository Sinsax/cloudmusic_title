# 项目简介

这是一个基于Rust的小工具，用于Linux系统下获取Wine版网易云音乐正在播放的歌曲名称，主要用于OBS直播场景。

## 主要特性

- 自动获取网易云音乐窗口标题
- 实时更新歌曲信息至文件
- 支持OBS文本源
- 低资源占用

## 依赖

- Linux系统

- Wine环境

- 网易云音乐(Wine版)

- xdotool

- Rust环境(源码编译需要)

## 安装

xdotool依赖

```bash
# Ubuntu/Debian
sudo apt-get install xdotool
# Arch Linux
sudo pacman -S xdotool
```

编译

```bash
cargo build --release
```

编译文件在``target\release``下



## 使用

1. 运行程序：`./cloudmusic_title`
2. 在OBS中添加文本源，选择生成的`title.txt`

## 注意

- 保持网易云音乐窗口开启
- 确保程序有写入权限

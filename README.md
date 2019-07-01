gfwlist-domains
------------
一个从 [gfwlist](https://github.com/gfwlist/gfwlist) 生成域名列表的辅助工具

### 安装

```
$ cargo install gfwlist-domains
```

### 用法

如果 `$HOME/.cargo/bin` 在你的 `$PATH` 环境变量里，即可执行：

```
$ gfwlist-domains
```

运行后，在 stdout 输出域名（每行一个域名），在 stderr 输出其它统计信息

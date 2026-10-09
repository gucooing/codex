# 维护 ccodex

本分支以官方 `rust-v0.162.0` / `c1382380de69521303b416720a52f42d51af6248` 为基线。原有 `main` 开发分支保留；持续维护的定制分支为 `ccodex`。

## 定制边界

- `config.toml` 顶层 `BASE_OAUTH_URL`；`ccodex -c 'BASE_OAUTH_URL="https://service.example/api/oauth/chatgpt"' login` 可临时覆盖。
- CLI 在读取凭据和创建网络客户端之前选择服务；共享 HTTP request draft、WebSocket connector、OAuth issuer 和后台配置使用同一服务根。进程内不切换服务，修改配置后重新启动。
- HTTP 方法、请求体、query、PKCE、state、client_id、JWT claim 命名空间、模型/工具元数据保持官方协议；内置第三方 provider/MCP 与普通网页地址不重写。
- 默认数据目录为 `.ccodex`，只有公开命令改为 `ccodex`；内部 crate、原生辅助程序名和协议标识维持原名，方便跟进官方。
- npm 使用官方的平台可选依赖布局，scope 为 `@gucooing`。原生包装保留 bwrap、ripgrep、zsh、Windows sandbox、code-mode host 等资源。未修改的语音运行库从同一官方稳定版下载，校验 `scripts/ccodex/upstream.json` 中固定 SHA-256 后仅复制 voice 子目录，保留其许可证；主 CLI 从本分支源码编译。

## 云端检查与发布

按用户要求，ccodex 不在本地编译，也不运行会编译 Rust 的测试、clippy 或 schema 生成。本地可执行源码审查、格式检查和 Python/JavaScript 静态打包检查；未执行或失败的检查须如实说明。不为获取云端 CI 验证而推送或新建分支。`.github/workflows/ccodex.yml` 在获准推送 `ccodex` 时执行六个平台的构建及打包，并上传 Actions artifacts。Windows x64 任务运行相关 Rust 测试、完整发布包的 daemon 安装回归测试及配置 schema 生成校验。普通分支推送不会发布 npm 或 GitHub Release。

首次发布前，在 GitHub 配置 `npm` environment，并配置 `NPM_TOKEN`（有 `@gucooing/ccodex` 发布权限），或者为该包配置 npm trusted publishing。这个包名及 scope 必须由维护者实际持有。

确认分支 CI 成功后，在工作区版本号对应的提交上创建并推送 tag：

```sh
git tag ccodex-v0.162.0+1
git push origin ccodex-v0.162.0+1
```

tag 的基础版本必须与 `codex-rs/Cargo.toml` 的 workspace version 一致；`+N` 只递增打包修订，不改变 CLI、npm 包或官方协议版本。流程先成功构建所有平台，然后发布平台包，最后发布 npm `latest` 和 GitHub Release。安装器、CLI 更新检查和 npm 更新命令使用本仓库/包，不使用官方 codex 更新源。未配置发布权限时构建产物仍可在 Actions 下载，不能声称已经发布到 npm。

没有配置 Homebrew tap、WinGet catalog 或 Apple 签名凭据；不要把这些渠道写成已发布。原生安装与 npm 均提供平台对应的构建。

## 跟进官方

1. 从 `openai/codex` 获取最新稳定版 tag，解析完整 commit，先审阅与当前基线的源码差异。
2. 在 `ccodex` 合并目标稳定版，保留本分支的最小定制；不要直接以 main 开发提交替代发布版。更新 `scripts/ccodex/upstream.json` 的 tag、commit 和官方资产摘要。
3. 检查 endpoint 调用链、配置、登录刷新/撤销、WS 及打包布局变化；本地只做不编译的检查，Rust 编译、测试和 schema 生成留待获准的构建环境执行。当前 release tag 的上游锁文件仍含 0.0.0，本分支的 Cargo.lock 仅同步继承 workspace version 的本地 crate 为 0.162.0；外部依赖采用该 release，包括固定 git rev 的 rmcp 3.3.0。
4. `ccodex` 验证后，再按 Codex2API 的 `docs/CODEX_UPDATES.md` 更新代理。代理的供应端协议常量仍来自官方 release，不能写成第三方服务地址。
5. 本地静态打包检查可运行 `python -m unittest discover -s scripts/ccodex -p 'test_*.py'`，它不会编译或发布。

服务端已有能力、尚未完成的文件/云插件等能力与在线验证边界，以 Codex2API 的接口审计和更新记录为准；可路由到第三方服务不代表该服务已经实现官方全部产品功能。

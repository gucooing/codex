# ccodex

支持第三方服务端的 Codex CLI，基于官方 **0.160.0**（`a956835d020762cb2b570053af06f643a11c0ecc`）。保留官方工具、权限、沙箱、登录及交互实现；定制项是服务地址、安装隔离、命令名和发布更新渠道。本项目由 gucooing 维护，不是 OpenAI 官方发行包。

## 安装与更新

首个 `ccodex-v*` Release 发布成功后，可使用 npm 安装或更新：

```sh
npm install -g @gucooing/ccodex@latest
ccodex
```

也支持 `pnpm add -g @gucooing/ccodex@latest`、`bun install -g @gucooing/ccodex@latest` 和 `npx @gucooing/ccodex`。

原生安装包提供 Linux、macOS、Windows 的 x64/ARM64 版本，位于 [Releases](https://github.com/gucooing/codex/releases)。无需 Node.js 的安装方式：

```sh
curl -fsSL https://raw.githubusercontent.com/gucooing/codex/ccodex/scripts/ccodex/install.sh | sh
```

```powershell
irm https://raw.githubusercontent.com/gucooing/codex/ccodex/scripts/ccodex/install.ps1 | iex
```

安装器核验 SHA-256，更新保留 ccodex 的配置及登录数据。它只安装 `ccodex`，不覆盖官方 `codex`。源码开发、发布配置和更新步骤见 [CCODEX.md](CCODEX.md)。

## 服务地址

配置文件默认为 `~/.ccodex/config.toml`（Windows：`%USERPROFILE%\.ccodex\config.toml`）。可通过 `CCODEX_HOME` 指定目录；显式的兼容 `CODEX_HOME` 覆盖仍受支持，勿将其指向官方数据目录。

```toml
BASE_OAUTH_URL = "https://oauth-ai.alsl.xyz/api/oauth/chatgpt"
```

未配置时使用上述地址。该地址需要部署 [Codex2API](https://github.com/gucooing/Codex2Api-rs) 后才能登录使用。修改地址后重启 ccodex。所有服务共用 ccodex 的登录数据，不按地址拆分凭据。

`BASE_OAUTH_URL` 是包含 `/api/oauth/chatgpt` 的服务根地址，不要填写 `/oauth/token` 或 `/responses`。OAuth、ChatGPT backend-api、OpenAI v1 和第一方 HTTP/WebSocket 请求从该地址派生；第三方 MCP、其他供应商、文档和软件分发地址继续使用原配置。

```sh
ccodex login
ccodex login --device-auth
ccodex login status
ccodex logout
```

浏览器登录保留官方 PKCE、state、本地回调与成功页流程；设备码登录由同一服务授权。服务不可用时不会回退官方认证服务器。服务端实际实现哪些云端能力，由 Codex2API 的接口与账户策略决定。

原版项目说明保存在 [README.upstream.md](README.upstream.md)，许可证为 [Apache-2.0](LICENSE)。

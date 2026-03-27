# 离线激活流程

本文档说明 `audio-courier` 当前的离线授权方案，包括用户如何激活、你如何签发许可证、应用如何校验，以及换机时怎么处理。

## 1. 方案概览

当前授权方案基于以下规则：

- 用户端生成设备请求码
- 你离线签发 `license.json`
- 应用启动时在 Rust 端校验许可证
- 校验失败时禁用聊天和录音等受控功能

许可证内容包含：

- `userId`
- `deviceFingerprint`
- `issuedAt`
- `expiresAt`
- `maxVersion`
- `features`
- `signature`

签名算法使用 `Ed25519`。

## 2. 用户激活流程

### 2.1 用户生成设备请求码

用户打开应用后：

1. 点击顶部 `许可证`
2. 在弹窗中填写 `用户标识`
3. 点击 `生成请求码`

应用会生成一段 JSON，并自动复制到剪贴板。

示例：

```json
{
  "appId": "audio-courier",
  "appVersion": "1.0.2",
  "userId": "customer_001",
  "deviceFingerprint": "a3f4...",
  "deviceHint": "DESKTOP-123456 / windows / x86_64",
  "requestTime": "2026-03-27T10:00:00Z"
}
```

用户把这段内容发给你。

### 2.2 你签发许可证

你收到设备请求码后，在本地运行签名工具生成 `license.json`。

### 2.3 用户导入许可证

你把签好的 `license.json` 内容发给用户后，用户：

1. 打开应用
2. 点击顶部 `许可证`
3. 把 `license.json` 内容粘贴到 `导入许可证` 输入框
4. 点击 `导入并激活`

如果校验通过，应用会显示 `许可证有效`，并开放受控功能。

## 3. 你如何生成密钥

先生成一对密钥：

```powershell
cargo run --manifest-path src-tauri/Cargo.toml --bin license_tool -- generate-keypair
```

输出示例：

```text
LICENSE_PRIVATE_KEY=...
LICENSE_PUBLIC_KEY=...
```

要求：

- `LICENSE_PRIVATE_KEY` 只保存在你自己手里，不能放进应用
- `LICENSE_PUBLIC_KEY` 放进应用运行环境，用于验签

建议把公钥写入 `src-tauri/.env`：

```env
LICENSE_PUBLIC_KEY=你的公钥
```

## 4. 你如何签发许可证

### 4.1 准备请求文件

把用户发来的设备请求码保存为本地文件，例如：

`activation_request.json`

### 4.2 运行签名命令

```powershell
cargo run --manifest-path src-tauri/Cargo.toml --bin license_tool -- sign --request activation_request.json --user-id customer_001 --expires-at 2027-03-27T23:59:59Z --max-version 1.9.99 --feature pro --output license.json
```

参数说明：

- `--request` 用户发来的设备请求文件
- `--user-id` 用户标识
- `--expires-at` 到期时间，必须是 UTC 时间
- `--max-version` 允许使用的最高版本
- `--feature` 授权功能，可重复传入多个
- `--output` 输出的许可证文件

如果你不想每次命令行传私钥，可以先设置环境变量：

```powershell
$env:LICENSE_PRIVATE_KEY="你的私钥"
```

然后再执行签发命令。

## 5. 许可证示例

```json
{
  "licenseId": "lic_xxxxx",
  "userId": "customer_001",
  "deviceFingerprint": "a3f4...",
  "issuedAt": "2026-03-27T10:10:00Z",
  "expiresAt": "2027-03-27T23:59:59Z",
  "maxVersion": "1.9.99",
  "features": ["pro"],
  "signature": "base64-signature"
}
```

## 6. 应用如何校验

应用在 Rust 端校验以下内容：

1. `license.json` 是否是合法 JSON
2. `signature` 是否能被内置公钥验证
3. 当前机器的 `deviceFingerprint` 是否与许可证一致
4. 当前时间是否超过 `expiresAt`
5. 当前版本是否超过 `maxVersion`

只要有一项失败，许可证状态就会变成无效。

常见失败原因：

- `未导入许可证`
- `许可证签名校验失败`
- `许可证绑定的设备与当前机器不匹配`
- `许可证已过期`
- `当前软件版本超出许可证授权范围`

## 7. 换机流程

换机本质上是重新签发。

处理步骤：

1. 用户在新机器上重新生成设备请求码
2. 用户把新请求码和原订单信息发给你
3. 你确认允许换机
4. 你重新签发新的 `license.json`
5. 用户在新机器导入新许可证

建议你自己维护一份台账，至少记录：

- `licenseId`
- `userId`
- 订单号
- 首次签发时间
- 到期时间
- 当前绑定设备摘要
- 换机次数

## 8. 发给用户的简版说明

你可以直接把下面这段发给用户：

```text
1. 打开软件，点击顶部“许可证”
2. 输入你的用户标识，点击“生成请求码”
3. 把生成的内容发给我
4. 收到我返回的许可证后，重新打开“许可证”
5. 把许可证内容粘贴到“导入许可证”
6. 点击“导入并激活”
```

## 9. 注意事项

- 私钥绝不能随软件一起分发
- 公钥可以内置在应用里
- 设备指纹是防普通复制，不是绝对防破解
- 纯离线授权只能提高倒卖和复制成本，不能彻底防破解
- 如果后续要支持“选择文件导入许可证”或“导出请求文件”，可以在当前基础上继续扩展

# 📦 dbg-logger

一个零依赖的 TypeScript/JavaScript 调试辅助工具，支持显示变量名、值、调用位置（文件/行号）、时间戳，并可灵活配置。

## ✨ 特性

* 自动格式化各种类型（对象、数组、Map、Set、Date、Error 等）
* 支持 `DBG.log(() => user)` 风格，自动提取变量名或表达式
* 支持相对路径输出（基于项目 `package.json` 所在目录）
* 可选显示时间戳、文件名、行号
* 支持标签、条件输出、批量输出
* 支持全局启用/禁用、默认配置

---

## 🚀 安装

```bash
npm install dbg-logger
# 或者
yarn add dbg-logger
# 或者
pnpm add dbg-logger
```

---

## 🛠 使用方法

### 1. 基本用法

```ts
import {dbg, DBG} from "dbg-logger";

const user = {id: 1, name: "Alice"};

dbg(user, "user");
// 输出: [src/index.ts:10] user = { "id": 1, "name": "Alice" }

DBG.log(() => user);
// 输出: [src/index.ts:11] user = { "id": 1, "name": "Alice" }
```

### 2. 批量调试

```ts
import {dbgMulti} from "dbg-logger";

const name = "Alice";
const age = 20;

dbgMulti({name, age});
// 输出: 
// [src/index.ts:5] name = "Alice"
// [src/index.ts:6] age = 20
```

### 3. 条件调试

```ts
import {dbgIf} from "dbg-logger";

const debugMode = true;
const count = 42;

dbgIf(debugMode, count, "count");
// 仅在 debugMode 为 true 时打印
```

### 4. 带标签调试

```ts
import {dbgWith} from "dbg-logger";

const token = "abcd1234";
dbgWith("auth", token, "token");
// 输出: [src/auth.ts:15] [auth] token = "abcd1234"
```

### 5. 配置默认选项

```ts
import {dbgConfig} from "dbg-logger";

dbgConfig.setDefaults({
    showTimestamp: true,
    isRelativePath: true,
});

dbgConfig.disable(); // 全局关闭调试
dbgConfig.enable();  // 重新启用
```

---

## ⚙️ 配置选项

```ts
interface DbgOptions {
    showLocation?: boolean; // 是否显示文件名和行号
    isRelativePath?: boolean; // 是否基于项目根目录
    showTimestamp?: boolean; // 是否显示时间戳
    label?: string;          // 自定义标签
    logger?: (msg: string) => void; // 自定义输出函数
    enabled?: boolean;       // 是否启用调试
    varName?: string;        // 强制指定变量名
}
```

---

## ⚠️ 注意事项

* **变量名推断**基于 `Function.toString()`，在生产环境被压缩时,变量名被混淆之后可能失效，建议传入 `varName` 明确指定。
* 文件路径相对化依赖 `package.json` 的位置，如果在子包/monorepo 使用，路径可能是相对当前子包。

---

## 📜 许可证

MIT License © 2025

 
import { dbg, DBG, dbgMulti } from "./index.js";

const name = "张三";
const age = 25;
const isStudent = true;
const score = null;
const extra = undefined;
const bigNum = 123456789012345678901234567890n;
const sym = Symbol("test");
const date = new Date();
const regex = /test/gi;
const error = new Error("测试错误");

console.log("--- 推荐使用方式 ---");
DBG.log(() => name);
DBG.log(() => age);
DBG.log(() => isStudent);
DBG.log(() => score);
DBG.log(() => extra);

const user = {
  name: "李四",
  age: 30,
  hobbies: ["读书", "游泳"],
  address: { city: "北京", district: "朝阳区" },
};
const numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
const map = new Map([
  ["key1", "value1"],
  ["key2", "value2"],
]);
const set = new Set([1, 2, 3, 4, 5]);

DBG.log(() => user);
DBG.log(() => numbers);
DBG.log(() => map);
DBG.log(() => set);
DBG.log(() => bigNum);
DBG.log(() => sym);
DBG.log(() => date);
DBG.log(() => regex);
DBG.log(() => error);

console.log("\n--- 多变量调试 ---");
DBG.log(() => ({ name, age, isStudent }));
DBG.log(() => [name, age, isStudent]);

console.log("\n--- 传统使用方式 ---");
dbg(name, "name");
dbg(user, "user");
dbgMulti({ name, age, user });

const testFunc = () => "hello world";
const asyncFunc = async () => await Promise.resolve(42);
DBG.log(() => testFunc);
DBG.log(() => asyncFunc);
const nested = {
  level1: {
    level2: {
      level3: {
        level4: {
          data: "深层嵌套数据",
        },
      },
    },
  },
};

DBG.log(() => nested);

const map2 = new Map([
  ["key1", "value1"],
  ["key2", "value2"],
  ["key3", "value2"],
  ["key4", "value2"],
  ["key5", "value2"],
  ["key6", "value2"],
  ["key7", "value2"],
  ["key8", "value2"],
  ["key9", "value2"],
]);
DBG.log(() => map2);
const objectArr = [
  1,
  2,
  3,
  4,
  5,
  6,
  7,
  8,
  9,
  [123, 123, 345],
  {
    name: "asefsef",
    age: 123,
  },
];
DBG.log(() => objectArr);

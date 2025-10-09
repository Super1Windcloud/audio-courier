import CryptoJS from "crypto-js";
import WebSocket from "ws";
import fs from "fs";
import dotenv from "dotenv";
dotenv.config({
  path: "../.env",
});

interface Config {
  hostUrl: string;
  host: string;
  appid: string;
  apiSecret: string;
  apiKey: string;
  file: string;
  uri: string;
  highWaterMark: number;
}

const config: Config = {
  hostUrl: "wss://iat-api.xfyun.cn/v2/iat",
  host: "iat-api.xfyun.cn",
  appid: "10787ef6", // 请替换成你的 APPID
  apiSecret: "N2QyNDBkMTAxYTBjNjVhYzZmOWZjOTky", // 请替换成你的 API Secret
  apiKey: "85e042b9ee54869eb38df6d4cf8cca76", // 请替换成你的 API Key
  file:  "../src-tauri/output.pcm",
  uri: "/v2/iat",
  highWaterMark: 1280,
};

// 帧定义
const FRAME = {
  STATUS_FIRST_FRAME: 0,
  STATUS_CONTINUE_FRAME: 1,
  STATUS_LAST_FRAME: 2,
};

// 获取当前时间 RFC1123 格式
let date = new Date().toUTCString();
// 设置当前临时状态为初始化
let status = FRAME.STATUS_FIRST_FRAME;
// 记录本次识别用 sid
let currentSid: string = "";
// 识别结果
let iatResult: any[] = [];

let wssUrl = `${config.hostUrl}?authorization=${getAuthStr(date)}&date=${date}&host=${config.host}`;
let ws = new WebSocket(wssUrl);

// 连接建立完毕，读取数据进行识别
ws.on("open", () => {
  console.log("WebSocket connected!");
  const readerStream = fs.createReadStream(config.file, {
    highWaterMark: config.highWaterMark,
  });

  readerStream.on("data", (chunk: Buffer) => {
    send(chunk);
  });

  // 最终帧发送结束
  readerStream.on("end", () => {
    status = FRAME.STATUS_LAST_FRAME;
    send(Buffer.from(""));
  });
});

// 得到识别结果后进行处理，仅供参考，具体业务具体对待
ws.on("message", (data: string, err: Error) => {
  if (err) {
    console.error(`Error: ${err}`);
    return;
  }

  const res = JSON.parse(data);
  if (res.code !== 0) {
    console.error(`Error code ${res.code}, reason: ${res.message}`);
    return;
  }

  let str = "";
  if (res.data.status === 2) {
    // 数据全部返回完毕，可以关闭连接，释放资源
    str += "Final recognition result: ";
    currentSid = res.sid;
    ws.close();
  } else {
    str += "Interim recognition result: ";
  }

  iatResult[res.data.result.sn] = res.data.result;
  if (res.data.result.pgs === "rpl") {
    res.data.result.rg.forEach((i: any) => {
      iatResult[i] = null;
    });
    str += "【Dynamic Correction】";
  }

  str += ": ";
  iatResult.forEach((i: any) => {
    if (i != null) {
      i.ws.forEach((j: any) => {
        j.cw.forEach((k: any) => {
          str += k.w;
        });
      });
    }
  });
  console.log(str);
});

// 资源释放
ws.on("close", () => {
  console.log(`Recognition SID: ${currentSid}`);
  console.log("Connection closed!");
});

// 建立连接错误
ws.on("error", (err: Error) => {
  console.error("WebSocket connection error: " + err);
});

// 鉴权签名
function getAuthStr(date: string): string {
  let signatureOrigin = `host: ${config.host}\ndate: ${date}\nGET ${config.uri} HTTP/1.1`;
  let signatureSha = CryptoJS.HmacSHA256(signatureOrigin, config.apiSecret);
  let signature = CryptoJS.enc.Base64.stringify(signatureSha);
  let authorizationOrigin = `api_key="${config.apiKey}", algorithm="hmac-sha256", headers="host date request-line", signature="${signature}"`;
  let authStr = CryptoJS.enc.Base64.stringify(
    CryptoJS.enc.Utf8.parse(authorizationOrigin),
  );
  return authStr;
}

// 传输数据
function send(data: Buffer | string) {
  let frame: any = "";
  let frameDataSection = {
    status: status,
    format: "audio/L16;rate=16000",
    audio: data.toString("base64"),
    encoding: "raw",
  };

  switch (status) {
    case FRAME.STATUS_FIRST_FRAME:
      frame = {
        // 填充 common
        common: {
          app_id: config.appid,
        },
        // 填充 business
        business: {
          language: "zh_cn",
          domain: "iat",
          accent: "mandarin",
          dwa: "wpgs", // 可选参数，动态修正
        },
        // 填充 data
        data: frameDataSection,
      };
      status = FRAME.STATUS_CONTINUE_FRAME;
      break;
    case FRAME.STATUS_CONTINUE_FRAME:
    case FRAME.STATUS_LAST_FRAME:
      // 填充 frame
      frame = {
        data: frameDataSection,
      };
      break;
  }
  ws.send(JSON.stringify(frame));
}

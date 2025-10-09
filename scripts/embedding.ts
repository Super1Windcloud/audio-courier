import axios from "axios";

const API_KEY = "";

const client = axios.create({
  baseURL: "https://api.siliconflow.cn/v1",
  headers: {
    Authorization: `Bearer ${API_KEY}`,
    "Content-Type": "application/json",
  },
});

async function embeddings() {
  try {
    const response = await client.post("/embeddings", {
      model: "BAAI/bge-large-zh-v1.5",
      input:
        "Silicon flow embedding online: fast, affordable, and high-quality embedding services. come try it out! 草死你妈的比",
    });

    const data = response.data.data[0].embedding;

    console.log("Embeddings result:", JSON.stringify(data, null, 2));
    console.log("Embeddings  lens:", data.length);
  } catch (err) {
    console.error("Embeddings error:", err);
  }
}

async function main() {
  await embeddings();
}

main();

import dotenv from "dotenv";
import fs from "fs";
import OpenAI from "openai";

dotenv.config({
	path: "../.env",
});

const openai = new OpenAI({
	apiKey: process.env.OPENAI_DAICHONG,
	baseURL: "https://api.laozhang.ai/v1",
});

// @ts-ignore
async function _getSTTResult() {
	const transcription = await openai.audio.transcriptions.create({
		file: fs.createReadStream("../src-tauri/recorded.wav"),
		model: "gpt-4o-transcribe",
	});

	console.log(transcription.text);
}

async function getChatResult() {
	const response = await openai.responses.create({
		model: "gpt-4",
		input: "gpt4 大概什么时候发布的",
	});

	console.log(response.output_text);
}

async function createFile(filePath: string) {
	const fileContent = fs.createReadStream(filePath);
	const result = await openai.files.create({
		file: fileContent,
		purpose: "vision",
	});
	return result.id;
}

// @ts-ignore
async function _getImageResult() {
	const fileId = await createFile("../img.png");

	const response = await openai.responses.create({
		model: "gpt-4.1-mini",
		input: [
			{
				role: "user",
				content: [
					{ type: "input_text", text: "what's in this image?" },
					// @ts-ignore
					{
						type: "input_image",
						image_url: fileId,
					},
				],
			},
		],
	});

	console.log(response.output_text);
}

getChatResult();

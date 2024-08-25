import Database from '@tauri-apps/plugin-sql';
import { fetch } from '@tauri-apps/plugin-http';
import { invoke } from '@tauri-apps/api/core';
//格式话小说文本
function formatNovelText(text: string): string {
    const regex = new RegExp(`([。])([”】]?)`, 'g');
    const newText = text.replace(regex, (match) => {
        return match + '\n';
    });
    return newText;
}

async function fetchNovels(): Promise<Novel[]> {
    try {
        const db = await Database.load('sqlite:bintool.db');
        const result = await db.select('SELECT * FROM novels') as INovel[]
        const novels = result.map(data => new Novel(data.content, data.audioSrc));
        return novels;
    } catch (error) {
        throw error;
    }
}

interface INovel {
    content: string;
    audioSrc: string;
    loading: boolean;
    printDetails(): void;
    generateAudio(): Promise<void>;
}

class Novel implements INovel {
    constructor(
        public content: string = '',
        public audioSrc: string = '未生成',
        public loading: boolean = false
    ) { }

    printDetails(): void {
        console.log(`Content: ${this.content}, Audio Source: ${this.audioSrc}`);
    }

    async generateAudio(): Promise<void> {
        this.loading = true;
        try {
            const response = await fetch("http://127.0.0.1:9880/", {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    text: this.content,
                    text_language: 'zh'
                })
            });
            const audioBlob = await response.blob();
            const audioData = Array.from(new Uint8Array(await audioBlob.arrayBuffer()));
            const audioName = `audio_${Date.now()}.wav`;
            this.audioSrc = await invoke("save_novel_audio", { audioData, audioName }) as string;
        } catch (error) {
            throw error;
        } finally {
            this.loading = false;
        }
    }
}

export { Novel, formatNovelText, fetchNovels };
export type { INovel };

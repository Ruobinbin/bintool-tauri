import { invoke } from "@tauri-apps/api/core";
import { ElMessage } from "element-plus";

interface IThumbnail {
    url: string;
    height: number;
    width: number;
}

interface IVideo {
    id: string;
    url: string;
    duration: number;
    thumbnails: IThumbnail[];
}

class Video implements IVideo {
    public id: string;
    public url: string;
    public duration: number;
    public thumbnails: IThumbnail[];

    constructor(id: string, url: string, duration: number | string, thumbnails: IThumbnail[]) {
        this.id = id;
        this.url = url;
        this.duration = this.validateDuration(duration);
        this.thumbnails = thumbnails;
    }

    public validateDuration(value: number | string): number {
        const num = typeof value === 'number' ? value : Number(value);
        return !isNaN(num) ? num : -1;
    }

    public getLargestThumbnailUrl(): string {
        if (this.thumbnails.length === 0) {
            return 'https://via.placeholder.com/600x400.png?text=No+Image';
        }

        const largestThumbnail = this.thumbnails.reduce((largest, current) => {
            const largestArea = largest.height * largest.width;
            const currentArea = current.height * current.width;
            return currentArea > largestArea ? current : largest;
        });

        return largestThumbnail.url;
    }

    public async downloadVideo(path: string): Promise<void> {
        const outputFilePath = `${path}/${this.id}.mp4`;

        const cmd = [
            this.url,
            '-f', 'bestvideo',
            '-o', outputFilePath
        ];

        try {
            await invoke<string>('run_yt_dlp_cmd', { cmd });
            ElMessage({
                message: '视频下载成功',
                type: 'success',
            });
        } catch (err) {
            ElMessage({
                message: `操作失败: ${err as string}`,
                type: 'error',
            });
        }
    }
}

export { Video };
export type { IVideo, IThumbnail };
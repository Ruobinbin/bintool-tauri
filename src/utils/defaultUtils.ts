// ISO 8601 转 秒
const ISO8601ToSeconds = (duration: string): number => {
    const match = duration.match(/PT(\d+H)?(\d+M)?(\d+S)?/);
    if (match) {
        const hours = (parseInt(match[1]) || 0);
        const minutes = (parseInt(match[2]) || 0);
        const seconds = (parseInt(match[3]) || 0);

        return hours * 3600 + minutes * 60 + seconds;
    } else {
        return 0;
    }
};

// 去除HTML标签
const stripHtmlTags = (html: string): string => {
    const div = document.createElement("div");
    div.innerHTML = html;
    return div.textContent || div.innerText || "";
};

// 获取文件名
function getFileNameFromPath(path: string): string {
    const match = path.match(/[^/\\]+$/);
    return match ? match[0] : '';
}

export { ISO8601ToSeconds, stripHtmlTags, getFileNameFromPath };
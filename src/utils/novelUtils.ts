//格式话小说文本
function formatNovelText(text: string): string {
    const regex = new RegExp(`([。])([”】]?)`, 'g');
    const newText = text.replace(regex, (match) => {
        return match + '\n';
    });
    return newText;
}

export { formatNovelText };
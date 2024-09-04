
export default function timeFormat(secs: number): string {
    const hours = Math.floor(secs / 3600);
    const minutes = Math.floor((secs % 3600) / 60);
    const seconds = secs % 60;
    let result = ``;
    if(hours > 0) result += `${hours}h`;
    if(minutes > 0) result += `${minutes}m`;

    result += `${seconds}s`;

    return result;
}

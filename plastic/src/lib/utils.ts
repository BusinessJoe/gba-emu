export function runPeriodically(callback: () => void, frameRate: number): number {
    let id = setInterval(callback, frameRate / 60);
    return id;
}

export function clearRunPeriodically(id: number): void {
    clearInterval(id)
}

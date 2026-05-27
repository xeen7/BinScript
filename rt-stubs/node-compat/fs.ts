export function readFileSync(path: string): string {
    return fs_read_file_sync(path);
}

export function writeFileSync(path: string, data: string): void {
    fs_write_file_sync(path, data);
}

export function existsSync(path: string): boolean {
    return fs_exists_sync(path);
}

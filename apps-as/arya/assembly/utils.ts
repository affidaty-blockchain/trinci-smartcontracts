export function arrayBufferToHexString(ab: ArrayBuffer): string {
    let result: string = '';
    let dataView = new DataView(ab);
    for (let i = 0; i < dataView.byteLength; i++) {
        let byteStr = dataView.getUint8(i).toString(16);
        for (let i = 0; i < 2 - byteStr.length; i++) {
            byteStr = '0' + byteStr;
        }
        result += byteStr;
    }
    return result;
}


export function arrayToBase64(arr: number[]): string {
  return btoa(String.fromCharCode.apply(null, arr));
}
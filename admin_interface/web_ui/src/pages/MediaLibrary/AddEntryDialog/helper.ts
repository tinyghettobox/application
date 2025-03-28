import Compressor from 'compressorjs';

export async function cropImage(image: File | Blob, width: number, height: number, quality: number): Promise<number[]> {
  return new Promise((resolve, reject) => {
    new Compressor(image, {
      quality,
      width,
      height,
      resize: 'cover',
      mimeType: 'image/jpeg',
      async success(file: File | Blob) {
        resolve(Array.from(new Uint8Array(await file.arrayBuffer())));
      },
      error(error: Error) {
        reject(error);
      }
    });
  });
}

export async function resizeImage(image: File | Blob, x: number, y: number, width: number, height: number): Promise<number[]> {
  return new Promise((resolve, reject) => {
    new Compressor(image, {
      width,
      height,
      resize: 'cover',
      mimeType: 'image/jpeg',
      async success(file: File | Blob) {
        resolve(Array.from(new Uint8Array(await file.arrayBuffer())));
      },
      error(error: Error) {
        reject(error);
      }
    });
  });
}

export async function fileToBinary(file: File): Promise<number[]> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      if (reader.result instanceof ArrayBuffer) {
        resolve(Array.from(new Uint8Array(reader.result)));
      } else {
        reject(new Error('Unexpected reader result'));
      }
    };
    reader.onerror = () => {
      reject(new Error('Error reading file'));
    };
    reader.readAsArrayBuffer(file);
  });
}

import sharp from 'sharp';
import fs from 'fs';
import path from 'path';

const src = 'd:/code/code-vs/study-reminder/frontend/public/floating-icon.jpg';
const dir = 'd:/code/code-vs/study-reminder/src-tauri/icons';

async function main() {
    // 生成 32x32
    await sharp(src).resize(32, 32, { fit: 'cover' }).png().toFile(path.join(dir, '32x32.png'));
    console.log('32x32.png OK');

    // 生成 128x128
    await sharp(src).resize(128, 128, { fit: 'cover' }).png().toFile(path.join(dir, '128x128.png'));
    console.log('128x128.png OK');

    // 生成 256x256
    await sharp(src).resize(256, 256, { fit: 'cover' }).png().toFile(path.join(dir, '128x128@2x.png'));
    console.log('128x128@2x.png OK');

    // 生成 icon.ico
    const p32 = await sharp(src).resize(32, 32, { fit: 'cover' }).png().toBuffer();
    const h = Buffer.alloc(6);
    h.writeUInt16LE(0, 0);
    h.writeUInt16LE(1, 2);
    h.writeUInt16LE(1, 4);
    const e = Buffer.alloc(16);
    e.writeUInt8(32, 0);
    e.writeUInt8(32, 1);
    e.writeUInt8(0, 2);
    e.writeUInt8(0, 3);
    e.writeUInt16LE(1, 4);
    e.writeUInt16LE(32, 6);
    e.writeUInt32LE(p32.length, 8);
    e.writeUInt32LE(22, 12);
    fs.writeFileSync(path.join(dir, 'icon.ico'), Buffer.concat([h, e, p32]));
    console.log('icon.ico OK');

    // 生成 icon.icns
    const p128 = await sharp(src).resize(128, 128, { fit: 'cover' }).png().toBuffer();
    fs.writeFileSync(path.join(dir, 'icon.icns'), p128);
    console.log('icon.icns OK');

    console.log('全部完成！');
}

main().catch(e => { console.error(e); process.exit(1); });

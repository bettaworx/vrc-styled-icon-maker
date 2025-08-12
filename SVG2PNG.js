const fs = require('fs');
const path = require('path');
const sharp = require('sharp');

const args = process.argv.slice(2);
if (args.length !== 2) {
  console.log('usage: node ./SVG2PNG.js <Input> <Output>');
  process.exit(1);
}

const inputDir = args[0];
const outputDir = args[1];

if (!fs.existsSync(inputDir)) {
  console.error('[\tNG\t]\tInput directory not found', inputDir);
  process.exit(1);
}

if (!fs.existsSync(outputDir)) {
  fs.mkdirSync(outputDir, { recursive: true });
}

fs.readdir(inputDir, (err, files) => {
  if (err) {
    console.error('[\tNG\t]\tFailed to read', err);
    return;
  }

  const svgFiles = files.filter(file => path.extname(file).toLowerCase() === '.svg');

  svgFiles.forEach(file => {
    const inputPath = path.join(inputDir, file);
    const outputFileName = path.basename(file, '.svg') + '.png';
    const outputPath = path.join(outputDir, outputFileName);

    const sizeX = 192;
    const sizeY = 192;
    const extendX = 32;
    const extendY = 32;

    sharp(inputPath)
      .resize({
        width: sizeX,
        height: sizeY,
        fit: 'contain',
        background: { r: 0, g: 0, b: 0, alpha: 0 }
      })
      .extend({
        top: extendY,
        bottom: extendY,
        left: extendX,
        right: extendX,
        background: { r: 0, g: 0, b: 0, alpha: 0 }
      })
      .png()
      .toFile(outputPath)
      .then(() => {
        console.log(`[\tOK\t]\t${file} => ${outputFileName} (X:${sizeX + (extendX * 2)}, Y:${sizeY + (extendY * 2)})`);
      })
      .catch(err => {
        console.error(`[\tNG\t]\t${file} => X : `, err);
      });
  });
});

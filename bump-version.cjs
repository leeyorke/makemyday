const fs = require('fs');
const path = require('path');

const version = process.argv[2];

if (!version) {
  console.error('Usage: node bump-version.cjs <version>');
  process.exit(1);
}

// 更新 package.json
const packagePath = path.join(__dirname, 'package.json');
const packageJson = JSON.parse(fs.readFileSync(packagePath, 'utf8'));
packageJson.version = version;
fs.writeFileSync(packagePath, JSON.stringify(packageJson, null, 2) + '\n');
console.log(`✓ Updated package.json to ${version}`);

// 更新 src-tauri/Cargo.toml
const cargoPath = path.join(__dirname, 'src-tauri', 'Cargo.toml');
let cargoToml = fs.readFileSync(cargoPath, 'utf8');
cargoToml = cargoToml.replace(/version = "[^"]*"/, `version = "${version}"`);
fs.writeFileSync(cargoPath, cargoToml);
console.log(`✓ Updated src-tauri/Cargo.toml to ${version}`);

// 更新 src-tauri/tauri.conf.json
const tauriConfPath = path.join(__dirname, 'src-tauri', 'tauri.conf.json');
const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'));
tauriConf.version = version;
fs.writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + '\n');
console.log(`✓ Updated src-tauri/tauri.conf.json to ${version}`);

console.log('\n✓ All versions updated to', version);
#!/usr/bin/env node

/**
 * AnyDoc Tag Release 自动化发版脚本
 * 
 * 功能:
 * 1. 自动同步 package.json、src-tauri/Cargo.toml、src-tauri/tauri.conf.json 的版本号
 * 2. 自动运行 cargo test 与前端构建测试
 * 3. 自动生成规范提交并打上 Git Tag (vX.Y.Z)
 * 4. 输出触发 GitHub Actions 自动发版的推送命令
 */

import fs from "fs";
import path from "path";
import { execSync } from "child_process";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, "..");

const pkgPath = path.join(rootDir, "package.json");
const cargoPath = path.join(rootDir, "src-tauri", "Cargo.toml");
const tauriConfPath = path.join(rootDir, "src-tauri", "tauri.conf.json");

function run(cmd, cwd = rootDir) {
  console.log(`\x1b[36m> ${cmd}\x1b[0m`);
  execSync(cmd, { cwd, stdio: "inherit" });
}

function parseSemver(ver) {
  const match = ver.match(/^(\d+)\.(\d+)\.(\d+)(?:-(.+))?$/);
  if (!match) return null;
  return {
    major: parseInt(match[1], 10),
    minor: parseInt(match[2], 10),
    patch: parseInt(match[3], 10),
    prerelease: match[4] || null,
  };
}

function getNextVersion(current, bumpType) {
  const parsed = parseSemver(current);
  if (!parsed) {
    throw new Error(`当前版本号格式不规范: ${current}`);
  }
  if (bumpType === "patch") {
    return `${parsed.major}.${parsed.minor}.${parsed.patch + 1}`;
  }
  if (bumpType === "minor") {
    return `${parsed.major}.${parsed.minor + 1}.0`;
  }
  if (bumpType === "major") {
    return `${parsed.major + 1}.0.0`;
  }
  // 用户直接传入了指定的新版本号
  if (parseSemver(bumpType)) {
    return bumpType;
  }
  throw new Error(`未知版本升级类型: ${bumpType} (可用: patch, minor, major 或指定如 1.0.0)`);
}

function updateVersions(newVersion) {
  console.log(`\n\x1b[34m[1/3] 同步更新项目版本号为 v${newVersion} ...\x1b[0m`);

  // 1. package.json
  const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf-8"));
  pkg.version = newVersion;
  fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\n");
  console.log(`✓ 已更新 package.json -> ${newVersion}`);

  // 2. Cargo.toml
  let cargoContent = fs.readFileSync(cargoPath, "utf-8");
  cargoContent = cargoContent.replace(
    /(\[package\][\s\S]*?version\s*=\s*)"[^"]+"/,
    `$1"${newVersion}"`
  );
  fs.writeFileSync(cargoPath, cargoContent);
  console.log(`✓ 已更新 src-tauri/Cargo.toml -> ${newVersion}`);

  // 3. tauri.conf.json
  const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, "utf-8"));
  tauriConf.version = newVersion;
  fs.writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + "\n");
  console.log(`✓ 已更新 src-tauri/tauri.conf.json -> ${newVersion}`);
}

function runVerification() {
  console.log(`\n\x1b[34m[2/3] 执行发版前健康检查 (Cargo 单元测试 & 前端打包) ...\x1b[0m`);
  run("cargo test", path.join(rootDir, "src-tauri"));
  run("pnpm run build", rootDir);
  console.log(`✓ 所有测试与前端构建全部通过！`);
}

function handleGit(newVersion) {
  console.log(`\n\x1b[34m[3/3] 检查 Git 仓库并创建 Release Tag ...\x1b[0m`);
  const tag = `v${newVersion}`;
  let isGit = false;
  try {
    execSync("git rev-parse --is-inside-work-tree", { stdio: "ignore", cwd: rootDir });
    isGit = true;
  } catch {
    isGit = false;
  }

  if (!isGit) {
    console.log(`\x1b[33m提示: 当前目录尚未初始化为 Git 仓库。\x1b[0m`);
    console.log(`若要将代码发布到 GitHub，可依次执行:`);
    console.log(`  git init`);
    console.log(`  git add .`);
    console.log(`  git commit -m "chore(release): ${tag}"`);
    console.log(`  git tag -a ${tag} -m "Release ${tag}"`);
    console.log(`  git remote add origin <你的GitHub仓库地址>`);
    console.log(`  git push -u origin main --tags`);
    return;
  }

  // Git 操作
  run("git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json", rootDir);
  run(`git commit -m "chore(release): ${tag}"`, rootDir);
  run(`git tag -a ${tag} -m "Release ${tag}"`, rootDir);

  console.log(`\n\x1b[32m====================================================\x1b[0m`);
  console.log(`\x1b[32m🎉 成功创建本地发布 Tag: ${tag}\x1b[0m`);
  console.log(`\x1b[32m====================================================\x1b[0m\n`);
  console.log(`执行以下命令推送到 GitHub，即可自动触发跨平台发版流水线：\n`);
  console.log(`  \x1b[36mgit push origin HEAD\x1b[0m`);
  console.log(`  \x1b[36mgit push origin ${tag}\x1b[0m\n`);
}

function main() {
  const arg = process.argv[2] || "patch";
  if (arg === "-h" || arg === "--help") {
    console.log(`\nAnyDoc Tag Release 工具`);
    console.log(`用法: node scripts/release.js [patch | minor | major | <指定版本号>]`);
    console.log(`示例:`);
    console.log(`  node scripts/release.js patch    # 升级补丁版本 (例如 0.1.0 -> 0.1.1)`);
    console.log(`  node scripts/release.js minor    # 升级次版本号 (例如 0.1.0 -> 0.2.0)`);
    console.log(`  node scripts/release.js 1.0.0    # 直接设定版本为 1.0.0\n`);
    process.exit(0);
  }

  const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf-8"));
  const currentVersion = pkg.version || "0.1.0";
  const newVersion = getNextVersion(currentVersion, arg);

  console.log(`\x1b[35m=== AnyDoc 自动化发版流程 ===\x1b[0m`);
  console.log(`当前版本: ${currentVersion} -> 目标版本: ${newVersion}`);

  updateVersions(newVersion);
  runVerification();
  handleGit(newVersion);
}

main();

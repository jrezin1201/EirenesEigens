#!/usr/bin/env node
import { promises as fs } from 'fs';
import path from 'path';

const repoRoot = process.cwd();
const markdownFiles = [];
const IGNORE_DIRS = new Set(['node_modules', 'target', 'dist', '.git', 'archive']);

function shouldSkipDir(dirPath) {
  return dirPath.includes(`${path.sep}_archive${path.sep}`) || dirPath.endsWith(`${path.sep}_archive`) || dirPath.includes(`${path.sep}archive${path.sep}`) || IGNORE_DIRS.has(path.basename(dirPath));
}

async function walk(dir) {
  if (shouldSkipDir(dir) && dir !== repoRoot) {
    return;
  }
  const entries = await fs.readdir(dir, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (IGNORE_DIRS.has(entry.name) || entry.name.startsWith('.')) {
        continue;
      }
      if (shouldSkipDir(fullPath)) {
        continue;
      }
      await walk(fullPath);
    } else if (entry.isFile() && entry.name.endsWith('.md')) {
      markdownFiles.push(fullPath);
    }
  }
}

const brokenLinks = [];

function extractLinks(markdown) {
  const linkRegex = /\[[^\]]+\]\(([^)]+)\)/g;
  const links = [];
  let match;
  while ((match = linkRegex.exec(markdown)) !== null) {
    links.push(match[1]);
  }
  return links;
}

async function checkFile(filePath) {
  const text = await fs.readFile(filePath, 'utf8');
  const links = extractLinks(text);
  for (const link of links) {
    if (link.startsWith('http://') || link.startsWith('https://') || link.startsWith('mailto:')) {
      continue;
    }
    const [targetPath] = link.split('#');
    if (!targetPath) {
      continue;
    }
    const baseDir = path.dirname(filePath);
    const resolved = path.resolve(baseDir, targetPath);
    try {
      await fs.access(resolved);
    } catch (err) {
      brokenLinks.push({ filePath, link });
    }
  }
}

(async () => {
  await walk(repoRoot);
  for (const file of markdownFiles) {
    await checkFile(file);
  }
  if (brokenLinks.length) {
    console.error('Broken links found:');
    for (const issue of brokenLinks) {
      console.error(`- ${path.relative(repoRoot, issue.filePath)} → ${issue.link}`);
    }
    process.exit(1);
  } else {
    console.log(`Checked ${markdownFiles.length} markdown files. No broken local links.`);
  }
})();

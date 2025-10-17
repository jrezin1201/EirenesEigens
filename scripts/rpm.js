#!/usr/bin/env node

/**
 * RavensOne Package Manager (RPM)
 *
 * A lightweight package manager for RavensOne modules
 * Inspired by npm, cargo, and pip but optimized for RavensOne
 */

const fs = require('fs');
const path = require('path');
const https = require('https');
const http = require('http');

const VERSION = '1.0.0';
const REGISTRY_DIR = path.join(process.cwd(), '.ravens');
const PACKAGES_DIR = path.join(REGISTRY_DIR, 'packages');
const CACHE_DIR = path.join(REGISTRY_DIR, 'cache');
const LOCK_FILE = 'raven.lock';
const MANIFEST_FILE = 'raven.json';

class PackageManager {
    constructor() {
        this.manifest = null;
        this.lockFile = null;
        this.installed = new Map();
    }

    /**
     * Initialize new RavensOne project
     */
    async init(options = {}) {
        console.log('🎯 Initializing RavensOne project...\n');

        // Check if already initialized
        if (fs.existsSync(MANIFEST_FILE)) {
            console.log('⚠️  Project already initialized (raven.json exists)');
            return;
        }

        // Prompt for project details
        const name = options.name || path.basename(process.cwd());
        const version = options.version || '0.1.0';
        const description = options.description || '';
        const author = options.author || '';

        const manifest = {
            name,
            version,
            description,
            author,
            license: 'MIT',
            main: 'src/main.raven',
            dependencies: {},
            devDependencies: {},
            scripts: {
                dev: 'raven dev',
                build: 'raven build',
                test: 'raven test'
            },
            ravens: {
                version: '>=0.1.0'
            }
        };

        // Write manifest
        fs.writeFileSync(MANIFEST_FILE, JSON.stringify(manifest, null, 2));

        // Create directory structure
        this._createDirectories([
            'src',
            'tests',
            'examples',
            '.ravens'
        ]);

        // Create example main file
        const exampleCode = `// ${name} - Main entry point

fn main() {
    console.log("Hello from RavensOne! 🎯");
}
`;
        fs.writeFileSync('src/main.raven', exampleCode);

        // Create .gitignore
        const gitignore = `.ravens/
*.wasm
node_modules/
dist/
.DS_Store
`;
        fs.writeFileSync('.gitignore', gitignore);

        console.log('✅ Project initialized!\n');
        console.log('📦 Package:', name);
        console.log('📝 Version:', version);
        console.log('📂 Structure:');
        console.log('   src/main.raven');
        console.log('   tests/');
        console.log('   examples/');
        console.log('   raven.json\n');
        console.log('🚀 Next steps:');
        console.log('   raven install <package>  - Install a package');
        console.log('   raven dev                - Start development server');
        console.log('   raven build              - Build for production\n');
    }

    /**
     * Install package(s)
     */
    async install(packageName, options = {}) {
        this._ensureDirectories();
        this._loadManifest();

        if (!packageName) {
            // Install all dependencies from manifest
            console.log('📦 Installing dependencies...\n');
            await this._installAll();
        } else {
            // Install specific package
            console.log(`📦 Installing ${packageName}...\n`);
            await this._installPackage(packageName, options);
        }

        this._writeLockFile();
        console.log('\n✅ Installation complete!');
    }

    /**
     * Uninstall package
     */
    async uninstall(packageName) {
        this._loadManifest();

        console.log(`🗑️  Uninstalling ${packageName}...\n`);

        // Remove from manifest
        const wasDep = delete this.manifest.dependencies[packageName];
        const wasDevDep = delete this.manifest.devDependencies[packageName];

        if (!wasDep && !wasDevDep) {
            console.log(`⚠️  Package ${packageName} is not installed`);
            return;
        }

        // Remove from file system
        const packageDir = path.join(PACKAGES_DIR, packageName);
        if (fs.existsSync(packageDir)) {
            fs.rmSync(packageDir, { recursive: true, force: true });
        }

        // Update manifest
        fs.writeFileSync(MANIFEST_FILE, JSON.stringify(this.manifest, null, 2));

        console.log(`✅ Uninstalled ${packageName}`);
    }

    /**
     * List installed packages
     */
    async list() {
        this._loadManifest();

        console.log('📦 Installed packages:\n');

        if (Object.keys(this.manifest.dependencies).length === 0 &&
            Object.keys(this.manifest.devDependencies).length === 0) {
            console.log('   (none)\n');
            return;
        }

        if (Object.keys(this.manifest.dependencies).length > 0) {
            console.log('Dependencies:');
            for (const [name, version] of Object.entries(this.manifest.dependencies)) {
                const installed = this._isInstalled(name);
                const status = installed ? '✓' : '✗';
                console.log(`   ${status} ${name}@${version}`);
            }
            console.log();
        }

        if (Object.keys(this.manifest.devDependencies).length > 0) {
            console.log('Dev Dependencies:');
            for (const [name, version] of Object.entries(this.manifest.devDependencies)) {
                const installed = this._isInstalled(name);
                const status = installed ? '✓' : '✗';
                console.log(`   ${status} ${name}@${version}`);
            }
            console.log();
        }
    }

    /**
     * Search for packages
     */
    async search(query) {
        console.log(`🔍 Searching for "${query}"...\n`);

        // For now, search built-in packages
        const builtinPackages = this._getBuiltinPackages();
        const results = builtinPackages.filter(pkg =>
            pkg.name.includes(query) ||
            pkg.description.includes(query)
        );

        if (results.length === 0) {
            console.log('No packages found.\n');
            return;
        }

        console.log(`Found ${results.length} package(s):\n`);
        for (const pkg of results) {
            console.log(`📦 ${pkg.name}@${pkg.version}`);
            console.log(`   ${pkg.description}`);
            console.log(`   Install: raven install ${pkg.name}\n`);
        }
    }

    /**
     * Publish package
     */
    async publish() {
        this._loadManifest();

        console.log('📤 Publishing package...\n');

        // Validation
        if (!this.manifest.name) {
            console.error('❌ Package name is required');
            return;
        }

        if (!this.manifest.version) {
            console.error('❌ Package version is required');
            return;
        }

        console.log(`📦 Package: ${this.manifest.name}@${this.manifest.version}`);
        console.log(`📝 Description: ${this.manifest.description || '(none)'}`);
        console.log(`👤 Author: ${this.manifest.author || '(none)'}`);
        console.log();

        // For now, just create a local package archive
        const archiveName = `${this.manifest.name}-${this.manifest.version}.tar.gz`;
        const archivePath = path.join(CACHE_DIR, archiveName);

        console.log('📦 Creating package archive...');
        console.log(`   → ${archivePath}`);

        // Create archive metadata
        const metadata = {
            ...this.manifest,
            publishedAt: new Date().toISOString(),
            files: this._getPackageFiles()
        };

        fs.writeFileSync(archivePath.replace('.tar.gz', '.json'), JSON.stringify(metadata, null, 2));

        console.log('\n✅ Package published locally!');
        console.log('💡 Tip: In the future, this will publish to a central registry\n');
    }

    /**
     * Show package info
     */
    async info(packageName) {
        console.log(`📦 Package info: ${packageName}\n`);

        const builtinPackages = this._getBuiltinPackages();
        const pkg = builtinPackages.find(p => p.name === packageName);

        if (!pkg) {
            console.log('Package not found.\n');
            return;
        }

        console.log(`Name:        ${pkg.name}`);
        console.log(`Version:     ${pkg.version}`);
        console.log(`Description: ${pkg.description}`);
        console.log(`Author:      ${pkg.author || 'RavensOne Team'}`);
        console.log(`License:     ${pkg.license || 'MIT'}`);
        console.log();
        console.log(`Install:     raven install ${pkg.name}`);
        console.log();
    }

    // ==================== Private Methods ====================

    _ensureDirectories() {
        this._createDirectories([
            REGISTRY_DIR,
            PACKAGES_DIR,
            CACHE_DIR
        ]);
    }

    _createDirectories(dirs) {
        for (const dir of dirs) {
            if (!fs.existsSync(dir)) {
                fs.mkdirSync(dir, { recursive: true });
            }
        }
    }

    _loadManifest() {
        if (!fs.existsSync(MANIFEST_FILE)) {
            console.error('❌ No raven.json found. Run "raven init" first.');
            process.exit(1);
        }

        this.manifest = JSON.parse(fs.readFileSync(MANIFEST_FILE, 'utf-8'));
    }

    _loadLockFile() {
        if (fs.existsSync(LOCK_FILE)) {
            this.lockFile = JSON.parse(fs.readFileSync(LOCK_FILE, 'utf-8'));
        } else {
            this.lockFile = { packages: {} };
        }
    }

    _writeLockFile() {
        const lockData = {
            version: '1.0.0',
            generated: new Date().toISOString(),
            packages: {}
        };

        // Add all installed packages
        for (const [name, info] of this.installed.entries()) {
            lockData.packages[name] = info;
        }

        fs.writeFileSync(LOCK_FILE, JSON.stringify(lockData, null, 2));
    }

    async _installAll() {
        const deps = Object.entries(this.manifest.dependencies || {});
        const devDeps = Object.entries(this.manifest.devDependencies || {});

        for (const [name, version] of [...deps, ...devDeps]) {
            await this._installPackage(name, { version });
        }
    }

    async _installPackage(packageName, options = {}) {
        console.log(`  📥 Fetching ${packageName}...`);

        // Check if it's a built-in package
        const builtinPackages = this._getBuiltinPackages();
        const builtinPkg = builtinPackages.find(p => p.name === packageName);

        if (builtinPkg) {
            await this._installBuiltinPackage(builtinPkg, options);
        } else {
            console.log(`  ⚠️  Package ${packageName} not found in registry`);
            console.log(`  💡 Available packages: ${builtinPackages.map(p => p.name).join(', ')}`);
            return;
        }

        // Update manifest
        if (!options.dev) {
            this.manifest.dependencies[packageName] = builtinPkg.version;
        } else {
            this.manifest.devDependencies[packageName] = builtinPkg.version;
        }

        fs.writeFileSync(MANIFEST_FILE, JSON.stringify(this.manifest, null, 2));
    }

    async _installBuiltinPackage(pkg, options) {
        const packageDir = path.join(PACKAGES_DIR, pkg.name);

        // Create package directory
        this._createDirectories([packageDir]);

        // Write package.json
        fs.writeFileSync(
            path.join(packageDir, 'raven.json'),
            JSON.stringify(pkg, null, 2)
        );

        // Write package files
        for (const [filename, content] of Object.entries(pkg.files || {})) {
            const filePath = path.join(packageDir, filename);
            fs.writeFileSync(filePath, content);
        }

        console.log(`  ✅ Installed ${pkg.name}@${pkg.version}`);

        // Track installation
        this.installed.set(pkg.name, {
            version: pkg.version,
            resolved: 'builtin',
            integrity: 'sha256-builtin'
        });
    }

    _isInstalled(packageName) {
        const packageDir = path.join(PACKAGES_DIR, packageName);
        return fs.existsSync(packageDir);
    }

    _getPackageFiles() {
        const files = [];
        const walk = (dir) => {
            const entries = fs.readdirSync(dir, { withFileTypes: true });
            for (const entry of entries) {
                const fullPath = path.join(dir, entry.name);
                const relativePath = path.relative(process.cwd(), fullPath);

                // Skip certain directories
                if (entry.name === 'node_modules' ||
                    entry.name === '.ravens' ||
                    entry.name === '.git') {
                    continue;
                }

                if (entry.isDirectory()) {
                    walk(fullPath);
                } else {
                    files.push(relativePath);
                }
            }
        };

        walk('src');
        return files;
    }

    _getBuiltinPackages() {
        return [
            {
                name: '@ravens/http-client',
                version: '1.0.0',
                description: 'Full-featured HTTP client with builder pattern',
                author: 'RavensOne Team',
                license: 'MIT',
                files: {
                    'main.raven': `// HTTP Client module
export class HttpClient {
    constructor(baseUrl = '') {
        this.baseUrl = baseUrl;
    }

    async get(url) {
        return await fetch(this.baseUrl + url);
    }

    async post(url, data) {
        return await fetch(this.baseUrl + url, {
            method: 'POST',
            body: JSON.stringify(data)
        });
    }
}
`
                }
            },
            {
                name: '@ravens/ui-components',
                version: '1.0.0',
                description: 'Reusable UI components (Button, Card, Modal, etc)',
                author: 'RavensOne Team',
                license: 'MIT',
                files: {
                    'main.raven': `// UI Components module
export { Button, Card, Modal, Input, Badge, List } from './components';
`
                }
            },
            {
                name: '@ravens/wasm-utils',
                version: '1.0.0',
                description: 'WebAssembly utilities and helpers',
                author: 'RavensOne Team',
                license: 'MIT',
                files: {
                    'main.raven': `// WASM utilities
export class WasmLoader {
    async load(url) {
        const response = await fetch(url);
        const bytes = await response.arrayBuffer();
        return await WebAssembly.instantiate(bytes);
    }
}
`
                }
            },
            {
                name: '@ravens/database',
                version: '1.0.0',
                description: 'Database ORM with query builder',
                author: 'RavensOne Team',
                license: 'MIT',
                files: {
                    'main.raven': `// Database ORM
export class Database {
    constructor(config) {
        this.config = config;
    }

    async query(sql, params) {
        // Execute query
    }
}
`
                }
            },
            {
                name: '@ravens/auth',
                version: '1.0.0',
                description: 'Authentication and authorization utilities',
                author: 'RavensOne Team',
                license: 'MIT',
                files: {
                    'main.raven': `// Authentication module
export class Auth {
    async login(username, password) {
        // Login logic
    }

    async logout() {
        // Logout logic
    }
}
`
                }
            },
            {
                name: '@ravens/router',
                version: '1.0.0',
                description: 'Client-side routing for SPAs',
                author: 'RavensOne Team',
                license: 'MIT',
                files: {
                    'main.raven': `// Router module
export class Router {
    constructor(routes) {
        this.routes = routes;
    }

    navigate(path) {
        // Navigation logic
    }
}
`
                }
            }
        ];
    }
}

// ==================== CLI ====================

function printHelp() {
    console.log(`
🎯 RavensOne Package Manager v${VERSION}

Usage:
  raven <command> [options]

Commands:
  init                    Initialize new RavensOne project
  install [package]       Install package(s)
  uninstall <package>     Uninstall package
  list                    List installed packages
  search <query>          Search for packages
  publish                 Publish package
  info <package>          Show package information
  help                    Show this help message

Examples:
  raven init
  raven install @ravens/http-client
  raven install --dev @ravens/test-utils
  raven list
  raven search http
  raven info @ravens/ui-components
  raven publish

Options:
  --version, -v           Show version
  --help, -h              Show help
  --dev, -D               Install as dev dependency
`);
}

async function main() {
    const args = process.argv.slice(2);
    const command = args[0];
    const pm = new PackageManager();

    if (!command || command === 'help' || command === '--help' || command === '-h') {
        printHelp();
        return;
    }

    if (command === '--version' || command === '-v') {
        console.log(`RavensOne Package Manager v${VERSION}`);
        return;
    }

    try {
        switch (command) {
            case 'init':
                await pm.init({
                    name: args[1],
                    description: args[2]
                });
                break;

            case 'install':
            case 'i':
                const installOptions = {
                    dev: args.includes('--dev') || args.includes('-D')
                };
                await pm.install(args[1], installOptions);
                break;

            case 'uninstall':
            case 'remove':
            case 'rm':
                if (!args[1]) {
                    console.error('❌ Package name required');
                    process.exit(1);
                }
                await pm.uninstall(args[1]);
                break;

            case 'list':
            case 'ls':
                await pm.list();
                break;

            case 'search':
                if (!args[1]) {
                    console.error('❌ Search query required');
                    process.exit(1);
                }
                await pm.search(args[1]);
                break;

            case 'publish':
                await pm.publish();
                break;

            case 'info':
                if (!args[1]) {
                    console.error('❌ Package name required');
                    process.exit(1);
                }
                await pm.info(args[1]);
                break;

            default:
                console.error(`❌ Unknown command: ${command}`);
                console.log('Run "raven help" for usage information');
                process.exit(1);
        }
    } catch (error) {
        console.error('❌ Error:', error.message);
        if (process.env.DEBUG) {
            console.error(error.stack);
        }
        process.exit(1);
    }
}

// Run CLI
if (require.main === module) {
    main();
}

module.exports = PackageManager;

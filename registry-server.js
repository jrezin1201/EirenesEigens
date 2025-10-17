// RavensOne Package Registry Server
// Handles package publishing, versioning, and distribution

const express = require('express');
const multer = require('multer');
const fs = require('fs').promises;
const path = require('path');
const crypto = require('crypto');

const app = express();
const PORT = process.env.PORT || 4000;

// Storage directories
const REGISTRY_DIR = path.join(__dirname, 'registry');
const PACKAGES_DIR = path.join(REGISTRY_DIR, 'packages');
const METADATA_DIR = path.join(REGISTRY_DIR, 'metadata');
const AUTH_DIR = path.join(REGISTRY_DIR, 'auth');

// Middleware
app.use(express.json());
app.use(express.static('public'));

// Configure multer for file uploads
const storage = multer.diskStorage({
    destination: async (req, file, cb) => {
        const packageName = req.body.name;
        const version = req.body.version;
        const packageDir = path.join(PACKAGES_DIR, packageName, version);
        await fs.mkdir(packageDir, { recursive: true });
        cb(null, packageDir);
    },
    filename: (req, file, cb) => {
        cb(null, file.originalname);
    }
});

const upload = multer({ storage });

// Initialize registry directories
async function initRegistry() {
    await fs.mkdir(PACKAGES_DIR, { recursive: true });
    await fs.mkdir(METADATA_DIR, { recursive: true });
    await fs.mkdir(AUTH_DIR, { recursive: true });
    console.log('✅ Registry directories initialized');
}

// Authentication middleware
function requireAuth(req, res, next) {
    const token = req.headers['authorization'];

    if (!token) {
        return res.status(401).json({ error: 'Authentication required' });
    }

    // Simple token validation (in production, use proper JWT)
    const tokenHash = crypto.createHash('sha256').update(token).digest('hex');
    const authFile = path.join(AUTH_DIR, `${tokenHash}.json`);

    fs.access(authFile)
        .then(() => next())
        .catch(() => res.status(401).json({ error: 'Invalid token' }));
}

// Routes

// Health check
app.get('/health', (req, res) => {
    res.json({ status: 'ok', version: '1.0.0' });
});

// Register a new user/token
app.post('/api/register', async (req, res) => {
    try {
        const { username, email } = req.body;

        if (!username || !email) {
            return res.status(400).json({ error: 'Username and email required' });
        }

        // Generate authentication token
        const token = crypto.randomBytes(32).toString('hex');
        const tokenHash = crypto.createHash('sha256').update(token).digest('hex');

        // Store user data
        const userData = {
            username,
            email,
            createdAt: new Date().toISOString(),
            tokenHash
        };

        await fs.writeFile(
            path.join(AUTH_DIR, `${tokenHash}.json`),
            JSON.stringify(userData, null, 2)
        );

        res.json({
            success: true,
            token,
            message: 'Registration successful. Save this token securely!'
        });
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// Publish a package
app.post('/api/publish', requireAuth, upload.single('package'), async (req, res) => {
    try {
        const { name, version, description, author } = req.body;

        if (!name || !version) {
            return res.status(400).json({ error: 'Package name and version required' });
        }

        // Validate semantic versioning
        const versionRegex = /^\d+\.\d+\.\d+$/;
        if (!versionRegex.test(version)) {
            return res.status(400).json({ error: 'Invalid version format. Use semantic versioning (e.g., 1.0.0)' });
        }

        // Check if version already exists
        const versionFile = path.join(METADATA_DIR, name, `${version}.json`);
        try {
            await fs.access(versionFile);
            return res.status(409).json({ error: 'Version already exists' });
        } catch {
            // Version doesn't exist, proceed
        }

        // Save package metadata
        const metadata = {
            name,
            version,
            description: description || '',
            author: author || 'anonymous',
            publishedAt: new Date().toISOString(),
            downloads: 0
        };

        const metadataDir = path.join(METADATA_DIR, name);
        await fs.mkdir(metadataDir, { recursive: true });
        await fs.writeFile(versionFile, JSON.stringify(metadata, null, 2));

        // Update package index
        await updatePackageIndex(name, version, metadata);

        res.json({
            success: true,
            message: `Package ${name}@${version} published successfully`
        });
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// Get package metadata
app.get('/api/packages/:name', async (req, res) => {
    try {
        const { name } = req.params;
        const indexFile = path.join(METADATA_DIR, name, 'index.json');

        const data = await fs.readFile(indexFile, 'utf-8');
        res.json(JSON.parse(data));
    } catch (error) {
        res.status(404).json({ error: 'Package not found' });
    }
});

// Get specific package version
app.get('/api/packages/:name/:version', async (req, res) => {
    try {
        const { name, version } = req.params;
        const versionFile = path.join(METADATA_DIR, name, `${version}.json`);

        const data = await fs.readFile(versionFile, 'utf-8');
        res.json(JSON.parse(data));
    } catch (error) {
        res.status(404).json({ error: 'Package version not found' });
    }
});

// Download package
app.get('/api/packages/:name/:version/download', async (req, res) => {
    try {
        const { name, version } = req.params;
        const packageDir = path.join(PACKAGES_DIR, name, version);

        // Increment download counter
        await incrementDownloads(name, version);

        // Send package files as tarball (simplified - just send the directory)
        const files = await fs.readdir(packageDir);

        res.json({
            name,
            version,
            files: files.map(f => ({
                name: f,
                url: `/api/packages/${name}/${version}/files/${f}`
            }))
        });
    } catch (error) {
        res.status(404).json({ error: 'Package not found' });
    }
});

// Download individual file
app.get('/api/packages/:name/:version/files/:filename', async (req, res) => {
    try {
        const { name, version, filename } = req.params;
        const filePath = path.join(PACKAGES_DIR, name, version, filename);

        const content = await fs.readFile(filePath, 'utf-8');
        res.send(content);
    } catch (error) {
        res.status(404).json({ error: 'File not found' });
    }
});

// Search packages
app.get('/api/search', async (req, res) => {
    try {
        const { q } = req.query;

        if (!q) {
            return res.status(400).json({ error: 'Search query required' });
        }

        const packages = await fs.readdir(METADATA_DIR);
        const results = [];

        for (const pkg of packages) {
            if (pkg.toLowerCase().includes(q.toLowerCase())) {
                const indexFile = path.join(METADATA_DIR, pkg, 'index.json');
                try {
                    const data = await fs.readFile(indexFile, 'utf-8');
                    results.push(JSON.parse(data));
                } catch {
                    // Skip if index doesn't exist
                }
            }
        }

        res.json({ results, count: results.length });
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// List all packages
app.get('/api/packages', async (req, res) => {
    try {
        const packages = await fs.readdir(METADATA_DIR);
        const results = [];

        for (const pkg of packages) {
            const indexFile = path.join(METADATA_DIR, pkg, 'index.json');
            try {
                const data = await fs.readFile(indexFile, 'utf-8');
                results.push(JSON.parse(data));
            } catch {
                // Skip if index doesn't exist
            }
        }

        res.json({ packages: results, count: results.length });
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// Helper functions

async function updatePackageIndex(name, version, metadata) {
    const indexFile = path.join(METADATA_DIR, name, 'index.json');

    let index;
    try {
        const data = await fs.readFile(indexFile, 'utf-8');
        index = JSON.parse(data);
    } catch {
        index = {
            name,
            versions: [],
            latestVersion: version,
            totalDownloads: 0
        };
    }

    // Add new version
    index.versions.push(version);
    index.latestVersion = version;
    index.description = metadata.description;
    index.author = metadata.author;
    index.updatedAt = metadata.publishedAt;

    await fs.writeFile(indexFile, JSON.stringify(index, null, 2));
}

async function incrementDownloads(name, version) {
    const versionFile = path.join(METADATA_DIR, name, `${version}.json`);

    try {
        const data = await fs.readFile(versionFile, 'utf-8');
        const metadata = JSON.parse(data);
        metadata.downloads = (metadata.downloads || 0) + 1;
        await fs.writeFile(versionFile, JSON.stringify(metadata, null, 2));

        // Also update package index
        const indexFile = path.join(METADATA_DIR, name, 'index.json');
        const indexData = await fs.readFile(indexFile, 'utf-8');
        const index = JSON.parse(indexData);
        index.totalDownloads = (index.totalDownloads || 0) + 1;
        await fs.writeFile(indexFile, JSON.stringify(index, null, 2));
    } catch (error) {
        console.error('Error incrementing downloads:', error);
    }
}

// Start server
initRegistry().then(() => {
    app.listen(PORT, () => {
        console.log(`\n🚀 RavensOne Package Registry Server`);
        console.log(`   📦 Running on http://localhost:${PORT}`);
        console.log(`   📂 Registry: ${REGISTRY_DIR}`);
        console.log(`\n   API Endpoints:`);
        console.log(`   - POST /api/register - Register new user`);
        console.log(`   - POST /api/publish - Publish package`);
        console.log(`   - GET  /api/packages - List all packages`);
        console.log(`   - GET  /api/packages/:name - Get package info`);
        console.log(`   - GET  /api/search?q=term - Search packages\n`);
    });
});

// Dev server: serves dist/ with live reload, mimicking Vercel's cleanUrls + trailingSlash: false.
const { existsSync, readFileSync } = require('node:fs');
const { join, extname } = require('node:path');

const OUT = 'dist';

// /blog/foo -> dist/blog/foo/index.html, served without the trailing-slash redirect.
function cleanUrls(req, res, next) {
    const [path, query] = req.url.split('?');
    if (!extname(path) && path !== '/' && existsSync(join(OUT, path, 'index.html'))) {
        req.url = `${path.replace(/\/$/, '')}/index.html${query ? `?${query}` : ''}`;
    }
    next();
}

module.exports = {
    server: { baseDir: OUT },
    files: [OUT],
    middleware: [cleanUrls],
    reloadDebounce: 300,
    notify: false,
    open: false,
    callbacks: {
        // Fallback for anything static serving didn't match.
        ready(err, bs) {
            bs.addMiddleware('*', (req, res) => {
                res.writeHead(404, { 'Content-Type': 'text/html; charset=utf-8' });
                res.end(readFileSync(join(OUT, '404.html')));
            });
        },
    },
};

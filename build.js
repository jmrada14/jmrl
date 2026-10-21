// Static site build: copies static files and renders posts/*.md into dist/.
import { readFileSync, writeFileSync, readdirSync, mkdirSync, rmSync, copyFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { marked } from 'marked';

const SITE = 'https://radaleon.com';
const OUT = 'dist';
const STATIC = ['404.html', 'robots.txt'];
const PER_PAGE = 5;      // posts per page on /blog
const ON_HOME = 3;      // posts listed on the homepage

const template = readFileSync('templates/page.html', 'utf8');
const style = readFileSync('templates/style.css', 'utf8');

const escape = (s) =>
    s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');

// Minimal frontmatter: `key: value` lines between --- fences.
function parse(raw) {
    const match = raw.match(/^---\r?\n([\s\S]*?)\r?\n---\r?\n?([\s\S]*)$/);
    if (!match) return { meta: {}, body: raw };
    const meta = {};
    for (const line of match[1].split(/\r?\n/)) {
        const kv = line.match(/^([A-Za-z_-]+):\s*(.*)$/);
        if (kv) meta[kv[1]] = kv[2].trim().replace(/^["'](.*)["']$/, '$1');
    }
    return { meta, body: match[2] };
}

const render = (vars) => template.replace(/\{\{(\w+)\}\}/g, (_, key) => vars[key] ?? '');

function write(path, html) {
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, html);
}

const formatDate = (iso) =>
    new Date(`${iso}T00:00:00Z`).toLocaleDateString('en-US', {
        year: 'numeric', month: 'long', day: 'numeric', timeZone: 'UTC',
    });

// Page 1 lives at /blog, later pages at /blog/2, /blog/3, ...
const pageUrl = (n) => (n === 1 ? '/blog' : `/blog/${n}`);

rmSync(OUT, { recursive: true, force: true });
mkdirSync(OUT, { recursive: true });

for (const file of STATIC) {
    copyFileSync(file, join(OUT, file));
}

// --- Posts ---------------------------------------------------------------
const posts = readdirSync('posts')
    .filter((f) => f.endsWith('.md'))
    .map((file) => {
        const slug = file.replace(/\.md$/, '');
        const { meta, body } = parse(readFileSync(join('posts', file), 'utf8'));
        if (!meta.title) throw new Error(`posts/${file}: missing "title" in frontmatter`);
        if (!/^\d{4}-\d{2}-\d{2}$/.test(meta.date || '')) {
            throw new Error(`posts/${file}: missing or malformed "date" (expected YYYY-MM-DD)`);
        }
        // A numeric slug would collide with the /blog/<n> pagination URLs.
        if (/^\d+$/.test(slug)) throw new Error(`posts/${file}: slug cannot be a number`);
        if (meta.draft === 'true') return null;
        return { slug, ...meta, html: marked.parse(body) };
    })
    .filter(Boolean)
    .sort((a, b) => b.date.localeCompare(a.date));

for (const post of posts) {
    const url = `${SITE}/blog/${post.slug}`;
    const jsonld = JSON.stringify({
        '@context': 'https://schema.org',
        '@type': 'BlogPosting',
        headline: post.title,
        datePublished: post.date,
        description: post.description || '',
        url,
        author: { '@type': 'Person', name: 'Juan M. Rada León', url: SITE },
    });
    write(join(OUT, 'blog', post.slug, 'index.html'), render({
        title: `${escape(post.title)} — Juan M. Rada León`,
        description: escape(post.description || post.title),
        canonical: url,
        ogType: 'article',
        style,
        crumbs: '<a href="/">Home</a><span aria-hidden="true">/</span><a href="/blog">Writing</a>',
        footerExtra: '<span aria-hidden="true">·</span><a href="/blog">All writing</a>',
        jsonld: `    <script type="application/ld+json">${jsonld}</script>`,
        content: `        <header>
            <h1>${escape(post.title)}</h1>
            <p class="meta"><time datetime="${post.date}">${formatDate(post.date)}</time></p>
        </header>

        <article>
${post.html.trimEnd()}
        </article>
`,
    }));
}

// --- Blog index, paginated ----------------------------------------------
const totalPages = Math.max(1, Math.ceil(posts.length / PER_PAGE));

for (let page = 1; page <= totalPages; page++) {
    const slice = posts.slice((page - 1) * PER_PAGE, page * PER_PAGE);

    const list = slice.map((p) => `                <li>
                    <a href="/blog/${p.slug}">${escape(p.title)}</a>
                    <p class="meta"><time datetime="${p.date}">${formatDate(p.date)}</time></p>${p.description ? `\n                    <p>${escape(p.description)}</p>` : ''}
                </li>`).join('\n');

    const newer = page > 1
        ? `<a href="${pageUrl(page - 1)}">← Newer</a>`
        : '<span></span>';
    const older = page < totalPages
        ? `<a href="${pageUrl(page + 1)}">Older →</a>`
        : '<span></span>';
    const nav = totalPages > 1
        ? `\n        <nav class="pagination">
            ${newer}
            ${older}
        </nav>\n`
        : '';

    write(join(OUT, 'blog', page === 1 ? '' : String(page), 'index.html'), render({
        title: page === 1 ? 'Writing — Juan M. Rada León' : `Writing, page ${page} — Juan M. Rada León`,
        description: 'Notes on software engineering, AI, and payments by Juan M. Rada León.',
        canonical: `${SITE}${pageUrl(page)}`,
        ogType: 'website',
        style,
        // Page 1 is the current page; later pages link back to it.
        crumbs: page === 1
            ? '<a href="/">Home</a><span aria-hidden="true">/</span><span aria-current="page">Writing</span>'
            : '<a href="/">Home</a><span aria-hidden="true">/</span><a href="/blog">Writing</a>',
        footerExtra: '',
        jsonld: '',
        content: `        <header>
            <h1>Writing</h1>${totalPages > 1 ? `\n            <p class="page-meta">Page ${page} of ${totalPages}</p>` : ''}
        </header>

        <section>
            <ul class="posts">
${list || '                <li><p>Nothing here yet.</p></li>'}
            </ul>
        </section>
${nav}`,
    }));
}

// --- Homepage ------------------------------------------------------------
// Same one-line .item shape as Experience: title, then date.
const recent = posts.slice(0, ON_HOME).map((p) => `            <p class="item"><span class="name"><a href="/blog/${p.slug}">${escape(p.title)}</a></span><span class="desc"><time datetime="${p.date}">${formatDate(p.date)}</time></span></p>`).join('\n');

const writingSection = posts.length
    ? `        <section>
            <h2 class="row">Writing${posts.length > ON_HOME ? '<a class="more" href="/blog">All posts</a>' : ''}</h2>
${recent}
        </section>

`
    : '';

const home = readFileSync('index.html', 'utf8');
if (!home.includes('<!--{{writing}}-->')) {
    throw new Error('index.html: missing <!--{{writing}}--> marker');
}
writeFileSync(join(OUT, 'index.html'), home.replace('<!--{{writing}}-->\n', writingSection));

// --- Sitemap -------------------------------------------------------------
const urls = [
    { loc: `${SITE}/`, priority: '1.0' },
    ...Array.from({ length: totalPages }, (_, i) => ({
        loc: `${SITE}${pageUrl(i + 1)}`,
        priority: i === 0 ? '0.8' : '0.5',
    })),
    ...posts.map((p) => ({ loc: `${SITE}/blog/${p.slug}`, priority: '0.6', lastmod: p.date })),
];
writeFileSync(join(OUT, 'sitemap.xml'), `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${urls.map((u) => `    <url>
        <loc>${u.loc}</loc>${u.lastmod ? `\n        <lastmod>${u.lastmod}</lastmod>` : ''}
        <priority>${u.priority}</priority>
    </url>`).join('\n')}
</urlset>
`);

console.log(`built ${posts.length} post(s) across ${totalPages} page(s) → ${OUT}/`);

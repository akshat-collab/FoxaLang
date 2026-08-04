# Foxa Playground (React)

Browser playground for Foxa: Home, Learn, online compiler, Colab-style ML Lab, Help, and Feedback.

## Run

```bash
cd frontend
npm install
npm run dev
```

Open the URL Vite prints (usually http://localhost:5173).

## Pages

| Route | Purpose |
|-------|---------|
| `/` | Home |
| `/learn` | Learn Foxa lessons with runnable examples |
| `/compiler` | Online editor, file upload/download, run |
| `/lab` | Notebook: code + train cells, metrics |
| `/help` | FAQ + cheatsheet |
| `/feedback` | Feedback form (saved in localStorage) |

## Deploy on Vercel

Repo root has `vercel.json`. Import [akshat-collab/FoxaLang](https://github.com/akshat-collab/FoxaLang) in Vercel:

1. **Root Directory:** leave empty (repo root), **or** set to `frontend` and use `frontend/vercel.json`.
2. Framework: Vite (auto) / Output: `frontend/dist` when deploying from root.
3. Redeploy after pushing `vercel.json`.

SPA routes need the rewrite to `index.html` (already in `vercel.json`). Without it, `/learn` etc. return Vercel `404 NOT_FOUND`.

```bash
# from repo root
npx vercel --prod
```

## Deploy on Netlify

Repo root includes `netlify.toml` (`base = frontend`, publish `dist`, SPA redirects).

1. Push this repo to GitHub/GitLab.
2. In [Netlify](https://app.netlify.com): **Add new site → Import an existing project**.
3. Leave build settings as detected from `netlify.toml` (or set manually):
   - **Base directory:** `frontend`
   - **Build command:** `npm run build`
   - **Publish directory:** `frontend/dist` (UI may show `dist` when base is set)
4. Deploy. Client routes (`/learn`, `/compiler`, …) are handled by the `/* → /index.html` rewrite.

### CLI (optional)

```bash
# from repo root, with Netlify CLI logged in
npx netlify deploy --prod
```

Or build locally and drag `frontend/dist` onto Netlify’s drag-and-drop UI.

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

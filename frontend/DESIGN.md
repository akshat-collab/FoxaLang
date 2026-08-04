/**
 * Foxa UI redesign — design plan (audit + tokens)
 *
 * SCREENS: / Home, /compiler Playground, /learn Docs, /lab Notebook, /help, /feedback
 *
 * OLD PROBLEMS: marketing hero + gradient blobs, 3 identical feature cards,
 * orange-on-green AI palette, Syne display marketing type, page-load fade-ups,
 * compiler as padded page not IDE, no syntax highlight, Lab as generic cards.
 *
 * COLORS: slate workspace #0f1115 / #171a21 / #1c212b, text #dce1ea / #8b93a7,
 * accent #3d8bfd, ok #3dd68c, err #f07178. Avoids cream+terracotta and neon-green/violet.
 *
 * TYPE: IBM Plex Sans (chrome/body) + IBM Plex Mono (code). Scale 11–24.
 *
 * LAYOUTS:
 *   Playground: appbar > toolbar > [editor | console] full-bleed split
 *   Lab:        toolbar > sequential cells (In/Out) + optional metrics rail
 *   Learn:      sticky TOC | prose (~68ch) + shared code panel
 *   Home:       short strip + live sample (no hero blob)
 *
 * SIGNATURE: status run-rail (3px left bar + mono status chip idle|running|ok|err)
 */

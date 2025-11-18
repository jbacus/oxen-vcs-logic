# Auxin Frontend

Modern web interface for Auxin Server - Logic Pro version control.

## Features

- 📦 **Repository Management** - Create, browse, and manage repositories
- 📝 **Commit History** - View detailed commit history with timestamps
- 🎵 **Logic Pro Metadata** - Display BPM, sample rate, key signature, and tags
- 🔒 **Lock Management** - Acquire, release, and monitor distributed locks
- 🎨 **Modern UI** - Built with React, TypeScript, and Tailwind CSS
- ⚡ **Fast** - Powered by Vite for instant HMR and optimal builds

## Tech Stack

- **React 18** - UI library
- **TypeScript** - Type safety
- **Vite** - Build tool and dev server
- **Tailwind CSS** - Utility-first styling
- **React Router** - Client-side routing
- **React Query** - Data fetching and caching
- **Axios** - HTTP client
- **Lucide React** - Icon library

## Development

### Prerequisites

- Node.js 18+ and npm/yarn/pnpm
- Auxin server running on `http://localhost:3000`

### Setup

```bash
# Install dependencies
npm install

# Start development server (with API proxy to :3000)
npm run dev

# Open browser to http://localhost:5173
```

The dev server proxies API requests to `http://localhost:3000`, so make sure your auxin-server is running.

### Build for Production

```bash
# Build optimized static files
npm run build

# Preview production build locally
npm run preview
```

The built files will be in the `dist/` directory, ready to be served by auxin-server.

## Project Structure

```
frontend/
├── src/
│   ├── components/       # Reusable UI components
│   │   ├── common/       # Generic components (Header, Loading, etc.)
│   │   ├── repos/        # Repository components
│   │   ├── commits/      # Commit history components
│   │   ├── metadata/     # Logic Pro metadata viewer
│   │   └── locks/        # Lock management UI
│   ├── pages/            # Route pages
│   │   ├── HomePage.tsx  # Repository list
│   │   ├── RepoPage.tsx  # Repository detail
│   │   └── NotFound.tsx  # 404 page
│   ├── services/         # API client
│   │   └── api.ts        # Axios API functions
│   ├── types/            # TypeScript types
│   │   └── index.ts      # Shared type definitions
│   ├── App.tsx           # Main app component
│   ├── main.tsx          # Entry point
│   └── index.css         # Global styles
├── public/               # Static assets
├── index.html            # HTML template
├── package.json          # Dependencies
├── vite.config.ts        # Vite configuration
├── tailwind.config.js    # Tailwind configuration
└── tsconfig.json         # TypeScript configuration
```

## API Integration

The frontend communicates with auxin-server via REST API:

- `GET /api/repos` - List repositories
- `POST /api/repos/{namespace}/{name}` - Create repository
- `GET /api/repos/{namespace}/{name}/commits` - Get commits
- `GET /api/repos/{namespace}/{name}/metadata/{commit}` - Get Logic Pro metadata
- `POST /api/repos/{namespace}/{name}/locks/acquire` - Acquire lock
- `POST /api/repos/{namespace}/{name}/locks/release` - Release lock
- `GET /api/repos/{namespace}/{name}/locks/status` - Get lock status

See `src/services/api.ts` for full API client implementation.

## Configuration

### API Base URL

In development, API requests are proxied to `http://localhost:3000` (configured in `vite.config.ts`).

For production, the frontend expects to be served from the same origin as the API (e.g., both on `http://localhost:3000`).

### Styling

Tailwind CSS utility classes are used throughout. To customize the theme, edit `tailwind.config.js`.

## License

MIT

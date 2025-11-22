# Auxin Frontend Setup Guide

Complete guide for setting up and developing the Auxin web frontend.

## Prerequisites

- **Node.js 18+** and npm
- **Auxin server** (Rust backend)

## Quick Setup

### Option 1: Automated Script (Recommended)

```bash
# Make script executable
chmod +x build-frontend.sh

# Build frontend
./build-frontend.sh

# Start server (serves frontend at http://localhost:3000)
cargo run --release
```

### Option 2: Manual Setup

```bash
# Navigate to frontend directory
cd frontend

# Install dependencies
npm install

# Build for production
npm run build

# Return to server directory and start
cd ..
cargo run --release
```

## Development Workflow

### 1. Start Backend Server

```bash
# In auxin-server directory
cargo run
```

Server runs on `http://localhost:3000` with API endpoints.

### 2. Start Frontend Dev Server

```bash
# In a new terminal
cd frontend
npm run dev
```

Frontend dev server runs on `http://localhost:5173` with:
- Hot Module Replacement (instant updates)
- API proxy to `http://localhost:3000`
- TypeScript type checking
- Fast Vite bundling

### 3. Make Changes

Edit files in `frontend/src/`:
- **Components**: `components/`
- **Pages**: `pages/`
- **API Client**: `services/api.ts`
- **Types**: `types/index.ts`

Changes appear instantly in your browser!

## Project Structure

```
frontend/
├── src/
│   ├── components/
│   │   ├── common/           # Reusable UI (Header, Loading, etc.)
│   │   ├── repos/            # Repository components
│   │   ├── commits/          # Commit history
│   │   ├── metadata/         # Logic Pro metadata viewer
│   │   └── locks/            # Lock management
│   ├── pages/
│   │   ├── HomePage.tsx      # Repository list
│   │   ├── RepoPage.tsx      # Repository detail
│   │   └── NotFound.tsx      # 404 page
│   ├── services/
│   │   └── api.ts            # Axios API client
│   ├── types/
│   │   └── index.ts          # TypeScript interfaces
│   ├── App.tsx               # Main app with routing
│   ├── main.tsx              # Entry point
│   └── index.css             # Global styles
├── public/                   # Static assets
├── package.json              # Dependencies
├── vite.config.ts            # Vite config
├── tailwind.config.js        # Tailwind config
└── tsconfig.json             # TypeScript config
```

## Available Scripts

```bash
npm run dev         # Start dev server (http://localhost:5173)
npm run build       # Build for production (output: dist/)
npm run preview     # Preview production build locally
npm run lint        # Run ESLint
npm run type-check  # Run TypeScript type checking
```

## Technology Stack

| Category | Technology | Purpose |
|----------|-----------|---------|
| **UI Library** | React 18 | Component-based UI |
| **Language** | TypeScript | Type safety |
| **Build Tool** | Vite | Fast dev server & bundling |
| **Styling** | Tailwind CSS | Utility-first CSS |
| **Routing** | React Router v6 | Client-side routing |
| **Data Fetching** | React Query | API calls & caching |
| **HTTP Client** | Axios | REST API requests |
| **Icons** | Lucide React | SVG icons |
| **Dates** | date-fns | Date formatting |

## Adding New Features

### 1. Add API Endpoint

Edit `frontend/src/services/api.ts`:

```typescript
export const getMyData = (namespace: string, name: string) =>
  api.get(`/repos/${namespace}/${name}/mydata`);
```

### 2. Create Type Definition

Edit `frontend/src/types/index.ts`:

```typescript
export interface MyData {
  id: string;
  value: string;
}
```

### 3. Create Component

Create `frontend/src/components/myfeature/MyComponent.tsx`:

```typescript
import { useQuery } from '@tanstack/react-query';
import { getMyData } from '@/services/api';

export function MyComponent({ namespace, name }: Props) {
  const { data, isLoading } = useQuery({
    queryKey: ['mydata', namespace, name],
    queryFn: () => getMyData(namespace, name),
  });

  if (isLoading) return <Loading />;

  return <div>{data?.value}</div>;
}
```

### 4. Use in Page

Add to `frontend/src/pages/RepoPage.tsx`:

```typescript
import { MyComponent } from '@/components/myfeature/MyComponent';

// Inside component:
<MyComponent namespace={namespace} name={name} />
```

## Styling Guide

### Using Tailwind CSS

```tsx
// Utility classes
<div className="bg-blue-500 text-white p-4 rounded-lg">
  Hello World
</div>

// Predefined component classes (see index.css)
<button className="btn-primary">Click Me</button>
<div className="card">Card content</div>
<span className="badge-blue">Tag</span>
```

### Custom Styles

Edit `frontend/src/index.css`:

```css
@layer components {
  .my-custom-class {
    @apply bg-gray-100 p-4 rounded;
  }
}
```

## Troubleshooting

### Port Already in Use

```bash
# Kill process on port 5173
lsof -ti:5173 | xargs kill -9

# Or use different port
npm run dev -- --port 5174
```

### API Proxy Not Working

Check `vite.config.ts` proxy configuration:

```typescript
server: {
  proxy: {
    '/api': {
      target: 'http://localhost:3000',
      changeOrigin: true,
    },
  },
}
```

### Build Errors

```bash
# Clear cache and reinstall
rm -rf node_modules package-lock.json
npm install

# Check TypeScript errors
npm run type-check
```

### CORS Issues

The Rust server should have CORS enabled in `src/main.rs`:

```rust
.wrap(
    actix_cors::Cors::default()
        .allow_any_origin()
        .allow_any_method()
        .allow_any_header(),
)
```

## Production Deployment

### Build Optimized Bundle

```bash
cd frontend
npm run build
```

Output in `frontend/dist/`:
- Minified JavaScript
- Optimized CSS
- Compressed assets
- Source maps

### Serve with Rust Backend

The Rust server automatically serves `frontend/dist/` at `http://localhost:3000`:

```bash
cargo run --release
```

Navigate to `http://localhost:3000` to see the web UI.

### Environment-Specific Builds

Create `.env.production`:

```bash
VITE_API_BASE_URL=https://api.example.com
```

Access in code:

```typescript
const apiUrl = import.meta.env.VITE_API_BASE_URL || '/api';
```

## Performance Tips

1. **Lazy Loading**: Use React.lazy() for code splitting
2. **Memoization**: Use React.memo() for expensive components
3. **Query Caching**: React Query caches API responses (30s default)
4. **Image Optimization**: Use WebP format for images
5. **Bundle Analysis**: `npm run build -- --report`

## Resources

- [React Documentation](https://react.dev/)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/)
- [Vite Guide](https://vitejs.dev/guide/)
- [Tailwind CSS Docs](https://tailwindcss.com/docs)
- [React Query Guide](https://tanstack.com/query/latest)

## Support

For issues or questions:
- Check [frontend/README.md](README.md)
- Open issue on GitHub
- Review API documentation in main README

---

*Happy coding! 🚀*

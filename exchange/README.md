# algofi-knit-web

[![Node.js CI](https://github.com/Algofiorg/algofi-knit-web/actions/workflows/node.js.yml/badge.svg)](https://github.com/Algofiorg/algofi-knit-web/actions/workflows/node.js.yml)

> Algofi platform web frontend

This is a **React SPA with TypeScript** with the following stack:

- [**Vite**](https://vitejs.dev): Dev server + production build tool
- [**React Query**](https://react-query.tanstack.com): Server state fetching, caching, synchronizing, and updating
- [**React Context API**](https://reactjs.org/docs/context.html): Client state management
- [**React Hook Form**](https://react-hook-form.com): Form state management
- [**Zod**](https://github.com/colinhacks/zod): Form schema validation
- [**React Router 6**](https://reactrouter.com): Client-side routing
- [**Suspense**](https://reactjs.org/docs/concurrent-mode-suspense.html): Component render wait mechanism
- [**Error Boundaries**](https://reactjs.org/docs/error-boundaries.html): Rendering error handling
- [**MUI**](https://mui.com): UI library / design system
- [**Storybook**](https://storybook.js.org): UI development, testing, documentation
- [**ESLint**](https://eslint.org): Code linting
- [**Prettier**](https://eslint.org): Code formatting

## Contents

- [Getting Started](#getting-started)
- [Using the development server](#using-the-development-server)
- [Patterns](#patterns)
- [Build for production](#build-for-production)

## Getting Started

### 1. Install dependencies

```sh
npm install
```

### 2. Configure environment variables

Some features require valid environment variables (defined, for example, in a [`root .env.development.local file`](./.env.development.local)) to function properly.  Environment variables prefixed with "`VITE_`" are exposed in client source code and can be consumed in the project using the `import.meta.env` object. Declare environment variable types in the [`.vite-env.d.ts file`](./src/vite-env.d.ts).

Below are listed only those variables necessary for base functionality.

_Note:_ Replace `__YOUR_{VARIABLE_NAME}__` placeholders with appropriate values.

#### Testnet

```sh
VITE_IS_TESTNET=__YOUR_IS_TESTNET__
```

## Using the development server

Launch your development server with this command:

```sh
npm run dev
```

Navigate to [http://localhost:3000](http://localhost:3000) in your browser to explore the app with hot module replacement  over native ESM using [React Fast Refresh](https://github.com/vitejs/vite/tree/main/packages/plugin-react).

## Patterns

### Server state (async)

Declare custom hooks for each `useQuery()` or `useMutation()` call, each will be tagged with a unique cache key

Trigger refetch of query data by invalidating cache keys, no need to call a `refetch()` function. All server-side data returned by the cache key will be updated

Use built-in selectors (`select()` param) to transform server-state into data for display

Use React memoization (`useMemo()`) to apply complex transformations on server-state into data for display (e.g. data transformations that depend on multiple queries or data transformations that integrate local state)

Page level code splitting: Lazy import page components injected into React Router. Wrap in Suspense and Error Boundary

Lazy import components that fetch data and wrap with React Query in Suspense and Error Boundary (use React Query’s `QueryErrorResetBoundary`). Build reset functions which manipulate local state to enable users to escape error conditions

### Local state

Providers can be injected at root app level, but better practice to only wrap components where the context is required

`React.useReducer()` should be used for general cases, will allow for the storage and manipulation of a compound state (much like Redux)

`React.useState()` should be used for simpler storage cases

Should not encapsulate server-side logic or data, which are instead handled by React Query

### Forms

Scaffold forms using [`Form.tsx`](./src/components/form/Form.tsx), which wraps `FormProvider` and allows for form context to be passed to all child components (e.g. styled input fields or an error banner)

Scaffold form UI components by injecting form context with `useFormContext()`

Declare validation for each input at the form level, using a Zod schema

### UI component development

Declare a `__COMPONENT_NAME__.stories.tsx` file for every custom UI component. Include relevant mockups for different statuses or themes

Do not use `style={}` prop except for css-module variable injection (e.g. `style={{ —my-variable: isOpen ? ’10 px’ : ’20 px’ }}`

Use `sx` prop sparingly

Do not use prop-types or class based components

## Build for production

Compile project TypeScript files definited by [`tsconfig.json`](./tsconfig.json) and build for production with [`index.html`](./index.html) as build entry point using [Rollup](https://rollupjs.org/guide/en/).

```sh
npm run build
```

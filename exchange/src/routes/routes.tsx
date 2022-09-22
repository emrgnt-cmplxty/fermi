import Loader from 'components/Loader/index'
import AppLayout from 'layouts/app'
import PageLayout from 'layouts/page'
import Landing from 'pages/Landing'
import NotFound from 'pages/NotFound'
import { lazy, Suspense } from 'react'
import { Navigate, RouteObject, useRoutes } from 'react-router-dom'

const TradePage = lazy(() => import('pages/Trade'))
const FuturesPage = lazy(() => import('pages/Futures'))
const MarketsPage = lazy(() => import('pages/Markets'))
const PortfolioPage = lazy(() => import('pages/Portfolio'))

function AppRoutes() {
  const routes: RouteObject[] = [
    {
      path: '/app',
      element: <AppLayout />,
      children: [
        {
          index: true,
          element: <Navigate to="trade/BTC-USDC" />,
        },
        {
          path: 'markets/*',
          element: (
            <Suspense fallback={<Loader />}>
              <MarketsPage />
            </Suspense>
          ),
        },
        {
          path: 'portfolio/*',
          element: (
            <Suspense fallback={<Loader />}>
              <PortfolioPage />
            </Suspense>
          ),
        },
        {
          path: 'trade/*',
          element: (
            <Suspense fallback={<Loader />}>
              <TradePage />
            </Suspense>
          ),
        },
        {
          path: 'futures/*',
          element: (
            <Suspense fallback={<Loader />}>
              <FuturesPage />
            </Suspense>
          ),
        },
        {
          path: '*',
          element: (
            <main style={{ padding: '1rem' }}>
              <p>Error - This page does not exist.</p>
            </main>
          ),
        },
      ],
    },
    {
      // element: <PageLayout />,
      element: <AppLayout />,
      children: [
        { index: true, element: <Landing /> },
        { path: '404', element: <NotFound /> },
        { path: '*', element: <Navigate to="/404" /> },
      ],
    },
    { path: '*', element: <Navigate to="/404" replace /> },
  ]

  return useRoutes(routes)
}

export default AppRoutes

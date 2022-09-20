import React, { ReactElement, ReactNode, Suspense } from 'react'
import { ErrorBoundary, FallbackProps } from 'react-error-boundary'
import { QueryErrorResetBoundary } from 'react-query'

function DefaultErrorBoundaryFallback({
  error,
  resetErrorBoundary,
}: FallbackProps): ReactElement {
  return (
    <div>
      There was an error!{' '}
      <button onClick={() => resetErrorBoundary()}>Reset Error Boundary</button>
      <pre style={{ whiteSpace: 'normal' }}>{error.message}</pre>
    </div>
  )
}

function DefaultSuspenseFallback(): ReactElement {
  return (
    <div
      style={{
        display: 'flex',
        height: '100%',
        width: '100%',
        alignItems: 'center',
        justifyContent: 'center',
      }}
    >
      QuerySuspenseWrapper wrapped component loading...
    </div>
  )
}

interface QuerySuspenseWrapperProps {
  errorBoundaryFallback?: (
    props: FallbackProps,
  ) => React.ReactElement<
    unknown,
    string | React.FunctionComponent | typeof React.Component
  > | null
  suspenseFallback?:
    | boolean
    | React.ReactChild
    | React.ReactFragment
    | React.ReactPortal
    | null
  children: ReactNode
}

/**
 * Wraps component with QueryErrorResetBoundary, ErrorBoundary, and Suspense
 *
 * @param {ReactNode} children useQuery() & useMutation() calls must include { suspense: true } in options
 */
function QuerySuspenseWrapper({
  children,
  errorBoundaryFallback,
  suspenseFallback,
}: QuerySuspenseWrapperProps) {
  return (
    <QueryErrorResetBoundary>
      {({ reset }) => (
        <ErrorBoundary
          // fallbackRender rather than FallbackComponent to allow local state
          // resetters to be called alongside resetErrorBoundary
          fallbackRender={errorBoundaryFallback || DefaultErrorBoundaryFallback}
          // reset QueryErrorResetBoundary on ErrorBoundary reset
          onReset={reset}
        >
          <Suspense fallback={suspenseFallback || DefaultSuspenseFallback}>
            {/* lazy-loaded components with useQuery or useMutation calls */}
            {children}
          </Suspense>
        </ErrorBoundary>
      )}
    </QueryErrorResetBoundary>
  )
}

export default QuerySuspenseWrapper

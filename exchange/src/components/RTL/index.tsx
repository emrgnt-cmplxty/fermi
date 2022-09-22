import createCache from '@emotion/cache'
import { CacheProvider } from '@emotion/react'
import type { FC, ReactNode } from 'react'
import { useEffect } from 'react'
//import stylisRTLPlugin from 'stylis-plugin-rtl'

interface RTLProps {
  children: ReactNode
  direction: 'ltr' | 'rtl' | undefined
}

const styleCache = () =>
  createCache({
    key: 'rtl',
    prepend: true,
    // We have to temporary ignore this due to incorrect definitions
    // in the stylis-plugin-rtl module
    // @see https://github.com/styled-components/stylis-plugin-rtl/issues/23
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    //stylisPlugins: [stylisRTLPlugin],
  })

export const RTL: FC<RTLProps> = (props) => {
  const { children, direction = 'ltr' } = props

  useEffect(() => {
    document.dir = direction
  }, [direction])

  if (direction === 'rtl') {
    return <CacheProvider value={styleCache()}>{children}</CacheProvider>
  }

  return <>{children}</>
}
